use egui::{Ui, TextureId, Sense};

pub struct ViewportPanel {
    texture_id: Option<TextureId>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self { texture_id: None }
    }

    pub fn set_texture(&mut self, texture_id: TextureId) {
        self.texture_id = Some(texture_id);
    }

    pub fn clear_texture(&mut self) {
        self.texture_id = None;
    }

    pub fn show(&mut self, ui: &mut Ui) -> egui::Response {
        ui.heading("Viewport");
        ui.separator();

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());

        if let Some(texture_id) = self.texture_id {
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

    pub fn texture_id(&self) -> Option<TextureId> {
        self.texture_id
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
