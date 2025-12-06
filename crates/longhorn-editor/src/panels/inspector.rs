use egui::Ui;
use longhorn_core::{World, Name, Transform, Sprite, Enabled, EntityHandle, Script, ScriptValue};
use crate::EditorState;

/// Actions that can be triggered from the Inspector panel
#[derive(Debug, Clone)]
pub enum EditorAction {
    None,
    OpenScriptEditor { path: String },
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

    pub fn show(&mut self, ui: &mut Ui, world: &mut World, state: &EditorState) -> EditorAction {
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

        // Sprite (read-only)
        if let Ok(sprite) = world.get::<Sprite>(handle) {
            ui.label("Sprite:");
            ui.label(format!("  Texture ID: {}", sprite.texture.0));
            ui.label(format!("  Size: ({:.1}, {:.1})", sprite.size.x, sprite.size.y));
            ui.label(format!("  Color: ({:.2}, {:.2}, {:.2}, {:.2})",
                sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]));
            ui.label(format!("  Flip: x={}, y={}", sprite.flip_x, sprite.flip_y));
        }

        ui.separator();

        // Enabled (checkbox)
        if let Ok(mut enabled) = world.get_mut::<Enabled>(handle) {
            ui.checkbox(&mut enabled.0, "Enabled");
        }

        ui.separator();

        // Script components (can have multiple)
        self.show_script_components(ui, world, handle);

        ui.separator();

        // Add Script button at bottom
        if ui.button("Add Script").clicked() {
            log::info!("Add Script button clicked (not yet implemented)");
            // TODO: Show dropdown of available scripts from ScriptRuntime
            // For now, add a test script
            let test_script = Script::new("TestScript.ts");
            if let Err(e) = world.set(handle, test_script) {
                log::error!("Failed to add script: {:?}", e);
            } else {
                log::info!("Added TestScript.ts to entity");
            }
        }

        // Return any pending action
        self.pending_action.clone()
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
                    // Remove button
                    if ui.button("Remove").clicked() {
                        should_remove = true;
                    }

                    // Kebab menu button
                    let kebab_id = ui.make_persistent_id(format!("script_kebab_{}", path));
                    let kebab_response = ui.button("⋮");
                    if kebab_response.clicked() {
                        ui.memory_mut(|m| m.toggle_popup(kebab_id));
                    }

                    // Popup menu below the kebab button
                    egui::popup_below_widget(ui, kebab_id, &kebab_response, egui::PopupCloseBehavior::CloseOnClickOutside, |ui: &mut Ui| {
                        if ui.button("Edit").clicked() {
                            log::info!("Edit button clicked for script: {}", path);
                            self.pending_action = EditorAction::OpenScriptEditor {
                                path: path.to_string(),
                            };
                            ui.memory_mut(|m| m.close_popup());
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
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}
