use egui::Ui;
use crate::state::{EditorMode, EditorState};

/// Actions that can be triggered from the toolbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    Play,
    Pause,
    Resume,
    Stop,
}

/// Toolbar with play controls
pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &EditorState) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal(|ui| {
            // Center the buttons
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                ui.set_max_width(300.0);

                match state.mode {
                    EditorMode::Scene => {
                        if ui.button("▶ Play").clicked() {
                            action = ToolbarAction::Play;
                        }
                        ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                        ui.add_enabled(false, egui::Button::new("⏹ Stop"));
                    }
                    EditorMode::Play => {
                        if state.paused {
                            if ui.button("▶ Resume").clicked() {
                                action = ToolbarAction::Resume;
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("▶ Play"));
                        }

                        if !state.paused {
                            if ui.button("⏸ Pause").clicked() {
                                action = ToolbarAction::Pause;
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                        }

                        if ui.button("⏹ Stop").clicked() {
                            action = ToolbarAction::Stop;
                        }
                    }
                }

                ui.separator();

                // Mode indicator
                let mode_text = match (state.mode, state.paused) {
                    (EditorMode::Scene, _) => "Scene Mode",
                    (EditorMode::Play, false) => "▶ Playing",
                    (EditorMode::Play, true) => "⏸ Paused",
                };
                ui.label(mode_text);
            });
        });

        action
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_action_default() {
        assert_eq!(ToolbarAction::None, ToolbarAction::None);
    }
}
