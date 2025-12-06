// crates/longhorn-editor/src/panels/console.rs
use egui::{RichText, ScrollArea, Ui};
use crate::console::{ConsoleLevel, ScriptConsole};
use crate::styling::Colors;

/// Console panel showing script output
pub struct ConsolePanel {
    /// Whether auto-scroll is enabled
    auto_scroll: bool,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            auto_scroll: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, console: &ScriptConsole) {
        // Header row with title and clear button
        ui.horizontal(|ui| {
            ui.heading("Console");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    console.clear();
                }
            });
        });

        ui.separator();

        // Scrollable log area
        let entries = console.entries();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                for entry in &entries {
                    let (prefix, color) = match entry.level {
                        ConsoleLevel::Log => ("", Colors::TEXT_SECONDARY),
                        ConsoleLevel::Warn => ("⚠ ", Colors::WARNING),
                        ConsoleLevel::Error => ("✖ ", Colors::ERROR),
                    };

                    ui.label(RichText::new(format!("{}{}", prefix, entry.message)).color(color));
                }

                if entries.is_empty() {
                    ui.label(RichText::new("No console output").color(Colors::TEXT_MUTED));
                }
            });
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}
