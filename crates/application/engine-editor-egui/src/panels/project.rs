// Project panel - displays project assets and file browser

use eframe::egui;
use crate::types::ProjectAsset;
use crate::editor_state::ConsoleMessage;

pub struct ProjectPanel {
    console_messages: Vec<ConsoleMessage>,
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            console_messages: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, project_assets: &[ProjectAsset], console_messages: &mut Vec<ConsoleMessage>) {
        ui.horizontal(|ui| {
            ui.label("Asset Browser");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").on_hover_text("Refresh assets").clicked() {
                    console_messages.push(ConsoleMessage::info("🔄 Refreshing project assets"));
                }
                if ui.button("➕").on_hover_text("Create new asset").clicked() {
                    console_messages.push(ConsoleMessage::info("➕ Create asset menu"));
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for asset in project_assets {
                self.show_project_asset(ui, asset, console_messages);
            }
        });
    }
    
    fn show_project_asset(&mut self, ui: &mut egui::Ui, asset: &ProjectAsset, console_messages: &mut Vec<ConsoleMessage>) {
        match &asset.children {
            Some(children) => {
                // Folder with children
                ui.collapsing(&asset.name, |ui| {
                    for child in children {
                        self.show_project_asset(ui, child, console_messages);
                    }
                });
            }
            None => {
                // File asset
                if ui.selectable_label(false, &asset.name).clicked() {
                    console_messages.push(ConsoleMessage::info(&format!("📄 Selected asset: {}", asset.name)));
                }
            }
        }
    }
}