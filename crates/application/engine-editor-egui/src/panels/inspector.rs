// Inspector panel - shows and edits components of selected entities

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Light, Visibility, MeshFilter, MeshRenderer};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_renderer_3d::Camera;

pub struct InspectorPanel {
    show_add_component_dialog: bool,
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {
            show_add_component_dialog: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>) {
        ui.label("Entity Inspector");
        ui.separator();
        
        if let Some(selected_entity) = selected_entity {
            ui.horizontal(|ui| {
                ui.label(format!("Entity ID: {}", selected_entity.id()));
                
                // Copy entity info button
                if ui.button("Copy Info").on_hover_text("Copy entity information to clipboard").clicked() {
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
                    if world.get_component::<MeshFilter>(selected_entity).is_some() {
                        info.push_str(&format!("\nMesh: MeshFilter + MeshRenderer\n"));
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
                
                // MeshFilter Component
                if let Some(mesh_filter) = world.get_component::<MeshFilter>(selected_entity) {
                    ui.collapsing("🔧 Mesh Filter", |ui| {
                        ui.label(format!("Mesh Handle ID: {}", mesh_filter.mesh.id().get()));
                        ui.label("Contains reference to mesh data");
                    });
                }
                
                // MeshRenderer Component
                if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(selected_entity).cloned() {
                    ui.collapsing("🎨 Mesh Renderer", |ui| {
                        let mut enabled = mesh_renderer.enabled;
                        let mut cast_shadows = mesh_renderer.cast_shadows;
                        let mut receive_shadows = mesh_renderer.receive_shadows;
                        let mut layer_mask = mesh_renderer.layer_mask;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("mesh_renderer_grid").show(ui, |ui| {
                            ui.label("Enabled:");
                            changed |= ui.checkbox(&mut enabled, "").changed();
                            ui.end_row();
                            
                            ui.label("Cast Shadows:");
                            changed |= ui.checkbox(&mut cast_shadows, "").changed();
                            ui.end_row();
                            
                            ui.label("Receive Shadows:");
                            changed |= ui.checkbox(&mut receive_shadows, "").changed();
                            ui.end_row();
                            
                            ui.label("Layer Mask:");
                            changed |= ui.add(egui::DragValue::new(&mut layer_mask).hexadecimal(8, false, true)).changed();
                            ui.end_row();
                            
                            ui.label("Materials:");
                            ui.label(format!("{} material(s)", mesh_renderer.materials.len()));
                            ui.end_row();
                        });
                        
                        if changed {
                            if let Some(renderer_mut) = world.get_component_mut::<MeshRenderer>(selected_entity) {
                                renderer_mut.enabled = enabled;
                                renderer_mut.cast_shadows = cast_shadows;
                                renderer_mut.receive_shadows = receive_shadows;
                                renderer_mut.layer_mask = layer_mask;
                            }
                        }
                    });
                }
                
                // Material Component
                if let Some(material) = world.get_component::<Material>(selected_entity).cloned() {
                    ui.collapsing("🎨 Material", |ui| {
                        let mut color = material.color;
                        let mut metallic = material.metallic;
                        let mut roughness = material.roughness;
                        let mut emissive = material.emissive;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("material_grid").show(ui, |ui| {
                            ui.label("Color:");
                            ui.end_row();
                            
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut color[0]).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut color[1]).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut color[2]).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut color[3]).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.end_row();
                            
                            ui.label("Metallic:");
                            changed |= ui.add(egui::DragValue::new(&mut metallic).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.end_row();
                            
                            ui.label("Roughness:");
                            changed |= ui.add(egui::DragValue::new(&mut roughness).speed(0.01).clamp_range(0.0..=1.0)).changed();
                            ui.end_row();
                            
                            ui.label("Emissive:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut emissive[0]).speed(0.01).clamp_range(0.0..=10.0)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut emissive[1]).speed(0.01).clamp_range(0.0..=10.0)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut emissive[2]).speed(0.01).clamp_range(0.0..=10.0)).changed();
                            ui.end_row();
                        });
                        
                        if changed {
                            if let Some(material_mut) = world.get_component_mut::<Material>(selected_entity) {
                                material_mut.color = color;
                                material_mut.metallic = metallic;
                                material_mut.roughness = roughness;
                                material_mut.emissive = emissive;
                            }
                        }
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
                    if world.get_component::<MeshFilter>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("MeshFilter");
                    }
                    if world.get_component::<MeshRenderer>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("MeshRenderer");
                    }
                    if world.get_component::<Material>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Material");
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
                    self.show_add_component_dialog(ui, world, selected_entity);
                }
            });
        } else {
            ui.label("No entity selected");
            ui.label("Select an entity in the Hierarchy to view its components.");
        }
    }
    
    fn show_add_component_dialog(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
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
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                        }
                    }
                }
                
                // Visibility Component
                if ui.button("👁️ Visibility Component").clicked() {
                    match world.add_component(entity, Visibility::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                        }
                    }
                }
                
                // Camera Component
                if ui.button("📷 Camera Component").clicked() {
                    match world.add_component(entity, Camera::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                        }
                    }
                }
                
                // Light Component
                if ui.button("💡 Light Component").clicked() {
                    match world.add_component(entity, Light::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                        }
                    }
                }
                
                ui.separator();
                ui.label("Camera Components:");
                
                // Basic 3D Camera Component
                if ui.button("📷 3D Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if world.get_component::<Camera>(entity).is_some() {
                    } else {
                        let camera = Camera::new(16.0 / 9.0).with_fov(60.0);
                        
                        match world.add_component(entity, camera) {
                            Ok(_) => {
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                            }
                        }
                    }
                }
                
                // Main Camera shortcut
                if ui.button("🎥 Main Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if world.get_component::<Camera>(entity).is_some() {
                    } else {
                        let camera = Camera::main_camera();
                        
                        match world.add_component(entity, camera) {
                            Ok(_) => {
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
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
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                        }
                    }
                }
                
                // Canvas Component
                if ui.button("🎨 Canvas Component").clicked() {
                    match world.add_component(entity, Canvas::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
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