use egui::Color32;
use pollster;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize,
};

struct MinimalUnityApp<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

impl<'a> MinimalUnityApp<'a> {
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
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            egui_ctx,
            egui_state,
            egui_renderer,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    fn input(&mut self, event: &WindowEvent, window: &winit::window::Window) -> bool {
        let response = self.egui_state.on_window_event(window, event);
        response.consumed
    }
    
    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Menu Bar
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New Scene").clicked() {
                            println!("New scene");
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo").clicked() {
                            println!("Undo");
                        }
                    });
                });
            });
            
            // Left panel - Hierarchy
            egui::SidePanel::left("hierarchy").resizable(true).show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();
                if ui.button("Main Camera").clicked() {
                    println!("Selected: Main Camera");
                }
                if ui.button("Directional Light").clicked() {
                    println!("Selected: Directional Light");
                }
            });
            
            // Right panel - Inspector
            egui::SidePanel::right("inspector").resizable(true).show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                ui.label("Transform");
                ui.label("Position: (0, 0, 0)");
                ui.label("Rotation: (0, 0, 0)");
                ui.label("Scale: (1, 1, 1)");
            });
            
            // Central panel - Scene View
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Scene View");
                
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                
                // Dark background
                painter.rect_filled(rect, 0.0, Color32::from_rgb(60, 60, 60));
                
                // Simple grid
                let grid_size = 20.0;
                let center_x = rect.center().x;
                let center_y = rect.center().y;
                
                for i in -5..=5 {
                    let x = center_x + i as f32 * grid_size;
                    let y = center_y + i as f32 * grid_size;
                    
                    painter.line_segment(
                        [egui::Pos2::new(x, rect.top()), egui::Pos2::new(x, rect.bottom())],
                        egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                    );
                    
                    painter.line_segment(
                        [egui::Pos2::new(rect.left(), y), egui::Pos2::new(rect.right(), y)],
                        egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                    );
                }
                
                // Draw a simple cube in the center
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::Pos2::new(center_x, center_y), 
                        egui::Vec2::new(40.0, 40.0)
                    ),
                    0.0,
                    Color32::from_rgb(100, 150, 255)
                );
            });
        });
        
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        
        let tris = self.egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Minimal Unity Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.2,
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
            self.egui_renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }
        
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Minimal Unity Editor - wgpu")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap());
        
    let mut app = pollster::block_on(MinimalUnityApp::new(window.clone()));
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
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
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                match app.render(&window) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    }).unwrap();
}