use egui::{Ui, TextureId, Sense};
use crate::styling::Colors;
use crate::CameraInput;
use glam::Vec2;

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui, texture_id: Option<TextureId>) -> CameraInput {
        ui.heading("Viewport");
        ui.separator();

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());

        if let Some(texture_id) = texture_id {
            // Draw the rendered game texture
            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Colors::TEXT_ON_ACCENT,
            );
        } else {
            // Placeholder when no texture is set
            ui.painter().rect_filled(
                rect,
                0.0,
                Colors::BG_VIEWPORT,
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Game Viewport",
                egui::FontId::proportional(20.0),
                Colors::TEXT_SECONDARY,
            );
        }

        // Capture camera input when hovered
        let mut camera_input = CameraInput::default();
        if response.hovered() {
            camera_input.mmb_held = ui.input(|i| {
                i.pointer.button_down(egui::PointerButton::Middle)
            });
            let drag_delta = response.drag_delta();
            camera_input.mouse_delta = Vec2::new(drag_delta.x, drag_delta.y);
            camera_input.scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        }

        camera_input
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
