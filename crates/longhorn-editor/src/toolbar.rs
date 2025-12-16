use egui::Ui;
use crate::state::{PlayMode, EditorState};

/// Actions that can be triggered from the toolbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    Play,
    Pause,
    Resume,
    Stop,
    ToggleConsole,
}

/// Toolbar with play controls
pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &EditorState) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal_centered(|ui| {
            // Add some spacing from left edge
            ui.add_space(ui.available_width() / 2.0 - 120.0);

            // Play/Resume button
            match state.mode {
                PlayMode::Scene => {
                    if ui.button("▶ Play").clicked() {
                        action = ToolbarAction::Play;
                    }
                }
                PlayMode::Play if state.paused => {
                    if ui.button("▶ Resume").clicked() {
                        action = ToolbarAction::Resume;
                    }
                }
                PlayMode::Play => {
                    ui.add_enabled(false, egui::Button::new("▶ Play"));
                }
            }

            // Pause button
            match state.mode {
                PlayMode::Scene => {
                    ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                }
                PlayMode::Play if !state.paused => {
                    if ui.button("⏸ Pause").clicked() {
                        action = ToolbarAction::Pause;
                    }
                }
                PlayMode::Play => {
                    ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                }
            }

            // Stop button
            match state.mode {
                PlayMode::Scene => {
                    ui.add_enabled(false, egui::Button::new("⏹ Stop"));
                }
                PlayMode::Play => {
                    if ui.button("⏹ Stop").clicked() {
                        action = ToolbarAction::Stop;
                    }
                }
            }

            ui.separator();

            // Mode indicator
            let mode_text = match (state.mode, state.paused) {
                (PlayMode::Scene, _) => "Scene Mode",
                (PlayMode::Play, false) => "Playing",
                (PlayMode::Play, true) => "Paused",
            };
            ui.label(mode_text);

            // Spacer to push console button to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Console").clicked() {
                    action = ToolbarAction::ToggleConsole;
                }
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
