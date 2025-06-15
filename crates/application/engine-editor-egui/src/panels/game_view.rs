// Game view panel - shows the game from the player's camera perspective

use eframe::egui;
use crate::types::PlayState;
use crate::editor_state::ConsoleMessage;

pub struct GameViewPanel {}

impl GameViewPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        play_state: PlayState,
    ) -> (Vec<ConsoleMessage>, Option<egui::Rect>) {
        let mut messages = Vec::new();
        
        // Game View header
        ui.horizontal(|ui| {
            ui.label("🎮 Game View");
            
            ui.separator();
            
            // Aspect ratio selector  
            ui.label("Aspect:");
            egui::ComboBox::from_id_source("game_view_aspect")
                .selected_text("16:9")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut "", "16:9", "16:9");
                    ui.selectable_value(&mut "", "4:3", "4:3");
                    ui.selectable_value(&mut "", "Free", "Free Aspect");
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔊").on_hover_text("Audio toggle").clicked() {
                    // Audio toggled
                }
                if ui.button("📊").on_hover_text("Stats").clicked() {
                    // Stats clicked
                }
            });
        });
        
        ui.separator();
        
        // Main game view area
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::hover());
        
        // Debug: Draw a background to ensure the game view is visible
        let painter = ui.painter();
        painter.rect_filled(
            response.rect,
            0.0,
            egui::Color32::from_rgb(30, 30, 40), // Dark blue background
        );
        
        // Always return the rect for rendering from main camera perspective
        // The game view should show the camera view regardless of play state
        (messages, Some(response.rect))
    }
}