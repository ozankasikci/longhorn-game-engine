// crates/longhorn-editor/src/dialogs/unsaved_changes.rs
use egui::Context;

/// Result of the unsaved changes dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesResult {
    None,
    Save,
    DontSave,
    Cancel,
}

/// Dialog shown when closing with unsaved changes
pub struct UnsavedChangesDialog {
    /// Whether the dialog is open
    pub is_open: bool,
    /// List of dirty files to display
    dirty_files: Vec<String>,
}

impl UnsavedChangesDialog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            dirty_files: Vec::new(),
        }
    }

    /// Open the dialog with a list of dirty files
    pub fn open(&mut self, dirty_files: Vec<String>) {
        self.is_open = true;
        self.dirty_files = dirty_files;
    }

    /// Show the dialog and return result
    pub fn show(&mut self, ctx: &Context) -> UnsavedChangesResult {
        let mut result = UnsavedChangesResult::None;

        if !self.is_open {
            return result;
        }

        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(350.0);

                ui.label("You have unsaved changes:");
                ui.add_space(5.0);

                for file in &self.dirty_files {
                    ui.label(format!("  • {}", file));
                }

                ui.add_space(10.0);
                ui.label("Save before closing?");

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Save").clicked() {
                            result = UnsavedChangesResult::Save;
                            self.is_open = false;
                        }
                        if ui.button("Don't Save").clicked() {
                            result = UnsavedChangesResult::DontSave;
                            self.is_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            result = UnsavedChangesResult::Cancel;
                            self.is_open = false;
                        }
                    });
                });
            });

        result
    }
}

impl Default for UnsavedChangesDialog {
    fn default() -> Self {
        Self::new()
    }
}
