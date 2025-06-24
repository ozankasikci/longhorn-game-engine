use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ImportErrorType {
    InvalidFormat,
    FileTooLarge,
    FileNotFound,
    PermissionDenied,
    UnsupportedFeature,
    OutOfMemory,
    Other,
}

#[derive(Debug, Clone)]
pub struct ImportError {
    pub file_path: PathBuf,
    pub error_type: ImportErrorType,
    pub message: String,
    pub recoverable: bool,
}

pub struct ImportErrorDialog {
    errors: Vec<ImportError>,
    visible: bool,
}

impl ImportErrorDialog {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            visible: false,
        }
    }

    pub fn add_error(&mut self, error: ImportError) {
        self.errors.push(error);
        self.visible = true;
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn recoverable_errors(&self) -> Vec<&ImportError> {
        self.errors.iter().filter(|e| e.recoverable).collect()
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.visible || self.errors.is_empty() {
            return;
        }

        let mut open = self.visible;
        let mut clear_errors = false;

        egui::Window::new("Import Errors")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for error in &self.errors {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let color = if error.recoverable {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::RED
                                };

                                ui.colored_label(color, format!("{:?}", error.error_type));
                                ui.label(error.file_path.display().to_string());
                            });

                            ui.label(&error.message);

                            if error.recoverable {
                                ui.label("This error may be recoverable.");
                            }
                        });
                    }
                });

                ui.separator();

                if ui.button("Clear All").clicked() {
                    clear_errors = true;
                }
            });

        self.visible = open;
        if clear_errors {
            self.errors.clear();
        }
    }
}
