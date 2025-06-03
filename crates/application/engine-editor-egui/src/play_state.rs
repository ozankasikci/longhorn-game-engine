// Play state management functionality

use crate::types::PlayState;
use crate::editor_state::ConsoleMessage;
use std::time::{Instant, Duration};

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
    pub fn start_play(&mut self) -> Vec<ConsoleMessage> {
        let mut messages = vec![];
        self.play_state = PlayState::Playing;
        self.game_start_time = Some(Instant::now());
        messages.push(ConsoleMessage::info("▶️ Game started"));
        messages.push(ConsoleMessage::info(&format!("🎮 Play mode active at {:.1} FPS", 1.0 / self.delta_time.max(0.001))));
        messages
    }
    
    /// Pause the game
    pub fn pause_play(&mut self) -> Vec<ConsoleMessage> {
        let mut messages = vec![];
        if self.play_state == PlayState::Playing {
            self.play_state = PlayState::Paused;
            messages.push(ConsoleMessage::info(&format!("⏸️ Game paused at {:.1}s", self.get_play_time())));
        }
        messages
    }
    
    /// Resume the game from pause
    pub fn resume_play(&mut self) -> Vec<ConsoleMessage> {
        let mut messages = vec![];
        if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
            messages.push(ConsoleMessage::info(&format!("▶️ Game resumed at {:.1}s", self.get_play_time())));
        }
        messages
    }
    
    /// Stop playing and return to edit mode
    pub fn stop_play(&mut self) -> Vec<ConsoleMessage> {
        let mut messages = vec![];
        self.play_state = PlayState::Editing;
        let play_time = self.get_play_time();
        self.game_start_time = None;
        messages.push(ConsoleMessage::info(&format!("⏹️ Game stopped after {:.1}s", play_time)));
        messages.push(ConsoleMessage::info("🎨 Returned to edit mode"));
        messages
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