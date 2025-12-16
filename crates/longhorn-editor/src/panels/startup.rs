// crates/longhorn-editor/src/panels/startup.rs
use egui::{Context, Align2, Color32, FontId, Pos2, Rect, UiBuilder, Vec2};

/// Actions that can be triggered from the startup screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupAction {
    None,
    NewProject,
    OpenProject,
}

/// Startup screen shown when no project is loaded
pub struct StartupPanel;

impl StartupPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show the startup screen and return any triggered action
    pub fn show(&mut self, ctx: &Context) -> StartupAction {
        let mut action = StartupAction::None;

        egui::CentralPanel::default().show(ctx, |ui| {
            let center = ui.max_rect().center();

            // Draw background
            ui.painter().rect_filled(
                ui.max_rect(),
                0.0,
                Color32::from_rgb(30, 30, 35),
            );

            // Title
            ui.painter().text(
                Pos2::new(center.x, center.y - 80.0),
                Align2::CENTER_CENTER,
                "Longhorn Engine",
                FontId::proportional(32.0),
                Color32::from_rgb(220, 220, 220),
            );

            // Centered button area
            let button_area = Rect::from_center_size(
                Pos2::new(center.x, center.y + 20.0),
                Vec2::new(300.0, 100.0),
            );

            ui.allocate_new_ui(UiBuilder::new().max_rect(button_area), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    let button_size = Vec2::new(200.0, 40.0);

                    if ui.add_sized(button_size, egui::Button::new("New Project")).clicked() {
                        action = StartupAction::NewProject;
                    }

                    ui.add_space(10.0);

                    if ui.add_sized(button_size, egui::Button::new("Open Project")).clicked() {
                        action = StartupAction::OpenProject;
                    }
                });
            });
        });

        action
    }
}

impl Default for StartupPanel {
    fn default() -> Self {
        Self::new()
    }
}
