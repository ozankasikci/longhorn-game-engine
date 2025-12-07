use egui::Ui;
use longhorn_core::{World, Name, Transform, Sprite, Enabled, EntityHandle, Script, ScriptValue};
use longhorn_engine::MainCamera;
use longhorn_renderer::Camera;
use crate::EditorState;

/// Actions that can be triggered from the Inspector panel
#[derive(Debug, Clone)]
pub enum EditorAction {
    None,
    OpenScriptEditor { path: String },
    OpenTexturePicker { entity: hecs::Entity },
}

pub struct InspectorPanel {
    pending_action: EditorAction,
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {
            pending_action: EditorAction::None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        state: &EditorState,
    ) -> EditorAction {
        // Reset pending action at the start
        self.pending_action = EditorAction::None;

        ui.heading("Inspector");
        ui.separator();

        let Some(selected) = state.selected_entity else {
            ui.label("Select an entity");
            return EditorAction::None;
        };

        let handle = EntityHandle::new(selected);

        // Check if entity still exists
        if !world.exists(handle) {
            ui.label("Entity no longer exists");
            return EditorAction::None;
        }

        ui.label(format!("Entity ID: {}", selected.id()));
        ui.separator();

        // Name (read-only)
        if let Ok(name) = world.get::<Name>(handle) {
            ui.label(format!("Name: {}", name.0));
        }

        ui.separator();

        // Transform (editable)
        if let Ok(mut transform) = world.get_mut::<Transform>(handle) {
            ui.label("Transform:");
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.add(egui::DragValue::new(&mut transform.position.x).prefix("x: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut transform.position.y).prefix("y: ").speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                let mut degrees = transform.rotation.to_degrees();
                if ui.add(egui::DragValue::new(&mut degrees).suffix("°").speed(1.0)).changed() {
                    transform.rotation = degrees.to_radians();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.add(egui::DragValue::new(&mut transform.scale.x).prefix("x: ").speed(0.01));
                ui.add(egui::DragValue::new(&mut transform.scale.y).prefix("y: ").speed(0.01));
            });
        }

        ui.separator();

        // Sprite (editable)
        self.show_sprite_component(ui, world, handle);

        ui.separator();

        // Enabled (checkbox)
        if let Ok(mut enabled) = world.get_mut::<Enabled>(handle) {
            ui.checkbox(&mut enabled.0, "Enabled");
        }

        ui.separator();

        // Script components (can have multiple)
        self.show_script_components(ui, world, handle);

        ui.separator();

        // Camera component (editable)
        self.show_camera_component(ui, world, handle);

        ui.separator();

        // MainCamera component (marker)
        self.show_main_camera_component(ui, world, handle);

        ui.separator();

        // Add Component dropdown at bottom
        ui.menu_button("Add Component", |ui| {
            // Sprite option
            let has_sprite = world.get::<Sprite>(handle).is_ok();
            if ui.add_enabled(!has_sprite, egui::Button::new("Sprite")).clicked() {
                log::info!("Adding Sprite component to entity");
                // Add sprite with default values (texture ID 0, 32x32 size, white color)
                // AssetId(0) is used as "no texture" placeholder until user selects one via texture picker
                let sprite = Sprite::new(
                    longhorn_core::AssetId::new(0),
                    glam::Vec2::new(32.0, 32.0)
                );
                if let Err(e) = world.set(handle, sprite) {
                    log::error!("Failed to add sprite: {:?}", e);
                } else {
                    log::info!("Added Sprite component to entity - opening texture picker");
                    // Open texture picker immediately after adding the component
                    self.pending_action = EditorAction::OpenTexturePicker {
                        entity: handle.id(),
                    };
                }
                ui.close_menu();
            }

            // Script option
            if ui.button("Script").clicked() {
                log::info!("Add Script button clicked (not yet implemented)");
                // TODO: Show dropdown of available scripts from ScriptRuntime
                // For now, add a test script
                let test_script = Script::new("TestScript.ts");
                if let Err(e) = world.set(handle, test_script) {
                    log::error!("Failed to add script: {:?}", e);
                } else {
                    log::info!("Added TestScript.ts to entity");
                }
                ui.close_menu();
            }

            // Camera option
            let has_camera = world.get::<Camera>(handle).is_ok();
            if ui.add_enabled(!has_camera, egui::Button::new("Camera")).clicked() {
                log::info!("Adding Camera component to entity");
                let camera = Camera::new(800.0, 600.0);
                if let Err(e) = world.set(handle, camera) {
                    log::error!("Failed to add camera: {:?}", e);
                } else {
                    log::info!("Added Camera component to entity");
                }
                ui.close_menu();
            }

            // MainCamera option
            let has_main_camera = world.get::<MainCamera>(handle).is_ok();
            if ui.add_enabled(!has_main_camera, egui::Button::new("MainCamera")).clicked() {
                log::info!("Adding MainCamera component to entity");
                if let Err(e) = world.set(handle, MainCamera) {
                    log::error!("Failed to add MainCamera: {:?}", e);
                } else {
                    log::info!("Added MainCamera component to entity");
                }
                ui.close_menu();
            }
        });

        // Return any pending action
        match &self.pending_action {
            EditorAction::None => {},
            action => log::info!("Inspector returning action: {:?}", action),
        }
        self.pending_action.clone()
    }

    fn show_sprite_component(&mut self, ui: &mut Ui, world: &mut World, handle: EntityHandle) {
        if let Ok(mut sprite) = world.get_mut::<Sprite>(handle) {
            ui.group(|ui| {
                ui.heading("Sprite");
                ui.separator();

                // Texture info (read-only for now, will be editable in Task 5)
                ui.horizontal(|ui| {
                    ui.label("Texture:");
                    if sprite.texture.0 == 0 {
                        ui.label("None");
                    } else {
                        ui.label(format!("ID: {}", sprite.texture.0));
                    }
                });

                // Change Texture button - opens texture picker popup
                if ui.button("Change Texture").clicked() {
                    log::info!("Change Texture button clicked - opening texture picker");
                    self.pending_action = EditorAction::OpenTexturePicker {
                        entity: handle.id(),
                    };
                }

                ui.separator();

                // Size (editable)
                ui.label("Size:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut sprite.size.x).prefix("W: ").speed(1.0).range(0.01..=f32::INFINITY));
                    ui.add(egui::DragValue::new(&mut sprite.size.y).prefix("H: ").speed(1.0).range(0.01..=f32::INFINITY));
                });

                ui.separator();

                // Color (RGBA)
                ui.label("Color:");
                ui.horizontal(|ui| {
                    // Use color picker for RGB
                    let mut color_rgb = [sprite.color[0], sprite.color[1], sprite.color[2]];
                    if ui.color_edit_button_rgb(&mut color_rgb).changed() {
                        sprite.color[0] = color_rgb[0];
                        sprite.color[1] = color_rgb[1];
                        sprite.color[2] = color_rgb[2];
                    }
                    // Separate slider for alpha
                    ui.add(egui::Slider::new(&mut sprite.color[3], 0.0..=1.0).text("A").fixed_decimals(2));
                });

                ui.separator();

                // Flip options
                ui.horizontal(|ui| {
                    ui.checkbox(&mut sprite.flip_x, "Flip X");
                    ui.checkbox(&mut sprite.flip_y, "Flip Y");
                });
            });
        }
    }

    fn show_script_components(&mut self, ui: &mut Ui, world: &mut World, handle: EntityHandle) {
        // Query for all Script components on this entity
        // Clone script data to avoid borrow checker issues with UI
        let script_data = if let Ok(script) = world.get::<Script>(handle) {
            Some((script.path.clone(), script.enabled, script.properties.clone()))
        } else {
            None
        };

        if let Some((path, enabled, properties)) = script_data {
            self.show_single_script(ui, world, handle, &path, enabled, properties);
        }
    }

    fn show_single_script(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        handle: EntityHandle,
        path: &str,
        enabled: bool,
        properties: std::collections::HashMap<String, ScriptValue>,
    ) {
        let mut should_remove = false;
        let mut new_enabled = enabled;
        let mut updated_properties = properties.clone();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Script");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Kebab menu button with Edit and Remove options
                    ui.menu_button("...", |ui| {
                        if ui.button("Edit").clicked() {
                            log::info!("Edit button clicked for script: {}", path);
                            self.pending_action = EditorAction::OpenScriptEditor {
                                path: path.to_string(),
                            };
                            ui.close_menu();
                        }
                        if ui.button("Remove").clicked() {
                            should_remove = true;
                            ui.close_menu();
                        }
                    });
                });
            });

            ui.separator();

            // Script path (read-only for now)
            ui.label(format!("Path: {}", path));

            // Enabled checkbox
            ui.checkbox(&mut new_enabled, "Enabled");

            ui.separator();

            // Script properties (editable)
            if !updated_properties.is_empty() {
                ui.label("Properties:");

                // Clone keys to avoid borrow checker issues
                let keys: Vec<String> = updated_properties.keys().cloned().collect();

                for key in keys {
                    if let Some(value) = updated_properties.get(&key).cloned() {
                        if let Some(new_value) = self.show_script_property_input(ui, &key, &value) {
                            updated_properties.insert(key, new_value);
                        }
                    }
                }
            } else {
                ui.label("(No properties)");
            }
        });

        // Apply changes after UI rendering
        if should_remove {
            if let Err(e) = world.remove::<Script>(handle) {
                log::error!("Failed to remove script: {:?}", e);
            } else {
                log::info!("Removed script from entity");
            }
        } else if new_enabled != enabled || updated_properties != properties {
            // Update the script component
            let mut updated_script = Script::new(path);
            updated_script.enabled = new_enabled;
            updated_script.properties = updated_properties;

            if let Err(e) = world.set(handle, updated_script) {
                log::error!("Failed to update script: {:?}", e);
            }
        }
    }

    fn show_script_property_input(&mut self, ui: &mut Ui, key: &str, value: &ScriptValue) -> Option<ScriptValue> {
        let mut result = None;

        ui.horizontal(|ui| {
            ui.label(format!("{}:", key));

            match value {
                ScriptValue::Number(n) => {
                    let mut val = *n;
                    if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                        result = Some(ScriptValue::Number(val));
                    }
                }
                ScriptValue::String(s) => {
                    let mut val = s.clone();
                    if ui.text_edit_singleline(&mut val).changed() {
                        result = Some(ScriptValue::String(val));
                    }
                }
                ScriptValue::Boolean(b) => {
                    let mut val = *b;
                    if ui.checkbox(&mut val, "").changed() {
                        result = Some(ScriptValue::Boolean(val));
                    }
                }
                ScriptValue::Vec2 { x, y } => {
                    let mut x_val = *x;
                    let mut y_val = *y;
                    let mut changed = false;

                    if ui.add(egui::DragValue::new(&mut x_val).prefix("x: ").speed(0.1)).changed() {
                        changed = true;
                    }
                    if ui.add(egui::DragValue::new(&mut y_val).prefix("y: ").speed(0.1)).changed() {
                        changed = true;
                    }

                    if changed {
                        result = Some(ScriptValue::Vec2 { x: x_val, y: y_val });
                    }
                }
            }
        });

        result
    }

    fn show_camera_component(&mut self, ui: &mut Ui, world: &mut World, handle: EntityHandle) {
        // Clone camera data to avoid borrow checker issues with UI
        let camera_data = if let Ok(camera) = world.get::<Camera>(handle) {
            Some((camera.zoom, camera.viewport_size))
        } else {
            None
        };

        if let Some((mut zoom, viewport_size)) = camera_data {
            let mut should_remove = false;

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Camera");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Remove").clicked() {
                            should_remove = true;
                        }
                    });
                });

                ui.separator();

                // Zoom (editable)
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    ui.add(egui::DragValue::new(&mut zoom).speed(0.01).range(0.1..=10.0));
                });

                // Viewport size (read-only)
                ui.horizontal(|ui| {
                    ui.label("Viewport:");
                    ui.label(format!("{}x{}", viewport_size.x, viewport_size.y));
                });
            });

            // Apply changes after UI rendering
            if should_remove {
                if let Err(e) = world.remove::<Camera>(handle) {
                    log::error!("Failed to remove camera: {:?}", e);
                } else {
                    log::info!("Removed Camera component from entity");
                }
            } else {
                // Update camera if zoom changed
                if let Ok(mut camera) = world.get_mut::<Camera>(handle) {
                    camera.zoom = zoom;
                }
            }
        }
    }

    fn show_main_camera_component(&mut self, ui: &mut Ui, world: &mut World, handle: EntityHandle) {
        if world.get::<MainCamera>(handle).is_ok() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("MainCamera");
                    ui.label("✓");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Remove").clicked() {
                            if let Err(e) = world.remove::<MainCamera>(handle) {
                                log::error!("Failed to remove MainCamera: {:?}", e);
                            } else {
                                log::info!("Removed MainCamera component from entity");
                            }
                        }
                    });
                });

                ui.separator();
                ui.label("This camera will be used in Game View during Play mode.");
            });
        }
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}
