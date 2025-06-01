//! Asset type definitions

use serde::{Deserialize, Serialize};

/// Unique identifier for an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

/// Asset type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Model,
    Audio,
    Script,
    Scene,
    Font,
    Shader,
    Material,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: AssetId,
    pub asset_type: AssetType,
    pub path: String,
    pub size: u64,
    pub checksum: String,
}