// Utility functions for the editor

use eframe::egui;
use engine_components_2d::Sprite;

/// Convert sprite color to EGUI color
pub fn get_sprite_color(sprite: &Sprite) -> egui::Color32 {
    let final_color = sprite.color;
    egui::Color32::from_rgba_unmultiplied(
        (final_color[0] * 255.0) as u8,
        (final_color[1] * 255.0) as u8,
        (final_color[2] * 255.0) as u8,
        (final_color[3] * 255.0) as u8,
    )
}

/// Create a default texture ID pattern
pub fn create_default_texture_id(base_id: u64, color_index: usize) -> u64 {
    base_id + color_index as u64
}
