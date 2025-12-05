use egui::{Ui, TextureId, Sense};

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui, texture_id: Option<TextureId>) -> egui::Response {
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
                egui::Color32::WHITE,
            );
        } else {
            // Placeholder when no texture is set
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Game Viewport",
                egui::FontId::proportional(20.0),
                egui::Color32::from_rgb(150, 150, 150),
            );
        }

        response
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
