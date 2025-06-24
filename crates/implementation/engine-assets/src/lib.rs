//! Asset management system for the mobile game engine
//!
//! This crate provides loading, caching, and management of game assets
//! including textures, models, audio files, and other resources.

pub mod cache;
pub mod io;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod types;

pub use cache::AssetCache;
pub use loader::{AssetLoader, LoaderRegistry};
pub use manager::AssetManager;
pub use types::{AssetId, AssetMetadata, AssetType};

/// Asset system errors
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Failed to load asset: {0}")]
    LoadError(String),
    #[error("Unsupported asset type: {0}")]
    UnsupportedType(String),
    #[error("Asset cache error: {0}")]
    CacheError(String),
}

/// Asset system result type
pub type AssetResult<T> = Result<T, AssetError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_manager_creation() {
        // Placeholder test
        assert!(true);
    }
}
