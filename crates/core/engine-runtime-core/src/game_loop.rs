use crate::{Application, TimeManager, RuntimeError};
use engine_input::InputManager;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
};

/// Core game loop implementation with fixed timestep and variable rendering
pub struct GameLoop {
    time_manager: TimeManager,
    input_manager: InputManager,
}

impl GameLoop {
    /// Create a new game loop with default settings
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            time_manager: TimeManager::new(),
            input_manager: InputManager::new().map_err(|e| RuntimeError::Application(e.to_string()))?,
        })
    }
    
    /// Create a new game loop with custom timestep
    pub fn with_timestep(timestep: std::time::Duration) -> crate::Result<Self> {
        Ok(Self {
            time_manager: TimeManager::with_timestep(timestep),
            input_manager: InputManager::new().map_err(|e| RuntimeError::Application(e.to_string()))?,
        })
    }
    
    /// Run the game loop with the given application
    pub fn run<A: Application + 'static>(mut self, mut app: A) -> crate::Result<()> {
        let event_loop = EventLoop::new()
            .map_err(|e| RuntimeError::EventLoop(e.to_string()))?;
        
        let window = WindowBuilder::new()
            .with_title("Longhorn Game Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)
            .map_err(|e| RuntimeError::WindowCreation(e.to_string()))?;
        
        app.initialize()?;
        
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    // Process input events
                    self.input_manager.process_window_event(&event);
                    
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        },
                        _ => {
                            if let Err(e) = app.handle_window_event(&event) {
                                eprintln!("Application window event error: {}", e);
                                elwt.exit();
                            }
                        }
                    }
                },
                Event::AboutToWait => {
                    // This is where the main game loop logic happens
                    if app.should_exit() {
                        elwt.exit();
                        return;
                    }
                    
                    let (updates, interpolation) = self.time_manager.update();
                    
                    // Fixed timestep updates
                    for _ in 0..updates {
                        if let Err(e) = app.update(self.time_manager.fixed_timestep(), &self.input_manager) {
                            eprintln!("Application update error: {}", e);
                            elwt.exit();
                            return;
                        }
                    }
                    
                    // Update input manager for next frame
                    self.input_manager.update();
                    
                    // Variable rendering with interpolation
                    if let Err(e) = app.render(interpolation) {
                        eprintln!("Application render error: {}", e);
                        elwt.exit();
                        return;
                    }
                    
                    window.request_redraw();
                },
                Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == window.id() => {
                    // Redraw is handled in AboutToWait for consistent timing
                },
                _ => {}
            }
            
            elwt.set_control_flow(ControlFlow::Poll);
        }).map_err(|e| RuntimeError::EventLoop(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get reference to the time manager (for testing)
    pub fn time_manager(&self) -> &TimeManager {
        &self.time_manager
    }
    
    /// Get mutable reference to the time manager (for testing)
    pub fn time_manager_mut(&mut self) -> &mut TimeManager {
        &mut self.time_manager
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create default GameLoop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::sync::{Arc, Mutex};
    
    #[derive(Default)]
    #[allow(dead_code)]
    struct TestApplication {
        pub update_calls: Arc<Mutex<Vec<Duration>>>,
        pub render_calls: Arc<Mutex<Vec<f32>>>,
        pub initialize_called: Arc<Mutex<bool>>,
        pub exit_after_calls: Option<usize>,
    }
    
    impl Application for TestApplication {
        fn initialize(&mut self) -> crate::Result<()> {
            *self.initialize_called.lock().unwrap() = true;
            Ok(())
        }
        
        fn update(&mut self, delta_time: Duration, _input: &engine_input::InputManager) -> crate::Result<()> {
            self.update_calls.lock().unwrap().push(delta_time);
            Ok(())
        }
        
        fn render(&mut self, interpolation: f32) -> crate::Result<()> {
            self.render_calls.lock().unwrap().push(interpolation);
            Ok(())
        }
        
        fn should_exit(&self) -> bool {
            if let Some(max_calls) = self.exit_after_calls {
                self.update_calls.lock().unwrap().len() >= max_calls
            } else {
                false
            }
        }
    }
    
    #[test]
    fn test_game_loop_creation() {
        let game_loop = GameLoop::new().unwrap();
        assert_eq!(
            game_loop.time_manager().fixed_timestep(),
            Duration::from_nanos(16_666_667)
        );
    }
    
    #[test]
    fn test_game_loop_with_custom_timestep() {
        let timestep = Duration::from_millis(10);
        let game_loop = GameLoop::with_timestep(timestep).unwrap();
        assert_eq!(game_loop.time_manager().fixed_timestep(), timestep);
    }
    
    #[test]
    fn test_time_manager_access() {
        let mut game_loop = GameLoop::new().unwrap();
        
        // Test immutable access
        let _ = game_loop.time_manager();
        
        // Test mutable access
        game_loop.time_manager_mut().set_max_updates_per_frame(5);
    }
    
    // Note: We can't easily test the full run() method in unit tests because
    // it requires a window system. Integration tests would be better for that.
    // For now, we test the components in isolation.
    
    #[test]
    fn test_default_trait() {
        let game_loop = GameLoop::default();
        assert_eq!(
            game_loop.time_manager().fixed_timestep(),
            Duration::from_nanos(16_666_667)
        );
    }
}