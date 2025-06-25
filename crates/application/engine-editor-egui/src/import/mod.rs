use serde::{Deserialize, Serialize};

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

#[allow(unused_imports)]
pub use batch::{BatchImportOptions, BatchImporter};
#[allow(unused_imports)]
pub use dialog::ImportResult;
pub use dialog::ImportSettings;
#[allow(unused_imports)]
pub use dialog::{CollisionType, ImportDialog};
#[allow(unused_imports)]
pub use drag_drop::{DragDropHandler, FileType};
#[allow(unused_imports)]
pub use error::{ImportError as ImportErr, ImportErrorDialog, ImportErrorType};
#[allow(unused_imports)]
pub use file_watcher::{FileWatchEvent, ImportFileWatcher};
#[allow(unused_imports)]
pub use history::{ImportHistory, ImportRecord};
#[allow(unused_imports)]
pub use hot_reload::{HotReloadAction, HotReloadEvent, HotReloadWatcher};
#[allow(unused_imports)]
pub use preview::ImportPreview;
#[allow(unused_imports)]
pub use preview::PreviewData;
#[allow(unused_imports)]
pub use progress::{ImportProgress, ImportTask};
pub use service::ImportService;
#[allow(unused_imports)]
pub use service::{ImportError, ImportStatus};
#[allow(unused_imports)]
pub use service::{
    ImportHandle, ImportNotification, ImportQueue, ImportSettingsConverter, ImportUIState,
};
#[allow(unused_imports)]
pub use wrappers::{
    AudioImporterWrapper, SerializedAssetData, StandardAudioImporterWrapper,
    StandardTextureImporterWrapper, TextureImporterWrapper,
};
#[allow(unused_imports)]
pub use wrappers::{MeshImporterWrapper, ObjImporterWrapper};

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

#[allow(dead_code)]
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
