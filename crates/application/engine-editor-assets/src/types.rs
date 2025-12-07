//! Core types for asset management

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// File type categorization for project panel display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Script,
    Text,
    Image,
    Audio,
    Scene,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext {
            Some("ts") | Some("js") | Some("cs") => FileType::Script,
            Some("json") | Some("txt") | Some("md") | Some("html") | Some("css") | Some("toml") | Some("yaml") | Some("yml") => FileType::Text,
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif") | Some("bmp") => FileType::Image,
            Some("wav") | Some("mp3") | Some("ogg") | Some("flac") => FileType::Audio,
            Some("scene") => FileType::Scene,
            _ => FileType::Unknown,
        }
    }

    /// Returns true if this file type should be opened in the script editor
    pub fn is_text_editable(&self) -> bool {
        matches!(self, FileType::Script | FileType::Text | FileType::Scene)
    }

    /// Get the color for this file type
    pub fn icon_color(&self) -> [u8; 3] {
        match self {
            FileType::Script => [100, 150, 255],   // Blue
            FileType::Text => [150, 150, 150],     // Gray
            FileType::Image => [100, 200, 100],    // Green
            FileType::Audio => [200, 100, 200],    // Purple
            FileType::Scene => [255, 150, 50],     // Orange
            FileType::Unknown => [128, 128, 128],  // Dark gray
        }
    }

    /// Get the icon character for this file type
    pub fn icon_char(&self) -> &'static str {
        match self {
            FileType::Script => "📜",
            FileType::Text => "📄",
            FileType::Image => "🖼",
            FileType::Audio => "🎵",
            FileType::Scene => "🎬",
            FileType::Unknown => "📦",
        }
    }
}

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
    vec![ProjectAsset::folder(
        "Assets",
        vec![
            ProjectAsset::folder(
                "Scripts",
                vec![
                    ProjectAsset::file("PlayerController.cs"),
                    ProjectAsset::file("GameManager.cs"),
                    ProjectAsset::file("UIController.cs"),
                ],
            ),
            ProjectAsset::folder(
                "Materials",
                vec![
                    ProjectAsset::file("DefaultMaterial.mat"),
                    ProjectAsset::file("WoodTexture.mat"),
                    ProjectAsset::file("MetalSurface.mat"),
                ],
            ),
            ProjectAsset::folder(
                "Textures",
                vec![
                    ProjectAsset::file("grass.png"),
                    ProjectAsset::file("brick_wall.jpg"),
                    ProjectAsset::file("sky_gradient.png"),
                ],
            ),
            ProjectAsset::folder(
                "Models",
                vec![
                    ProjectAsset::file("character.fbx"),
                    ProjectAsset::file("tree.obj"),
                    ProjectAsset::file("building.gltf"),
                ],
            ),
            ProjectAsset::folder(
                "Audio",
                vec![
                    ProjectAsset::file("bgm_main.ogg"),
                    ProjectAsset::file("sfx_jump.wav"),
                    ProjectAsset::file("sfx_collect.wav"),
                ],
            ),
        ],
    )]
}
