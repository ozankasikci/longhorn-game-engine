//! Asset registry for managing asset metadata

use crate::{AssetId, AssetMetadata, AssetType};
use std::collections::HashMap;

/// Asset registry for tracking all assets
pub struct AssetRegistry {
    assets: HashMap<AssetId, AssetMetadata>,
    next_id: u64,
}

impl AssetRegistry {
    /// Create a new asset registry
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Register a new asset
    pub fn register(&mut self, asset_type: AssetType, path: String, size: u64, checksum: String) -> AssetId {
        let id = AssetId(self.next_id);
        self.next_id += 1;
        
        let metadata = AssetMetadata {
            id,
            asset_type,
            path,
            size,
            checksum,
        };
        
        self.assets.insert(id, metadata);
        id
    }
    
    /// Get asset metadata
    pub fn get_metadata(&self, id: AssetId) -> Option<&AssetMetadata> {
        self.assets.get(&id)
    }
    
    /// List all assets of a specific type
    pub fn list_by_type(&self, asset_type: AssetType) -> Vec<&AssetMetadata> {
        self.assets.values()
            .filter(|metadata| metadata.asset_type == asset_type)
            .collect()
    }
}