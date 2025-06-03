// Console panel - displays log messages and debug output

use eframe::egui;
use crate::editor_state::{ConsoleMessage, ConsoleMessageType};

pub struct ConsolePanel {
    pub console_messages: Vec<ConsoleMessage>,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            console_messages: Vec::new(),
        }
    }
    
    pub fn add_messages(&mut self, mut messages: Vec<ConsoleMessage>) {
        self.console_messages.append(&mut messages);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, console_messages: &mut Vec<ConsoleMessage>) {
        ui.horizontal(|ui| {
            ui.label("Output Log");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🧹 Clear").clicked() {
                    console_messages.clear();
                    console_messages.push(ConsoleMessage::info("🧹 Console cleared"));
                }
                
                if ui.button("📋 Copy All").on_hover_text("Copy all logs to clipboard").clicked() {
                    let all_logs = ConsoleMessage::get_all_logs_as_string(console_messages);
                    ui.output_mut(|o| o.copied_text = all_logs);
                    console_messages.push(ConsoleMessage::info("📋 Logs copied to clipboard"));
                }
                
                if ui.button("💾 Export").on_hover_text("Export logs to file").clicked() {
                    let all_logs = ConsoleMessage::get_all_logs_as_string(console_messages);
                    match std::fs::write("console_export.log", all_logs) {
                        Ok(_) => console_messages.push(ConsoleMessage::info("💾 Logs exported to console_export.log")),
                        Err(e) => console_messages.push(ConsoleMessage::info(&format!("❌ Export failed: {}", e))),
                    }
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in console_messages.iter() {
                    if let ConsoleMessage::Message { message, message_type, .. } = message {
                        let color = match message_type {
                            ConsoleMessageType::Info => egui::Color32::WHITE,
                            ConsoleMessageType::Warning => egui::Color32::YELLOW,
                            ConsoleMessageType::Error => egui::Color32::RED,
                        };
                        
                        ui.colored_label(color, message);
                    }
                }
            });
    }
}