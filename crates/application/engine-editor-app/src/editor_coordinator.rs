// Editor coordinator - manages play states and inter-panel communication

use crate::editor_state::ConsoleMessage;
use crate::types::PlayState;
use crate::play_state::PlayStateManager;

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
        if self.play_state_manager.play_state == PlayState::Editing {
            self.play_state_manager.start_play();
        }
    }
    
    /// Pause the game (only from playing state)
    pub fn pause_play(&mut self) {
        self.play_state_manager.pause_play();
    }
    
    /// Resume from paused state
    pub fn resume_play(&mut self) {
        self.play_state_manager.resume_play();
    }
    
    /// Stop play mode and return to editing
    pub fn stop_play(&mut self) {
        self.play_state_manager.stop_play();
    }
    
    /// Update delta time for game loop
    pub fn update_delta_time(&mut self) {
        self.play_state_manager.update_delta_time();
        
        // Scene navigation needs delta time even in editing mode
        // Only pause delta time calculation during actual pause state
        if self.play_state_manager.play_state == PlayState::Paused {
            self.play_state_manager.delta_time = 0.0;
        }
    }
    
    /// Get current play time in seconds
    pub fn get_play_time(&self) -> f32 {
        self.play_state_manager.get_play_time()
    }
    
    /// Get current play state
    pub fn get_play_state(&self) -> PlayState {
        self.play_state_manager.play_state
    }
    
    /// Get mutable reference to play state
    pub fn get_play_state_mut(&mut self) -> &mut PlayState {
        &mut self.play_state_manager.play_state
    }
}