// Inspector panel - shows and edits components of selected entities

use eframe::egui;
use engine_components_2d::SpriteRenderer;
use engine_components_3d::{Light, Material, MeshFilter, MeshRenderer, Transform, Visibility};
use engine_components_ui::{Canvas, Name};
use engine_ecs_core::{Entity, World};
use engine_renderer_3d::Camera;
use engine_scripting::components::{LuaScript, TypeScriptScript};
use engine_scripting::examples::typescript_examples::{
    get_all_typescript_examples, get_typescript_examples_by_category, 
    get_typescript_examples_by_difficulty, get_beginner_typescript_examples,
    get_typescript_example_by_name
};

#[cfg(test)]
mod typescript_tests;

#[cfg(test)]
mod typescript_component_tests;

#[cfg(test)]
mod typescript_example_integration_tests;

#[derive(Default)]
pub struct InspectorPanel {
    show_add_component_dialog: bool,
    show_script_selection_dialog: bool,
    pub show_script_creation_dialog: bool,
    pub script_creation_name: String,
    pub script_creation_template: ScriptTemplate,
    pub script_creation_language: ScriptLanguage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptTemplate {
    Entity,
    Behavior,
    System,
}

impl Default for ScriptTemplate {
    fn default() -> Self {
        Self::Entity
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptLanguage {
    TypeScript,
    Lua, // Still supported but not prominent in UI
}

/// Item for script selection dialog
#[derive(Debug, Clone)]
pub struct ScriptSelectionItem {
    pub name: String,
    pub path: String,
    pub item_type: String, // "example", "template", "project"
    pub description: String,
    pub category: String,
}

impl Default for ScriptLanguage {
    fn default() -> Self {
        Self::TypeScript // Default to TypeScript for new scripts
    }
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>) {
        ui.label("Entity Inspector");
        ui.separator();

        if let Some(selected_entity) = selected_entity {
            ui.horizontal(|ui| {
                ui.label(format!("Entity ID: {}", selected_entity.id()));

                // Copy entity info button
                if ui
                    .button("Copy Info")
                    .on_hover_text("Copy entity information to clipboard")
                    .clicked()
                {
                    let mut info = String::new();
                    info.push_str("=== Entity Information ===\n");
                    info.push_str(&format!("Entity ID: {}\n", selected_entity.id()));

                    // Get entity name
                    if let Some(name) = world.get_component::<Name>(selected_entity) {
                        info.push_str(&format!("Name: {}\n", name.name));
                    }

                    // Get transform with bounds checking
                    if let Some(transform) = world.get_component::<Transform>(selected_entity) {
                        info.push_str("\nTransform:\n");
                        
                        // Safe position access with bounds checking
                        if transform.position.len() >= 3 {
                            info.push_str(&format!(
                                "  Position: [{:.2}, {:.2}, {:.2}]\n",
                                transform.position[0], transform.position[1], transform.position[2]
                            ));
                        } else {
                            info.push_str("  Position: [Invalid array length]\n");
                        }
                        
                        // Safe rotation access with bounds checking
                        if transform.rotation.len() >= 3 {
                            info.push_str(&format!(
                                "  Rotation: [{:.2}°, {:.2}°, {:.2}°]\n",
                                transform.rotation[0].to_degrees(),
                                transform.rotation[1].to_degrees(),
                                transform.rotation[2].to_degrees()
                            ));
                        } else {
                            info.push_str("  Rotation: [Invalid array length]\n");
                        }
                        
                        // Safe scale access with bounds checking
                        if transform.scale.len() >= 3 {
                            info.push_str(&format!(
                                "  Scale: [{:.2}, {:.2}, {:.2}]\n",
                                transform.scale[0], transform.scale[1], transform.scale[2]
                            ));
                        } else {
                            info.push_str("  Scale: [Invalid array length]\n");
                        }
                    }

                    // Get mesh info
                    if world.get_component::<MeshFilter>(selected_entity).is_some() {
                        info.push_str("\nMesh: MeshFilter + MeshRenderer\n");
                    }

                    // Get material info
                    if let Some(material) = world.get_component::<Material>(selected_entity) {
                        info.push_str("\nMaterial:\n");
                        info.push_str(&format!(
                            "  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n",
                            material.color[0],
                            material.color[1],
                            material.color[2],
                            material.color[3]
                        ));
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
                if let Some(transform) = world.get_component::<Transform>(selected_entity).cloned()
                {
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
                            changed |= ui
                                .add(egui::DragValue::new(&mut pos[0]).speed(0.1))
                                .changed();
                            ui.label("Y:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut pos[1]).speed(0.1))
                                .changed();
                            ui.label("Z:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut pos[2]).speed(0.1))
                                .changed();
                            ui.end_row();

                            // Rotation
                            ui.label("Rotation:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut rot[0]).speed(1.0).suffix("°"))
                                .changed();
                            ui.label("Y:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut rot[1]).speed(1.0).suffix("°"))
                                .changed();
                            ui.label("Z:");
                            changed |= ui
                                .add(egui::DragValue::new(&mut rot[2]).speed(1.0).suffix("°"))
                                .changed();
                            ui.end_row();

                            // Scale
                            ui.label("Scale:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut scale[0])
                                        .speed(0.01)
                                        .range(0.01..=10.0),
                                )
                                .changed();
                            ui.label("Y:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut scale[1])
                                        .speed(0.01)
                                        .range(0.01..=10.0),
                                )
                                .changed();
                            ui.label("Z:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut scale[2])
                                        .speed(0.01)
                                        .range(0.01..=10.0),
                                )
                                .changed();
                            ui.end_row();
                        });

                        // Update the ECS component if values changed
                        if changed {
                            log::info!(
                                "Transform changed for entity {:?}: pos={:?}, rot={:?}, scale={:?}",
                                selected_entity,
                                pos,
                                rot,
                                scale
                            );

                            if let Some(transform_mut) =
                                world.get_component_mut::<Transform>(selected_entity)
                            {
                                transform_mut.position = pos;
                                transform_mut.rotation = rot;
                                transform_mut.scale = scale;
                                log::info!("Updated transform in ECS");
                            } else {
                                log::error!("Failed to get mutable transform component");
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
                if let Some(mesh_renderer) = world
                    .get_component::<MeshRenderer>(selected_entity)
                    .cloned()
                {
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
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut layer_mask)
                                        .hexadecimal(8, false, true),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Materials:");
                            ui.label(format!("{} material(s)", mesh_renderer.materials.len()));
                            ui.end_row();
                        });

                        if changed {
                            if let Some(renderer_mut) =
                                world.get_component_mut::<MeshRenderer>(selected_entity)
                            {
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
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[0])
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.label("G:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[1])
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.label("B:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[2])
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.label("A:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[3])
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Metallic:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut metallic)
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Roughness:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut roughness)
                                        .speed(0.01)
                                        .range(0.0..=1.0),
                                )
                                .changed();
                            ui.end_row();

                            ui.label("Emissive:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut emissive[0])
                                        .speed(0.01)
                                        .range(0.0..=10.0),
                                )
                                .changed();
                            ui.label("G:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut emissive[1])
                                        .speed(0.01)
                                        .range(0.0..=10.0),
                                )
                                .changed();
                            ui.label("B:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut emissive[2])
                                        .speed(0.01)
                                        .range(0.0..=10.0),
                                )
                                .changed();
                            ui.end_row();
                        });

                        if changed {
                            if let Some(material_mut) =
                                world.get_component_mut::<Material>(selected_entity)
                            {
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
                        ui.label(format!(
                            "Color: [{:.2}, {:.2}, {:.2}]",
                            light.color[0], light.color[1], light.color[2]
                        ));
                        ui.label(format!("Intensity: {:.2}", light.intensity));
                    });
                }

                // Sprite Renderer Component
                if let Some(sprite_renderer) = world
                    .get_component::<SpriteRenderer>(selected_entity)
                    .cloned()
                {
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
                            changed |= ui
                                .add(egui::DragValue::new(&mut layer).range(-32768..=32767))
                                .changed();
                            ui.end_row();

                            // Color tint
                            ui.label("Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[0])
                                        .range(0.0..=1.0)
                                        .speed(0.01),
                                )
                                .changed();
                            ui.label("G:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[1])
                                        .range(0.0..=1.0)
                                        .speed(0.01),
                                )
                                .changed();
                            ui.label("B:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[2])
                                        .range(0.0..=1.0)
                                        .speed(0.01),
                                )
                                .changed();
                            ui.label("A:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut color[3])
                                        .range(0.0..=1.0)
                                        .speed(0.01),
                                )
                                .changed();
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
                            if let Some(sprite_mut) =
                                world.get_component_mut::<SpriteRenderer>(selected_entity)
                            {
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

                // LuaScript Component
                if let Some(lua_script) = world.get_component::<LuaScript>(selected_entity).cloned() {
                    ui.collapsing(&format!("📜 Lua Scripts ({})", lua_script.script_count()), |ui| {
                        let mut lua_script_modified = lua_script.clone();
                        let mut changed = false;
                        
                        ui.horizontal(|ui| {
                            ui.label("Component enabled:");
                            changed |= ui.checkbox(&mut lua_script_modified.enabled, "").changed();
                            ui.label("Execution order:");
                            changed |= ui.add(egui::DragValue::new(&mut lua_script_modified.execution_order).range(-1000..=1000)).changed();
                        });
                        
                        ui.separator();
                        
                        // Display all scripts
                        let all_scripts = lua_script.get_all_scripts();
                        for (index, script_path) in all_scripts.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if index == 0 {
                                    ui.label("📜 Primary:");
                                } else {
                                    ui.label(&format!("📜 Script {}:", index + 1));
                                }
                                ui.label(*script_path);
                                
                                if ui.small_button("🗑️").on_hover_text("Remove this script").clicked() {
                                    if lua_script_modified.remove_script(script_path) {
                                        changed = true;
                                    } else {
                                        // Primary script removed and no other scripts - remove component
                                        let _ = world.remove_component::<LuaScript>(selected_entity);
                                        return; // Exit early since component is gone
                                    }
                                }
                            });
                        }
                        
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("➕ Add Script").clicked() {
                                self.show_script_selection_dialog = true;
                            }
                            if ui.button("🔄 Reload All").clicked() {
                                // TODO: Implement script reloading
                            }
                            if ui.button("🗑️ Remove Component").clicked() {
                                let _ = world.remove_component::<LuaScript>(selected_entity);
                            }
                        });
                        
                        if let Some(instance_id) = lua_script.instance_id {
                            ui.separator();
                            ui.label(format!("Instance ID: {}", instance_id));
                        }
                        
                        // Apply changes
                        if changed {
                            if let Some(script_mut) = world.get_component_mut::<LuaScript>(selected_entity) {
                                *script_mut = lua_script_modified;
                            }
                        }
                    });
                }

                // TypeScript Script Component
                if let Some(typescript_script) = world.get_component::<TypeScriptScript>(selected_entity).cloned() {
                    ui.collapsing("🔷 TypeScript Script", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("🔷 Script:");
                            ui.label(typescript_script.get_path());
                            
                            if ui.small_button("🗑️").on_hover_text("Remove TypeScript script").clicked() {
                                let _ = world.remove_component::<TypeScriptScript>(selected_entity);
                                return; // Exit early since component is gone
                            }
                        });
                        
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("🔄 Reload Script").clicked() {
                                // TODO: Implement TypeScript script reloading
                            }
                            if ui.button("📝 Edit Script").clicked() {
                                // TODO: Open script in editor
                            }
                        });
                        
                        ui.separator();
                        ui.label("TypeScript script information:");
                        ui.label(&format!("Language: TypeScript"));
                        ui.label(&format!("File Extension: .{}", typescript_script.get_file_extension()));
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
                    if world
                        .get_component::<SpriteRenderer>(selected_entity)
                        .is_some()
                    {
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
                    if world
                        .get_component::<MeshRenderer>(selected_entity)
                        .is_some()
                    {
                        component_count += 1;
                        component_list.push("MeshRenderer");
                    }
                    if world.get_component::<Material>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Material");
                    }
                    if world.get_component::<LuaScript>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("LuaScript");
                    }
                    if world.get_component::<TypeScriptScript>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("TypeScriptScript");
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
                
                // Script Selection Dialog
                if self.show_script_selection_dialog {
                    self.show_script_selection_dialog(ui, world, selected_entity);
                }
                
                // Script Creation Dialog
                if self.show_script_creation_dialog {
                    self.show_script_creation_dialog(ui, world, selected_entity);
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
                        Err(_e) => {}
                    }
                }

                // Visibility Component
                if ui.button("👁️ Visibility Component").clicked() {
                    match world.add_component(entity, Visibility::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(_e) => {}
                    }
                }

                // Camera Component
                if ui.button("📷 Camera Component").clicked() {
                    match world.add_component(entity, Camera::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(_e) => {}
                    }
                }

                // Light Component
                if ui.button("💡 Light Component").clicked() {
                    match world.add_component(entity, Light::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(_e) => {}
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
                            Err(_e) => {}
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
                            Err(_e) => {}
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
                        Err(_e) => {}
                    }
                }

                // Canvas Component
                if ui.button("🎨 Canvas Component").clicked() {
                    match world.add_component(entity, Canvas::default()) {
                        Ok(_) => {
                            self.show_add_component_dialog = false;
                        }
                        Err(_e) => {}
                    }
                }

                ui.separator();
                ui.label("Scripting Components:");
                
                // TypeScript Script Component (primary option)
                if ui.button("🔷 TypeScript Script Component").clicked() {
                    self.show_add_component_dialog = false;
                    self.script_creation_language = ScriptLanguage::TypeScript; // Set to TypeScript
                    self.show_script_selection_dialog = true; // Show script selection with examples
                }
                
                // Lua Script Component (secondary option)
                if ui.button("🌙 Lua Script Component").clicked() {
                    self.show_add_component_dialog = false;
                    self.script_creation_language = ScriptLanguage::Lua; // Set to Lua
                    self.show_script_selection_dialog = true;
                }

                ui.separator();
                if ui.button("Cancel").clicked() {
                    self.show_add_component_dialog = false;
                }
            });
        self.show_add_component_dialog = dialog_open;
    }

    fn show_script_selection_dialog(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut dialog_open = self.show_script_selection_dialog;
        egui::Window::new("Select Script")
            .open(&mut dialog_open)
            .resizable(true)
            .default_width(500.0)
            .show(ui.ctx(), |ui| {
                ui.label("Choose an existing script or create a new one:");
                ui.separator();

                // Create new script section
                ui.horizontal(|ui| {
                    ui.label("Create new script:");
                    if ui.button("📝 New Script").clicked() {
                        self.show_script_selection_dialog = false;
                        self.show_script_creation_dialog = true;
                    }
                });

                ui.separator();
                ui.label("Available scripts:");

                // Scripts from assets/scripts/
                ui.collapsing("Project Scripts", |ui| {
                    if let Ok(entries) = std::fs::read_dir("assets/scripts") {
                        for entry in entries.flatten() {
                            if let Some(extension) = entry.path().extension() {
                                match self.script_creation_language {
                                    ScriptLanguage::TypeScript => {
                                        if extension == "ts" {
                                            let script_name = entry.file_name().to_string_lossy().to_string();
                                            let script_path = format!("assets/scripts/{}", script_name);
                                            
                                            if ui.button(&script_name).clicked() {
                                                // Check if entity already has TypeScript component
                                                if let Some(mut ts_script) = world.get_component_mut::<TypeScriptScript>(entity) {
                                                    // Add to existing component
                                                    ts_script.add_script(script_path.clone());
                                                    self.show_script_selection_dialog = false;
                                                } else {
                                                    // Create new component
                                                    match self.attach_script_to_entity(world, entity, &script_path) {
                                                        Ok(_) => {
                                                            self.show_script_selection_dialog = false;
                                                        }
                                                        Err(_e) => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    ScriptLanguage::Lua => {
                                        if extension == "lua" {
                                            let script_name = entry.file_name().to_string_lossy().to_string();
                                            let script_path = format!("assets/scripts/{}", script_name);
                                            
                                            if ui.button(&script_name).clicked() {
                                                // Check if entity already has LuaScript component
                                                if let Some(mut lua_script) = world.get_component_mut::<LuaScript>(entity) {
                                                    // Add to existing component
                                                    lua_script.add_script(script_path.clone());
                                                    self.show_script_selection_dialog = false;
                                                } else {
                                                    // Create new component
                                                    match world.add_component(entity, LuaScript::new(script_path)) {
                                                        Ok(_) => {
                                                            self.show_script_selection_dialog = false;
                                                        }
                                                        Err(_e) => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        ui.label("No project scripts found");
                    }
                });

                // Example scripts
                ui.collapsing("Example Scripts", |ui| {
                    match self.script_creation_language {
                        ScriptLanguage::TypeScript => {
                            // Show TypeScript examples from our examples system
                            let examples = self.get_typescript_example_scripts();
                            
                            ui.collapsing("Basic Examples", |ui| {
                                let metadata = self.get_typescript_example_metadata();
                                for (name, description, difficulty, _category) in metadata {
                                    if difficulty == "beginner" {
                                        let button_text = format!("🔷 {} - {}", name, description);
                                        if ui.button(&button_text).clicked() {
                                            // Attach TypeScript example
                                            match self.attach_typescript_example(world, entity, &name) {
                                                Ok(_) => {
                                                    self.show_script_selection_dialog = false;
                                                }
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                }
                            });
                            
                            ui.collapsing("Advanced Examples", |ui| {
                                let metadata = self.get_typescript_example_metadata();
                                for (name, description, difficulty, _category) in metadata {
                                    if difficulty == "intermediate" || difficulty == "advanced" {
                                        let button_text = format!("🔶 {} - {}", name, description);
                                        if ui.button(&button_text).clicked() {
                                            // Attach TypeScript example
                                            match self.attach_typescript_example(world, entity, &name) {
                                                Ok(_) => {
                                                    self.show_script_selection_dialog = false;
                                                }
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        ScriptLanguage::Lua => {
                            // Show Lua examples (original behavior)
                            let example_scripts = [
                                ("Entity Template", "crates/implementation/engine-scripting/lua/examples/entity_template.lua"),
                                ("Player Controller", "crates/implementation/engine-scripting/lua/examples/player_controller.lua"),
                                ("Enemy AI", "crates/implementation/engine-scripting/lua/examples/enemy_ai.lua"),
                                ("Game Manager", "crates/implementation/engine-scripting/lua/examples/game_manager.lua"),
                                ("Basic Template", "crates/implementation/engine-scripting/lua/examples/basic_template.lua"),
                            ];

                            for (name, path) in example_scripts {
                                if ui.button(name).clicked() {
                                    // Check if entity already has LuaScript component
                                    if let Some(mut lua_script) = world.get_component_mut::<LuaScript>(entity) {
                                        // Add to existing component
                                        lua_script.add_script(path.to_string());
                                        self.show_script_selection_dialog = false;
                                    } else {
                                        // Create new component
                                        match world.add_component(entity, LuaScript::new(path.to_string())) {
                                            Ok(_) => {
                                                self.show_script_selection_dialog = false;
                                            }
                                            Err(_e) => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                ui.separator();
                if ui.button("Cancel").clicked() {
                    self.show_script_selection_dialog = false;
                }
            });
        self.show_script_selection_dialog = dialog_open;
    }

    fn show_script_creation_dialog(&mut self, ui: &mut egui::Ui, world: &mut World, entity: Entity) {
        let mut dialog_open = self.show_script_creation_dialog;
        let language_name = match self.script_creation_language {
            ScriptLanguage::TypeScript => "TypeScript",
            ScriptLanguage::Lua => "Lua",
        };
        
        egui::Window::new("Create New Script")
            .open(&mut dialog_open)
            .resizable(false)
            .default_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.label(&format!("Create a new {} script:", language_name));
                ui.separator();

                // Script name input
                ui.horizontal(|ui| {
                    ui.label("Script name:");
                    ui.text_edit_singleline(&mut self.script_creation_name);
                });

                // Template selection
                ui.horizontal(|ui| {
                    ui.label("Template:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.script_creation_template))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.script_creation_template, ScriptTemplate::Entity, "Entity");
                            ui.selectable_value(&mut self.script_creation_template, ScriptTemplate::Behavior, "Behavior");
                            ui.selectable_value(&mut self.script_creation_template, ScriptTemplate::System, "System");
                        });
                });

                ui.separator();

                // Template description
                match self.script_creation_template {
                    ScriptTemplate::Entity => {
                        ui.label("Entity Script: Attached to a specific entity with lifecycle methods (start, update, etc.)");
                    }
                    ScriptTemplate::Behavior => {
                        ui.label("Behavior Script: Reusable behavior that can be attached to multiple entities");
                    }
                    ScriptTemplate::System => {
                        ui.label("System Script: Global system that operates on multiple entities");
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Create & Attach").clicked() {
                        if !self.script_creation_name.is_empty() {
                            let (_file_extension, script_name) = match self.script_creation_language {
                                ScriptLanguage::TypeScript => {
                                    let ext = ".ts";
                                    let name = if !self.script_creation_name.ends_with(ext) {
                                        format!("{}{}", self.script_creation_name, ext)
                                    } else {
                                        self.script_creation_name.clone()
                                    };
                                    (ext, name)
                                }
                                ScriptLanguage::Lua => {
                                    let ext = ".lua";
                                    let name = if !self.script_creation_name.ends_with(ext) {
                                        format!("{}{}", self.script_creation_name, ext)
                                    } else {
                                        self.script_creation_name.clone()
                                    };
                                    (ext, name)
                                }
                            };

                            let script_path = format!("assets/scripts/{}", script_name);
                            
                            // Create the script file
                            if let Err(_) = std::fs::create_dir_all("assets/scripts") {
                                // Directory creation failed, but continue anyway
                            }

                            let template_content = match (self.script_creation_language.clone(), self.script_creation_template.clone()) {
                                (ScriptLanguage::TypeScript, ScriptTemplate::Entity) => {
                                    let class_name = script_name.replace(".ts", "").replace("-", "_")
                                        .chars().enumerate().map(|(i, c)| {
                                            if i == 0 { c.to_uppercase().collect::<String>() } 
                                            else if c == '_' { "".to_string() }
                                            else { c.to_string() }
                                        }).collect::<String>();
                                    
                                    format!(r#"export class {class_name} {{
    private position: Vector3 = new Vector3(0, 0, 0);
    
    init(): void {{
        console.log("{class_name} initialized");
        
        // Get current entity and its transform
        const entity = Engine.world.getCurrentEntity();
        const transform = entity.getComponent<Transform>();
        
        // Initialize your entity here
        if (transform) {{
            this.position = transform.position;
        }}
    }}
    
    update(deltaTime: number): void {{
        // Update logic here
        const entity = Engine.world.getCurrentEntity();
        const transform = entity.getComponent<Transform>();
        
        // Example: simple rotation
        if (transform) {{
            transform.rotation.y += deltaTime;
            this.position = transform.position;
        }}
    }}
    
    destroy(): void {{
        console.log("{class_name} destroyed");
        
        // Cleanup logic here
    }}
}}
"#, class_name = class_name)
                                }
                                (ScriptLanguage::Lua, ScriptTemplate::Entity) => {
                                    format!(r#"-- Entity script: {}
-- This script is attached to a specific entity

local {module_name} = {{}}

function {module_name}:init()
    -- Called when the entity is created
    print("[{module_name}] Entity started!")
end

function {module_name}:update(dt)
    -- Called every frame
    -- dt is the delta time in seconds
end

function {module_name}:destroy()
    -- Called when the entity is destroyed
    print("[{module_name}] Entity destroyed!")
end

return {module_name}
"#, script_name=script_name, module_name=script_name.replace(".lua", "").replace("-", "_"))
                                }
                                (ScriptLanguage::TypeScript, ScriptTemplate::Behavior) => {
                                    let class_name = script_name.replace(".ts", "").replace("-", "_")
                                        .chars().enumerate().map(|(i, c)| {
                                            if i == 0 { c.to_uppercase().collect::<String>() } 
                                            else if c == '_' { "".to_string() }
                                            else { c.to_string() }
                                        }).collect::<String>();
                                    
                                    format!(r#"interface Behavior {{
    start(entity: Entity): void;
    update(entity: Entity, deltaTime: number): void;
    destroy(entity: Entity): void;
}}

export class {class_name} implements Behavior {{
    start(entity: Entity): void {{
        console.log("{class_name} behavior started on entity: " + entity.id);
        
        // Initialization logic here
    }}
    
    update(entity: Entity, deltaTime: number): void {{
        // Behavior update logic here
        const transform = entity.getComponent<Transform>();
        if (transform) {{
            // Example behavior logic
        }}
    }}
    
    destroy(entity: Entity): void {{
        console.log("{class_name} behavior removed from entity: " + entity.id);
        
        // Cleanup logic here
    }}
}}
"#, class_name = class_name)
                                }
                                (ScriptLanguage::Lua, ScriptTemplate::Behavior) => {
                                    format!(r#"-- Behavior script: {}
-- Reusable behavior that can be attached to entities

local behavior = {{}}

function behavior.start(entity)
    -- Called when attached to an entity
    print("Behavior started on entity: " .. tostring(entity))
end

function behavior.update(entity, dt)
    -- Called every frame for each entity with this behavior
end

function behavior.on_destroy(entity)
    -- Called when detached from an entity
    print("Behavior removed from entity: " .. tostring(entity))
end

return behavior
"#, script_name)
                                }
                                (ScriptLanguage::TypeScript, ScriptTemplate::System) => {
                                    let class_name = script_name.replace(".ts", "").replace("-", "_")
                                        .chars().enumerate().map(|(i, c)| {
                                            if i == 0 { c.to_uppercase().collect::<String>() } 
                                            else if c == '_' { "".to_string() }
                                            else { c.to_string() }
                                        }).collect::<String>();
                                    
                                    format!(r#"interface System {{
    initialize(): void;
    update(world: World, deltaTime: number): void;
    shutdown(): void;
}}

export class {class_name} implements System {{
    initialize(): void {{
        console.log("{class_name} system initialized");
        
        // System initialization logic here
    }}
    
    update(world: World, deltaTime: number): void {{
        // System update logic here
        // Example: query for entities with specific components
        const entities = Engine.world.query<Transform>();
        for (const entity of entities) {{
            const transform = entity.getComponent<Transform>();
            if (transform) {{
                // Process entity
            }}
        }}
    }}
    
    shutdown(): void {{
        console.log("{class_name} system shutdown");
        
        // Cleanup logic here
    }}
}}
"#, class_name = class_name)
                                }
                                (ScriptLanguage::Lua, ScriptTemplate::System) => {
                                    format!(r#"-- System script: {}
-- Global system that operates on multiple entities

local system = {{}}

function system.initialize()
    -- Called once when the system starts
    print("System initialized!")
end

function system.update(dt)
    -- Called every frame
    -- Process entities here
end

function system.shutdown()
    -- Called when the system shuts down
    print("System shutdown!")
end

return system
"#, script_name)
                                }
                            };

                            if let Ok(_) = std::fs::write(&script_path, template_content) {
                                // Script created successfully, attach it based on language
                                match self.script_creation_language {
                                    ScriptLanguage::TypeScript => {
                                        // Attach TypeScript script
                                        match self.attach_script_to_entity(world, entity, &script_path) {
                                            Ok(_) => {
                                                self.show_script_creation_dialog = false;
                                                self.script_creation_name.clear();
                                            }
                                            Err(_e) => {}
                                        }
                                    }
                                    ScriptLanguage::Lua => {
                                        // Check if entity already has LuaScript component
                                        if let Some(mut lua_script) = world.get_component_mut::<LuaScript>(entity) {
                                            // Add to existing component
                                            lua_script.add_script(script_path.clone());
                                            self.show_script_creation_dialog = false;
                                            self.script_creation_name.clear();
                                        } else {
                                            // Create new component
                                            match world.add_component(entity, LuaScript::new(script_path)) {
                                                Ok(_) => {
                                                    self.show_script_creation_dialog = false;
                                                    self.script_creation_name.clear();
                                                }
                                                Err(_e) => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_script_creation_dialog = false;
                        self.script_creation_name.clear();
                    }
                });
            });
        self.show_script_creation_dialog = dialog_open;
    }

    // TypeScript integration methods
    pub fn create_script_file(&mut self, _world: &mut World, _entity: Entity) -> Result<(), String> {
        use std::fs;
        use std::path::Path;

        let script_name = &self.script_creation_name;
        if script_name.is_empty() {
            return Err("Script name cannot be empty".to_string());
        }

        // Create assets/scripts directory if it doesn't exist
        let scripts_dir = Path::new("assets/scripts");
        if !scripts_dir.exists() {
            fs::create_dir_all(scripts_dir).map_err(|e| e.to_string())?;
        }

        let file_extension = match self.script_creation_language {
            ScriptLanguage::TypeScript => "ts",
            ScriptLanguage::Lua => "lua",
        };

        let file_path = scripts_dir.join(format!("{}.{}", script_name, file_extension));
        
        let content = match (&self.script_creation_template, &self.script_creation_language) {
            (ScriptTemplate::Entity, ScriptLanguage::TypeScript) => {
                let class_name = to_pascal_case(script_name);
                format!(
                    r#"// TypeScript Entity Script: {}

export class {} {{
    private entity: Entity;

    init(): void {{
        this.entity = Engine.world.getCurrentEntity();
        console.log("Entity script {} initialized");
    }}

    update(deltaTime: number): void {{
        // Update logic here
    }}

    destroy(): void {{
        console.log("Entity script {} destroyed");
    }}
}}
"#,
                    script_name, class_name, script_name, script_name
                )
            }
            (ScriptTemplate::Behavior, ScriptLanguage::TypeScript) => {
                let class_name = to_pascal_case(script_name);
                format!(
                    r#"// TypeScript Behavior Script: {}

interface Behavior {{
    start(entity: Entity): void;
    update(entity: Entity, deltaTime: number): void;
    stop(entity: Entity): void;
}}

export class {} implements Behavior {{
    start(entity: Entity): void {{
        console.log("Behavior {} started");
    }}

    update(entity: Entity, deltaTime: number): void {{
        // Behavior logic here
    }}

    stop(entity: Entity): void {{
        console.log("Behavior {} stopped");
    }}
}}
"#,
                    script_name, class_name, script_name, script_name
                )
            }
            (ScriptTemplate::System, ScriptLanguage::TypeScript) => {
                let class_name = to_pascal_case(script_name);
                format!(
                    r#"// TypeScript System Script: {}

interface System {{
    initialize(): void;
    update(world: World, deltaTime: number): void;
    cleanup(): void;
}}

export class {} implements System {{
    initialize(): void {{
        console.log("System {} initialized");
    }}

    update(world: World, deltaTime: number): void {{
        // Query entities and update
        const entities = Engine.world.query(/* components */);
        for (const entity of entities) {{
            // System logic here
        }}
    }}

    cleanup(): void {{
        console.log("System {} cleaned up");
    }}
}}
"#,
                    script_name, class_name, script_name, script_name
                )
            }
            // Fallback for Lua (existing implementation)
            _ => format!("-- Script: {}\n\nprint(\"Hello from {}\")", script_name, script_name),
        };

        fs::write(&file_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn attach_script_to_entity(&mut self, world: &mut World, entity: Entity, script_path: &str) -> Result<(), String> {
        if script_path.ends_with(".ts") {
            // Check if entity already has TypeScriptScript component
            if let Some(mut typescript_script) = world.get_component_mut::<TypeScriptScript>(entity) {
                // Add to existing component
                typescript_script.add_script(script_path.to_string());
                Ok(())
            } else {
                // Create new component
                let script = TypeScriptScript::new(script_path.to_string());
                world.add_component(entity, script).map_err(|e| format!("Failed to attach TypeScript script: {:?}", e))?;
                Ok(())
            }
        } else if script_path.ends_with(".lua") {
            // Check if entity already has LuaScript component
            if let Some(mut lua_script) = world.get_component_mut::<LuaScript>(entity) {
                // Add to existing component
                lua_script.add_script(script_path.to_string());
                Ok(())
            } else {
                // Create new component
                let script = LuaScript::new(script_path.to_string());
                world.add_component(entity, script).map_err(|e| format!("Failed to attach Lua script: {:?}", e))?;
                Ok(())
            }
        } else {
            Err("Unsupported script file type".to_string())
        }
    }

    pub fn is_valid_script_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check for valid identifier characters
        name.chars().all(|c| c.is_alphanumeric() || c == '_') &&
        !name.contains(' ') &&
        !name.contains('-') &&
        !name.contains('/')
    }

    pub fn is_valid_template_language_combination(&self) -> bool {
        // All templates are valid for both languages in this implementation
        true
    }

    // TypeScript example integration methods

    /// Get all available TypeScript example scripts for the UI
    pub fn get_typescript_example_scripts(&self) -> Vec<String> {
        get_all_typescript_examples()
            .into_iter()
            .map(|example| example.name)
            .collect()
    }

    /// Get TypeScript examples by difficulty level
    pub fn get_typescript_examples_by_difficulty(&self, difficulty: &str) -> Vec<String> {
        use engine_scripting::examples::{DifficultyLevel};
        
        let difficulty_level = match difficulty {
            "beginner" => DifficultyLevel::Beginner,
            "intermediate" => DifficultyLevel::Intermediate,
            "advanced" => DifficultyLevel::Advanced,
            _ => return vec![],
        };
        
        get_typescript_examples_by_difficulty(difficulty_level)
            .into_iter()
            .map(|example| example.name)
            .collect()
    }

    /// Get TypeScript examples by category
    pub fn get_typescript_examples_by_category(&self, category: &str) -> Vec<String> {
        use engine_scripting::examples::{ExampleCategory};
        
        let category_enum = match category {
            "basic_syntax" => ExampleCategory::BasicSyntax,
            "input_handling" => ExampleCategory::InputHandling,
            "physics" => ExampleCategory::Physics,
            "event_system" => ExampleCategory::EventSystem,
            "game_logic" => ExampleCategory::GameLogic,
            "debugging" => ExampleCategory::Debugging,
            "performance" => ExampleCategory::Performance,
            "integration" => ExampleCategory::Integration,
            _ => return vec![],
        };
        
        get_typescript_examples_by_category(category_enum)
            .into_iter()
            .map(|example| example.name)
            .collect()
    }

    /// Get metadata for TypeScript examples for UI display
    pub fn get_typescript_example_metadata(&self) -> Vec<(String, String, String, String)> {
        get_all_typescript_examples()
            .into_iter()
            .map(|example| {
                let difficulty = match example.difficulty_level {
                    engine_scripting::examples::DifficultyLevel::Beginner => "beginner",
                    engine_scripting::examples::DifficultyLevel::Intermediate => "intermediate",
                    engine_scripting::examples::DifficultyLevel::Advanced => "advanced",
                }.to_string();
                
                let category = match example.category {
                    engine_scripting::examples::ExampleCategory::BasicSyntax => "basic_syntax",
                    engine_scripting::examples::ExampleCategory::InputHandling => "input_handling",
                    engine_scripting::examples::ExampleCategory::Physics => "physics",
                    engine_scripting::examples::ExampleCategory::EventSystem => "event_system",
                    engine_scripting::examples::ExampleCategory::GameLogic => "game_logic",
                    engine_scripting::examples::ExampleCategory::Debugging => "debugging",
                    engine_scripting::examples::ExampleCategory::Performance => "performance",
                    engine_scripting::examples::ExampleCategory::Integration => "integration",
                }.to_string();
                
                (example.name, example.description, difficulty, category)
            })
            .collect()
    }

    /// Attach a TypeScript example script to an entity
    pub fn attach_typescript_example(&mut self, world: &mut World, entity: Entity, example_name: &str) -> Result<(), String> {
        let example = get_typescript_example_by_name(example_name)
            .ok_or_else(|| format!("TypeScript example '{}' not found", example_name))?;

        // Create a script file in assets/scripts with the example content
        let file_name = format!("{}.ts", example_name);
        let script_path = format!("assets/scripts/{}", file_name);
        
        // Ensure directory exists
        if let Err(_) = std::fs::create_dir_all("assets/scripts") {
            return Err("Failed to create scripts directory".to_string());
        }
        
        // Write example content to file
        std::fs::write(&script_path, &example.code)
            .map_err(|e| format!("Failed to write example file: {}", e))?;
        
        // Attach the script to the entity
        self.attach_script_to_entity(world, entity, &script_path)
    }

    /// Create a TypeScript example file with a custom name
    pub fn create_typescript_example_file(&mut self, world: &mut World, entity: Entity, example_name: &str, file_name: &str) -> Result<(), String> {
        if file_name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }
        
        let example = get_typescript_example_by_name(example_name)
            .ok_or_else(|| format!("TypeScript example '{}' not found", example_name))?;

        // Validate file path
        let file_path = std::path::Path::new(file_name);
        if !file_path.is_relative() && !file_path.starts_with("assets/scripts/") {
            return Err("Invalid file path".to_string());
        }
        
        let script_path = if file_path.starts_with("assets/scripts/") {
            file_name.to_string()
        } else {
            format!("assets/scripts/{}", file_name)
        };
        
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&script_path).parent() {
            if let Err(_) = std::fs::create_dir_all(parent) {
                return Err("Failed to create script directory".to_string());
            }
        }
        
        // Write example content to file
        std::fs::write(&script_path, &example.code)
            .map_err(|e| format!("Failed to write example file: {}", e))?;
        
        Ok(())
    }

    /// Search TypeScript examples by keyword
    pub fn search_typescript_examples(&self, keyword: &str) -> Vec<String> {
        let keyword_lower = keyword.to_lowercase();
        get_all_typescript_examples()
            .into_iter()
            .filter(|example| {
                example.name.to_lowercase().contains(&keyword_lower) ||
                example.description.to_lowercase().contains(&keyword_lower) ||
                example.api_features.iter().any(|api| api.to_lowercase().contains(&keyword_lower))
            })
            .map(|example| example.name)
            .collect()
    }

    /// Get TypeScript examples filtered by difficulty and category
    pub fn get_typescript_examples_filtered(&self, difficulty: &str, category: &str) -> Vec<String> {
        let difficulty_examples = self.get_typescript_examples_by_difficulty(difficulty);
        let category_examples = self.get_typescript_examples_by_category(category);
        
        difficulty_examples
            .into_iter()
            .filter(|name| category_examples.contains(name))
            .collect()
    }

    /// Get popular TypeScript examples for recommendations
    pub fn get_popular_typescript_examples(&self) -> Vec<String> {
        // Return beginner and commonly used examples
        vec![
            "typescript_hello_world".to_string(),
            "typescript_input_handling".to_string(),
            "typescript_entity_controller".to_string(),
        ]
    }

    /// Get newcomer-friendly TypeScript examples
    pub fn get_newcomer_friendly_typescript_examples(&self) -> Vec<String> {
        self.get_typescript_examples_by_difficulty("beginner")
    }

    /// Get display information for a TypeScript example
    pub fn get_typescript_example_display_info(&self, example_name: &str) -> Option<(String, String, String)> {
        let example = get_typescript_example_by_name(example_name)?;
        
        let display_name = example_name.replace("typescript_", "").replace("_", " ");
        let display_name = display_name
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        let icon = match example.category {
            engine_scripting::examples::ExampleCategory::BasicSyntax => "📝",
            engine_scripting::examples::ExampleCategory::InputHandling => "🎮",
            engine_scripting::examples::ExampleCategory::Physics => "⚡",
            engine_scripting::examples::ExampleCategory::EventSystem => "📡",
            engine_scripting::examples::ExampleCategory::GameLogic => "🎲",
            engine_scripting::examples::ExampleCategory::Debugging => "🐛",
            engine_scripting::examples::ExampleCategory::Performance => "⚡",
            engine_scripting::examples::ExampleCategory::Integration => "🔗",
        }.to_string();
        
        let tooltip = format!("{} ({})", example.description, 
            match example.difficulty_level {
                engine_scripting::examples::DifficultyLevel::Beginner => "Beginner",
                engine_scripting::examples::DifficultyLevel::Intermediate => "Intermediate", 
                engine_scripting::examples::DifficultyLevel::Advanced => "Advanced",
            }
        );
        
        Some((display_name, icon, tooltip))
    }

    /// Check if TypeScript should show script selection dialog
    pub fn should_show_script_selection_for_typescript(&self) -> bool {
        true // TypeScript should support script selection like Lua
    }

    /// Get items for TypeScript script selection dialog
    pub fn get_typescript_script_selection_items(&self) -> Vec<ScriptSelectionItem> {
        let mut items = Vec::new();
        
        // Add example scripts
        for example in get_all_typescript_examples() {
            items.push(ScriptSelectionItem {
                name: example.name.replace("typescript_", "").replace("_", " "),
                path: format!("example:{}", example.name),
                item_type: "example".to_string(),
                description: example.description,
                category: match example.category {
                    engine_scripting::examples::ExampleCategory::BasicSyntax => "Basic Syntax",
                    engine_scripting::examples::ExampleCategory::InputHandling => "Input Handling",
                    engine_scripting::examples::ExampleCategory::Physics => "Physics",
                    engine_scripting::examples::ExampleCategory::EventSystem => "Event System",
                    engine_scripting::examples::ExampleCategory::GameLogic => "Game Logic",
                    engine_scripting::examples::ExampleCategory::Debugging => "Debugging",
                    engine_scripting::examples::ExampleCategory::Performance => "Performance",
                    engine_scripting::examples::ExampleCategory::Integration => "Integration",
                }.to_string(),
            });
        }
        
        // Add template options
        items.push(ScriptSelectionItem {
            name: "Entity Script Template".to_string(),
            path: "template:entity".to_string(),
            item_type: "template".to_string(),
            description: "Basic entity script with init, update, and destroy methods".to_string(),
            category: "Templates".to_string(),
        });
        
        items.push(ScriptSelectionItem {
            name: "Behavior Script Template".to_string(),
            path: "template:behavior".to_string(),
            item_type: "template".to_string(),
            description: "Reusable behavior script for multiple entities".to_string(),
            category: "Templates".to_string(),
        });
        
        items.push(ScriptSelectionItem {
            name: "System Script Template".to_string(),
            path: "template:system".to_string(),
            item_type: "template".to_string(),
            description: "Global system script for world-level operations".to_string(),
            category: "Templates".to_string(),
        });
        
        items
    }
}

// Helper function to convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

