// Settings dialog for editor preferences

use crate::types::EditorSettings;
use eframe::egui;

pub struct SettingsDialog {
    pub open: bool,
    pub settings: EditorSettings,
    temp_settings: EditorSettings,
    active_tab: SettingsTab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsTab {
    Camera,
    Grid,
    Snap,
    Theme,
}

impl SettingsDialog {
    pub fn new(settings: EditorSettings) -> Self {
        Self {
            open: false,
            settings: settings.clone(),
            temp_settings: settings,
            active_tab: SettingsTab::Camera,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        let mut save_clicked = false;
        let mut cancel_clicked = false;

        egui::Window::new("Preferences")
            .default_width(600.0)
            .default_height(400.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Tab selection
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, SettingsTab::Camera, "🎥 Camera");
                    ui.selectable_value(&mut self.active_tab, SettingsTab::Grid, "⊞ Grid");
                    ui.selectable_value(&mut self.active_tab, SettingsTab::Snap, "🧲 Snap");
                    ui.selectable_value(&mut self.active_tab, SettingsTab::Theme, "🎨 Theme");
                });

                ui.separator();

                // Tab content
                egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                    SettingsTab::Camera => self.show_camera_settings(ui),
                    SettingsTab::Grid => self.show_grid_settings(ui),
                    SettingsTab::Snap => self.show_snap_settings(ui),
                    SettingsTab::Theme => self.show_theme_settings(ui),
                });

                ui.separator();

                // Buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }

                        if ui.button("Apply").clicked() {
                            self.settings = self.temp_settings.clone();
                        }

                        if ui.button("Save").clicked() {
                            save_clicked = true;
                        }
                    });
                });
            });

        // Handle button actions
        if save_clicked {
            self.settings = self.temp_settings.clone();
            let _ = self.settings.save();
            self.open = false;
        }

        if cancel_clicked {
            self.temp_settings = self.settings.clone();
            self.open = false;
        }
    }

    fn show_camera_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Camera Settings");
        ui.add_space(10.0);

        egui::Grid::new("camera_settings_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                // Movement speed
                ui.label("Movement Speed:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.camera.movement_speed, 0.5..=20.0)
                        .suffix(" units/sec"),
                );
                ui.end_row();

                // Fast movement multiplier
                ui.label("Sprint Multiplier:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.camera.fast_multiplier, 1.5..=10.0)
                        .suffix("x"),
                );
                ui.end_row();

                // Rotation sensitivity
                ui.label("Look Sensitivity:");
                ui.add(
                    egui::Slider::new(
                        &mut self.temp_settings.camera.rotation_sensitivity,
                        0.001..=0.02,
                    )
                    .suffix(" rad/px")
                    .custom_formatter(|n, _| format!("{:.3}", n)),
                );
                ui.end_row();

                // Smoothing
                ui.label("Mouse Smoothing:");
                ui.checkbox(&mut self.temp_settings.camera.smoothing_enabled, "");
                ui.end_row();

                // Invert Y
                ui.label("Invert Y-Axis:");
                ui.checkbox(&mut self.temp_settings.camera.invert_y, "");
                ui.end_row();

                // Invert X
                ui.label("Invert X-Axis:");
                ui.checkbox(&mut self.temp_settings.camera.invert_x, "");
                ui.end_row();
            });

        ui.add_space(10.0);

        // Reset button
        if ui.button("Reset to Defaults").clicked() {
            self.temp_settings.camera = Default::default();
        }
    }

    fn show_grid_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Grid Settings");
        ui.add_space(10.0);

        egui::Grid::new("grid_settings_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Show Grid:");
                ui.checkbox(&mut self.temp_settings.grid.enabled, "");
                ui.end_row();

                ui.label("Grid Size:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.grid.size, 0.1..=10.0)
                        .suffix(" units"),
                );
                ui.end_row();

                ui.label("Subdivisions:");
                ui.add(egui::Slider::new(
                    &mut self.temp_settings.grid.subdivisions,
                    1..=20,
                ));
                ui.end_row();
            });
    }

    fn show_snap_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Snap Settings");
        ui.add_space(10.0);

        egui::Grid::new("snap_settings_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("Enable Snapping:");
                ui.checkbox(&mut self.temp_settings.snap.enabled, "");
                ui.end_row();

                ui.label("Position Snap:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.snap.position_increment, 0.01..=2.0)
                        .suffix(" units"),
                );
                ui.end_row();

                ui.label("Rotation Snap:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.snap.rotation_increment, 1.0..=90.0)
                        .suffix("°"),
                );
                ui.end_row();

                ui.label("Scale Snap:");
                ui.add(egui::Slider::new(
                    &mut self.temp_settings.snap.scale_increment,
                    0.01..=1.0,
                ));
                ui.end_row();
            });
    }

    fn show_theme_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Theme Settings");
        ui.add_space(10.0);

        egui::Grid::new("theme_settings_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                ui.label("UI Scale:");
                ui.add(egui::Slider::new(&mut self.temp_settings.font_size, 0.5..=2.0).suffix("x"));
                ui.end_row();

                ui.label("Font Size:");
                ui.add(
                    egui::Slider::new(&mut self.temp_settings.font_size, 8.0..=20.0).suffix(" pt"),
                );
                ui.end_row();
            });
    }
}
