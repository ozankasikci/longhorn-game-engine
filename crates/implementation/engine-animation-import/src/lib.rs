use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError as AssetImportError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod compression;
pub mod processing;
pub mod retargeting;
pub mod validation;

#[derive(Error, Debug)]
pub enum AnimationError {
    #[error("Unsupported animation format")]
    UnsupportedFormat,

    #[error("Invalid animation data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationFormat {
    GlTF,
    FBX,
    Collada,
    Custom,
    Compressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Step,
    Cubic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyType {
    Position,
    Rotation,
    Scale,
    BlendShape(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: Vec<f32>,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub target_node: String,
    pub property: PropertyType,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub name: String,
    pub duration_seconds: f32,
    pub channels: Vec<Channel>,
    pub format: AnimationFormat,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compressed_data: Option<Vec<u8>>,
}

impl AnimationData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            duration_seconds: 0.0,
            channels: Vec::new(),
            format: AnimationFormat::Custom,
            compressed_data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    pub optimize_keyframes: bool,
    pub compression_tolerance: f32,
    pub target_fps: Option<f32>,
    pub import_bone_animations: bool,
    pub import_blend_shapes: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            optimize_keyframes: true,
            compression_tolerance: 0.001,
            target_fps: None,
            import_bone_animations: true,
            import_blend_shapes: true,
        }
    }
}

pub struct AnimationImporter;

impl Default for AnimationImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import(
        &self,
        data: &[u8],
        _context: &ImportContext,
    ) -> Result<AnimationData, AnimationError> {
        // Try to detect format from data
        if data.len() < 16 {
            return Err(AnimationError::InvalidData("Data too short".to_string()));
        }

        // Check glTF (JSON or binary)
        if data.starts_with(b"{") || data.starts_with(b"glTF") {
            return Err(AnimationError::UnsupportedFormat);
        }

        // Check FBX
        if data.starts_with(b"Kaydara FBX Binary") {
            return Err(AnimationError::UnsupportedFormat);
        }

        // Check Collada/DAE (XML)
        if data.starts_with(b"<?xml") || data.starts_with(b"<COLLADA") {
            return Err(AnimationError::UnsupportedFormat);
        }

        Err(AnimationError::UnsupportedFormat)
    }
}

#[async_trait]
impl AssetImporter for AnimationImporter {
    type Asset = AnimationData;

    fn supported_extensions(&self) -> &[&str] {
        &["gltf", "glb", "fbx", "dae"]
    }

    async fn import(
        &self,
        path: &Path,
        context: &ImportContext,
    ) -> Result<Self::Asset, AssetImportError> {
        // Read file data
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| AssetImportError::IoError(e.to_string()))?;

        // Import animation
        self.import(&data, context)
            .map_err(|e| AssetImportError::ProcessingError(e.to_string()))
    }
}
