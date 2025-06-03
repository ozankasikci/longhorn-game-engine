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
) -> Vec<ConsoleMessage> {
    let mut messages = Vec::new();
    
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
        
        // SIMPLIFIED: Just position camera at a good viewing distance
        let view_distance = (object_radius * 8.0).max(5.0); // Simple distance calculation
        
        // Position camera behind and above the object (simple, predictable positioning)
        scene_navigation.scene_camera_transform.position = [
            object_pos[0],              // Same X as object
            object_pos[1] + 3.0,        // 3 units above object
            object_pos[2] + view_distance, // Behind object (positive Z)
        ];
        
        // Reset rotation to look straight at the object
        scene_navigation.scene_camera_transform.rotation = [
            -0.2,  // Slight downward pitch to look at object
            0.0,   // No yaw rotation
            0.0    // No roll
        ];
        
        // Get object name for logging
        let name = world.get_component::<Name>(selected_entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", selected_entity.id()));
        
        messages.push(ConsoleMessage::info(&format!(
            "🔍 Focused on {} (radius: {:.1}, distance: {:.1})",
            name, object_radius, view_distance
        )));
    } else {
        messages.push(ConsoleMessage::info("⚠️ Selected entity has no transform"));
    }
    
    messages
}

/// Scene view panel for 3D scene rendering and manipulation
pub struct SceneViewPanel {
    pub scene_view_active: bool,
    console_messages: Vec<ConsoleMessage>,
}

impl SceneViewPanel {
    pub fn new() -> Self {
        Self {
            scene_view_active: true,
            console_messages: Vec::new(),
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
    ) -> Vec<ConsoleMessage> {
        let mut console_messages = Vec::new();
        
        // Scene view toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.scene_view_active, true, "Scene");
            ui.selectable_value(&mut self.scene_view_active, false, "Game");
            
            ui.separator();
            
            if ui.button("🔍").on_hover_text("Focus on selected (F)").clicked() {
                if let Some(entity) = selected_entity {
                    let messages = focus_on_selected_object(world, entity, scene_navigation);
                    console_messages.extend(messages);
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
        let nav_messages = scene_input::handle_scene_input(
            world,
            ui,
            &response,
            rect,
            scene_navigation,
            gizmo_system,
            selected_entity,
        );
        console_messages.extend(nav_messages);
        
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
                let messages = scene_renderer.draw_scene(
                    world,
                    ui,
                    rect,
                    &response,
                    scene_navigation,
                    selected_entity,
                    play_state,
                );
                console_messages.extend(messages);
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
                    let messages = focus_on_selected_object(world, entity, scene_navigation);
                    console_messages.extend(messages);
                }
            }
        });
        
        console_messages
    }
}