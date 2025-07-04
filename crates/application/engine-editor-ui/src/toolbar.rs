// Toolbar - Longhorn-style editor toolbar with scene tools and play controls

use crate::types::{GizmoSystem, PlayState, SceneNavigation, SceneTool};
use eframe::egui;
use engine_components_3d::Transform;
use engine_ecs_core::{Entity, World};

pub struct Toolbar {}

impl Toolbar {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Toolbar {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        play_state: &mut PlayState,
        gizmo_system: &mut dyn GizmoSystem,
        scene_navigation: &mut SceneNavigation,
        world: &World,
        selected_entity: Option<Entity>,
        selected_object: &Option<String>,
    ) -> ToolbarActions {
        let mut actions = ToolbarActions::default();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Scene manipulation tools
            let current_tool = scene_navigation.current_tool;

            // Selection tool (Q)
            let select_pressed = ui
                .add(
                    egui::Button::new("Select").fill(if current_tool == SceneTool::Select {
                        egui::Color32::from_rgb(100, 150, 255)
                    } else {
                        egui::Color32::TRANSPARENT
                    }),
                )
                .on_hover_text("Select Tool (Q)")
                .clicked();

            if select_pressed {
                scene_navigation.current_tool = SceneTool::Select;
                gizmo_system.set_active_tool(SceneTool::Select);
                gizmo_system.disable_move_gizmo();
                // Selection tool activated
            }

            // Move tool (W)
            let move_pressed = ui
                .add(
                    egui::Button::new("Move").fill(if current_tool == SceneTool::Move {
                        egui::Color32::from_rgb(100, 150, 255)
                    } else {
                        egui::Color32::TRANSPARENT
                    }),
                )
                .on_hover_text("Move Tool (W)")
                .clicked();

            if move_pressed {
                scene_navigation.current_tool = SceneTool::Move;
                gizmo_system.set_active_tool(SceneTool::Move);
                // Enable move gizmo if an entity is selected
                if let Some(entity) = selected_entity {
                    if let Some(_transform) = world.get_component::<Transform>(entity) {
                        gizmo_system.enable_move_gizmo();
                    }
                }
                // Move tool activated
            }

            // Rotate tool (E) - Future implementation
            let rotate_pressed = ui
                .add(
                    egui::Button::new("Rotate").fill(if current_tool == SceneTool::Rotate {
                        egui::Color32::from_rgb(100, 150, 255)
                    } else {
                        egui::Color32::TRANSPARENT
                    }),
                )
                .on_hover_text("Rotate Tool (E) - Coming Soon")
                .clicked();

            if rotate_pressed {
                scene_navigation.current_tool = SceneTool::Rotate;
                gizmo_system.set_active_tool(SceneTool::Rotate);
                // Rotate tool - coming soon
            }

            // Scale tool (R) - Future implementation
            let scale_pressed = ui
                .add(
                    egui::Button::new("Scale").fill(if current_tool == SceneTool::Scale {
                        egui::Color32::from_rgb(100, 150, 255)
                    } else {
                        egui::Color32::TRANSPARENT
                    }),
                )
                .on_hover_text("Scale Tool (R) - Coming Soon")
                .clicked();

            if scale_pressed {
                scene_navigation.current_tool = SceneTool::Scale;
                gizmo_system.set_active_tool(SceneTool::Scale);
                // Scale tool - coming soon
            }

            ui.separator();

            // Play controls - state-aware buttons
            match play_state {
                PlayState::Editing => {
                    if ui.button("Play").on_hover_text("Play").clicked() {
                        actions.start_play = true;
                    }
                    // Show disabled pause/stop buttons
                    ui.add_enabled(false, egui::Button::new("Pause"));
                    ui.add_enabled(false, egui::Button::new("Stop"));
                }
                PlayState::Playing => {
                    // Show highlighted play button (active state)
                    ui.add_enabled(
                        false,
                        egui::Button::new("Play").fill(egui::Color32::from_rgb(100, 200, 100)),
                    );
                    if ui.button("Pause").on_hover_text("Pause").clicked() {
                        actions.pause_play = true;
                    }
                    if ui.button("Stop").on_hover_text("Stop").clicked() {
                        actions.stop_play = true;
                    }
                }
                PlayState::Paused => {
                    if ui.button("Play").on_hover_text("Resume").clicked() {
                        actions.resume_play = true;
                    }
                    // Show highlighted pause button (active state)
                    ui.add_enabled(
                        false,
                        egui::Button::new("Pause").fill(egui::Color32::from_rgb(200, 200, 100)),
                    );
                    if ui.button("Stop").on_hover_text("Stop").clicked() {
                        actions.stop_play = true;
                    }
                }
            }

            ui.separator();

            // DEBUG: Test transform mutation
            if ui
                .button("Test Move")
                .on_hover_text("Debug: Move selected object 1 unit in X")
                .clicked()
            {
                actions.test_move = true;
            }

            ui.separator();

            // View options
            ui.label("Layers:");
            egui::ComboBox::from_id_source("layers")
                .selected_text("Default")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut "", "Default", "Default");
                    ui.selectable_value(&mut "", "UI", "UI");
                    ui.selectable_value(&mut "", "Background", "Background");
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("Focus Selected")
                    .on_hover_text("Focus camera on selected object")
                    .clicked()
                {
                    if let Some(ref _obj) = selected_object {
                        // Focused on object
                    }
                }

                ui.separator();

                ui.label("Layout:");
                egui::ComboBox::from_id_source("layout")
                    .selected_text("Default")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut "", "Default", "Default");
                        ui.selectable_value(&mut "", "2 by 3", "2 by 3");
                        ui.selectable_value(&mut "", "4 Split", "4 Split");
                    });
            });
        });

        actions
    }
}

/// Actions triggered by toolbar interactions
#[derive(Default)]
pub struct ToolbarActions {
    pub start_play: bool,
    pub pause_play: bool,
    pub resume_play: bool,
    pub stop_play: bool,
    pub test_move: bool,
}
