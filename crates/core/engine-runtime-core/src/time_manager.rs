use std::time::Duration;
use instant::Instant;

/// Manages timing for the game loop with fixed timestep and accumulator pattern
#[derive(Debug)]
pub struct TimeManager {
    fixed_timestep: Duration,
    accumulator: Duration,
    previous_time: Instant,
    max_updates_per_frame: u32,
    total_time: Duration,
    frame_count: u64,
    delta_time: f32,
}

impl TimeManager {
    /// Create a new TimeManager with default 60Hz fixed timestep
    pub fn new() -> Self {
        Self::with_timestep(Duration::from_nanos(16_666_667)) // ~60Hz
    }
    
    /// Create a new TimeManager with custom fixed timestep
    pub fn with_timestep(fixed_timestep: Duration) -> Self {
        Self {
            fixed_timestep,
            accumulator: Duration::ZERO,
            previous_time: Instant::now(),
            max_updates_per_frame: 10, // Death spiral prevention
            total_time: Duration::ZERO,
            frame_count: 0,
            delta_time: 0.0,
        }
    }
    
    /// Update timing and return number of fixed updates needed and interpolation factor
    pub fn update(&mut self) -> (u32, f32) {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.previous_time);
        self.previous_time = current_time;
        
        // Update delta time and frame count
        self.delta_time = frame_time.as_secs_f32();
        self.frame_count += 1;
        
        // Clamp frame time to prevent death spiral
        let clamped_frame_time = frame_time.min(Duration::from_millis(100));
        self.accumulator += clamped_frame_time;
        
        let mut updates = 0;
        while self.accumulator >= self.fixed_timestep && updates < self.max_updates_per_frame {
            self.accumulator -= self.fixed_timestep;
            self.total_time += self.fixed_timestep;
            updates += 1;
        }
        
        // Calculate interpolation factor (0.0 to 1.0)
        let interpolation = if self.fixed_timestep.as_nanos() > 0 {
            self.accumulator.as_nanos() as f32 / self.fixed_timestep.as_nanos() as f32
        } else {
            0.0
        };
        
        (updates, interpolation.clamp(0.0, 1.0))
    }
    
    /// Get the fixed timestep duration
    pub fn fixed_timestep(&self) -> Duration {
        self.fixed_timestep
    }
    
    /// Get total elapsed time
    pub fn total_time(&self) -> Duration {
        self.total_time
    }
    
    /// Get current accumulator value (for testing)
    pub fn accumulator(&self) -> Duration {
        self.accumulator
    }
    
    /// Set maximum updates per frame (death spiral prevention)
    pub fn set_max_updates_per_frame(&mut self, max_updates: u32) {
        self.max_updates_per_frame = max_updates;
    }
    
    /// Get the current delta time in seconds
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
    
    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Get the target FPS based on fixed timestep
    pub fn target_fps(&self) -> f64 {
        1.0 / self.fixed_timestep.as_secs_f64()
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_time_manager_creation() {
        let tm = TimeManager::new();
        assert_eq!(tm.fixed_timestep(), Duration::from_nanos(16_666_667));
        assert_eq!(tm.total_time(), Duration::ZERO);
        assert_eq!(tm.accumulator(), Duration::ZERO);
    }
    
    #[test]
    fn test_time_manager_with_custom_timestep() {
        let timestep = Duration::from_millis(10);
        let tm = TimeManager::with_timestep(timestep);
        assert_eq!(tm.fixed_timestep(), timestep);
    }
    
    #[test]
    fn test_time_manager_update_no_time_passed() {
        let mut tm = TimeManager::new();
        let (updates, interpolation) = tm.update();
        
        // Should be no updates and minimal interpolation
        assert_eq!(updates, 0);
        assert!(interpolation >= 0.0 && interpolation <= 1.0);
    }
    
    #[test]
    fn test_time_manager_update_with_sleep() {
        let mut tm = TimeManager::with_timestep(Duration::from_millis(10));
        
        // Sleep for more than one timestep
        thread::sleep(Duration::from_millis(25));
        
        let (updates, interpolation) = tm.update();
        
        // Should have at least 2 updates (25ms / 10ms = 2.5)
        assert!(updates >= 2);
        assert!(interpolation >= 0.0 && interpolation <= 1.0);
        
        // Total time should reflect the updates
        assert!(tm.total_time() >= Duration::from_millis(20));
    }
    
    #[test]
    fn test_death_spiral_prevention() {
        let mut tm = TimeManager::with_timestep(Duration::from_millis(1));
        tm.set_max_updates_per_frame(5);
        
        // Sleep for a very long time to trigger death spiral prevention
        thread::sleep(Duration::from_millis(100));
        
        let (updates, _) = tm.update();
        
        // Should be capped at max_updates_per_frame
        assert_eq!(updates, 5);
    }
    
    #[test]
    fn test_interpolation_calculation() {
        let timestep = Duration::from_millis(10);
        let mut tm = TimeManager::with_timestep(timestep);
        
        // Manually set accumulator to half a timestep
        tm.accumulator = Duration::from_millis(5);
        
        let (_updates, interpolation) = tm.update();
        
        // Should have interpolation around 0.5 (5ms / 10ms)
        assert_relative_eq!(interpolation, 0.5, epsilon = 0.1);
    }
    
    #[test]
    fn test_max_updates_per_frame_setter() {
        let mut tm = TimeManager::new();
        tm.set_max_updates_per_frame(3);
        
        // This is tested indirectly through the death spiral test
        // The setter should work without panicking
        assert_eq!(tm.max_updates_per_frame, 3);
    }
}