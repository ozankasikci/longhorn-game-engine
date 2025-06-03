// Scene view module - handles 3D scene rendering and interaction

pub mod rendering;
pub mod navigation;
pub mod gizmos;
pub mod scene_renderer;
pub mod object_renderer;
pub mod scene_view_impl;
pub mod scene_input;

#[cfg(test)]
mod navigation_tests;
#[cfg(test)]
mod camera_movement_tests;

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType};
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
        
        // Calculate object bounds - consider scale and any mesh bounds
        let scale = transform.scale;
        let max_scale = scale[0].max(scale[1]).max(scale[2]);
        
        // Try to get mesh bounds for better framing
        let object_radius = if let Some(mesh) = world.get_component::<Mesh>(selected_entity) {
            // If we have mesh bounds, use them
            match &mesh.mesh_type {
                MeshType::Cube => max_scale * 0.866, // Cube diagonal / 2
                MeshType::Sphere => max_scale * 0.5,  // Sphere radius
                MeshType::Plane => max_scale * 1.0,   // Plane size
                MeshType::Custom(_) => max_scale * 1.0, // Default for custom meshes
            }
        } else {
            // No mesh, use scale as a rough estimate
            max_scale * 1.0
        };
        
        // Calculate good viewing distance
        let view_distance = (object_radius * 5.0).max(5.0); // Reasonable distance for viewing
        
        // FIXED: Position camera to properly view the object
        // In our coordinate system: +Y is up, -Z is forward
        // So to look at an object, camera should be at higher Z (behind) and look forward (-Z)
        scene_navigation.scene_camera_transform.position = [
            object_pos[0],              // Same X as object  
            object_pos[1] + view_distance * 0.5,  // Above object (45 degree angle)
            object_pos[2] + view_distance,        // Behind object in +Z
        ];
        
        // Calculate rotation to look at the object
        // We need to point the camera toward the object
        let dx = object_pos[0] - scene_navigation.scene_camera_transform.position[0];
        let dy = object_pos[1] - scene_navigation.scene_camera_transform.position[1];
        let dz = object_pos[2] - scene_navigation.scene_camera_transform.position[2];
        
        // Calculate pitch (rotation around X axis) - looking down at object
        let horizontal_dist = (dx * dx + dz * dz).sqrt();
        let pitch = dy.atan2(horizontal_dist); // Negative because we look down
        
        // Calculate yaw (rotation around Y axis) - pointing toward object
        let yaw = dx.atan2(-dz); // atan2(x, -z) for proper orientation
        
        scene_navigation.scene_camera_transform.rotation = [
            pitch,  // Pitch down to look at object
            yaw,    // Yaw to face object  
            0.0     // No roll
        ];
    }
    // Focus operation complete
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
    ) {
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
        scene_input::handle_scene_input(
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
        // Scene view rendered
    }
}