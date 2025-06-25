use engine_resource_core::ResourceId;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HotReloadEvent {
    SourceModified {
        resource_id: ResourceId,
        source_path: PathBuf,
    },
    #[allow(dead_code)]
    SourceDeleted {
        resource_id: ResourceId,
        source_path: PathBuf,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HotReloadAction {
    Reimport {
        #[allow(dead_code)]
        resource_id: ResourceId,
        #[allow(dead_code)]
        source_path: PathBuf,
    },
    RemoveAsset {
        #[allow(dead_code)]
        resource_id: ResourceId,
    },
}

#[derive(Debug)]
struct WatchedAsset {
    // Allow dead code as these fields may be used in future hot reload implementation
    #[allow(dead_code)]
    asset_path: PathBuf,
    #[allow(dead_code)]
    source_path: PathBuf,
}

pub struct HotReloadWatcher {
    watched_assets: HashMap<ResourceId, WatchedAsset>,
}

impl Default for HotReloadWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl HotReloadWatcher {
    pub fn new() -> Self {
        Self {
            watched_assets: HashMap::new(),
        }
    }

    pub fn watch_asset(
        &mut self,
        resource_id: ResourceId,
        asset_path: PathBuf,
        source_path: PathBuf,
    ) {
        self.watched_assets.insert(
            resource_id,
            WatchedAsset {
                asset_path,
                source_path,
            },
        );
    }

    #[allow(dead_code)]
    pub fn unwatch_asset(&mut self, resource_id: &ResourceId) {
        self.watched_assets.remove(resource_id);
    }

    pub fn is_watching(&self, resource_id: &ResourceId) -> bool {
        self.watched_assets.contains_key(resource_id)
    }

    pub fn handle_event(&self, event: HotReloadEvent) -> Vec<HotReloadAction> {
        match event {
            HotReloadEvent::SourceModified {
                resource_id,
                source_path,
            } => {
                if self.watched_assets.contains_key(&resource_id) {
                    vec![HotReloadAction::Reimport {
                        resource_id,
                        source_path,
                    }]
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

    #[allow(dead_code)]
    pub fn get_watched_count(&self) -> usize {
        self.watched_assets.len()
    }
}
