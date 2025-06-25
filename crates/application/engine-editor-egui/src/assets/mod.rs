// Re-export asset creation functions from the asset crate

pub use engine_editor_assets::create_default_textures;

pub mod database;

pub use database::{AssetDatabase, AssetType};
#[allow(unused_imports)]
pub use database::{AssetEntry, AssetMetadata};
