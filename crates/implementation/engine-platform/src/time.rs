//! Platform time utilities

use std::time::{Duration, Instant};

/// High-resolution timer
pub struct Timer {
    start_time: Instant,
}

/// Platform clock for time measurements
pub struct Clock {
    last_frame: Instant,
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// Get elapsed time since timer creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }
}

impl Clock {
    /// Create a new clock
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
        }
    }

    /// Tick the clock and return delta time
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;
        delta
    }

    /// Get delta time in seconds
    pub fn delta_secs(&mut self) -> f32 {
        self.tick().as_secs_f32()
    }
}
