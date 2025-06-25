use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Audio,
    Animation,
    Material,
    Shader,
    Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub id: Uuid,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub import_time: std::time::SystemTime,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub file_size: u64,
}

pub struct AssetDatabase {
    assets: HashMap<Uuid, AssetEntry>,
    path_index: HashMap<PathBuf, Uuid>,
}

impl Default for AssetDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl AssetDatabase {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_index: HashMap::new(),
        }
    }

    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    pub fn add_imported_asset(&mut self, id: Uuid, path: PathBuf, asset_type: AssetType) {
        let entry = AssetEntry {
            id,
            path: path.clone(),
            asset_type,
            import_time: std::time::SystemTime::now(),
            metadata: AssetMetadata::default(),
        };

        self.assets.insert(id, entry);
        self.path_index.insert(path, id);
    }

    pub fn get_asset(&self, id: Uuid) -> Option<&AssetEntry> {
        self.assets.get(&id)
    }

    pub fn get_asset_by_path(&self, path: &PathBuf) -> Option<&AssetEntry> {
        self.path_index.get(path).and_then(|id| self.assets.get(id))
    }

    pub fn remove_asset(&mut self, id: Uuid) -> Option<AssetEntry> {
        if let Some(asset) = self.assets.remove(&id) {
            self.path_index.remove(&asset.path);
            Some(asset)
        } else {
            None
        }
    }

    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<&AssetEntry> {
        self.assets
            .values()
            .filter(|asset| asset.asset_type == asset_type)
            .collect()
    }

    pub fn update_metadata(&mut self, id: Uuid, metadata: AssetMetadata) {
        if let Some(asset) = self.assets.get_mut(&id) {
            asset.metadata = metadata;
        }
    }
}
