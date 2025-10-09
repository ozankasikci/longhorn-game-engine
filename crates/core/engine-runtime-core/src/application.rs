use std::time::Duration;
use winit::event::WindowEvent;
use engine_input::InputManager;

/// Trait that applications must implement to work with the game loop
pub trait Application {
    /// Called once when the application starts
    fn initialize(&mut self) -> crate::Result<()> {
        Ok(())
    }
    
    /// Called at fixed timestep intervals for game logic updates
    fn update(&mut self, delta_time: Duration, input: &InputManager) -> crate::Result<()>;
    
    /// Called for rendering with interpolation factor between physics frames
    fn render(&mut self, interpolation: f32) -> crate::Result<()>;
    
    /// Called when window events are received
    fn handle_window_event(&mut self, event: &WindowEvent) -> crate::Result<()> {
        let _ = event;
        Ok(())
    }
    
    /// Called when the application should exit
    fn should_exit(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    #[derive(Default)]
    struct MockApplication {
        pub update_calls: Arc<Mutex<Vec<Duration>>>,
        pub render_calls: Arc<Mutex<Vec<f32>>>,
        pub should_exit: bool,
    }
    
    impl Application for MockApplication {
        fn update(&mut self, delta_time: Duration, _input: &InputManager) -> crate::Result<()> {
            self.update_calls.lock().unwrap().push(delta_time);
            Ok(())
        }
        
        fn render(&mut self, interpolation: f32) -> crate::Result<()> {
            self.render_calls.lock().unwrap().push(interpolation);
            Ok(())
        }
        
        fn should_exit(&self) -> bool {
            self.should_exit
        }
    }
    
    #[test]
    fn test_application_trait_default_implementations() {
        let mut app = MockApplication::default();
        
        // Test default initialize
        assert!(app.initialize().is_ok());
        
        // Test default should_exit
        assert!(!app.should_exit());
        
        // Test default handle_window_event
        let event = WindowEvent::CloseRequested;
        assert!(app.handle_window_event(&event).is_ok());
    }
    
    #[test]
    fn test_application_update_and_render() {
        let mut app = MockApplication::default();
        let update_calls = app.update_calls.clone();
        let render_calls = app.render_calls.clone();
        
        let delta = Duration::from_millis(16);
        let interpolation = 0.5;
        
        let input = InputManager::new().unwrap();
        assert!(app.update(delta, &input).is_ok());
        assert!(app.render(interpolation).is_ok());
        
        assert_eq!(update_calls.lock().unwrap().len(), 1);
        assert_eq!(update_calls.lock().unwrap()[0], delta);
        
        assert_eq!(render_calls.lock().unwrap().len(), 1);
        assert_eq!(render_calls.lock().unwrap()[0], interpolation);
    }
}