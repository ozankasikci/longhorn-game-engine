//! Asset loading functionality

use crate::{AssetResult, AssetType};

/// Asset loader trait
pub trait AssetLoader<T> {
    /// Load an asset from a path
    fn load(&self, path: &str) -> AssetResult<T>;
    
    /// Get the supported asset type
    fn asset_type(&self) -> AssetType;
}

/// Asset loader registry
pub struct LoaderRegistry {
    // TODO: Implement loader registry
}

impl LoaderRegistry {
    /// Create a new loader registry
    pub fn new() -> Self {
        Self {
            // TODO: Initialize loader registry
        }
    }
    
    /// Register a loader for an asset type
    pub fn register<T, L>(&mut self, _loader: L) 
    where 
        T: 'static,
        L: AssetLoader<T> + 'static 
    {
        // TODO: Implement loader registration
    }
}