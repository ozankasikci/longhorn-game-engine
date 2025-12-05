use egui::Ui;
use longhorn_core::{World, Name, Transform, Sprite, Enabled, EntityHandle};
use crate::EditorState;

pub struct InspectorPanel;

impl InspectorPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, world: &mut World, state: &EditorState) {
        ui.heading("Inspector");
        ui.separator();

        let Some(selected) = state.selected_entity else {
            ui.label("Select an entity");
            return;
        };

        let handle = EntityHandle::new(selected);

        // Check if entity still exists
        if !world.exists(handle) {
            ui.label("Entity no longer exists");
            return;
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
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}
