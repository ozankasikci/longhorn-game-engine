//! Main game loop implementation

use crate::{RuntimeResult, RuntimeError, Application, ApplicationEvent};
use std::time::{Duration, Instant};

/// Main game loop
pub struct GameLoop<A: Application> {
    application: A,
    target_fps: u32,
    last_frame: Instant,
    frame_count: u64,
}

impl<A: Application> GameLoop<A> {
    /// Create a new game loop
    pub fn new(application: A) -> RuntimeResult<Self> {
        Ok(Self {
            application,
            target_fps: 60,
            last_frame: Instant::now(),
            frame_count: 0,
        })
    }
    
    /// Set target FPS
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
    
    /// Run the game loop
    pub fn run(mut self) -> RuntimeResult<()> {
        self.application.initialize()?;
        
        let target_frame_time = Duration::from_secs_f32(1.0 / self.target_fps as f32);
        
        loop {
            let frame_start = Instant::now();
            let delta_time = frame_start.duration_since(self.last_frame).as_secs_f32();
            self.last_frame = frame_start;
            
            // Check for exit condition
            if self.application.should_exit() {
                break;
            }
            
            // Update application
            self.application.update(delta_time)?;
            
            // Render application
            self.application.render()?;
            
            self.frame_count += 1;
            
            // Frame rate limiting
            let frame_time = frame_start.elapsed();
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
        }
        
        log::info!("Game loop completed after {} frames", self.frame_count);
        Ok(())
    }
    
    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}