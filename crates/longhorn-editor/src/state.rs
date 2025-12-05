use hecs::Entity;

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
