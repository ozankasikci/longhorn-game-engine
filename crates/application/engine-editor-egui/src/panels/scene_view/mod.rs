// Scene view module - handles 3D scene rendering and interaction

pub mod rendering;
pub mod navigation;
pub mod camera_movement;
pub mod gizmos;
pub mod scene_renderer;
pub mod object_renderer;
pub mod scene_view_impl;
pub mod scene_input;
pub mod debug_overlay;
pub mod improved_grid;

#[cfg(test)]
mod navigation_tests;
#[cfg(test)]
mod camera_movement_tests;

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, MeshFilter};
use engine_components_ui::Name;
use crate::types::{SceneNavigation, GizmoSystem};
use crate::editor_state::ConsoleMessage;

/// Focus the scene camera on the selected object
fn focus_on_selected_object(
    world: &World,
    selected_entity: Entity,
    scene_navigation: &mut SceneNavigation,
) {
    if let Some(transform) = world.get_component::<Transform>(selected_entity) {
        // Get object position
        let object_pos = transform.position;
        
        // The camera actually looks in +Z direction when rotation is [0,0,0]
        // So to look at an object, we need to place the camera in FRONT (negative Z)
        
        scene_navigation.scene_camera_transform.position = [
            object_pos[0],          // Same X as object
            object_pos[1] + 1.5,    // 1.5 units above
            object_pos[2] - 5.0,    // 5 units in FRONT (negative Z)
        ];
        
        // With camera in front looking back (+Z direction), no rotation needed
        scene_navigation.scene_camera_transform.rotation = [0.0, 0.0, 0.0];
        
        // Debug output
        let name = world.get_component::<Name>(selected_entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", selected_entity.id()));
        
        eprintln!("\n=== FOCUS: Fixed for +Z look direction ===");
        eprintln!("Object '{}' at: [{:.2}, {:.2}, {:.2}]", 
            name, object_pos[0], object_pos[1], object_pos[2]);
        eprintln!("Camera pos: [{:.2}, {:.2}, {:.2}] (in front, looking back)",
            scene_navigation.scene_camera_transform.position[0],
            scene_navigation.scene_camera_transform.position[1],
            scene_navigation.scene_camera_transform.position[2]);
        eprintln!("Rotation: [0, 0, 0] (looking in +Z direction)");
        eprintln!("==================\n");
    }
}

/// Scene view panel for 3D scene rendering and manipulation
pub struct SceneViewPanel {
    pub scene_view_active: bool,
}

impl SceneViewPanel {
    pub fn new() -> Self {
        Self {
            scene_view_active: true,
        }
    }

    /// Main entry point for rendering the scene view
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: Option<Entity>,
        scene_navigation: &mut SceneNavigation,
        gizmo_system: &mut GizmoSystem,
        scene_renderer: &mut scene_view_impl::SceneViewRenderer,
        play_state: crate::types::PlayState,
    ) -> Vec<crate::editor_state::ConsoleMessage> {
        // Scene view toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.scene_view_active, true, "Scene");
            ui.selectable_value(&mut self.scene_view_active, false, "Game");
            
            ui.separator();
            
            if ui.button("🔍").on_hover_text("Focus on selected (F)").clicked() {
                if let Some(entity) = selected_entity {
                    focus_on_selected_object(world, entity, scene_navigation);
                }
            }
        });
        
        ui.separator();
        
        // Main view area - allocate space first
        let available_size = ui.available_size();
        let (rect, mut response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
        
        // CRITICAL: Create an interactive area that captures mouse events
        // This ensures the scene view gets mouse input even in a docked panel
        response = ui.interact(rect, response.id, egui::Sense::click_and_drag());
        
        // Force focus when hovering to ensure we get input priority
        if response.hovered() {
            response.request_focus();
        }
        
        // Handle scene navigation FIRST before drawing
        let console_messages = scene_input::handle_scene_input(
            world,
            ui,
            &response,
            rect,
            scene_navigation,
            gizmo_system,
            selected_entity,
        );
        
        // Get painter from UI
        let painter = ui.painter();
        
        // Draw background
        painter.rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(35, 35, 35)
        );
        
        
        // Scene content
        ui.allocate_ui_at_rect(rect, |ui| {
            if self.scene_view_active {
                // Draw 3D scene
                scene_renderer.draw_scene(
                    world,
                    ui,
                    rect,
                    &response,
                    scene_navigation,
                    selected_entity,
                    play_state,
                );
                
                // Draw debug overlay
                debug_overlay::draw_movement_debug_overlay(ui, rect, scene_navigation);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Runtime game preview");
                        ui.small("Press Play to see game running");
                    });
                });
            }
        });
        
        // Handle keyboard shortcuts for scene view
        ui.input(|i| {
            // F key to focus on selected object
            if i.key_pressed(egui::Key::F) && selected_entity.is_some() {
                if let Some(entity) = selected_entity {
                    focus_on_selected_object(world, entity, scene_navigation);
                }
            }
        });
        
        console_messages
    }
}