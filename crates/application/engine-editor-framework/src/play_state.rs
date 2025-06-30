// Play state management functionality

use instant::Instant;
use crate::world_snapshot::{WorldSnapshot, SnapshotError};
use engine_ecs_core::World;

// Re-export PlayState from scene view
pub use engine_editor_scene_view::types::PlayState;

/// Manages play mode state and timing for the editor
pub struct PlayStateManager {
    pub play_state: PlayState,
    pub game_start_time: Option<Instant>,
    pub last_frame_time: Instant,
    pub delta_time: f32,
    /// Snapshot captured when entering play mode for state restoration
    world_snapshot: Option<WorldSnapshot>,
}

impl PlayStateManager {
    pub fn new() -> Self {
        Self {
            play_state: PlayState::default(),
            game_start_time: None,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            world_snapshot: None,
        }
    }

    /// Start playing the game
    pub fn start(&mut self) {
        self.play_state = PlayState::Playing;
        self.game_start_time = Some(Instant::now());
    }

    /// Pause the game
    pub fn pause(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play_state = PlayState::Paused;
        }
    }

    /// Resume the game from pause
    pub fn resume(&mut self) {
        if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
        }
    }

    /// Stop playing and return to edit mode
    pub fn stop(&mut self) {
        self.play_state = PlayState::Editing;
        self.game_start_time = None;
        self.world_snapshot = None; // Clear snapshot when stopping
    }

    /// Update delta time calculation
    pub fn update_time(&mut self, dt: f32) {
        self.delta_time = dt;

        // Also update based on actual time
        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_frame_time);
        let actual_dt = frame_duration.as_secs_f32();
        self.last_frame_time = now;

        // Use the smaller of the two to prevent huge jumps
        self.delta_time = self.delta_time.min(actual_dt);

        // Clamp delta time to prevent huge jumps
        if self.delta_time > 0.1 {
            self.delta_time = 0.016; // Cap at ~60 FPS equivalent
        }
    }

    /// Get the current play time in seconds
    pub fn get_play_time(&self) -> f32 {
        if let Some(start_time) = self.game_start_time {
            let duration = Instant::now().duration_since(start_time);
            duration.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Get current state
    pub fn get_state(&self) -> PlayState {
        self.play_state
    }

    /// Start playing with snapshot capture for state restoration
    pub fn start_with_snapshot(&mut self, world: &World) {
        // Capture snapshot before starting play mode
        self.world_snapshot = Some(WorldSnapshot::capture(world));
        self.play_state = PlayState::Playing;
        self.game_start_time = Some(Instant::now());
    }

    /// Stop playing and restore world state from snapshot
    pub fn stop_with_restore(&mut self, world: &mut World) -> Result<(), SnapshotError> {
        if let Some(snapshot) = &self.world_snapshot {
            // Restore world state from snapshot
            snapshot.restore(world)?;
        }
        
        // Stop play mode and clear snapshot
        self.play_state = PlayState::Editing;
        self.game_start_time = None;
        self.world_snapshot = None;
        
        Ok(())
    }

    /// Get the current world snapshot if available
    pub fn get_snapshot(&self) -> Option<&WorldSnapshot> {
        self.world_snapshot.as_ref()
    }
}

impl Default for PlayStateManager {
    fn default() -> Self {
        Self::new()
    }
}
