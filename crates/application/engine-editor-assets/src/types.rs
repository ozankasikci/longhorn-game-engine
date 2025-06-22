//! Core types for asset management

use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Texture asset for displaying in editor
#[derive(Clone, Debug)]
pub struct TextureAsset {
    pub id: egui::TextureId,
    pub name: String,
    pub size: egui::Vec2,
    pub path: String,
}

/// Project asset representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectAsset {
    pub name: String,
    pub children: Option<Vec<ProjectAsset>>,
}

impl ProjectAsset {
    pub fn file(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: None,
        }
    }
    
    pub fn folder(name: &str, children: Vec<ProjectAsset>) -> Self {
        Self {
            name: name.to_string(),
            children: Some(children),
        }
    }
}

/// Asset handle for referencing loaded assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetHandle(u64);

impl AssetHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn invalid() -> Self {
        Self(0)
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
    
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Asset handle generator for creating unique handles
pub struct AssetHandleGenerator {
    next_id: AtomicU64,
}

impl AssetHandleGenerator {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1), // Start at 1, 0 is reserved for invalid
        }
    }
    
    pub fn generate(&mut self) -> AssetHandle {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        AssetHandle::new(id)
    }
}

impl Default for AssetHandleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset loading errors
#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    
    #[error("Failed to load asset: {0}")]
    LoadFailed(String),
    
    #[error("Invalid asset format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Creates default project assets for the editor UI
pub fn create_default_project_assets() -> Vec<ProjectAsset> {
    vec![
        ProjectAsset::folder("Assets", vec![
            ProjectAsset::folder("Scripts", vec![
                ProjectAsset::file("PlayerController.cs"),
                ProjectAsset::file("GameManager.cs"),
                ProjectAsset::file("UIController.cs"),
            ]),
            ProjectAsset::folder("Materials", vec![
                ProjectAsset::file("DefaultMaterial.mat"),
                ProjectAsset::file("WoodTexture.mat"),
                ProjectAsset::file("MetalSurface.mat"),
            ]),
            ProjectAsset::folder("Textures", vec![
                ProjectAsset::file("grass.png"),
                ProjectAsset::file("brick_wall.jpg"),
                ProjectAsset::file("sky_gradient.png"),
            ]),
            ProjectAsset::folder("Models", vec![
                ProjectAsset::file("character.fbx"),
                ProjectAsset::file("tree.obj"),
                ProjectAsset::file("building.gltf"),
            ]),
            ProjectAsset::folder("Audio", vec![
                ProjectAsset::file("bgm_main.ogg"),
                ProjectAsset::file("sfx_jump.wav"),
                ProjectAsset::file("sfx_collect.wav"),
            ]),
        ])
    ]
}