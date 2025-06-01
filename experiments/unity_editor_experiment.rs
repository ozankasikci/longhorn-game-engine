use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use pollster;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize,
};

struct UnityEditorApp<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    
    // Unity Editor State
    hierarchy_open: bool,
    inspector_open: bool,
    project_open: bool,
    console_open: bool,
    selected_object: Option<String>,
    console_messages: Vec<String>,
}

impl<'a> UnityEditorApp<'a> {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        
        // Create wgpu instance
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
        
        // Setup egui
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
            hierarchy_open: true,
            inspector_open: true,
            project_open: true,
            console_open: true,
            selected_object: Some("Main Camera".to_string()),
            console_messages: vec![
                "Unity Editor simulation started".to_string(),
                "Scene loaded successfully".to_string(),
                "All systems initialized".to_string(),
            ],
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
    
    fn update(&mut self) {
        // Update logic here
    }
    
    fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Prepare egui
        let raw_input = self.egui_state.take_egui_input(window);
        
        // Update Unity Editor state for UI
        let mut hierarchy_open = self.hierarchy_open;
        let mut inspector_open = self.inspector_open;
        let mut project_open = self.project_open;
        let mut console_open = self.console_open;
        let mut selected_object = self.selected_object.clone();
        let mut console_messages = self.console_messages.clone();
        
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            draw_unity_ui(ctx, &mut hierarchy_open, &mut inspector_open, &mut project_open, 
                         &mut console_open, &mut selected_object, &mut console_messages);
        });
        
        // Update state back
        self.hierarchy_open = hierarchy_open;
        self.inspector_open = inspector_open;
        self.project_open = project_open;
        self.console_open = console_open;
        self.selected_object = selected_object;
        self.console_messages = console_messages;
        
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        
        let tris = self.egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Unity Editor Render Pass"),
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
        
        // Cleanup
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
}

fn draw_unity_ui(
    ctx: &egui::Context, 
    hierarchy_open: &mut bool,
    inspector_open: &mut bool,
    project_open: &mut bool,
    console_open: &mut bool,
    selected_object: &mut Option<String>,
    console_messages: &mut Vec<String>,
) {
        // Debug: Always show something
        egui::Window::new("Debug").show(ctx, |ui| {
            ui.label("If you see this, egui is working!");
            ui.label(format!("Hierarchy open: {}", hierarchy_open));
            ui.label(format!("Inspector open: {}", inspector_open));
            ui.label(format!("Console messages: {}", console_messages.len()));
        });

        // Menu Bar (must be first)
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        console_messages.push("New scene created".to_string());
                    }
                    if ui.button("Open Scene").clicked() {
                        console_messages.push("Scene opened".to_string());
                    }
                    if ui.button("Save Scene").clicked() {
                        console_messages.push("Scene saved".to_string());
                    }
                    ui.separator();
                    if ui.button("Build Settings").clicked() {
                        console_messages.push("Build settings opened".to_string());
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {}
                    if ui.button("Redo").clicked() {}
                    ui.separator();
                    if ui.button("Project Settings").clicked() {}
                });
                
                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Empty").clicked() {
                        console_messages.push("Empty GameObject created".to_string());
                    }
                    ui.menu_button("3D Object", |ui| {
                        if ui.button("Cube").clicked() {
                            *selected_object = Some("Cube".to_string());
                            console_messages.push("Cube created".to_string());
                        }
                        if ui.button("Sphere").clicked() {
                            *selected_object = Some("Sphere".to_string());
                            console_messages.push("Sphere created".to_string());
                        }
                        if ui.button("Plane").clicked() {
                            *selected_object = Some("Plane".to_string());
                            console_messages.push("Plane created".to_string());
                        }
                    });
                    ui.menu_button("Light", |ui| {
                        if ui.button("Directional Light").clicked() {
                            *selected_object = Some("Directional Light".to_string());
                            console_messages.push("Directional Light created".to_string());
                        }
                    });
                });
                
                ui.menu_button("Window", |ui| {
                    ui.checkbox(hierarchy_open, "Hierarchy");
                    ui.checkbox(inspector_open, "Inspector");
                    ui.checkbox(project_open, "Project");
                    ui.checkbox(console_open, "Console");
                });
            });
        });
        
        // Hierarchy Panel
        if *hierarchy_open {
            egui::SidePanel::left("hierarchy").resizable(true).show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if ui.selectable_label(selected_object.as_ref().map_or(false, |s| s == "Main Camera"), "Main Camera").clicked() {
                        *selected_object = Some("Main Camera".to_string());
                    }
                    
                    if ui.selectable_label(selected_object.as_ref().map_or(false, |s| s == "Directional Light"), "Directional Light").clicked() {
                        *selected_object = Some("Directional Light".to_string());
                    }
                    
                    ui.collapsing("Environment", |ui| {
                        if ui.selectable_label(selected_object.as_ref().map_or(false, |s| s == "Skybox"), "Skybox").clicked() {
                            *selected_object = Some("Skybox".to_string());
                        }
                    });
                    
                    if let Some(ref obj) = selected_object.clone() {
                        if obj != "Main Camera" && obj != "Directional Light" && obj != "Skybox" {
                            if ui.selectable_label(true, obj).clicked() {
                                // Keep selection
                            }
                        }
                    }
                });
            });
        }
        
        // Inspector Panel
        if *inspector_open {
            egui::SidePanel::right("inspector").resizable(true).show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                
                if let Some(ref obj) = selected_object {
                    ui.label(format!("Selected: {}", obj));
                    ui.separator();
                    
                    // Transform component
                    ui.collapsing("Transform", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("Y: "));
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("Z: "));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Rotation:");
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("Y: "));
                            ui.add(egui::DragValue::new(&mut 0.0).prefix("Z: "));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            ui.add(egui::DragValue::new(&mut 1.0).prefix("X: "));
                            ui.add(egui::DragValue::new(&mut 1.0).prefix("Y: "));
                            ui.add(egui::DragValue::new(&mut 1.0).prefix("Z: "));
                        });
                    });
                    
                    // Object-specific components
                    match obj.as_str() {
                        "Main Camera" => {
                            ui.collapsing("Camera", |ui| {
                                ui.label("Field of View: 60");
                                ui.label("Near Clipping: 0.3");
                                ui.label("Far Clipping: 1000");
                            });
                        }
                        "Directional Light" => {
                            ui.collapsing("Light", |ui| {
                                ui.label("Type: Directional");
                                ui.label("Intensity: 1.0");
                                ui.color_edit_button_rgb(&mut [1.0, 1.0, 1.0]);
                            });
                        }
                        _ => {
                            if obj.contains("Cube") || obj.contains("Sphere") || obj.contains("Plane") {
                                ui.collapsing("Mesh Renderer", |ui| {
                                    ui.label("Material: Default");
                                    ui.label("Cast Shadows: On");
                                    ui.label("Receive Shadows: On");
                                });
                            }
                        }
                    }
                } else {
                    ui.label("No object selected");
                }
            });
        }
        
        // Scene View (Central Panel)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Scene");
            
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            
            // Draw a simple 3D-like scene background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(60, 60, 60));
            
            // Draw grid
            let grid_size = 20.0;
            let center_x = rect.center().x;
            let center_y = rect.center().y;
            
            for i in -10..=10 {
                let x = center_x + i as f32 * grid_size;
                let y = center_y + i as f32 * grid_size;
                
                // Vertical lines
                painter.line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                );
                
                // Horizontal lines
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                );
            }
            
            // Draw objects in scene
            if let Some(ref obj) = selected_object {
                let obj_pos = Pos2::new(center_x, center_y);
                match obj.as_str() {
                    "Cube" => {
                        painter.rect_filled(
                            Rect::from_center_size(obj_pos, Vec2::new(40.0, 40.0)),
                            0.0,
                            Color32::from_rgb(100, 150, 255)
                        );
                    }
                    "Sphere" => {
                        painter.circle_filled(obj_pos, 20.0, Color32::from_rgb(255, 100, 100));
                    }
                    "Plane" => {
                        painter.rect_filled(
                            Rect::from_center_size(obj_pos, Vec2::new(80.0, 10.0)),
                            0.0,
                            Color32::from_rgb(100, 255, 100)
                        );
                    }
                    _ => {}
                }
            }
            
            // Scene controls
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("🎯").clicked() {
                        console_messages.push("Focus on selected object".to_string());
                    }
                    if ui.button("🔄").clicked() {
                        console_messages.push("Reset camera view".to_string());
                    }
                });
            });
        });
        
        // Project Panel
        if *project_open {
            egui::TopBottomPanel::bottom("project").resizable(true).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Project");
                    ui.separator();
                    if ui.button("Assets").clicked() {}
                    if ui.button("Packages").clicked() {}
                });
                ui.separator();
                
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("📁 Scripts");
                        ui.label("📁 Materials");
                        ui.label("📁 Textures");
                        ui.label("📁 Prefabs");
                        ui.label("📁 Scenes");
                        ui.label("📁 Audio");
                    });
                });
            });
        }
        
        // Console Panel
        if *console_open {
            egui::TopBottomPanel::bottom("console").resizable(true).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Console");
                    ui.separator();
                    if ui.button("Clear").clicked() {
                        console_messages.clear();
                    }
                });
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for message in console_messages {
                        ui.label(format!("ℹ️ {}", message));
                    }
                });
            });
        }
}

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Unity Editor Simulation - wgpu")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap());
        
    let mut app = pollster::block_on(UnityEditorApp::new(window.clone()));
    
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
                app.update();
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