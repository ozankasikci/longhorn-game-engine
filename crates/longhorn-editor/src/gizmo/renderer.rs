use super::types::{GizmoHandle, GizmoMode};
use egui::{Color32, Painter, Pos2, Stroke};
use glam::Vec2;

/// Configuration for gizmo visual appearance
pub struct GizmoConfig {
    pub arrow_length: f32,
    pub arrow_thickness: f32,
    pub center_handle_size: f32,
    pub tip_length: f32,
    pub tip_width: f32,
    pub scale_cube_size: f32,

    // Colors
    pub color_x_axis: Color32,
    pub color_y_axis: Color32,
    pub color_center: Color32,
    pub color_hover: Color32,
    pub color_active: Color32,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            arrow_length: 60.0,
            arrow_thickness: 3.0,
            center_handle_size: 12.0,
            tip_length: 12.0,
            tip_width: 6.0,
            scale_cube_size: 10.0,

            // Unity-style colors
            color_x_axis: Color32::from_rgb(220, 50, 50),    // Red for X
            color_y_axis: Color32::from_rgb(80, 200, 80),    // Green for Y
            color_center: Color32::from_rgb(200, 200, 200),  // Gray for center
            color_hover: Color32::from_rgb(255, 220, 100),   // Yellow on hover
            color_active: Color32::from_rgb(255, 255, 255),  // White when dragging
        }
    }
}

/// Helper to convert Vec2 to egui Pos2
fn to_pos2(v: Vec2) -> Pos2 {
    Pos2::new(v.x, v.y)
}

/// Draw a 2D arrow (inspired by Bevy's implementation)
fn draw_arrow_2d(
    painter: &Painter,
    start: Vec2,
    end: Vec2,
    color: Color32,
    thickness: f32,
    tip_length: f32,
    tip_width: f32,
) {
    // Draw arrow shaft
    painter.line_segment(
        [to_pos2(start), to_pos2(end)],
        Stroke::new(thickness, color),
    );

    // Calculate arrowhead
    let dir = (end - start).normalize();
    let perpendicular = Vec2::new(-dir.y, dir.x);

    let tip1 = end - dir * tip_length + perpendicular * tip_width;
    let tip2 = end - dir * tip_length - perpendicular * tip_width;

    // Draw arrowhead lines
    painter.line_segment([to_pos2(end), to_pos2(tip1)], Stroke::new(thickness, color));
    painter.line_segment([to_pos2(end), to_pos2(tip2)], Stroke::new(thickness, color));
}

/// Helper to get color based on hover/active state
fn get_handle_color(
    handle: GizmoHandle,
    default_color: Color32,
    hover_handle: Option<GizmoHandle>,
    active_handle: Option<GizmoHandle>,
    config: &GizmoConfig,
) -> Color32 {
    if Some(handle) == active_handle {
        config.color_active
    } else if Some(handle) == hover_handle {
        config.color_hover
    } else {
        default_color
    }
}

/// Draw the move gizmo at the given screen position
pub fn draw_move_gizmo(
    painter: &Painter,
    config: &GizmoConfig,
    screen_pos: Vec2,
    hover_handle: Option<GizmoHandle>,
    active_handle: Option<GizmoHandle>,
) {
    // Draw X-axis arrow (horizontal right)
    let x_color = get_handle_color(GizmoHandle::MoveX, config.color_x_axis, hover_handle, active_handle, config);
    let x_end = screen_pos + Vec2::new(config.arrow_length, 0.0);
    draw_arrow_2d(
        painter,
        screen_pos,
        x_end,
        x_color,
        config.arrow_thickness,
        config.tip_length,
        config.tip_width,
    );

    // Draw Y-axis arrow (vertical up - note: screen Y is down, so we use negative)
    let y_color = get_handle_color(GizmoHandle::MoveY, config.color_y_axis, hover_handle, active_handle, config);
    let y_end = screen_pos - Vec2::new(0.0, config.arrow_length); // Negative Y for screen coords
    draw_arrow_2d(
        painter,
        screen_pos,
        y_end,
        y_color,
        config.arrow_thickness,
        config.tip_length,
        config.tip_width,
    );

    // Draw center square for XY movement
    let center_color = get_handle_color(GizmoHandle::MoveXY, config.color_center, hover_handle, active_handle, config);
    let rect = egui::Rect::from_center_size(
        to_pos2(screen_pos),
        egui::Vec2::splat(config.center_handle_size),
    );
    painter.rect_filled(rect, 0.0, center_color);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::BLACK));
}

/// Draw the scale gizmo at the given screen position
/// Unity-style: axis lines with cubes at the ends + center cube for uniform scaling
pub fn draw_scale_gizmo(
    painter: &Painter,
    config: &GizmoConfig,
    screen_pos: Vec2,
    hover_handle: Option<GizmoHandle>,
    active_handle: Option<GizmoHandle>,
) {
    // Draw X-axis line (horizontal right)
    let x_color = get_handle_color(GizmoHandle::ScaleX, config.color_x_axis, hover_handle, active_handle, config);
    let x_end = screen_pos + Vec2::new(config.arrow_length, 0.0);
    painter.line_segment(
        [to_pos2(screen_pos), to_pos2(x_end)],
        Stroke::new(config.arrow_thickness, x_color),
    );

    // Draw X-axis cube at end
    let x_cube_rect = egui::Rect::from_center_size(
        to_pos2(x_end),
        egui::Vec2::splat(config.scale_cube_size),
    );
    painter.rect_filled(x_cube_rect, 0.0, x_color);
    painter.rect_stroke(x_cube_rect, 0.0, Stroke::new(1.0, Color32::BLACK));

    // Draw Y-axis line (vertical up - negative Y for screen coords)
    let y_color = get_handle_color(GizmoHandle::ScaleY, config.color_y_axis, hover_handle, active_handle, config);
    let y_end = screen_pos - Vec2::new(0.0, config.arrow_length);
    painter.line_segment(
        [to_pos2(screen_pos), to_pos2(y_end)],
        Stroke::new(config.arrow_thickness, y_color),
    );

    // Draw Y-axis cube at end
    let y_cube_rect = egui::Rect::from_center_size(
        to_pos2(y_end),
        egui::Vec2::splat(config.scale_cube_size),
    );
    painter.rect_filled(y_cube_rect, 0.0, y_color);
    painter.rect_stroke(y_cube_rect, 0.0, Stroke::new(1.0, Color32::BLACK));

    // Draw center cube for uniform (XY) scaling
    let center_color = get_handle_color(GizmoHandle::ScaleXY, config.color_center, hover_handle, active_handle, config);
    let center_rect = egui::Rect::from_center_size(
        to_pos2(screen_pos),
        egui::Vec2::splat(config.center_handle_size),
    );
    painter.rect_filled(center_rect, 0.0, center_color);
    painter.rect_stroke(center_rect, 0.0, Stroke::new(1.0, Color32::BLACK));
}

/// Main drawing function - draws the appropriate gizmo based on mode
pub fn draw_gizmo(
    painter: &Painter,
    config: &GizmoConfig,
    mode: GizmoMode,
    screen_pos: Vec2,
    hover_handle: Option<GizmoHandle>,
    active_handle: Option<GizmoHandle>,
) {
    match mode {
        GizmoMode::None => {}
        GizmoMode::Move => {
            draw_move_gizmo(painter, config, screen_pos, hover_handle, active_handle);
        }
        GizmoMode::Scale => {
            draw_scale_gizmo(painter, config, screen_pos, hover_handle, active_handle);
        }
        GizmoMode::Rotate => {
            // TODO: Implement rotate gizmo
        }
    }
}
