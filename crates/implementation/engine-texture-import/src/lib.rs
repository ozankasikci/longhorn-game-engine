use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError as AssetImportError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod compression;
pub mod dds;
pub mod processing;
pub mod tga;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Unsupported texture format")]
    UnsupportedFormat,

    #[error("Invalid texture data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image decoding error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Compression error: {0}")]
    CompressionError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    R8,
    Rg8,
    Rgb8,
    Rgba8,
    R16,
    Rg16,
    Rgb16,
    Rgba16,
    R32F,
    Rg32F,
    Rgb32F,
    Rgba32F,
    BC1,  // DXT1
    BC2,  // DXT3
    BC3,  // DXT5
    BC4,  // RGTC1
    BC5,  // RGTC2
    BC6H, // BPTC float
    BC7,  // BPTC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    BC1,
    BC3,
    BC7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    Nearest,
    Linear,
    Trilinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapMode {
    Repeat,
    MirrorRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureSettings {
    pub generate_mipmaps: bool,
    pub compression: CompressionType,
    pub filter_mode: FilterMode,
    pub wrap_mode: WrapMode,
    pub max_size: Option<u32>,
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            generate_mipmaps: true,
            compression: CompressionType::None,
            filter_mode: FilterMode::Linear,
            wrap_mode: WrapMode::Repeat,
            max_size: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportSettings {
    #[serde(default)]
    pub texture_settings: TextureSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MipmapLevel {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
    pub mipmaps: Vec<MipmapLevel>,
}

pub struct TextureImporter;

impl Default for TextureImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import(
        &self,
        data: &[u8],
        _context: &ImportContext,
    ) -> Result<TextureData, TextureError> {
        // Try to detect format from data
        if data.len() < 8 {
            return Err(TextureError::InvalidData("Data too short".to_string()));
        }

        // Check PNG signature
        if data.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
            return self.import_png(data);
        }

        // Check JPEG signature
        if data.starts_with(&[0xFF, 0xD8]) {
            return self.import_jpeg(data);
        }

        Err(TextureError::UnsupportedFormat)
    }

    fn import_png(&self, data: &[u8]) -> Result<TextureData, TextureError> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)?;
        let rgba = img.to_rgba8();

        Ok(TextureData {
            width: rgba.width(),
            height: rgba.height(),
            format: TextureFormat::Rgba8,
            data: rgba.into_raw(),
            mipmaps: vec![],
        })
    }

    fn import_jpeg(&self, data: &[u8]) -> Result<TextureData, TextureError> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)?;
        let rgba = img.to_rgba8();

        Ok(TextureData {
            width: rgba.width(),
            height: rgba.height(),
            format: TextureFormat::Rgba8,
            data: rgba.into_raw(),
            mipmaps: vec![],
        })
    }
}

#[async_trait]
impl AssetImporter for TextureImporter {
    type Asset = TextureData;

    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "tga", "bmp", "dds"]
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

        // Import texture
        self.import(&data, context)
            .map_err(|e| AssetImportError::ProcessingError(e.to_string()))
    }
}

// Helper function to calculate bounds
pub fn calculate_bounds(texture: &TextureData) -> (u32, u32) {
    (texture.width, texture.height)
}
