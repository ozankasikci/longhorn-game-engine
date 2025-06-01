use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use pollster;
use std::collections::HashMap;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize,
};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    pub transform: Transform,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
    pub active: bool,
}

impl GameObject {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            transform: Transform::default(),
            children: Vec::new(),
            parent: None,
            active: true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ConsoleMessageType {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ConsoleMessage {
    pub message: String,
    pub message_type: ConsoleMessageType,
    pub timestamp: std::time::Instant,
}

pub struct EditorState {
    pub scene_objects: HashMap<u32, GameObject>,
    pub selected_object: Option<u32>,
    pub next_object_id: u32,
    pub console_messages: Vec<ConsoleMessage>,
    pub scene_name: String,
    
    // Panel visibility
    pub hierarchy_open: bool,
    pub inspector_open: bool,
    pub project_open: bool,
    pub console_open: bool,
    
    // Scene view state
    pub scene_pan: [f32; 2],
    pub scene_zoom: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut state = Self {
            scene_objects: HashMap::new(),
            selected_object: None,
            next_object_id: 1,
            console_messages: Vec::new(),
            scene_name: "Untitled Scene".to_string(),
            hierarchy_open: true,
            inspector_open: true,
            project_open: true,
            console_open: true,
            scene_pan: [0.0, 0.0],
            scene_zoom: 1.0,
        };
        
        // Create default scene objects
        state.add_default_objects();
        state
    }
}

impl EditorState {
    pub fn add_default_objects(&mut self) {
        // Main Camera
        let camera = GameObject::new(self.next_object_id, "Main Camera".to_string());
        self.scene_objects.insert(self.next_object_id, camera);
        self.next_object_id += 1;
        
        // Directional Light
        let mut light = GameObject::new(self.next_object_id, "Directional Light".to_string());
        light.transform.rotation = [50.0, -30.0, 0.0];
        self.scene_objects.insert(self.next_object_id, light);
        self.next_object_id += 1;
        
        self.log_info("Scene initialized with default objects");
    }
    
    pub fn create_object(&mut self, name: String) -> u32 {
        let obj = GameObject::new(self.next_object_id, name.clone());
        let id = self.next_object_id;
        self.scene_objects.insert(id, obj);
        self.next_object_id += 1;
        self.log_info(&format!("Created object: {}", name));
        id
    }
    
    pub fn delete_object(&mut self, id: u32) {
        if let Some(obj) = self.scene_objects.remove(&id) {
            self.log_info(&format!("Deleted object: {}", obj.name));
            if self.selected_object == Some(id) {
                self.selected_object = None;
            }
        }
    }
    
    pub fn select_object(&mut self, id: u32) {
        if self.scene_objects.contains_key(&id) {
            self.selected_object = Some(id);
            if let Some(obj) = self.scene_objects.get(&id) {
                self.log_info(&format!("Selected: {}", obj.name));
            }
        }
    }
    
    pub fn log_info(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Info,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn log_warning(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Warning,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn log_error(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Error,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn clear_console(&mut self) {
        self.console_messages.clear();
        self.log_info("Console cleared");
    }
}

struct UnityEditor<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    editor_state: EditorState,
}

impl<'a> UnityEditor<'a> {
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
            editor_state: EditorState::default(),
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
        let mut editor_state = std::mem::take(&mut self.editor_state);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            draw_editor_ui(ctx, &mut editor_state);
        });
        self.editor_state = editor_state;
        
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        
        let tris = self.egui_ctx.tessellate(full_output.shapes, window.scale_factor() as f32);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
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
        
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
}

fn draw_editor_ui(ctx: &egui::Context, editor_state: &mut EditorState) {
        // Menu Bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        editor_state.scene_objects.clear();
                        editor_state.selected_object = None;
                        editor_state.next_object_id = 1;
                        editor_state.add_default_objects();
                        editor_state.scene_name = "Untitled Scene".to_string();
                    }
                    if ui.button("Open Scene").clicked() {
                        editor_state.log_info("Open Scene - Not implemented");
                    }
                    if ui.button("Save Scene").clicked() {
                        editor_state.log_info("Save Scene - Not implemented");
                    }
                    ui.separator();
                    if ui.button("Build Settings").clicked() {
                        editor_state.log_info("Build Settings opened");
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        editor_state.log_info("Undo - Not implemented");
                    }
                    if ui.button("Redo").clicked() {
                        editor_state.log_info("Redo - Not implemented");
                    }
                    ui.separator();
                    if ui.button("Project Settings").clicked() {
                        editor_state.log_info("Project Settings - Not implemented");
                    }
                });
                
                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Empty").clicked() {
                        let id = editor_state.create_object("GameObject".to_string());
                        editor_state.select_object(id);
                    }
                    ui.separator();
                    ui.menu_button("3D Object", |ui| {
                        if ui.button("Cube").clicked() {
                            let id = editor_state.create_object("Cube".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Sphere").clicked() {
                            let id = editor_state.create_object("Sphere".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Plane").clicked() {
                            let id = editor_state.create_object("Plane".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Capsule").clicked() {
                            let id = editor_state.create_object("Capsule".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Cylinder").clicked() {
                            let id = editor_state.create_object("Cylinder".to_string());
                            editor_state.select_object(id);
                        }
                    });
                    ui.menu_button("2D Object", |ui| {
                        if ui.button("Sprite").clicked() {
                            let id = editor_state.create_object("Sprite".to_string());
                            editor_state.select_object(id);
                        }
                    });
                    ui.menu_button("Light", |ui| {
                        if ui.button("Directional Light").clicked() {
                            let id = editor_state.create_object("Directional Light".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Point Light").clicked() {
                            let id = editor_state.create_object("Point Light".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Spot Light").clicked() {
                            let id = editor_state.create_object("Spot Light".to_string());
                            editor_state.select_object(id);
                        }
                    });
                    ui.menu_button("Audio", |ui| {
                        if ui.button("Audio Source").clicked() {
                            let id = editor_state.create_object("Audio Source".to_string());
                            editor_state.select_object(id);
                        }
                    });
                    ui.menu_button("UI", |ui| {
                        if ui.button("Canvas").clicked() {
                            let id = editor_state.create_object("Canvas".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Button").clicked() {
                            let id = editor_state.create_object("Button".to_string());
                            editor_state.select_object(id);
                        }
                        if ui.button("Text").clicked() {
                            let id = editor_state.create_object("Text".to_string());
                            editor_state.select_object(id);
                        }
                    });
                });
                
                ui.menu_button("Component", |ui| {
                    if ui.button("Mesh").clicked() {
                        editor_state.log_info("Add Mesh Component - Not implemented");
                    }
                    if ui.button("Material").clicked() {
                        editor_state.log_info("Add Material Component - Not implemented");
                    }
                    if ui.button("Rigidbody").clicked() {
                        editor_state.log_info("Add Rigidbody Component - Not implemented");
                    }
                    if ui.button("Collider").clicked() {
                        editor_state.log_info("Add Collider Component - Not implemented");
                    }
                });
                
                ui.menu_button("Window", |ui| {
                    ui.checkbox(&mut editor_state.hierarchy_open, "Hierarchy");
                    ui.checkbox(&mut editor_state.inspector_open, "Inspector");
                    ui.checkbox(&mut editor_state.project_open, "Project");
                    ui.checkbox(&mut editor_state.console_open, "Console");
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        editor_state.log_info("Mobile Game Engine v0.1.0 - Built with Rust + wgpu");
                    }
                    if ui.button("Documentation").clicked() {
                        editor_state.log_info("Documentation - Not available yet");
                    }
                });
                
                ui.separator();
                ui.label(format!("Scene: {}", editor_state.scene_name));
            });
        });
        
        // Console Panel (bottom)
        if editor_state.console_open {
            egui::TopBottomPanel::bottom("console").resizable(true).default_height(150.0).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Console");
                    ui.separator();
                    if ui.button("Clear").clicked() {
                        editor_state.clear_console();
                    }
                    ui.separator();
                    ui.label(format!("{} messages", editor_state.console_messages.len()));
                });
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    for msg in &editor_state.console_messages {
                        let (icon, color) = match msg.message_type {
                            ConsoleMessageType::Info => ("ℹ️", Color32::WHITE),
                            ConsoleMessageType::Warning => ("⚠️", Color32::YELLOW),
                            ConsoleMessageType::Error => ("❌", Color32::RED),
                        };
                        ui.horizontal(|ui| {
                            ui.label(icon);
                            ui.colored_label(color, &msg.message);
                        });
                    }
                });
            });
        }
        
        // Project Panel (bottom)
        if editor_state.project_open {
            egui::TopBottomPanel::bottom("project").resizable(true).default_height(200.0).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Project");
                    ui.separator();
                    if ui.button("Assets").clicked() {}
                    if ui.button("Packages").clicked() {}
                });
                ui.separator();
                
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        // Asset folders
                        ui.vertical(|ui| {
                            ui.label("📁 Scripts");
                            ui.label("📁 Materials");
                            ui.label("📁 Textures");
                            ui.label("📁 Prefabs");
                            ui.label("📁 Scenes");
                            ui.label("📁 Audio");
                            ui.label("📁 Animations");
                            ui.label("📁 Fonts");
                        });
                    });
                });
            });
        }
        
        // Hierarchy Panel (left side)
        if editor_state.hierarchy_open {
            egui::SidePanel::left("hierarchy").resizable(true).default_width(250.0).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Hierarchy");
                    ui.separator();
                    if ui.button("🔍").clicked() {
                        editor_state.log_info("Search in hierarchy - Not implemented");
                    }
                });
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect object data to avoid borrowing issues
                    let mut objects: Vec<(u32, String, bool)> = editor_state.scene_objects
                        .iter()
                        .map(|(id, obj)| (*id, obj.name.clone(), obj.active))
                        .collect();
                    objects.sort_by_key(|(id, _, _)| *id);
                    
                    let mut actions = Vec::new();
                    
                    for (id, name, active) in objects {
                        let is_selected = editor_state.selected_object == Some(id);
                        
                        ui.horizontal(|ui| {
                            // Active checkbox
                            let mut active_state = active;
                            if ui.checkbox(&mut active_state, "").changed() {
                                actions.push(("toggle_active", id, active_state));
                            }
                            
                            // Object name (selectable)
                            if ui.selectable_label(is_selected, &name).clicked() {
                                actions.push(("select", id, false));
                            }
                        });
                        
                        // Right-click context menu
                        ui.allocate_response(ui.available_size(), egui::Sense::click()).context_menu(|ui| {
                            if ui.button("Duplicate").clicked() {
                                actions.push(("duplicate", id, false));
                                ui.close_menu();
                            }
                            if ui.button("Delete").clicked() {
                                actions.push(("delete", id, false));
                                ui.close_menu();
                            }
                            if ui.button("Rename").clicked() {
                                actions.push(("rename", id, false));
                                ui.close_menu();
                            }
                        });
                    }
                    
                    // Process actions after UI rendering
                    for (action, id, value) in actions {
                        match action {
                            "toggle_active" => {
                                if let Some(obj) = editor_state.scene_objects.get_mut(&id) {
                                    obj.active = value;
                                    let name = obj.name.clone();
                                    drop(obj);
                                    editor_state.log_info(&format!("Toggled {} active state", name));
                                }
                            }
                            "select" => {
                                editor_state.select_object(id);
                            }
                            "duplicate" => {
                                if let Some(obj) = editor_state.scene_objects.get(&id) {
                                    let new_name = format!("{} (1)", obj.name);
                                    let new_id = editor_state.create_object(new_name);
                                    editor_state.select_object(new_id);
                                }
                            }
                            "delete" => {
                                editor_state.delete_object(id);
                            }
                            "rename" => {
                                editor_state.log_info("Rename - Not implemented");
                            }
                            _ => {}
                        }
                    }
                });
            });
        }
        
        // Inspector Panel (right side)
        if editor_state.inspector_open {
            egui::SidePanel::right("inspector").resizable(true).default_width(300.0).show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                
                if let Some(selected_id) = editor_state.selected_object {
                    if let Some(obj) = editor_state.scene_objects.get(&selected_id).cloned() {
                        // Object header
                        ui.horizontal(|ui| {
                            let mut active = obj.active;
                            if ui.checkbox(&mut active, "").changed() {
                                if let Some(obj_mut) = editor_state.scene_objects.get_mut(&selected_id) {
                                    obj_mut.active = active;
                                }
                            }
                            ui.text_edit_singleline(&mut obj.name.clone());
                        });
                        
                        ui.separator();
                        
                        // Transform component
                        ui.collapsing("Transform", |ui| {
                            let transform = &obj.transform;
                            
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.add(egui::DragValue::new(&mut transform.position[0].clone()).prefix("X: ").speed(0.1));
                                ui.add(egui::DragValue::new(&mut transform.position[1].clone()).prefix("Y: ").speed(0.1));
                                ui.add(egui::DragValue::new(&mut transform.position[2].clone()).prefix("Z: ").speed(0.1));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Rotation:");
                                ui.add(egui::DragValue::new(&mut transform.rotation[0].clone()).prefix("X: ").speed(1.0));
                                ui.add(egui::DragValue::new(&mut transform.rotation[1].clone()).prefix("Y: ").speed(1.0));
                                ui.add(egui::DragValue::new(&mut transform.rotation[2].clone()).prefix("Z: ").speed(1.0));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Scale:");
                                ui.add(egui::DragValue::new(&mut transform.scale[0].clone()).prefix("X: ").speed(0.1));
                                ui.add(egui::DragValue::new(&mut transform.scale[1].clone()).prefix("Y: ").speed(0.1));
                                ui.add(egui::DragValue::new(&mut transform.scale[2].clone()).prefix("Z: ").speed(0.1));
                            });
                        });
                        
                        // Object-specific components
                        match obj.name.as_str() {
                            name if name.contains("Camera") => {
                                ui.collapsing("Camera", |ui| {
                                    ui.label("Field of View: 60°");
                                    ui.label("Near Clipping: 0.3");
                                    ui.label("Far Clipping: 1000");
                                    ui.label("Projection: Perspective");
                                });
                            }
                            name if name.contains("Light") => {
                                ui.collapsing("Light", |ui| {
                                    ui.label("Type: Directional");
                                    ui.label("Intensity: 1.0");
                                    ui.horizontal(|ui| {
                                        ui.label("Color:");
                                        ui.color_edit_button_rgb(&mut [1.0, 1.0, 1.0]);
                                    });
                                    ui.label("Shadows: Soft Shadows");
                                });
                            }
                            name if name.contains("Cube") || name.contains("Sphere") || name.contains("Plane") || name.contains("Capsule") || name.contains("Cylinder") => {
                                ui.collapsing("Mesh Filter", |ui| {
                                    ui.label(format!("Mesh: {}", obj.name));
                                });
                                ui.collapsing("Mesh Renderer", |ui| {
                                    ui.label("Material: Default");
                                    ui.label("Cast Shadows: On");
                                    ui.label("Receive Shadows: On");
                                    ui.label("Light Probes: Blend Probes");
                                });
                            }
                            name if name.contains("Sprite") => {
                                ui.collapsing("Sprite Renderer", |ui| {
                                    ui.label("Sprite: None");
                                    ui.label("Color: White");
                                    ui.label("Flip X: false");
                                    ui.label("Flip Y: false");
                                });
                            }
                            _ => {
                                ui.label("(No additional components)");
                            }
                        }
                        
                        ui.separator();
                        if ui.button("Add Component").clicked() {
                            editor_state.log_info("Add Component - Not implemented");
                        }
                    }
                } else {
                    ui.label("No object selected");
                    ui.separator();
                    ui.label("Select an object in the Hierarchy to view its properties here.");
                }
            });
        }
        
        // Scene View (central panel)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Scene");
                ui.separator();
                
                // Scene view controls
                if ui.button("🎯").clicked() {
                    editor_state.scene_pan = [0.0, 0.0];
                    editor_state.scene_zoom = 1.0;
                    editor_state.log_info("Reset scene view");
                }
                if ui.button("🔄").clicked() {
                    editor_state.log_info("Refresh scene view");
                }
                
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", editor_state.scene_zoom));
                ui.label(format!("Objects: {}", editor_state.scene_objects.len()));
            });
            
            ui.separator();
            
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            
            // Dark scene background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(60, 60, 60));
            
            // Grid
            let grid_size = 20.0 * editor_state.scene_zoom;
            let center_x = rect.center().x + editor_state.scene_pan[0];
            let center_y = rect.center().y + editor_state.scene_pan[1];
            
            for i in -20..=20 {
                let x = center_x + i as f32 * grid_size;
                let y = center_y + i as f32 * grid_size;
                
                if x >= rect.left() && x <= rect.right() {
                    painter.line_segment(
                        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                        Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                    );
                }
                
                if y >= rect.top() && y <= rect.bottom() {
                    painter.line_segment(
                        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                        Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
                    );
                }
            }
            
            // Draw scene objects
            for (id, obj) in &editor_state.scene_objects {
                if !obj.active {
                    continue;
                }
                
                let obj_x = center_x + obj.transform.position[0] * 20.0 * editor_state.scene_zoom;
                let obj_y = center_y - obj.transform.position[1] * 20.0 * editor_state.scene_zoom; // Flip Y for screen coords
                let obj_pos = Pos2::new(obj_x, obj_y);
                
                let is_selected = editor_state.selected_object == Some(*id);
                let base_size = 30.0 * editor_state.scene_zoom;
                
                match obj.name.as_str() {
                    name if name.contains("Cube") => {
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(100, 150, 255) };
                        painter.rect_filled(
                            Rect::from_center_size(obj_pos, Vec2::new(base_size, base_size)),
                            0.0,
                            color
                        );
                        if is_selected {
                            painter.rect_stroke(
                                Rect::from_center_size(obj_pos, Vec2::new(base_size + 4.0, base_size + 4.0)),
                                0.0,
                                Stroke::new(2.0, Color32::YELLOW)
                            );
                        }
                    }
                    name if name.contains("Sphere") => {
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(255, 100, 100) };
                        painter.circle_filled(obj_pos, base_size * 0.5, color);
                        if is_selected {
                            painter.circle_stroke(obj_pos, base_size * 0.5 + 2.0, Stroke::new(2.0, Color32::YELLOW));
                        }
                    }
                    name if name.contains("Plane") => {
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(100, 255, 100) };
                        painter.rect_filled(
                            Rect::from_center_size(obj_pos, Vec2::new(base_size * 1.5, base_size * 0.3)),
                            0.0,
                            color
                        );
                        if is_selected {
                            painter.rect_stroke(
                                Rect::from_center_size(obj_pos, Vec2::new(base_size * 1.5 + 4.0, base_size * 0.3 + 4.0)),
                                0.0,
                                Stroke::new(2.0, Color32::YELLOW)
                            );
                        }
                    }
                    name if name.contains("Camera") => {
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(200, 200, 200) };
                        // Camera icon (triangle)
                        let points = [
                            Pos2::new(obj_x - base_size * 0.3, obj_y + base_size * 0.3),
                            Pos2::new(obj_x + base_size * 0.3, obj_y),
                            Pos2::new(obj_x - base_size * 0.3, obj_y - base_size * 0.3),
                        ];
                        painter.add(egui::Shape::convex_polygon(points.to_vec(), color, Stroke::NONE));
                        if is_selected {
                            painter.circle_stroke(obj_pos, base_size * 0.4, Stroke::new(2.0, Color32::YELLOW));
                        }
                    }
                    name if name.contains("Light") => {
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(255, 255, 150) };
                        // Light icon (star-like)
                        painter.circle_filled(obj_pos, base_size * 0.3, color);
                        for i in 0..8 {
                            let angle = i as f32 * std::f32::consts::PI / 4.0;
                            let start = obj_pos;
                            let end = Pos2::new(
                                obj_x + angle.cos() * base_size * 0.5,
                                obj_y + angle.sin() * base_size * 0.5
                            );
                            painter.line_segment([start, end], Stroke::new(2.0, color));
                        }
                        if is_selected {
                            painter.circle_stroke(obj_pos, base_size * 0.6, Stroke::new(2.0, Color32::YELLOW));
                        }
                    }
                    _ => {
                        // Default object (circle)
                        let color = if is_selected { Color32::from_rgb(255, 255, 100) } else { Color32::from_rgb(150, 150, 150) };
                        painter.circle_filled(obj_pos, base_size * 0.4, color);
                        if is_selected {
                            painter.circle_stroke(obj_pos, base_size * 0.4 + 2.0, Stroke::new(2.0, Color32::YELLOW));
                        }
                    }
                }
                
                // Object name label
                let text_pos = Pos2::new(obj_x, obj_y + base_size * 0.7);
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_TOP,
                    &obj.name,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE
                );
            }
            
            // Handle scene view interactions
            let response = ui.allocate_response(rect.size(), egui::Sense::click_and_drag());
            
            if response.clicked() {
                // Check if we clicked on an object
                let click_pos = response.interact_pointer_pos().unwrap_or_default();
                let mut clicked_object = None;
                
                for (id, obj) in &editor_state.scene_objects {
                    if !obj.active {
                        continue;
                    }
                    
                    let obj_x = center_x + obj.transform.position[0] * 20.0 * editor_state.scene_zoom;
                    let obj_y = center_y - obj.transform.position[1] * 20.0 * editor_state.scene_zoom;
                    let obj_pos = Pos2::new(obj_x, obj_y);
                    let base_size = 30.0 * editor_state.scene_zoom;
                    
                    let distance = (click_pos - obj_pos).length();
                    if distance < base_size * 0.5 {
                        clicked_object = Some(*id);
                        break;
                    }
                }
                
                if let Some(id) = clicked_object {
                    editor_state.select_object(id);
                } else {
                    editor_state.selected_object = None;
                }
            }
            
            if response.dragged() {
                let delta = response.drag_delta();
                editor_state.scene_pan[0] += delta.x;
                editor_state.scene_pan[1] += delta.y;
            }
            
            // Mouse wheel for zoom
            ctx.input(|i| {
                let scroll_delta = i.raw_scroll_delta.y;
                if scroll_delta != 0.0 {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    editor_state.scene_zoom = (editor_state.scene_zoom * zoom_factor).clamp(0.1, 5.0);
                }
            });
        });
}

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Unity-Style Game Editor - wgpu + egui")
        .with_inner_size(winit::dpi::LogicalSize::new(1400, 900))
        .build(&event_loop)
        .unwrap());
        
    let mut editor = pollster::block_on(UnityEditor::new(window.clone()));
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !editor.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            editor.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
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
                    Err(wgpu::SurfaceError::Lost) => editor.resize(editor.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    }).unwrap();
}