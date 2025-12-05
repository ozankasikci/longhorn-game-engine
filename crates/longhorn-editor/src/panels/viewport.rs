use egui::Ui;

pub struct ViewportPanel;

impl ViewportPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("Viewport");
        ui.separator();

        let available = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());

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
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
