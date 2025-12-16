use egui::Context;
use std::path::PathBuf;

/// Result of the new project dialog
#[derive(Debug, Clone)]
pub enum NewProjectResult {
    None,
    Create { path: PathBuf, name: String },
    Cancel,
}

/// Dialog for creating a new project
pub struct NewProjectDialog {
    /// Whether the dialog is open
    pub is_open: bool,
    /// Project name input
    name: String,
    /// Location path
    location: PathBuf,
    /// Error message to display
    error: Option<String>,
}

impl NewProjectDialog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            name: String::new(),
            location: dirs::home_dir().unwrap_or_default(),
            error: None,
        }
    }

    /// Open the dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.name = "My Game".to_string();
        self.error = None;
    }

    /// Convert project name to folder name (lowercase, hyphens)
    fn to_folder_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    /// Get the final project path
    fn final_path(&self) -> PathBuf {
        let folder_name = Self::to_folder_name(&self.name);
        self.location.join(folder_name)
    }

    /// Show the dialog and return result
    pub fn show(&mut self, ctx: &Context) -> NewProjectResult {
        let mut result = NewProjectResult::None;

        if !self.is_open {
            return result;
        }

        egui::Window::new("New Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                egui::Grid::new("new_project_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Project Name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label("Location:");
                        ui.horizontal(|ui| {
                            let location_str = self.location.to_string_lossy();
                            ui.add(egui::TextEdit::singleline(&mut location_str.to_string()).interactive(false));
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&self.location)
                                    .pick_folder()
                                {
                                    self.location = path;
                                }
                            }
                        });
                        ui.end_row();

                        ui.label("Final path:");
                        ui.label(self.final_path().to_string_lossy().to_string());
                        ui.end_row();
                    });

                // Error message
                if let Some(error) = &self.error {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_create = !self.name.trim().is_empty() && !self.final_path().exists();

                        if ui.add_enabled(can_create, egui::Button::new("Create")).clicked() {
                            result = NewProjectResult::Create {
                                path: self.final_path(),
                                name: self.name.clone(),
                            };
                            self.is_open = false;
                        }

                        if ui.button("Cancel").clicked() {
                            result = NewProjectResult::Cancel;
                            self.is_open = false;
                        }

                        // Show why create is disabled
                        if self.name.trim().is_empty() {
                            self.error = Some("Project name is required".to_string());
                        } else if self.final_path().exists() {
                            self.error = Some("Folder already exists".to_string());
                        } else {
                            self.error = None;
                        }
                    });
                });
            });

        result
    }
}

impl Default for NewProjectDialog {
    fn default() -> Self {
        Self::new()
    }
}
