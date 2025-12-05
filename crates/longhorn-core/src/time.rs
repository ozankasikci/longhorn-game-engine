use std::time::{Duration, Instant};

/// Time tracking for game loop
#[derive(Debug, Clone)]
pub struct Time {
    startup: Instant,
    last_update: Instant,
    delta: Duration,
    total_elapsed: Duration,
}

impl Time {
    /// Create a new Time tracker
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            startup: now,
            last_update: now,
            delta: Duration::ZERO,
            total_elapsed: Duration::ZERO,
        }
    }

    /// Update time tracking (call once per frame)
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_update);
        self.total_elapsed = now.duration_since(self.startup);
        self.last_update = now;
    }

    /// Get delta time since last frame (in seconds)
    pub fn delta(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Get delta time as Duration
    pub fn delta_duration(&self) -> Duration {
        self.delta
    }

    /// Get total elapsed time since startup (in seconds)
    pub fn elapsed(&self) -> f32 {
        self.total_elapsed.as_secs_f32()
    }

    /// Get total elapsed time as Duration
    pub fn elapsed_duration(&self) -> Duration {
        self.total_elapsed
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed timestep accumulator for physics updates
#[derive(Debug, Clone)]
pub struct FixedTimestep {
    timestep: Duration,
    accumulator: Duration,
}

impl FixedTimestep {
    /// Create a new fixed timestep with the given rate (in Hz)
    pub fn new(rate: f32) -> Self {
        Self {
            timestep: Duration::from_secs_f32(1.0 / rate),
            accumulator: Duration::ZERO,
        }
    }

    /// Create a fixed timestep for 60 FPS physics
    pub fn from_fps(fps: u32) -> Self {
        Self::new(fps as f32)
    }

    /// Add delta time and return number of fixed updates to perform
    pub fn tick(&mut self, delta: Duration) -> u32 {
        self.accumulator += delta;
        let mut steps = 0;

        while self.accumulator >= self.timestep {
            self.accumulator -= self.timestep;
            steps += 1;
        }

        steps
    }

    /// Get the fixed timestep duration
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    /// Get the fixed timestep in seconds
    pub fn timestep_secs(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    /// Reset the accumulator
    pub fn reset(&mut self) {
        self.accumulator = Duration::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_time_tracking() {
        let mut time = Time::new();

        // Initial state
        assert_eq!(time.delta(), 0.0);
        assert!(time.elapsed() >= 0.0);

        // Sleep and update
        thread::sleep(Duration::from_millis(10));
        time.update();

        // Delta should be > 0
        assert!(time.delta() > 0.0);
        assert!(time.elapsed() > 0.0);
    }

    #[test]
    fn test_fixed_timestep() {
        let mut timestep = FixedTimestep::new(60.0);

        // No update initially
        assert_eq!(timestep.tick(Duration::from_millis(0)), 0);

        // Less than one frame
        assert_eq!(timestep.tick(Duration::from_millis(8)), 0);

        // About one frame (16.666ms for 60 FPS, with 8ms already accumulated)
        assert_eq!(timestep.tick(Duration::from_millis(8)), 0);

        // Now slightly over one frame total
        assert_eq!(timestep.tick(Duration::from_millis(1)), 1);

        // Two frames
        assert_eq!(timestep.tick(Duration::from_millis(33)), 1);
    }

    #[test]
    fn test_fixed_timestep_from_fps() {
        let timestep = FixedTimestep::from_fps(60);
        assert!((timestep.timestep_secs() - 1.0 / 60.0).abs() < 0.0001);
    }

    #[test]
    fn test_fixed_timestep_reset() {
        let mut timestep = FixedTimestep::new(60.0);

        // Accumulate some time
        timestep.tick(Duration::from_millis(8));

        // Reset
        timestep.reset();

        // Should not trigger update with small delta
        assert_eq!(timestep.tick(Duration::from_millis(8)), 0);
    }
}
