//! Asset management system for the Longhorn Game Engine editor
//! 
//! This crate provides:
//! - Texture asset management
//! - Project asset organization
//! - Asset loading and caching
//! - Default asset creation

pub mod types;
pub mod texture_manager;
pub mod asset_loader;
pub mod asset_cache;

// Re-export main types and functions
pub use types::{TextureAsset, ProjectAsset, AssetHandle, AssetLoadError, AssetHandleGenerator};
pub use texture_manager::TextureManager;
pub use asset_loader::{AssetLoader, TextureLoader};
pub use asset_cache::AssetCache;

// Re-export default creation functions
pub use texture_manager::create_default_textures;
pub use types::create_default_project_assets;