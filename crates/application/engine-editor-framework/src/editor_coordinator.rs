// Editor coordinator - manages play states and inter-panel communication

use crate::play_state::{PlayState, PlayStateManager};

/// Coordinates editor state transitions and high-level operations
pub struct EditorCoordinator {
    pub play_state_manager: PlayStateManager,
}

impl EditorCoordinator {
    pub fn new() -> Self {
        Self {
            play_state_manager: PlayStateManager::new(),
        }
    }

    /// Transition to playing state
    pub fn start_play(&mut self) {
        if self.play_state_manager.get_state() == PlayState::Editing {
            self.play_state_manager.start();
        }
    }

    /// Pause the game (only from playing state)
    pub fn pause_play(&mut self) {
        self.play_state_manager.pause();
    }

    /// Resume from paused state
    pub fn resume_play(&mut self) {
        self.play_state_manager.resume();
    }

    /// Stop play mode and return to editing
    pub fn stop_play(&mut self) {
        self.play_state_manager.stop();
    }

    /// Update delta time for game loop
    pub fn update_delta_time(&mut self) {
        self.play_state_manager.update_time(0.016); // Default to 60 FPS

        // Scene navigation needs delta time even in editing mode
        // Only pause delta time calculation during actual pause state
        if self.play_state_manager.get_state() == PlayState::Paused {
            self.play_state_manager.delta_time = 0.0;
        }
    }

    /// Get current play time in seconds
    pub fn get_play_time(&self) -> f32 {
        self.play_state_manager.get_play_time()
    }

    /// Get current play state
    pub fn get_play_state(&self) -> PlayState {
        self.play_state_manager.get_state()
    }

    /// Get mutable reference to play state
    pub fn get_play_state_mut(&mut self) -> &mut PlayState {
        &mut self.play_state_manager.play_state
    }

    /// Get current delta time
    pub fn get_delta_time(&self) -> f32 {
        self.play_state_manager.delta_time
    }
}

impl Default for EditorCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
