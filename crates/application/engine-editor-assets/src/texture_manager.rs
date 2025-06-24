//! Texture asset management

use crate::types::{AssetHandle, AssetHandleGenerator, TextureAsset};
use egui::{TextureId, Vec2};
use std::collections::HashMap;

/// Manages texture assets for the editor
pub struct TextureManager {
    textures: HashMap<AssetHandle, TextureAsset>,
    texture_id_map: HashMap<String, AssetHandle>,
    handle_generator: AssetHandleGenerator,
    default_texture_handle: AssetHandle,
}

impl TextureManager {
    pub fn new() -> Self {
        let mut manager = Self {
            textures: HashMap::new(),
            texture_id_map: HashMap::new(),
            handle_generator: AssetHandleGenerator::new(),
            default_texture_handle: AssetHandle::invalid(),
        };

        // Create default texture
        let default_handle = manager.register_texture(
            "default".to_string(),
            TextureId::default(),
            Vec2::new(64.0, 64.0),
            "builtin:default".to_string(),
        );
        manager.default_texture_handle = default_handle;

        manager
    }

    /// Register a new texture
    pub fn register_texture(
        &mut self,
        name: String,
        id: TextureId,
        size: Vec2,
        path: String,
    ) -> AssetHandle {
        let handle = self.handle_generator.generate();

        let texture = TextureAsset {
            id,
            name: name.clone(),
            size,
            path,
        };

        self.textures.insert(handle, texture);
        self.texture_id_map.insert(name, handle);

        handle
    }

    /// Get a texture by handle
    pub fn get_texture(&self, handle: AssetHandle) -> Option<&TextureAsset> {
        self.textures.get(&handle)
    }

    /// Get a texture by name
    pub fn get_texture_by_name(&self, name: &str) -> Option<&TextureAsset> {
        self.texture_id_map
            .get(name)
            .and_then(|handle| self.textures.get(handle))
    }

    /// Get the default texture
    pub fn get_default_texture(&self) -> &TextureAsset {
        self.textures
            .get(&self.default_texture_handle)
            .expect("Default texture should always exist")
    }

    /// Get all texture handles
    pub fn all_handles(&self) -> Vec<AssetHandle> {
        self.textures.keys().copied().collect()
    }

    /// Remove a texture
    pub fn remove_texture(&mut self, handle: AssetHandle) -> Option<TextureAsset> {
        if handle == self.default_texture_handle {
            return None; // Don't remove the default texture
        }

        if let Some(texture) = self.textures.remove(&handle) {
            self.texture_id_map.remove(&texture.name);
            Some(texture)
        } else {
            None
        }
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

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
        textures.insert(
            handle,
            TextureAsset {
                id: TextureId::default(), // Placeholder for now
                name: name.to_string(),
                size: Vec2::new(64.0, 64.0),
                path: format!("builtin:{}", name.to_lowercase().replace(' ', "_")),
            },
        );
    }

    textures
}
