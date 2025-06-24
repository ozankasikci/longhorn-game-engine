use engine_resource_core::ResourceId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod batch;
pub mod dialog;
mod drag_drop;
mod error;
pub mod file_watcher;
mod history;
mod hot_reload;
mod preview;
mod progress;
pub mod service;
pub mod wrappers;

#[cfg(test)]
mod tests;

pub use batch::{BatchImportOptions, BatchImporter};
pub use dialog::{CollisionType, ImportDialog, ImportResult, ImportSettings};
pub use drag_drop::{DragDropHandler, FileType};
pub use error::{ImportError as ImportErr, ImportErrorDialog, ImportErrorType};
pub use file_watcher::{FileWatchEvent, ImportFileWatcher};
pub use history::{ImportHistory, ImportRecord};
pub use hot_reload::{HotReloadAction, HotReloadEvent, HotReloadWatcher};
pub use preview::{ImportPreview, PreviewData};
pub use progress::{ImportProgress, ImportTask};
pub use service::{
    ImportError, ImportHandle, ImportNotification, ImportQueue, ImportService,
    ImportSettingsConverter, ImportStatus, ImportUIState,
};
pub use wrappers::{
    AudioImporterWrapper, MeshImporterWrapper, ObjImporterWrapper, SerializedAssetData,
    StandardAudioImporterWrapper, StandardTextureImporterWrapper, TextureImporterWrapper,
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
