// Asset management module for the editor

use std::collections::HashMap;
use eframe::egui;
use crate::types::TextureAsset;

/// Creates default colored textures for sprites
pub fn create_default_textures() -> HashMap<u64, TextureAsset> {
    let mut textures = HashMap::new();
    
    // Create basic colored square textures
    let textures_data = [
        (1000, "White Square", [1.0, 1.0, 1.0, 1.0]),
        (1001, "Red Square", [1.0, 0.2, 0.2, 1.0]),
        (1002, "Green Square", [0.2, 1.0, 0.2, 1.0]),
        (1003, "Blue Square", [0.2, 0.2, 1.0, 1.0]),
        (1004, "Yellow Square", [1.0, 1.0, 0.2, 1.0]),
        (1005, "Purple Square", [1.0, 0.2, 1.0, 1.0]),
        (1006, "Cyan Square", [0.2, 1.0, 1.0, 1.0]),
        (1007, "Orange Square", [1.0, 0.5, 0.2, 1.0]),
    ];
    
    for (handle, name, _color) in textures_data {
        textures.insert(handle, TextureAsset {
            id: egui::TextureId::default(), // Placeholder for now
            name: name.to_string(),
            size: egui::Vec2::new(64.0, 64.0),
            path: format!("builtin:{}", name.to_lowercase().replace(' ', "_")),
        });
    }
    
    textures
}