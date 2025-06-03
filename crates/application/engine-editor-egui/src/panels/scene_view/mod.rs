// Scene view module - handles 3D scene rendering and interaction

pub mod rendering;
pub mod navigation;
pub mod gizmos;
pub mod scene_renderer;
pub mod object_renderer;
pub mod scene_view_impl;
pub mod scene_input;

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Mesh, MeshType, Light, Visibility};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::Camera;
use crate::types::{SceneNavigation, SceneTool, GizmoSystem, GizmoComponent, GizmoInteractionState, GizmoAxis};
use crate::editor_state::ConsoleMessage;
use std::collections::HashMap;

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
        
        // Calculate a good viewing distance based on object scale
        let avg_scale = (transform.scale[0] + transform.scale[1] + transform.scale[2]) / 3.0;
        let view_distance = avg_scale * 5.0 + 3.0; // Base distance of 3 units plus scale factor
        
        // Get current camera rotation to maintain viewing angle
        let camera_rot = scene_navigation.scene_camera_transform.rotation;
        let pitch = camera_rot[0];
        let yaw = camera_rot[1];
        
        // Calculate camera position offset from object
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        
        // Camera looks along -Z, so we position it behind the object along its forward vector
        let camera_offset = [
            sin_yaw * cos_pitch * view_distance,
            -sin_pitch * view_distance,
            cos_yaw * cos_pitch * view_distance,
        ];
        
        // Set new camera position
        scene_navigation.scene_camera_transform.position = [
            object_pos[0] + camera_offset[0],
            object_pos[1] + camera_offset[1],
            object_pos[2] + camera_offset[2],
        ];
        
        // Get object name for logging
        let name = world.get_component::<Name>(selected_entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", selected_entity.id()));
        
        messages.push(ConsoleMessage::info(&format!(
            "🔍 Focused on {} at [{:.1}, {:.1}, {:.1}] (distance: {:.1})",
            name, object_pos[0], object_pos[1], object_pos[2], view_distance
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
        
        // Main view area
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());
        
        // Draw background
        ui.painter().rect_filled(
            response.rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(35, 35, 35)
        );
        
        // Scene content
        ui.allocate_ui_at_rect(response.rect, |ui| {
            if self.scene_view_active {
                // Draw 3D scene
                let messages = scene_renderer.draw_scene(
                    world,
                    ui,
                    response.rect,
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
        
        // Handle gizmo and scene interactions
        let messages = scene_input::handle_scene_input(
            world,
            ui,
            &response,
            response.rect,
            scene_navigation,
            gizmo_system,
            selected_entity,
        );
        console_messages.extend(messages);
        
        console_messages
    }
}