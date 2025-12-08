use super::{renderer::GizmoConfig, types::{GizmoHandle, GizmoMode}};
use glam::Vec2;
use longhorn_core::Transform;

/// Check if a point is near a line segment
fn point_near_line(point: Vec2, line_start: Vec2, line_end: Vec2, threshold: f32) -> bool {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    // Project point onto line
    let line_len_sq = line_vec.length_squared();
    if line_len_sq == 0.0 {
        return false;
    }

    let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;

    // Check distance from point to projection
    (point - projection).length() < threshold
}

/// Check if a point is inside a rectangle
fn point_in_rect(point: Vec2, center: Vec2, half_size: f32) -> bool {
    let diff = (point - center).abs();
    diff.x < half_size && diff.y < half_size
}

/// Perform hit testing to find which gizmo handle (if any) is under the mouse
pub fn hit_test_gizmo(
    mouse_pos: Vec2,
    screen_pos: Vec2,
    mode: GizmoMode,
    config: &GizmoConfig,
) -> Option<GizmoHandle> {
    match mode {
        GizmoMode::None => None,
        GizmoMode::Move => hit_test_move_gizmo(mouse_pos, screen_pos, config),
        GizmoMode::Scale => hit_test_scale_gizmo(mouse_pos, screen_pos, config),
        GizmoMode::Rotate => {
            // TODO: Implement for rotate
            None
        }
    }
}

/// Hit test for move gizmo handles
fn hit_test_move_gizmo(
    mouse_pos: Vec2,
    screen_pos: Vec2,
    config: &GizmoConfig,
) -> Option<GizmoHandle> {
    let hit_threshold = 8.0; // Pixels
    let center_half_size = config.center_handle_size / 2.0;

    // Check center square first (highest priority)
    // Use a much larger hit area for better UX (25 pixels radius = 50x50 square)
    let center_hit_size = center_half_size.max(25.0);
    if point_in_rect(mouse_pos, screen_pos, center_hit_size) {
        return Some(GizmoHandle::MoveXY);
    }

    // Check X-axis arrow (starting from outside the center area)
    let x_start = screen_pos + Vec2::new(center_hit_size, 0.0);
    let x_end = screen_pos + Vec2::new(config.arrow_length, 0.0);
    if point_near_line(mouse_pos, x_start, x_end, hit_threshold) {
        return Some(GizmoHandle::MoveX);
    }

    // Check Y-axis arrow (starting from outside the center area, negative Y for screen coords)
    let y_start = screen_pos - Vec2::new(0.0, center_hit_size);
    let y_end = screen_pos - Vec2::new(0.0, config.arrow_length);
    if point_near_line(mouse_pos, y_start, y_end, hit_threshold) {
        return Some(GizmoHandle::MoveY);
    }

    None
}

/// Hit test for scale gizmo handles
fn hit_test_scale_gizmo(
    mouse_pos: Vec2,
    screen_pos: Vec2,
    config: &GizmoConfig,
) -> Option<GizmoHandle> {
    let cube_half_size = config.scale_cube_size / 2.0;
    let center_half_size = config.center_handle_size / 2.0;

    // Check center cube first (highest priority for uniform scaling)
    if point_in_rect(mouse_pos, screen_pos, center_half_size) {
        return Some(GizmoHandle::ScaleXY);
    }

    // Check X-axis cube at end of X axis
    let x_cube_pos = screen_pos + Vec2::new(config.arrow_length, 0.0);
    if point_in_rect(mouse_pos, x_cube_pos, cube_half_size) {
        return Some(GizmoHandle::ScaleX);
    }

    // Check Y-axis cube at end of Y axis (negative Y for screen coords)
    let y_cube_pos = screen_pos - Vec2::new(0.0, config.arrow_length);
    if point_in_rect(mouse_pos, y_cube_pos, cube_half_size) {
        return Some(GizmoHandle::ScaleY);
    }

    None
}

/// Calculate new transform based on drag delta
pub fn update_transform_from_drag(
    handle: GizmoHandle,
    drag_start_transform: Transform,
    world_delta: Vec2, // Already in world space
) -> Transform {
    let mut new_transform = drag_start_transform;

    match handle {
        // Move gizmo handles
        GizmoHandle::MoveX => {
            // Only move horizontally
            new_transform.position.x += world_delta.x;
        }
        GizmoHandle::MoveY => {
            // Only move vertically
            new_transform.position.y += world_delta.y;
        }
        GizmoHandle::MoveXY => {
            // Free movement
            new_transform.position.x += world_delta.x;
            new_transform.position.y += world_delta.y;
        }

        // Scale gizmo handles
        GizmoHandle::ScaleX => {
            // Scale horizontally based on X delta
            // Use a multiplier for more intuitive scaling (0.01 = 1% per pixel)
            let scale_factor = 1.0 + (world_delta.x * 0.01);
            new_transform.scale.x = (drag_start_transform.scale.x * scale_factor).max(0.01);
        }
        GizmoHandle::ScaleY => {
            // Scale vertically based on Y delta (inverted because screen Y is flipped)
            let scale_factor = 1.0 - (world_delta.y * 0.01);
            new_transform.scale.y = (drag_start_transform.scale.y * scale_factor).max(0.01);
        }
        GizmoHandle::ScaleXY => {
            // Uniform scaling based on combined delta magnitude
            // Use the larger absolute component for more responsive scaling
            let delta_magnitude = world_delta.x.abs().max(world_delta.y.abs());
            let direction = if world_delta.x + world_delta.y > 0.0 { 1.0 } else { -1.0 };
            let scale_factor = 1.0 + (delta_magnitude * direction * 0.01);

            new_transform.scale.x = (drag_start_transform.scale.x * scale_factor).max(0.01);
            new_transform.scale.y = (drag_start_transform.scale.y * scale_factor).max(0.01);
        }

        _ => {
            // TODO: Implement rotate handles
        }
    }

    new_transform
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_near_line() {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 0.0);

        // Point on the line
        assert!(point_near_line(Vec2::new(50.0, 0.0), start, end, 5.0));

        // Point near the line
        assert!(point_near_line(Vec2::new(50.0, 3.0), start, end, 5.0));

        // Point far from the line
        assert!(!point_near_line(Vec2::new(50.0, 10.0), start, end, 5.0));
    }

    #[test]
    fn test_point_in_rect() {
        let center = Vec2::new(100.0, 100.0);
        let half_size = 10.0;

        // Point inside
        assert!(point_in_rect(Vec2::new(105.0, 105.0), center, half_size));

        // Point outside
        assert!(!point_in_rect(Vec2::new(120.0, 120.0), center, half_size));
    }
}
