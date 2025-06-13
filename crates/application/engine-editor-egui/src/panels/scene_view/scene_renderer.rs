// Scene rendering implementation - handles the actual 3D scene rendering logic

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material, Light, Visibility};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_renderer_3d::Camera;
use crate::types::{SceneNavigation, SceneTool, GizmoSystem, GizmoComponent, GizmoInteractionState, GizmoAxis};
use crate::editor_state::ConsoleMessage;
use std::collections::HashMap;

/// Renders the 3D scene view content
pub fn draw_simple_scene_view(
    world: &World,
    ui: &mut egui::Ui,
    rect: egui::Rect,
    scene_navigation: &SceneNavigation,
    gizmo_system: &GizmoSystem,
    selected_entity: Option<Entity>,
    last_rendered_entity_count: &mut usize,
) -> Vec<ConsoleMessage> {
    let mut console_messages = Vec::new();
    let painter = ui.painter();
    
    // Get camera position and rotation for view transformation
    let camera_pos = scene_navigation.scene_camera_transform.position;
    let camera_rot = scene_navigation.scene_camera_transform.rotation;
    
    // Draw grid background
    let _grid_size = 50.0;
    let view_center = rect.center();
    
    // Apply camera offset to grid rendering
    let camera_offset_x = -camera_pos[0] * 50.0; // 50 pixels per world unit
    let camera_offset_y = camera_pos[2] * 50.0;  // Z becomes Y in screen space
    
    // Draw grid lines with camera offset
    painter.line_segment(
        [egui::pos2(rect.left(), view_center.y + camera_offset_y), 
         egui::pos2(rect.right(), view_center.y + camera_offset_y)],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
    );
    painter.line_segment(
        [egui::pos2(view_center.x + camera_offset_x, rect.top()), 
         egui::pos2(view_center.x + camera_offset_x, rect.bottom())],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
    );
    
    // Draw scene objects (simplified 2D representation with camera transform)
    // Phase 10.2: Query entities with both Transform AND Mesh components
    // This is the ECS-to-renderer bridge implementation
    let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
    
    // Track entity changes for Phase 10.2
    let current_entity_count = entities_with_transforms.len();
    if current_entity_count != *last_rendered_entity_count {
        console_messages.push(ConsoleMessage::info(&format!(
            "🎮 Phase 10.2: Total entities with Transform: {}",
            current_entity_count
        )));
        *last_rendered_entity_count = current_entity_count;
    }
    
    // TODO: Move the rest of the massive rendering logic here
    // This is just the beginning - the full function needs to be extracted
    
    console_messages
}