// Project panel - displays project assets and file browser

use eframe::egui;
use engine_editor_assets::ProjectAsset;

pub struct ProjectPanel {}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, project_assets: &[ProjectAsset]) {
        ui.horizontal(|ui| {
            ui.label("Asset Browser");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").on_hover_text("Refresh assets").clicked() {
                    // Refresh assets
                }
                if ui.button("➕").on_hover_text("Create new asset").clicked() {
                    // Create asset menu
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for asset in project_assets {
                self.show_project_asset(ui, asset);
            }
        });
    }
    
    fn show_project_asset(&mut self, ui: &mut egui::Ui, asset: &ProjectAsset) {
        match &asset.children {
            Some(children) => {
                // Folder with children
                ui.collapsing(&asset.name, |ui| {
                    for child in children {
                        self.show_project_asset(ui, child);
                    }
                });
            }
            None => {
                // File asset
                if ui.selectable_label(false, &asset.name).clicked() {
                    // Asset selected
                }
            }
        }
    }
}