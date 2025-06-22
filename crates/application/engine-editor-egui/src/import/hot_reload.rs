use std::path::PathBuf;
use std::collections::HashMap;
use engine_resource_core::ResourceId;

#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    SourceModified {
        resource_id: ResourceId,
        source_path: PathBuf,
    },
    SourceDeleted {
        resource_id: ResourceId,
        source_path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub enum HotReloadAction {
    Reimport {
        resource_id: ResourceId,
        source_path: PathBuf,
    },
    RemoveAsset {
        resource_id: ResourceId,
    },
}

#[derive(Debug)]
struct WatchedAsset {
    asset_path: PathBuf,
    source_path: PathBuf,
}

pub struct HotReloadWatcher {
    watched_assets: HashMap<ResourceId, WatchedAsset>,
}

impl HotReloadWatcher {
    pub fn new() -> Self {
        Self {
            watched_assets: HashMap::new(),
        }
    }
    
    pub fn watch_asset(&mut self, resource_id: ResourceId, asset_path: PathBuf, source_path: PathBuf) {
        self.watched_assets.insert(resource_id, WatchedAsset {
            asset_path,
            source_path,
        });
    }
    
    pub fn unwatch_asset(&mut self, resource_id: &ResourceId) {
        self.watched_assets.remove(resource_id);
    }
    
    pub fn is_watching(&self, resource_id: &ResourceId) -> bool {
        self.watched_assets.contains_key(resource_id)
    }
    
    pub fn handle_event(&self, event: HotReloadEvent) -> Vec<HotReloadAction> {
        match event {
            HotReloadEvent::SourceModified { resource_id, source_path } => {
                if self.watched_assets.contains_key(&resource_id) {
                    vec![HotReloadAction::Reimport { resource_id, source_path }]
                } else {
                    vec![]
                }
            }
            HotReloadEvent::SourceDeleted { resource_id, .. } => {
                if self.watched_assets.contains_key(&resource_id) {
                    vec![HotReloadAction::RemoveAsset { resource_id }]
                } else {
                    vec![]
                }
            }
        }
    }
    
    pub fn get_watched_count(&self) -> usize {
        self.watched_assets.len()
    }
}