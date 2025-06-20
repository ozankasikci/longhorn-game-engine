// 3D projection utilities for gizmos
use eframe::egui;
use glam::{Mat4, Vec3, Vec4};

/// Projects a 3D world position to 2D screen coordinates using proper camera matrices
pub fn world_to_screen(
    world_pos: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_rect: egui::Rect,
) -> Option<egui::Pos2> {
    // Transform world position to clip space
    let world_pos4 = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let clip_pos = projection_matrix * view_matrix * world_pos4;
    
    // Check if behind camera
    if clip_pos.w <= 0.0 {
        return None;
    }
    
    // Perspective divide to get NDC coordinates
    let ndc = Vec3::new(
        clip_pos.x / clip_pos.w,
        clip_pos.y / clip_pos.w,
        clip_pos.z / clip_pos.w,
    );
    
    // Check if within NDC bounds
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
        return None;
    }
    
    // Convert NDC to screen coordinates
    let screen_x = viewport_rect.left() + (ndc.x + 1.0) * 0.5 * viewport_rect.width();
    let screen_y = viewport_rect.top() + (1.0 - ndc.y) * 0.5 * viewport_rect.height();
    
    let result = egui::pos2(screen_x, screen_y);
    
    Some(result)
}

/// Calculate the screen-space direction vector for a world axis
pub fn world_axis_to_screen_direction(
    origin: Vec3,
    axis_direction: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_rect: egui::Rect,
) -> Option<egui::Vec2> {
    // Project origin
    let origin_screen = world_to_screen(origin, view_matrix, projection_matrix, viewport_rect)?;
    
    // Project point along axis
    let axis_end = origin + axis_direction;
    let axis_end_screen = world_to_screen(axis_end, view_matrix, projection_matrix, viewport_rect)?;
    
    // Calculate screen direction
    let screen_dir = axis_end_screen - origin_screen;
    Some(screen_dir)
}

/// Calculate a consistent gizmo size that appears the same regardless of distance
pub fn calculate_gizmo_scale(
    world_pos: Vec3,
    view_matrix: Mat4,
    viewport_height: f32,
) -> f32 {
    // Get camera position from view matrix
    let camera_pos = view_matrix.inverse().w_axis.truncate();
    
    // Calculate distance from camera to gizmo
    let distance = (world_pos - camera_pos).length();
    
    // Scale based on distance and viewport size
    // This ensures gizmo appears roughly the same size on screen
    let base_scale = 0.1; // Base size when distance = 1
    let viewport_factor = viewport_height / 600.0; // Normalize for 600px height
    
    base_scale * distance * viewport_factor
}

/// Convert mouse movement to world-space movement along an axis
pub fn screen_delta_to_world_movement(
    mouse_delta: egui::Vec2,
    screen_axis_direction: egui::Vec2,
    world_axis_direction: Vec3,
    gizmo_scale: f32,
) -> Vec3 {
    // Project mouse movement onto screen axis
    let screen_axis_normalized = screen_axis_direction.normalized();
    let movement_along_axis = mouse_delta.dot(screen_axis_normalized);
    
    // Convert to world movement
    let world_movement = world_axis_direction * movement_along_axis * 0.01 * gizmo_scale;
    world_movement
}

/// Calculate movement for plane constraints (e.g., XY, XZ, YZ planes)
pub fn screen_delta_to_plane_movement(
    mouse_delta: egui::Vec2,
    plane_normal: Vec3,
    view_matrix: Mat4,
    gizmo_scale: f32,
) -> Vec3 {
    // Get camera forward direction
    let camera_forward = -view_matrix.z_axis.truncate().normalize();
    
    // Calculate plane basis vectors
    let u = if plane_normal.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y.cross(plane_normal).normalize()
    };
    let v = plane_normal.cross(u).normalize();
    
    // Project camera forward onto plane to determine movement mapping
    let forward_on_plane = camera_forward - plane_normal * camera_forward.dot(plane_normal);
    let forward_on_plane = forward_on_plane.normalize();
    
    // Map screen axes to plane axes
    let screen_x_in_plane = u * forward_on_plane.dot(u) + v * forward_on_plane.dot(v);
    let screen_y_in_plane = Vec3::Y; // Simplified for now
    
    // Convert mouse movement to world movement
    let world_movement = screen_x_in_plane * mouse_delta.x * 0.01 * gizmo_scale
                       + screen_y_in_plane * -mouse_delta.y * 0.01 * gizmo_scale;
    
    world_movement
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_to_screen_center() {
        let view_matrix = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),  // camera position
            Vec3::ZERO,                 // look at origin
            Vec3::Y,                    // up
        );
        let projection_matrix = Mat4::perspective_rh(
            45.0_f32.to_radians(),      // fov
            1.0,                        // aspect ratio
            0.1,                        // near
            100.0,                      // far
        );
        let viewport = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        
        // Object at origin should appear at screen center
        let screen_pos = world_to_screen(Vec3::ZERO, view_matrix, projection_matrix, viewport);
        assert!(screen_pos.is_some());
        let pos = screen_pos.unwrap();
        assert!((pos.x - 400.0).abs() < 0.1);
        assert!((pos.y - 300.0).abs() < 0.1);
    }
    
    #[test]
    fn test_world_to_screen_behind_camera() {
        let view_matrix = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        let projection_matrix = Mat4::perspective_rh(45.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let viewport = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        
        // Object behind camera should return None
        let screen_pos = world_to_screen(Vec3::new(0.0, 0.0, 10.0), view_matrix, projection_matrix, viewport);
        assert!(screen_pos.is_none());
    }
    
    #[test]
    fn test_calculate_gizmo_scale() {
        let view_matrix = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        
        let scale = calculate_gizmo_scale(Vec3::ZERO, view_matrix, 600.0);
        assert!(scale > 0.0);
        
        // Farther objects should have larger scale
        let far_scale = calculate_gizmo_scale(Vec3::new(0.0, 0.0, -10.0), view_matrix, 600.0);
        assert!(far_scale > scale);
    }
}