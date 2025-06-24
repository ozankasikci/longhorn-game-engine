// Debug overlay for camera movement visualization

use crate::types::SceneNavigation;
use eframe::egui;

pub fn draw_movement_debug_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    scene_navigation: &SceneNavigation,
) {
    let painter = ui.painter();

    // Draw debug info in top-left corner
    let debug_rect =
        egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), egui::vec2(300.0, 200.0));

    painter.rect_filled(
        debug_rect,
        egui::Rounding::same(5.0),
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
    );

    let text_pos = debug_rect.min + egui::vec2(10.0, 10.0);
    let line_height = 20.0;
    let mut y_offset = 0.0;

    let yaw = scene_navigation.scene_camera_transform.rotation[1];
    let pitch = scene_navigation.scene_camera_transform.rotation[0];

    // Camera info
    painter.text(
        text_pos + egui::vec2(0.0, y_offset),
        egui::Align2::LEFT_TOP,
        format!("Camera Yaw: {:.1}°", yaw.to_degrees()),
        egui::FontId::monospace(14.0),
        egui::Color32::WHITE,
    );
    y_offset += line_height;

    painter.text(
        text_pos + egui::vec2(0.0, y_offset),
        egui::Align2::LEFT_TOP,
        format!("Camera Pitch: {:.1}°", pitch.to_degrees()),
        egui::FontId::monospace(14.0),
        egui::Color32::WHITE,
    );
    y_offset += line_height;

    // Position
    let pos = &scene_navigation.scene_camera_transform.position;
    painter.text(
        text_pos + egui::vec2(0.0, y_offset),
        egui::Align2::LEFT_TOP,
        format!("Position: [{:.1}, {:.1}, {:.1}]", pos[0], pos[1], pos[2]),
        egui::FontId::monospace(14.0),
        egui::Color32::WHITE,
    );
    y_offset += line_height * 1.5;

    // Movement direction indicator
    painter.text(
        text_pos + egui::vec2(0.0, y_offset),
        egui::Align2::LEFT_TOP,
        "Movement Direction:",
        egui::FontId::monospace(14.0),
        egui::Color32::YELLOW,
    );
    y_offset += line_height;

    // Calculate forward direction (matching our movement calculation)
    let forward_x = -yaw.sin() * pitch.cos();
    let forward_y = pitch.sin();
    let forward_z = yaw.cos() * pitch.cos();

    painter.text(
        text_pos + egui::vec2(0.0, y_offset),
        egui::Align2::LEFT_TOP,
        format!(
            "Forward: [{:.2}, {:.2}, {:.2}]",
            forward_x, forward_y, forward_z
        ),
        egui::FontId::monospace(14.0),
        egui::Color32::GREEN,
    );
    y_offset += line_height;

    // Draw compass in bottom-right
    let compass_center = rect.max - egui::vec2(100.0, 100.0);
    let compass_radius = 60.0;

    // Background circle
    painter.circle_filled(
        compass_center,
        compass_radius,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
    );

    painter.circle_stroke(
        compass_center,
        compass_radius,
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );

    // Draw cardinal directions
    painter.text(
        compass_center + egui::vec2(0.0, -compass_radius - 10.0),
        egui::Align2::CENTER_BOTTOM,
        "+Z",
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    painter.text(
        compass_center + egui::vec2(compass_radius + 10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "+X",
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    // Draw camera direction arrow
    let arrow_length = compass_radius * 0.8;
    let arrow_end = compass_center
        + egui::vec2(
            forward_x * arrow_length,
            -forward_z * arrow_length, // Negative because screen Y is down
        );

    painter.arrow(
        compass_center,
        arrow_end - compass_center,
        egui::Stroke::new(3.0, egui::Color32::RED),
    );

    // Label
    painter.text(
        compass_center + egui::vec2(0.0, compass_radius + 20.0),
        egui::Align2::CENTER_TOP,
        "Camera Look Direction",
        egui::FontId::proportional(12.0),
        egui::Color32::GRAY,
    );
}
