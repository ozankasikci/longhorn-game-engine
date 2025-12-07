use hecs::Entity;
use longhorn_core::{Scene, World};
use std::error::Error;

/// Editor operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    Scene,  // Editing - no game logic runs
    Play,   // Running - game loop active
}

/// Editor state
#[derive(Debug, Default)]
pub struct EditorState {
    /// Current operating mode
    pub mode: EditorMode,
    /// Currently selected entity
    pub selected_entity: Option<Entity>,
    /// Whether game is paused (only relevant in Play mode)
    pub paused: bool,
    /// Path to loaded game
    pub game_path: Option<String>,
    /// Snapshot of the world state when entering play mode
    play_mode_snapshot: Option<Scene>,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, entity: Option<Entity>) {
        self.selected_entity = entity;
    }

    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entity == Some(entity)
    }

    pub fn is_playing(&self) -> bool {
        self.mode == EditorMode::Play
    }

    pub fn is_scene_mode(&self) -> bool {
        self.mode == EditorMode::Scene
    }

    /// Enter play mode by saving a snapshot of the current world state
    pub fn enter_play_mode<R: longhorn_core::AssetRegistry>(
        &mut self,
        world: &World,
        registry: &R,
    ) -> Result<(), Box<dyn Error>> {
        // Serialize world to snapshot
        let snapshot = Scene::from_world(world, registry);
        self.play_mode_snapshot = Some(snapshot);
        self.mode = EditorMode::Play;
        Ok(())
    }

    /// Exit play mode by restoring the world state from the snapshot
    pub fn exit_play_mode<L: longhorn_core::AssetLoader>(
        &mut self,
        world: &mut World,
        asset_loader: &mut L,
    ) -> Result<(), Box<dyn Error>> {
        // Restore world from snapshot
        if let Some(snapshot) = &self.play_mode_snapshot {
            // Restore entities in-place to preserve entity IDs
            snapshot.restore_into(world, asset_loader)?;
        }

        self.mode = EditorMode::Scene;
        self.play_mode_snapshot = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longhorn_core::{AssetId, World};
    use std::collections::HashMap;

    // Mock asset registry for testing
    struct MockRegistry {
        id_to_path: HashMap<u64, String>,
    }

    impl MockRegistry {
        fn new() -> Self {
            Self {
                id_to_path: HashMap::new(),
            }
        }
    }

    impl longhorn_core::AssetRegistry for MockRegistry {
        fn get_path(&self, id: AssetId) -> Option<&str> {
            self.id_to_path.get(&id.0).map(|s| s.as_str())
        }

        fn get_id(&self, _path: &str) -> Option<AssetId> {
            None
        }
    }

    // Mock asset loader for testing
    struct MockAssetLoader;

    impl longhorn_core::AssetLoader for MockAssetLoader {
        fn load_texture(&mut self, _path: &str) -> std::io::Result<AssetId> {
            Ok(AssetId::new(1))
        }

        fn load_texture_by_id(&mut self, id: AssetId) -> std::io::Result<AssetId> {
            Ok(id)
        }
    }

    #[test]
    fn test_default_mode_is_scene() {
        let state = EditorState::new();
        assert_eq!(state.mode, EditorMode::Scene);
        assert!(state.is_scene_mode());
        assert!(!state.is_playing());
    }

    #[test]
    fn test_mode_transitions() {
        let mut state = EditorState::new();

        state.mode = EditorMode::Play;
        assert!(state.is_playing());
        assert!(!state.is_scene_mode());

        state.mode = EditorMode::Scene;
        assert!(state.is_scene_mode());
    }

    #[test]
    fn test_enter_play_mode_saves_snapshot() {
        let mut state = EditorState::new();
        let world = World::new();
        let registry = MockRegistry::new();

        assert_eq!(state.mode, EditorMode::Scene);

        state.enter_play_mode(&world, &registry).unwrap();

        assert_eq!(state.mode, EditorMode::Play);
        assert!(state.play_mode_snapshot.is_some());
    }

    #[test]
    fn test_exit_play_mode_clears_snapshot() {
        let mut state = EditorState::new();
        let mut world = World::new();
        let registry = MockRegistry::new();
        let mut asset_loader = MockAssetLoader;

        state.enter_play_mode(&world, &registry).unwrap();
        state.exit_play_mode(&mut world, &mut asset_loader).unwrap();

        assert_eq!(state.mode, EditorMode::Scene);
        assert!(state.play_mode_snapshot.is_none());
    }
}
