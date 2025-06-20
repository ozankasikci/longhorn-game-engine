// 2D gizmo overlay rendering
use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use crate::types::{GizmoSystem, SceneTool};
use glam::{Mat4, Vec3};

/// Draw 2D gizmo overlay on top of the 3D scene
pub fn draw_gizmo_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    world: &World,
    selected_entity: Option<Entity>,
    gizmo_system: &GizmoSystem,
    camera_view_matrix: Option<Mat4>,
    camera_projection_matrix: Option<Mat4>,
) {
    let Some(entity) = selected_entity else { return };
    let Some(transform) = world.get_component::<Transform>(entity) else { return };
    
    // Only draw for move tool
    if gizmo_system.get_active_tool() != SceneTool::Move {
        return;
    }
    
    // Get gizmo screen position
    let gizmo_center = if let (Some(view_mat), Some(proj_mat)) = (camera_view_matrix, camera_projection_matrix) {
        let world_pos = Vec3::from_array(transform.position);
        super::gizmo_3d_projection::world_to_screen(world_pos, view_mat, proj_mat, rect)
            .unwrap_or_else(|| rect.center())
    } else {
        // Fallback to simple 2D projection
        let scene_center = rect.center();
        let scale = 50.0;
        egui::pos2(
            scene_center.x + transform.position[0] * scale,
            scene_center.y - transform.position[2] * scale
        )
    };
    
    let painter = ui.painter();
    let axis_length = 80.0;
    let arrow_size = 10.0;
    
    // Draw X axis (red)
    let x_end = gizmo_center + egui::vec2(axis_length, 0.0);
    painter.line_segment(
        [gizmo_center, x_end],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0))
    );
    // X axis arrow
    painter.line_segment(
        [x_end, x_end + egui::vec2(-arrow_size, arrow_size/2.0)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0))
    );
    painter.line_segment(
        [x_end, x_end + egui::vec2(-arrow_size, -arrow_size/2.0)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0))
    );
    
    // Draw Y axis (green) - points up
    let y_end = gizmo_center + egui::vec2(0.0, -axis_length);
    painter.line_segment(
        [gizmo_center, y_end],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0))
    );
    // Y axis arrow
    painter.line_segment(
        [y_end, y_end + egui::vec2(arrow_size/2.0, arrow_size)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0))
    );
    painter.line_segment(
        [y_end, y_end + egui::vec2(-arrow_size/2.0, arrow_size)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0))
    );
    
    // Draw Z axis (blue) - diagonal
    let z_end = gizmo_center + egui::vec2(-axis_length * 0.7, axis_length * 0.7);
    painter.line_segment(
        [gizmo_center, z_end],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 255))
    );
    // Z axis arrow
    let z_arrow_dir = egui::vec2(0.7, -0.7).normalized() * arrow_size;
    painter.line_segment(
        [z_end, z_end + z_arrow_dir + egui::vec2(-arrow_size/3.0, 0.0)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 255))
    );
    painter.line_segment(
        [z_end, z_end + z_arrow_dir + egui::vec2(0.0, arrow_size/3.0)],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 255))
    );
    
    // Draw center sphere
    painter.circle_filled(
        gizmo_center,
        6.0,
        egui::Color32::from_rgb(255, 255, 255)
    );
    
    // Highlight based on interaction state
    if let Some(gizmo) = gizmo_system.get_move_gizmo() {
        use crate::types::{GizmoInteractionState, GizmoComponent, GizmoAxis};
        
        match gizmo.get_interaction_state() {
            GizmoInteractionState::Hovering(component) | 
            GizmoInteractionState::Dragging { component, .. } => {
                match component {
                    GizmoComponent::Axis(GizmoAxis::X) => {
                        painter.line_segment(
                            [gizmo_center, x_end],
                            egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 0))
                        );
                    }
                    GizmoComponent::Axis(GizmoAxis::Y) => {
                        painter.line_segment(
                            [gizmo_center, y_end],
                            egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 0))
                        );
                    }
                    GizmoComponent::Axis(GizmoAxis::Z) => {
                        painter.line_segment(
                            [gizmo_center, z_end],
                            egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 255, 0))
                        );
                    }
                    GizmoComponent::Center => {
                        painter.circle_filled(
                            gizmo_center,
                            8.0,
                            egui::Color32::from_rgb(255, 255, 0)
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}