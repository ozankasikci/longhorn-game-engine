// Time management for the game engine

use std::time::{Instant, Duration};

pub struct Timer {
    start_time: Instant,
    last_frame: Instant,
    delta_time: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta_time: Duration::ZERO,
        }
    }
}

impl Timer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.last_frame = now;
    }
    
    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }
    
    pub fn elapsed(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}