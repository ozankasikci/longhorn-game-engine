use egui::Color32;
use pollster;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize,
};

struct DebugEditor<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    frame_count: u32,
}

impl<'a> DebugEditor<'a> {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
            
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
            
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
            
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
        surface.configure(&device, &config);
        
        let egui_ctx = egui::Context::default();
        let viewport_id = egui_ctx.viewport_id();
        let egui_state = egui_winit::State::new(egui_ctx.clone(), viewport_id, &window, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        
        println!("Debug Editor initialized successfully");
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            egui_ctx,
            egui_state,
            egui_renderer,
            frame_count: 0,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            println!("Resized to: {}x{}", new_size.width, new_size.height);
        }
    }
    
    fn input(&mut self, event: &WindowEvent, window: &winit::window::Window) -> bool {
        let response = self.egui_state.on_window_event(window, event);
        response.consumed
    }
    
    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;
        
        if self.frame_count % 60 == 0 {
            println!("Frame: {}", self.frame_count);
        }
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Debug Render Encoder"),
        });
        
        let raw_input = self.egui_state.take_egui_input(window);
        
        println!("About to run egui context...");
        
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            println!("Inside egui context callback!");
            
            // Always show a window first
            egui::Window::new("Debug Window")
                .default_size([300.0, 200.0])
                .show(ctx, |ui| {
                    ui.label("Debug window is working!");
                    ui.label(format!("Frame: {}", self.frame_count));
                    ui.separator();
                    ui.label("If you see this, egui is rendering.");
                });
                
            // Try the menu bar
            egui::TopBottomPanel::top("debug_menu").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("Debug", |ui| {
                        if ui.button("Test").clicked() {
                            println!("Debug menu clicked!");
                        }
                    });
                    ui.label("Menu Bar Working");
                });
            });
            
            // Try central panel
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Central Panel Debug");
                ui.label("This should be visible in the center");
                
                if ui.button("Click Me").clicked() {
                    println!("Central panel button clicked!");
                }
                
                ui.separator();
                ui.label(format!("Window size: {}x{}", self.size.width, self.size.height));
                ui.label(format!("Frame count: {}", self.frame_count));
            });
        });
        
        println!("egui context finished, handling platform output...");
        
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        
        let tris = self.egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);
        
        println!("Tessellated {} triangles", tris.len());
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.3,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: window.scale_factor() as f32,
            };
            
            println!("About to render egui...");
            self.egui_renderer.render(&mut render_pass, &tris, &screen_descriptor);
            println!("egui render finished");
        }
        
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        if self.frame_count % 60 == 0 {
            println!("Frame {} presented successfully", self.frame_count);
        }
        
        Ok(())
    }
}

fn main() {
    env_logger::init();
    
    println!("Starting debug Unity editor...");
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Debug Unity Editor")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap());
        
    let mut editor = pollster::block_on(DebugEditor::new(window.clone()));
    
    println!("Editor created, starting event loop...");
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !editor.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => {
                            println!("Close requested");
                            elwt.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            println!("Window resized: {:?}", physical_size);
                            editor.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            println!("Scale factor changed: {}", scale_factor);
                            let new_size = PhysicalSize::new(
                                (editor.size.width as f64 * scale_factor) as u32,
                                (editor.size.height as f64 * scale_factor) as u32,
                            );
                            editor.resize(new_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                match editor.render(&window) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        println!("Surface lost, resizing...");
                        editor.resize(editor.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        println!("Out of memory!");
                        elwt.exit();
                    }
                    Err(e) => eprintln!("Render error: {:?}", e),
                }
            }
            _ => {}
        }
    }).unwrap();
}