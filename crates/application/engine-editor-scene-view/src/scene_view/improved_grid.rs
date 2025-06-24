// Improved grid rendering with proper clipping and dynamic sizing

use eframe::egui;

pub struct GridLevel {
    pub spacing: f32,
    pub extent: f32,
    pub minor_alpha: f32,
    pub major_alpha: f32,
    pub major_interval: i32,
}

/// Get appropriate grid level based on camera height
pub fn get_grid_level(camera_height: f32) -> GridLevel {
    let height = camera_height.abs();

    if height < 10.0 {
        // Close view - fine grid
        GridLevel {
            spacing: 1.0,
            extent: 50.0,
            minor_alpha: 0.4,
            major_alpha: 0.7,
            major_interval: 10,
        }
    } else if height < 50.0 {
        // Medium view
        GridLevel {
            spacing: 5.0,
            extent: 200.0,
            minor_alpha: 0.3,
            major_alpha: 0.6,
            major_interval: 10,
        }
    } else if height < 200.0 {
        // Far view
        GridLevel {
            spacing: 10.0,
            extent: 500.0,
            minor_alpha: 0.25,
            major_alpha: 0.5,
            major_interval: 10,
        }
    } else {
        // Very far view
        GridLevel {
            spacing: 50.0,
            extent: 2000.0,
            minor_alpha: 0.2,
            major_alpha: 0.4,
            major_interval: 10,
        }
    }
}

/// Calculate line opacity based on distance with smooth fade
pub fn calculate_line_opacity(distance: f32, base_alpha: f32, camera_height: f32) -> f32 {
    // Dynamic fade distances based on camera height
    let fade_start = camera_height.abs() * 2.0;
    let fade_end = camera_height.abs() * 10.0;

    if distance < fade_start {
        base_alpha
    } else if distance > fade_end {
        0.0
    } else {
        let t = (distance - fade_start) / (fade_end - fade_start);
        // Smooth fade using ease-out curve
        let fade = 1.0 - (t * t);
        base_alpha * fade
    }
}

/// Clip a line segment to the near plane
pub fn clip_line_to_near_plane(
    start_world: [f32; 3],
    end_world: [f32; 3],
    start_depth: f32,
    end_depth: f32,
    near_threshold: f32,
) -> Option<([f32; 3], [f32; 3], f32, f32)> {
    // If both points are in front of near plane, return as-is
    if start_depth > near_threshold && end_depth > near_threshold {
        return Some((start_world, end_world, start_depth, end_depth));
    }

    // If both points are behind near plane, cull the line
    if start_depth <= near_threshold && end_depth <= near_threshold {
        return None;
    }

    // Line crosses near plane - clip it
    let t = (near_threshold - start_depth) / (end_depth - start_depth);
    let clip_point = [
        start_world[0] + t * (end_world[0] - start_world[0]),
        start_world[1] + t * (end_world[1] - start_world[1]),
        start_world[2] + t * (end_world[2] - start_world[2]),
    ];

    if start_depth > near_threshold {
        // Start is visible, end is clipped
        Some((start_world, clip_point, start_depth, near_threshold))
    } else {
        // Start is clipped, end is visible
        Some((clip_point, end_world, near_threshold, end_depth))
    }
}

/// Check if a screen-space line segment is within reasonable bounds
pub fn is_line_in_bounds(
    start_screen: egui::Pos2,
    end_screen: egui::Pos2,
    rect: egui::Rect,
    margin: f32,
) -> bool {
    let expanded_rect = rect.expand(margin);

    // Check if either endpoint is in bounds
    if expanded_rect.contains(start_screen) || expanded_rect.contains(end_screen) {
        return true;
    }

    // Check if line crosses the rectangle
    // This is a simplified check - could be more precise
    let line_bounds = egui::Rect::from_two_pos(start_screen, end_screen);
    expanded_rect.intersects(line_bounds)
}

/// Get grid line color and width based on type and distance
pub fn get_line_style(
    grid_coord: i32,
    is_x_axis: bool,
    distance: f32,
    level: &GridLevel,
    camera_height: f32,
) -> (f32, egui::Color32) {
    if grid_coord == 0 {
        // Axis lines
        let base_color = if is_x_axis {
            egui::Color32::from_rgba_unmultiplied(200, 100, 100, 255) // Red for X
        } else {
            egui::Color32::from_rgba_unmultiplied(100, 100, 200, 255) // Blue for Z
        };

        let alpha = calculate_line_opacity(distance, 1.0, camera_height);
        let color = egui::Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (alpha * 255.0) as u8,
        );

        (2.0, color)
    } else if grid_coord % level.major_interval == 0 {
        // Major grid lines
        let alpha = calculate_line_opacity(distance, level.major_alpha, camera_height);
        let color = egui::Color32::from_rgba_unmultiplied(120, 120, 120, (alpha * 255.0) as u8);

        (1.0, color)
    } else {
        // Minor grid lines
        let alpha = calculate_line_opacity(distance, level.minor_alpha, camera_height);
        let color = egui::Color32::from_rgba_unmultiplied(80, 80, 80, (alpha * 255.0) as u8);

        (0.5, color)
    }
}
