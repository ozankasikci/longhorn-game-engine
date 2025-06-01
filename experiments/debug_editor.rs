use mobile_game_engine::debug_logger::*;
use egui::Color32;
use pollster;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize,
};

struct DebugApp<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    render_count: u32,
}

impl<'a> DebugApp<'a> {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        debug_log("INIT", "Starting DebugApp initialization");
        
        let size = window.inner_size();
        debug_log_with_data("INIT", "Window size", &size);
        
        debug_log("WGPU", "Creating wgpu instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        debug_log("WGPU", "Instance created successfully");
        
        debug_log("WGPU", "Creating surface");
        let surface = instance.create_surface(window.clone()).unwrap();
        debug_log("WGPU", "Surface created successfully");
        
        debug_log("WGPU", "Requesting adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug_log("WGPU", "Adapter obtained successfully");
        
        debug_log("WGPU", "Requesting device and queue");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("Debug Device"),
                },
                None,
            )
            .await
            .unwrap();
        debug_log("WGPU", "Device and queue created successfully");
        
        debug_log("WGPU", "Getting surface capabilities");
        let surface_caps = surface.get_capabilities(&adapter);
        debug_log_with_data("WGPU", "Surface formats", &surface_caps.formats);
        debug_log_with_data("WGPU", "Surface present modes", &surface_caps.present_modes);
        
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        debug_log_with_data("WGPU", "Selected surface format", &surface_format);
        
        debug_log("WGPU", "Creating surface configuration");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        debug_log_with_data("WGPU", "Surface configuration", &config);
        
        debug_log("WGPU", "Configuring surface");
        surface.configure(&device, &config);
        debug_log("WGPU", "Surface configured successfully");
        
        debug_log("EGUI", "Creating egui context");
        let egui_ctx = egui::Context::default();
        let viewport_id = egui_ctx.viewport_id();
        debug_log_with_data("EGUI", "Viewport ID", &viewport_id);
        
        debug_log("EGUI", "Creating egui winit state");
        let egui_state = egui_winit::State::new(egui_ctx.clone(), viewport_id, &window, None, None);
        debug_log("EGUI", "Egui winit state created successfully");
        
        debug_log("EGUI", "Creating egui wgpu renderer");
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        debug_log("EGUI", "Egui wgpu renderer created successfully");
        
        debug_log("INIT", "DebugApp initialization completed successfully");
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            egui_ctx,
            egui_state,
            egui_renderer,
            render_count: 0,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        debug_log_with_data("RESIZE", "Resize requested", &new_size);
        
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            
            debug_log("RESIZE", "Reconfiguring surface");
            self.surface.configure(&self.device, &self.config);
            debug_log("RESIZE", "Surface reconfigured successfully");
        } else {
            debug_log("RESIZE", "Invalid size - skipping resize");
        }
    }
    
    fn input(&mut self, event: &WindowEvent, window: &winit::window::Window) -> bool {
        debug_log_with_data("INPUT", "Processing window event", &format!("{:?}", event));
        
        let response = self.egui_state.on_window_event(window, event);
        debug_log_with_data("INPUT", "Egui response", &response.consumed);
        
        response.consumed
    }
    
    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        self.render_count += 1;
        increment_frame();
        
        debug_log_with_data("RENDER", "Starting render", &self.render_count);
        
        debug_log("RENDER", "Getting current texture");
        let output = self.surface.get_current_texture()?;
        debug_log_with_data("RENDER", "Surface texture obtained", &output.texture.size());
        
        debug_log("RENDER", "Creating texture view");
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        debug_log("RENDER", "Texture view created");
        
        debug_log("RENDER", "Creating command encoder");
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Debug Render Encoder"),
        });
        debug_log("RENDER", "Command encoder created");
        
        debug_log("EGUI", "Taking egui input");
        let raw_input = self.egui_state.take_egui_input(window);
        debug_log_with_data("EGUI", "Raw input taken", &format!("events: {}, time: {:?}", raw_input.events.len(), raw_input.time));
        
        debug_log("EGUI", "Running egui context");
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            debug_log("UI", "Inside egui context callback");
            
            // Always show debug info
            egui::Window::new("Debug Info")
                .default_size([300.0, 200.0])
                .show(ctx, |ui| {
                    debug_log("UI", "Rendering debug window");
                    ui.heading("🔧 Debug Information");
                    ui.separator();
                    ui.label(format!("Render count: {}", self.render_count));
                    ui.label(format!("Frame: {}", get_frame_count()));
                    ui.label(format!("Window size: {}x{}", self.size.width, self.size.height));
                    ui.label(format!("Scale factor: {:.2}", window.scale_factor()));
                    
                    if ui.button("Force Repaint").clicked() {
                        debug_log("UI", "Force repaint button clicked");
                        ctx.request_repaint();
                    }
                    
                    ui.separator();
                    ui.colored_label(Color32::GREEN, "✅ egui is working!");
                    ui.colored_label(Color32::BLUE, "✅ wgpu is working!");
                    ui.colored_label(Color32::YELLOW, "✅ Events are working!");
                });
            
            // Menu bar
            egui::TopBottomPanel::top("menu").show(ctx, |ui| {
                debug_log("UI", "Rendering menu bar");
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("Test Menu", |ui| {
                        if ui.button("Test Action").clicked() {
                            debug_log("UI", "Test action clicked");
                        }
                    });
                    ui.label(format!("Render: {}", self.render_count));
                });
            });
            
            // Central panel
            egui::CentralPanel::default().show(ctx, |ui| {
                debug_log("UI", "Rendering central panel");
                ui.vertical_centered(|ui| {
                    ui.heading("🎮 Mobile Game Engine Debug");
                    ui.separator();
                    
                    ui.label("This debug version shows if rendering is working.");
                    ui.label("If you can see this text, the engine is functioning correctly!");
                    
                    ui.separator();
                    
                    if ui.button("🔴 Big Red Button").clicked() {
                        debug_log("UI", "Big red button clicked");
                    }
                    
                    if ui.button("🟢 Big Green Button").clicked() {
                        debug_log("UI", "Big green button clicked");
                    }
                    
                    if ui.button("🔵 Big Blue Button").clicked() {
                        debug_log("UI", "Big blue button clicked");
                    }
                    
                    ui.separator();
                    
                    // Test different UI elements
                    ui.horizontal(|ui| {
                        ui.label("Slider:");
                        let mut value = 0.5;
                        ui.add(egui::Slider::new(&mut value, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Checkbox:");
                        let mut checked = true;
                        ui.checkbox(&mut checked, "Test checkbox");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Text input:");
                        let mut text = "Hello World".to_string();
                        ui.text_edit_singleline(&mut text);
                    });
                });
            });
            
            debug_log("UI", "egui context callback completed");
        });
        
        debug_log_with_data("EGUI", "Egui run completed", &format!("shapes: {}, platform_output events: {}", full_output.shapes.len(), full_output.platform_output.events.len()));
        
        debug_log("EGUI", "Handling platform output");
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        debug_log("EGUI", "Platform output handled");
        
        debug_log_with_data("EGUI", "Tessellating shapes", &full_output.shapes.len());
        let tris = self.egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);
        debug_log_with_data("EGUI", "Tessellation completed", &format!("{} triangle meshes", tris.len()));
        
        debug_log("EGUI", "Updating textures");
        for (id, image_delta) in &full_output.textures_delta.set {
            debug_log_with_data("EGUI", "Updating texture", &id);
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        debug_log("EGUI", "Texture updates completed");
        
        debug_log("RENDER", "Beginning render pass");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,  // Slightly blue background to distinguish from grey
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            debug_log("RENDER", "Render pass created");
            
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: window.scale_factor() as f32,
            };
            debug_log("RENDER", &format!("Screen descriptor: {}x{} @ {:.2} ppp", 
                screen_descriptor.size_in_pixels[0], 
                screen_descriptor.size_in_pixels[1], 
                screen_descriptor.pixels_per_point));
            
            debug_log("EGUI", "Rendering egui to render pass");
            self.egui_renderer.render(&mut render_pass, &tris, &screen_descriptor);
            debug_log("EGUI", "Egui render completed");
        }
        debug_log("RENDER", "Render pass ended");
        
        debug_log("EGUI", "Freeing old textures");
        for x in &full_output.textures_delta.free {
            debug_log_with_data("EGUI", "Freeing texture", &x);
            self.egui_renderer.free_texture(x);
        }
        debug_log("EGUI", "Texture cleanup completed");
        
        debug_log("RENDER", "Submitting command buffer");
        self.queue.submit(std::iter::once(encoder.finish()));
        debug_log("RENDER", "Command buffer submitted");
        
        debug_log("RENDER", "Presenting frame");
        output.present();
        debug_log_with_data("RENDER", "Frame presented successfully", &self.render_count);
        
        Ok(())
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    debug_log("MAIN", "🚀 Starting debug Unity editor with comprehensive logging");
    
    debug_log("MAIN", "Creating event loop");
    let event_loop = EventLoop::new().unwrap();
    debug_log("MAIN", "Event loop created successfully");
    
    debug_log("MAIN", "Creating window");
    let window = Arc::new(WindowBuilder::new()
        .with_title("🔧 Debug Unity Editor - Comprehensive Logging")
        .with_inner_size(winit::dpi::LogicalSize::new(1000, 700))
        .build(&event_loop)
        .unwrap());
    debug_log_with_data("MAIN", "Window created", &window.inner_size());
    
    debug_log("MAIN", "Initializing debug app");
    let mut app = pollster::block_on(DebugApp::new(window.clone()));
    debug_log("MAIN", "Debug app initialized successfully");
    
    debug_log("MAIN", "Starting event loop");
    let mut event_count = 0;
    
    event_loop.run(move |event, elwt| {
        event_count += 1;
        
        if event_count % 100 == 1 {
            debug_log_with_data("EVENT", "Event loop iteration", &event_count);
        }
        
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                debug_log_with_data("EVENT", "Window event received", &format!("{:?}", event));
                
                // Handle RedrawRequested BEFORE passing to input
                match event {
                    WindowEvent::RedrawRequested => {
                        debug_log("EVENT", "RedrawRequested - starting render");
                        match app.render(&window) {
                            Ok(_) => {
                                if app.render_count % 60 == 1 {
                                    debug_log("RENDER", "✅ Render successful");
                                }
                            }
                            Err(wgpu::SurfaceError::Lost) => {
                                debug_log("ERROR", "🔄 Surface lost - attempting resize");
                                app.resize(app.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                debug_log("ERROR", "💥 Out of memory - exiting");
                                elwt.exit();
                            }
                            Err(e) => {
                                debug_log_with_data("ERROR", "❌ Render error", &e);
                            }
                        }
                    }
                    _ => {
                        // For all other events, process through input first
                        if !app.input(event, &window) {
                            match event {
                                WindowEvent::CloseRequested => {
                                    debug_log("EVENT", "Close requested - exiting");
                                    elwt.exit();
                                }
                                WindowEvent::Resized(physical_size) => {
                                    debug_log_with_data("EVENT", "Resize event", &physical_size);
                                    app.resize(*physical_size);
                                }
                                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                                    debug_log_with_data("EVENT", "Scale factor changed", &scale_factor);
                                    let new_size = PhysicalSize::new(
                                        (app.size.width as f64 * scale_factor) as u32,
                                        (app.size.height as f64 * scale_factor) as u32,
                                    );
                                    app.resize(new_size);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::AboutToWait => {
                if event_count % 100 == 1 {
                    debug_log("EVENT", "AboutToWait - requesting redraw");
                }
                window.request_redraw();
            }
            _ => {
                if event_count <= 10 {
                    debug_log_with_data("EVENT", "Other event", &format!("{:?}", event));
                }
            }
        }
    }).unwrap();
}