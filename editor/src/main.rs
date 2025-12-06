use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use longhorn_editor::{Editor, EditorMode, EditorViewportRenderer, RemoteServer, apply_theme};
use longhorn_engine::Engine;
use longhorn_core::{Name, Transform, Sprite, Enabled, AssetId, Script};
use glam::Vec2;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::non_blocking::WorkerGuard;

// Use wgpu from egui_wgpu to ensure version compatibility
use egui_wgpu::wgpu;

struct EditorApp {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    egui_state: Option<EguiState>,
    viewport_renderer: Option<EditorViewportRenderer>,
    engine: Engine,
    editor: Editor,
    remote_server: Option<RemoteServer>,
}

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct EguiState {
    ctx: egui::Context,
    winit_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EditorApp {
    fn new() -> Self {
        let mut engine = Engine::new_headless();

        // Auto-load test_project
        let test_project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("test_project");
        if let Err(e) = engine.load_game(&test_project) {
            log::warn!("Failed to auto-load test_project: {}", e);
        } else {
            log::info!("Auto-loaded game from: {:?}", test_project);
        }

        // Spawn test entities
        engine.world_mut()
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Sprite::new(AssetId::new(1), Vec2::new(32.0, 32.0)))
            .with(Enabled::default())
            .with(Script::new("PlayerController.ts"))
            .build();

        engine.world_mut()
            .spawn()
            .with(Name::new("Enemy"))
            .with(Transform::from_position(Vec2::new(300.0, 150.0)))
            .with(Sprite::new(AssetId::new(2), Vec2::new(64.0, 64.0)))
            .with(Enabled::default())
            .build();

        // Start remote control server
        let remote_server = match RemoteServer::start() {
            Ok(server) => Some(server),
            Err(e) => {
                log::warn!("Failed to start remote server: {}", e);
                None
            }
        };

        Self {
            window: None,
            gpu_state: None,
            egui_state: None,
            viewport_renderer: None,
            engine,
            editor: Editor::new(),
            remote_server,
        }
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
        let size = window.inner_size();

        // Use PRIMARY backends (Metal on macOS, Vulkan on Linux/Windows, DX12 on Windows)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = match instance.create_surface(window.clone()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to create surface: {:?}", e);
                panic!("Cannot create wgpu surface. Make sure you have proper GPU drivers installed.");
            }
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Editor Device"),
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // egui prefers non-sRGB formats (Bgra8Unorm or Rgba8Unorm)
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Bgra8Unorm || *f == wgpu::TextureFormat::Rgba8Unorm)
            .unwrap_or(surface_caps.formats[0]);

        log::info!("Surface format: {:?}", surface_format);
        log::info!("Available formats: {:?}", surface_caps.formats);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create egui state
        let ctx = egui::Context::default();

        // Apply Longhorn theme
        apply_theme(&ctx);

        let winit_state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let mut renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        // Create viewport renderer before moving device/queue into GpuState
        let mut viewport_renderer = EditorViewportRenderer::new(&device, &queue, size.width, size.height);
        viewport_renderer.register_with_egui(&mut renderer, &device);

        self.gpu_state = Some(GpuState {
            device,
            queue,
            surface,
            surface_config,
        });

        self.egui_state = Some(EguiState {
            ctx,
            winit_state,
            renderer,
        });

        self.viewport_renderer = Some(viewport_renderer);

        self.window = Some(window.clone());

        // Request initial redraw
        window.request_redraw();
    }

    fn render(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(gpu) = &mut self.gpu_state else { return };
        let Some(egui_state) = &mut self.egui_state else { return };

        // Process remote commands
        if let Some(ref server) = self.remote_server {
            while let Ok(pending) = server.command_rx.try_recv() {
                let response = self.editor.process_remote_command(pending.command, &mut self.engine);
                let _ = pending.response_tx.send(response);
            }
        }

        // Update game if in play mode and not paused
        let editor_state = self.editor.state();
        if editor_state.mode == EditorMode::Play && !editor_state.paused {
            let _ = self.engine.update();
        }

        // Render game viewport
        if let Some(viewport_renderer) = &mut self.viewport_renderer {
            viewport_renderer.render(&gpu.device, &gpu.queue, self.engine.world());
        }

        // Get surface texture
        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => {
                gpu.surface.configure(&gpu.device, &gpu.surface_config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of memory");
                return;
            }
            Err(e) => {
                log::warn!("Surface error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Run egui frame
        let raw_input = egui_state.winit_state.take_egui_input(window);

        let viewport_texture = self.viewport_renderer
            .as_ref()
            .and_then(|vr| vr.egui_texture_id());

        let mut should_exit = false;
        let full_output = egui_state.ctx.run(raw_input, |ctx| {
            should_exit = self.editor.show(ctx, &mut self.engine, viewport_texture);
        });

        if should_exit {
            std::process::exit(0);
        }

        // Handle platform output
        egui_state.winit_state.handle_platform_output(window, full_output.platform_output);

        // Create command encoder
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Editor Encoder"),
        });

        // Render egui
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.surface_config.width, gpu.surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let tris = egui_state.ctx.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        // Debug: log shape counts on first few frames
        static FRAME_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if frame < 5 {
            log::info!("Frame {}: {} shapes, {} primitives, ppp={}",
                frame, full_output.shapes.len(), tris.len(), full_output.pixels_per_point);
        }

        for (id, delta) in &full_output.textures_delta.set {
            egui_state.renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
        }

        egui_state.renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &tris, &screen_descriptor);

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Convert to 'static lifetime for egui_wgpu compatibility (wgpu 22+)
            let mut render_pass = render_pass.forget_lifetime();
            egui_state.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            egui_state.renderer.free_texture(id);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Request redraw
        window.request_redraw();
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            if let Some(gpu) = &mut self.gpu_state {
                gpu.surface_config.width = size.width;
                gpu.surface_config.height = size.height;
                gpu.surface.configure(&gpu.device, &gpu.surface_config);

                // Update viewport renderer
                if let (Some(vr), Some(egui_state)) =
                    (&mut self.viewport_renderer, &mut self.egui_state)
                {
                    vr.resize(&gpu.device, size.width, size.height);
                    vr.update_egui_texture(&mut egui_state.renderer, &gpu.device);
                }
            }
        }
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Longhorn Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.init_gpu(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle events first
        if let Some(egui_state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let _ = egui_state.winit_state.on_window_event(window, &event);
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
}

/// Set up logging with both console and file output.
/// Returns a guard that must be held for the lifetime of the program.
fn setup_logging() -> WorkerGuard {
    // Log file location: <project>/logs/editor.log
    let log_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    // Create file appender (non-blocking for performance)
    let file_appender = tracing_appender::rolling::never(&log_dir, "editor.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Env filter: respects RUST_LOG, defaults to "debug" for file logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    // Console layer: info level, compact
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_filter(EnvFilter::new("info"));

    // File layer: debug level, with timestamps
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    // Bridge log crate to tracing (so existing log::info!() calls work)
    tracing_log::LogTracer::init().ok();

    log::info!("Logging initialized. File: {}", log_dir.join("editor.log").display());

    guard
}

fn main() {
    // Set up logging - guard must be held until program exit
    let _log_guard = setup_logging();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop.run_app(&mut app).unwrap();
}
