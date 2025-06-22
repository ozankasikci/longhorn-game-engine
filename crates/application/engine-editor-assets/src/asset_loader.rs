//! Asset loading interfaces and implementations

use std::path::Path;
use crate::types::{AssetLoadError, TextureAsset};

/// Trait for loading assets
pub trait AssetLoader: Send + Sync {
    type Asset;
    
    /// Load an asset from the given path
    fn load(&self, path: &Path) -> Result<Self::Asset, AssetLoadError>;
    
    /// Check if this loader can handle the given file extension
    fn supports_extension(&self, extension: &str) -> bool;
}

/// Texture asset loader
pub struct TextureLoader {
    supported_extensions: Vec<String>,
}

impl TextureLoader {
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "bmp".to_string(),
                "tga".to_string(),
                "dds".to_string(),
                "webp".to_string(),
            ],
        }
    }
}

impl Default for TextureLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for TextureLoader {
    type Asset = TextureAsset;
    
    fn load(&self, path: &Path) -> Result<Self::Asset, AssetLoadError> {
        // In a real implementation, this would load the texture from disk
        // For now, we'll create a placeholder
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AssetLoadError::InvalidFormat("Invalid file name".to_string()))?;
        
        Ok(TextureAsset {
            id: egui::TextureId::default(),
            name: file_name.to_string(),
            size: egui::Vec2::new(256.0, 256.0), // Placeholder size
            path: path.to_string_lossy().to_string(),
        })
    }
    
    fn supports_extension(&self, extension: &str) -> bool {
        self.supported_extensions.iter()
            .any(|ext| ext.eq_ignore_ascii_case(extension))
    }
}

/// Asset loader registry
pub struct AssetLoaderRegistry {
    texture_loader: TextureLoader,
}

impl AssetLoaderRegistry {
    pub fn new() -> Self {
        Self {
            texture_loader: TextureLoader::new(),
        }
    }
    
    /// Get the appropriate loader for a file path
    pub fn get_loader_for_path(&self, path: &Path) -> Option<&dyn AssetLoader<Asset = TextureAsset>> {
        let extension = path.extension()?.to_str()?;
        
        if self.texture_loader.supports_extension(extension) {
            Some(&self.texture_loader)
        } else {
            None
        }
    }
}

impl Default for AssetLoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}