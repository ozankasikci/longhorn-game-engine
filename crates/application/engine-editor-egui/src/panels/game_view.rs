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
        
        if play_state == PlayState::Editing {
            // Show "Press Play" message when not in play mode
            ui.allocate_ui_at_rect(response.rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Press Play button to see game from camera");
                        ui.small("This view shows what the player will see");
                    });
                });
            });
        } else {
            // Return the rect for rendering from main camera perspective
            return (messages, Some(response.rect));
        }
        
        (messages, None)
    }
}