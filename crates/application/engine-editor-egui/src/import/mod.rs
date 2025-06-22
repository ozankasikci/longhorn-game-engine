use std::path::PathBuf;
use engine_resource_core::ResourceId;
use serde::{Serialize, Deserialize};

pub mod dialog;
mod preview;
mod drag_drop;
mod progress;
mod error;
mod history;
mod batch;
mod hot_reload;
pub mod service;
pub mod file_watcher;
pub mod wrappers;

#[cfg(test)]
mod tests;

pub use dialog::{ImportDialog, ImportSettings, CollisionType, ImportResult};
pub use preview::{ImportPreview, PreviewData};
pub use drag_drop::{DragDropHandler, FileType};
pub use progress::{ImportProgress, ImportTask};
pub use error::{ImportError as ImportErr, ImportErrorDialog, ImportErrorType};
pub use history::{ImportHistory, ImportRecord};
pub use batch::{BatchImporter, BatchImportOptions};
pub use hot_reload::{HotReloadWatcher, HotReloadEvent, HotReloadAction};
pub use service::{ImportService, ImportHandle, ImportStatus, ImportError, ImportNotification, ImportQueue, ImportSettingsConverter, ImportUIState};
pub use file_watcher::{ImportFileWatcher, FileWatchEvent};
pub use wrappers::{
    MeshImporterWrapper, TextureImporterWrapper, AudioImporterWrapper,
    ObjImporterWrapper, StandardTextureImporterWrapper, StandardAudioImporterWrapper,
    SerializedAssetData
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Audio,
    Animation,
    Material,
    Shader,
    Script,
    Other,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "obj" | "gltf" | "glb" | "dae" | "3ds" => Some(AssetType::Mesh),
            "fbx" => Some(AssetType::Mesh), // FBX can be mesh or animation, default to mesh
            "png" | "jpg" | "jpeg" | "tga" | "bmp" | "dds" => Some(AssetType::Texture),
            "wav" | "mp3" | "ogg" | "flac" => Some(AssetType::Audio),
            "anim" => Some(AssetType::Animation),
            "mat" | "material" => Some(AssetType::Material),
            "glsl" | "hlsl" | "shader" => Some(AssetType::Shader),
            "lua" | "py" | "js" => Some(AssetType::Script),
            _ => None,
        }
    }
}