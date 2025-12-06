// crates/longhorn-editor/src/panels/script_editor.rs
use egui::{Color32, FontId, RichText, ScrollArea, TextEdit, Ui};
use crate::script_editor_state::ScriptEditorState;

/// Script Editor panel with basic text editing and error display
pub struct ScriptEditorPanel;

impl ScriptEditorPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show the script editor panel
    /// Returns true if save was triggered (Ctrl+S or Cmd+S)
    pub fn show(&mut self, ui: &mut Ui, state: &mut ScriptEditorState) -> bool {
        let mut save_triggered = false;

        // Check if a file is open
        if !state.is_open() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No script open").color(Color32::DARK_GRAY));
                ui.add_space(10.0);
                ui.label(RichText::new("Select a script from the Project panel to begin editing")
                    .color(Color32::DARK_GRAY)
                    .size(12.0));
            });
            return false;
        }

        // Header with filename and dirty indicator
        ui.horizontal(|ui| {
            // Filename with dirty indicator
            if let Some(filename) = state.filename() {
                let display_text = if state.is_dirty() {
                    format!("{}*", filename)
                } else {
                    filename.to_string()
                };
                ui.heading(display_text);
            }

            // Save shortcut hint
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("Ctrl+S / Cmd+S to save")
                    .color(Color32::DARK_GRAY)
                    .size(12.0));
            });
        });

        ui.separator();

        // Check for save shortcut
        let ctrl_or_cmd = ui.input(|i| i.modifiers.command);
        let s_pressed = ui.input(|i| i.key_pressed(egui::Key::S));

        if ctrl_or_cmd && s_pressed {
            save_triggered = true;
        }

        // Code editor area
        let available_height = ui.available_height();

        // Reserve space for error panel if there are errors
        let editor_height = if state.errors.is_empty() {
            available_height
        } else {
            // Reserve ~100px for error panel, but make it responsive
            (available_height * 0.7).max(available_height - 150.0)
        };

        // Code editor
        ui.push_id("script_editor_code", |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(editor_height)
                .show(ui, |ui| {
                    // Use a monospace font for code
                    let font_id = FontId::monospace(14.0);

                    let text_edit = TextEdit::multiline(&mut state.content)
                        .font(font_id)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .lock_focus(true);

                    ui.add(text_edit);
                });
        });

        // Error panel (shown below the editor)
        if !state.errors.is_empty() {
            ui.separator();

            ui.push_id("script_editor_errors", |ui| {
                // Error panel header
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Errors ({})", state.errors.len()))
                        .color(Color32::from_rgb(255, 100, 100))
                        .strong());
                });

                ui.separator();

                // Error list
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        for error in &state.errors {
                            ui.horizontal(|ui| {
                                // Red dot indicator
                                ui.label(RichText::new("●").color(Color32::from_rgb(255, 80, 80)));

                                // Line number
                                ui.label(RichText::new(format!("Line {}:", error.line))
                                    .color(Color32::from_rgb(255, 150, 150))
                                    .strong());

                                // Error message
                                ui.label(RichText::new(&error.message)
                                    .color(Color32::from_rgb(255, 200, 200)));
                            });
                        }
                    });
            });
        }

        save_triggered
    }
}

impl Default for ScriptEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}
