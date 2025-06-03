// Inspector panel - shows and edits components of selected entities

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Mesh, Light, Visibility};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_camera::{Camera, Camera2D, CameraComponent, CameraType};
use crate::editor_state::ConsoleMessage;

pub struct InspectorPanel {
    show_add_component_dialog: bool,
    console_messages: Vec<ConsoleMessage>,
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {
            show_add_component_dialog: false,
            console_messages: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>) -> Vec<ConsoleMessage> {
        ui.label("Entity Inspector");
        ui.separator();
        
        let mut messages = Vec::new();
        
        if let Some(selected_entity) = selected_entity {
            ui.horizontal(|ui| {
                ui.label(format!("Entity ID: {}", selected_entity.id()));
                
                // Copy entity info button
                if ui.button("📋 Copy Info").on_hover_text("Copy entity information to clipboard").clicked() {
                    let mut info = String::new();
                    info.push_str(&format!("=== Entity Information ===\n"));
                    info.push_str(&format!("Entity ID: {}\n", selected_entity.id()));
                    
                    // Get entity name
                    if let Some(name) = world.get_component::<Name>(selected_entity) {
                        info.push_str(&format!("Name: {}\n", name.name));
                    }
                    
                    // Get transform
                    if let Some(transform) = world.get_component::<Transform>(selected_entity) {
                        info.push_str(&format!("\nTransform:\n"));
                        info.push_str(&format!("  Position: [{:.2}, {:.2}, {:.2}]\n", 
                            transform.position[0], transform.position[1], transform.position[2]));
                        info.push_str(&format!("  Rotation: [{:.2}°, {:.2}°, {:.2}°]\n", 
                            transform.rotation[0].to_degrees(), 
                            transform.rotation[1].to_degrees(), 
                            transform.rotation[2].to_degrees()));
                        info.push_str(&format!("  Scale: [{:.2}, {:.2}, {:.2}]\n", 
                            transform.scale[0], transform.scale[1], transform.scale[2]));
                    }
                    
                    // Get mesh info
                    if let Some(mesh) = world.get_component::<Mesh>(selected_entity) {
                        info.push_str(&format!("\nMesh: {:?}\n", mesh.mesh_type));
                    }
                    
                    // Get material info
                    if let Some(material) = world.get_component::<Material>(selected_entity) {
                        info.push_str(&format!("\nMaterial:\n"));
                        info.push_str(&format!("  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n", 
                            material.color[0], material.color[1], material.color[2], material.color[3]));
                        info.push_str(&format!("  Metallic: {:.2}\n", material.metallic));
                        info.push_str(&format!("  Roughness: {:.2}\n", material.roughness));
                    }
                    
                    // Copy to clipboard
                    ui.output_mut(|o| o.copied_text = info.clone());
                    messages.push(ConsoleMessage::info("📋 Entity information copied to clipboard"));
                }
            });
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Get Transform component from ECS v2 (clone it to avoid borrowing issues)
                if let Some(transform) = world.get_component::<Transform>(selected_entity).cloned() {
                    ui.collapsing("📐 Transform", |ui| {
                        // Clone the transform to make it mutable for editing
                        let mut pos = transform.position;
                        let mut rot = transform.rotation;
                        let mut scale = transform.scale;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("transform_grid").show(ui, |ui| {
                            // Position
                            ui.label("Position:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[0]).speed(0.1)).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[1]).speed(0.1)).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[2]).speed(0.1)).changed();
                            ui.end_row();
                            
                            // Rotation
                            ui.label("Rotation:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[0]).speed(1.0).suffix("°")).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[1]).speed(1.0).suffix("°")).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[2]).speed(1.0).suffix("°")).changed();
                            ui.end_row();
                            
                            // Scale
                            ui.label("Scale:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[0]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[1]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[2]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.end_row();
                        });
                        
                        // Update the ECS component if values changed
                        if changed {
                            if let Some(transform_mut) = world.get_component_mut::<Transform>(selected_entity) {
                                transform_mut.position = pos;
                                transform_mut.rotation = rot;
                                transform_mut.scale = scale;
                            }
                        }
                    });
                } else {
                    ui.label("❌ No Transform component");
                }
                
                // Name Component
                if let Some(name) = world.get_component::<Name>(selected_entity) {
                    ui.collapsing("📝 Name", |ui| {
                        ui.label(format!("Name: {}", name.name));
                    });
                }
                
                // Visibility Component
                if let Some(visibility) = world.get_component::<Visibility>(selected_entity) {
                    ui.collapsing("👁️ Visibility", |ui| {
                        ui.label(format!("Visible: {}", visibility.visible));
                    });
                }
                
                // Camera Component
                if let Some(camera) = world.get_component::<Camera>(selected_entity) {
                    ui.collapsing("📷 Camera", |ui| {
                        ui.label(format!("FOV: {:.1}°", camera.fov));
                        ui.label(format!("Near: {:.2}", camera.near));
                        ui.label(format!("Far: {:.0}", camera.far));
                        ui.label(format!("Main Camera: {}", camera.is_main));
                    });
                }
                
                // Light Component
                if let Some(light) = world.get_component::<Light>(selected_entity) {
                    ui.collapsing("💡 Light", |ui| {
                        ui.label(format!("Type: {:?}", light.light_type));
                        ui.label(format!("Color: [{:.2}, {:.2}, {:.2}]", 
                                 light.color[0], light.color[1], light.color[2]));
                        ui.label(format!("Intensity: {:.2}", light.intensity));
                    });
                }
                
                // Sprite Renderer Component
                if let Some(sprite_renderer) = world.get_component::<SpriteRenderer>(selected_entity).cloned() {
                    ui.collapsing("🖼️ Sprite Renderer", |ui| {
                        let mut enabled = sprite_renderer.enabled;
                        let mut layer = sprite_renderer.layer;
                        let mut color = sprite_renderer.sprite.color;
                        let mut flip_x = sprite_renderer.sprite.flip_x;
                        let mut flip_y = sprite_renderer.sprite.flip_y;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("sprite_renderer_grid").show(ui, |ui| {
                            // Enabled checkbox
                            ui.label("Enabled:");
                            changed |= ui.checkbox(&mut enabled, "").changed();
                            ui.end_row();
                            
                            // Layer
                            ui.label("Layer:");
                            changed |= ui.add(egui::DragValue::new(&mut layer).range(-32768..=32767)).changed();
                            ui.end_row();
                            
                            // Color tint
                            ui.label("Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                            
                            // Flip options
                            ui.label("Flip X:");
                            changed |= ui.checkbox(&mut flip_x, "").changed();
                            ui.end_row();
                            ui.label("Flip Y:");
                            changed |= ui.checkbox(&mut flip_y, "").changed();
                            ui.end_row();
                        });
                        
                        // Show texture handle if present
                        if let Some(handle) = sprite_renderer.sprite.texture_handle {
                            ui.label(format!("Texture Handle: {}", handle));
                        } else {
                            ui.label("No texture assigned");
                        }
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(sprite_mut) = world.get_component_mut::<SpriteRenderer>(selected_entity) {
                                sprite_mut.enabled = enabled;
                                sprite_mut.layer = layer;
                                sprite_mut.sprite.color = color;
                                sprite_mut.sprite.flip_x = flip_x;
                                sprite_mut.sprite.flip_y = flip_y;
                            }
                        }
                    });
                }
                
                // Canvas Component
                if let Some(canvas) = world.get_component::<Canvas>(selected_entity) {
                    ui.collapsing("🎨 Canvas", |ui| {
                        ui.label(format!("Render Mode: {:?}", canvas.render_mode));
                        ui.label(format!("Sorting Layer: {}", canvas.sorting_layer));
                        ui.label(format!("Order in Layer: {}", canvas.order_in_layer));
                        ui.label(format!("Pixel Perfect: {}", canvas.pixel_perfect));
                    });
                }
                
                // Camera2D Component
                if let Some(camera_2d) = world.get_component::<Camera2D>(selected_entity).cloned() {
                    ui.collapsing("📷 Camera 2D", |ui| {
                        let mut size = camera_2d.size;
                        let mut aspect_ratio = camera_2d.aspect_ratio;
                        let mut near = camera_2d.near;
                        let mut far = camera_2d.far;
                        let mut is_main = camera_2d.is_main;
                        let mut bg_color = camera_2d.background_color;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("camera_2d_grid").show(ui, |ui| {
                            // Orthographic size
                            ui.label("Size:");
                            changed |= ui.add(egui::DragValue::new(&mut size).speed(0.1).range(0.1..=100.0)).changed();
                            ui.end_row();
                            
                            // Aspect ratio
                            ui.label("Aspect Ratio:");
                            changed |= ui.add(egui::DragValue::new(&mut aspect_ratio).speed(0.01).range(0.0..=10.0)).changed();
                            ui.end_row();
                            
                            // Near/Far clipping
                            ui.label("Near:");
                            changed |= ui.add(egui::DragValue::new(&mut near).speed(0.1).range(-100.0..=100.0)).changed();
                            ui.end_row();
                            ui.label("Far:");
                            changed |= ui.add(egui::DragValue::new(&mut far).speed(0.1).range(-100.0..=100.0)).changed();
                            ui.end_row();
                            
                            // Main camera
                            ui.label("Main Camera:");
                            changed |= ui.checkbox(&mut is_main, "").changed();
                            ui.end_row();
                            
                            // Background color
                            ui.label("Background Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                        });
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(camera_mut) = world.get_component_mut::<Camera2D>(selected_entity) {
                                camera_mut.size = size;
                                camera_mut.aspect_ratio = aspect_ratio;
                                camera_mut.near = near;
                                camera_mut.far = far;
                                camera_mut.is_main = is_main;
                                camera_mut.background_color = bg_color;
                            }
                        }
                    });
                }
                
                // Camera Component (Advanced)
                if let Some(camera_comp) = world.get_component::<CameraComponent>(selected_entity).cloned() {
                    ui.collapsing("📷 Camera (Advanced)", |ui| {
                        let mut is_main = camera_comp.is_main;
                        let mut camera_type = camera_comp.camera.camera_type().clone();
                        let mut clear_color = camera_comp.camera.clear_color();
                        let mut render_order = camera_comp.camera.render_order();
                        let mut enabled = camera_comp.camera.enabled();
                        
                        let mut changed = false;
                        
                        egui::Grid::new("camera_comp_grid").show(ui, |ui| {
                            // Main camera checkbox
                            ui.label("Main Camera:");
                            changed |= ui.checkbox(&mut is_main, "").changed();
                            ui.end_row();
                            
                            // Enabled checkbox
                            ui.label("Enabled:");
                            changed |= ui.checkbox(&mut enabled, "").changed();
                            ui.end_row();
                            
                            // Render order
                            ui.label("Render Order:");
                            changed |= ui.add(egui::DragValue::new(&mut render_order).range(-100..=100)).changed();
                            ui.end_row();
                            
                            // Camera type
                            ui.label("Camera Type:");
                            ui.end_row();
                            
                            // Camera type specific settings
                            match &mut camera_type {
                                CameraType::Orthographic2D { size, near, far } => {
                                    ui.label("Type: Orthographic 2D");
                                    ui.end_row();
                                    ui.label("Size:");
                                    changed |= ui.add(egui::DragValue::new(size).speed(0.1).range(0.1..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Near:");
                                    changed |= ui.add(egui::DragValue::new(near).speed(0.1).range(-100.0..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Far:");
                                    changed |= ui.add(egui::DragValue::new(far).speed(0.1).range(-100.0..=100.0)).changed();
                                    ui.end_row();
                                }
                                CameraType::Perspective3D { fov_degrees, near, far } => {
                                    ui.label("Type: Perspective 3D");
                                    ui.end_row();
                                    ui.label("FOV (degrees):");
                                    changed |= ui.add(egui::DragValue::new(fov_degrees).speed(1.0).range(1.0..=179.0)).changed();
                                    ui.end_row();
                                    ui.label("Near:");
                                    changed |= ui.add(egui::DragValue::new(near).speed(0.01).range(0.01..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Far:");
                                    changed |= ui.add(egui::DragValue::new(far).speed(1.0).range(1.0..=10000.0)).changed();
                                    ui.end_row();
                                }
                                CameraType::Custom { .. } => {
                                    ui.label("Type: Custom Matrix");
                                    ui.end_row();
                                    ui.label("(Custom matrices not editable)");
                                    ui.end_row();
                                }
                            }
                            
                            // Clear color
                            ui.label("Clear Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                        });
                        
                        // Show viewport info (read-only)
                        ui.separator();
                        ui.label("Viewport Information:");
                        let viewport = camera_comp.camera.viewport();
                        ui.label(format!("Size: {}x{}", viewport.width, viewport.height));
                        ui.label(format!("Aspect Ratio: {:.2}", viewport.aspect_ratio()));
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(camera_mut) = world.get_component_mut::<CameraComponent>(selected_entity) {
                                camera_mut.is_main = is_main;
                                camera_mut.camera.set_camera_type(camera_type);
                                camera_mut.camera.set_clear_color(clear_color);
                                camera_mut.camera.set_render_order(render_order);
                                camera_mut.camera.set_enabled(enabled);
                                
                                // Update projection matrix if camera type changed
                                if let Err(e) = camera_mut.camera.update_projection_matrix() {
                                    messages.push(ConsoleMessage::info(&format!("⚠️ Camera update error: {}", e)));
                                }
                            }
                        }
                    });
                }
                
                // ECS v2 Entity Info
                ui.separator();
                ui.collapsing("🔧 Entity Debug", |ui| {
                    ui.label(format!("Entity ID: {}", selected_entity.id()));
                    ui.label(format!("ID: {}", selected_entity.id()));
                    
                    // Count components
                    let mut component_count = 0;
                    let mut component_list = Vec::new();
                    
                    if world.get_component::<Transform>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Transform");
                    }
                    if world.get_component::<Name>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Name");
                    }
                    if world.get_component::<Visibility>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Visibility");
                    }
                    if world.get_component::<Camera>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Camera");
                    }
                    if world.get_component::<Light>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Light");
                    }
                    if world.get_component::<SpriteRenderer>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("SpriteRenderer");
                    }
                    if world.get_component::<Canvas>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Canvas");
                    }
                    if world.get_component::<Camera2D>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Camera2D");
                    }
                    if world.get_component::<CameraComponent>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("CameraComponent");
                    }
                    
                    ui.label(format!("Component Count: {}", component_count));
                    ui.label(format!("Components: {}", component_list.join(", ")));
                });
                
                ui.separator();
                if ui.button("➕ Add Component").clicked() {
                    self.show_add_component_dialog = true;
                }
                
                // Add Component Dialog
                if self.show_add_component_dialog {
                    self.show_add_component_dialog(ui, world, selected_entity, &mut messages);
                }
            });
        } else {
            ui.label("No entity selected");
            ui.label("Select an entity in the Hierarchy to view its components.");
        }
        
        // Return any console messages
        let mut final_messages = self.console_messages.drain(..).collect::<Vec<_>>();
        final_messages.append(&mut messages);
        final_messages
    }
    
    fn show_add_component_dialog(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity, messages: &mut Vec<ConsoleMessage>) {
        let mut dialog_open = self.show_add_component_dialog;
        egui::Window::new("Add Component")
            .open(&mut dialog_open)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Choose a component to add:");
                ui.separator();
                
                // Name Component
                if ui.button("📝 Name Component").clicked() {
                    match world.add_component(entity, Name::new("New Object")) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Name component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Name: {}", e)));
                        }
                    }
                }
                
                // Visibility Component
                if ui.button("👁️ Visibility Component").clicked() {
                    match world.add_component(entity, Visibility::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Visibility component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Visibility: {}", e)));
                        }
                    }
                }
                
                // Camera Component
                if ui.button("📷 Camera Component").clicked() {
                    match world.add_component(entity, Camera::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Camera component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Camera: {}", e)));
                        }
                    }
                }
                
                // Light Component
                if ui.button("💡 Light Component").clicked() {
                    match world.add_component(entity, Light::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Light component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Light: {}", e)));
                        }
                    }
                }
                
                ui.separator();
                ui.label("Camera Components:");
                
                // Basic 3D Camera Component
                if ui.button("📷 3D Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if world.get_component::<Camera>(entity).is_some() {
                        messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let camera = Camera::new().with_fov(60.0);
                        
                        match world.add_component(entity, camera) {
                            Ok(_) => {
                                messages.push(ConsoleMessage::info("✅ Added 3D Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                messages.push(ConsoleMessage::info(&format!("❌ Failed to add 3D Camera: {}", e)));
                            }
                        }
                    }
                }
                
                // Main Camera shortcut
                if ui.button("🎥 Main Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if world.get_component::<Camera>(entity).is_some() {
                        messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let camera = Camera::main_camera();
                        
                        match world.add_component(entity, camera) {
                            Ok(_) => {
                                messages.push(ConsoleMessage::info("✅ Added Main Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                messages.push(ConsoleMessage::info(&format!("❌ Failed to add Main Camera: {}", e)));
                            }
                        }
                    }
                }
                
                ui.separator();
                ui.label("2D Components:");
                
                // Sprite Renderer Component
                if ui.button("🖼️ Sprite Renderer Component").clicked() {
                    match world.add_component(entity, SpriteRenderer::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Sprite Renderer component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Sprite Renderer: {}", e)));
                        }
                    }
                }
                
                // Canvas Component
                if ui.button("🎨 Canvas Component").clicked() {
                    match world.add_component(entity, Canvas::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Canvas component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Canvas: {}", e)));
                        }
                    }
                }
                
                // Camera2D Component
                if ui.button("📷 Camera 2D Component").clicked() {
                    match world.add_component(entity, Camera2D::default()) {
                        Ok(_) => {
                            messages.push(ConsoleMessage::info("✅ Added Camera 2D component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            messages.push(ConsoleMessage::info(&format!("❌ Failed to add Camera 2D: {}", e)));
                        }
                    }
                }
                
                ui.separator();
                if ui.button("Cancel").clicked() {
                    self.show_add_component_dialog = false;
                }
            });
        self.show_add_component_dialog = dialog_open;
    }
}