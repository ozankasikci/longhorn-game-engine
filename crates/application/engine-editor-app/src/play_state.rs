// Play state management functionality

use crate::editor_state::ConsoleMessage;
use crate::types::PlayState;
use std::time::{Duration, Instant};

/// Manages play mode state and timing for the editor
pub struct PlayStateManager {
    pub play_state: PlayState,
    pub game_start_time: Option<Instant>,
    pub last_frame_time: Instant,
    pub delta_time: f32,
}

impl PlayStateManager {
    pub fn new() -> Self {
        Self {
            play_state: PlayState::default(),
            game_start_time: None,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
        }
    }

    /// Start playing the game
    pub fn start_play(&mut self) {
        self.play_state = PlayState::Playing;
        self.game_start_time = Some(Instant::now());
    }

    /// Pause the game
    pub fn pause_play(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play_state = PlayState::Paused;
        }
    }

    /// Resume the game from pause
    pub fn resume_play(&mut self) {
        if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
        }
    }

    /// Stop playing and return to edit mode
    pub fn stop_play(&mut self) {
        self.play_state = PlayState::Editing;
        self.game_start_time = None;
    }

    /// Update delta time calculation
    pub fn update_delta_time(&mut self) {
        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_frame_time);
        self.delta_time = frame_duration.as_secs_f32();
        self.last_frame_time = now;

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
}
