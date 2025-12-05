use crate::PlatformEvent;
use longhorn_engine::{Engine, EngineError};
use std::path::Path;

/// Mobile application wrapper for the Longhorn engine.
pub struct MobileApp {
    engine: Engine,
    running: bool,
}

impl MobileApp {
    /// Create a new headless mobile app (for testing).
    pub fn new_headless() -> Self {
        Self {
            engine: Engine::new_headless(),
            running: false,
        }
    }

    /// Load a game from a directory.
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<(), EngineError> {
        self.engine.load_game(path)
    }

    /// Start the game.
    pub fn start(&mut self) -> Result<(), EngineError> {
        self.engine.start()?;
        self.running = true;
        Ok(())
    }

    /// Handle a platform event.
    pub fn handle_event(&mut self, event: PlatformEvent) {
        match event {
            PlatformEvent::Touch(touch_event) => {
                self.engine.handle_touch(touch_event);
            }
            PlatformEvent::Resize { width, height } => {
                self.engine.resize(width, height);
            }
            PlatformEvent::Suspend => {
                log::info!("App suspended");
                // Future: pause game, save state
            }
            PlatformEvent::Resume => {
                log::info!("App resumed");
                // Future: resume game, reload resources if needed
            }
            PlatformEvent::Quit => {
                log::info!("App quit requested");
                self.running = false;
            }
        }
    }

    /// Update the app (main game loop step).
    pub fn update(&mut self) -> Result<(), EngineError> {
        if !self.running {
            return Ok(());
        }
        self.engine.update()
    }

    /// Check if the app is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get a reference to the engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get a mutable reference to the engine.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use longhorn_core::Vec2;
    use longhorn_input::TouchEvent;
    use std::fs;

    fn setup_test_game() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "longhorn_mobile_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a minimal game.json
        let manifest = r#"{
            "name": "Test Mobile Game",
            "version": "1.0.0",
            "entry": "main.ts",
            "viewport": {
                "width": 800,
                "height": 600
            },
            "assets": {
                "preload": []
            }
        }"#;

        fs::write(temp_dir.join("game.json"), manifest).unwrap();
        temp_dir
    }

    #[test]
    fn test_new_headless() {
        let app = MobileApp::new_headless();
        assert!(!app.is_running());
    }

    #[test]
    fn test_load_and_start() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        assert!(!app.is_running());

        app.start().unwrap();
        assert!(app.is_running());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_handle_touch_event() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        app.start().unwrap();

        let touch_event = TouchEvent::Start { x: 100.0, y: 200.0 };
        app.handle_event(PlatformEvent::Touch(touch_event));

        // Verify the touch was processed by checking input state
        assert!(app.engine().input().just_pressed());
        assert_eq!(app.engine().input().position(), Vec2::new(100.0, 200.0));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_handle_resize_event() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        app.start().unwrap();

        app.handle_event(PlatformEvent::Resize {
            width: 1920,
            height: 1080,
        });

        assert_eq!(app.engine().config().viewport_width, 1920);
        assert_eq!(app.engine().config().viewport_height, 1080);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_handle_quit_event() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        app.start().unwrap();
        assert!(app.is_running());

        app.handle_event(PlatformEvent::Quit);
        assert!(!app.is_running());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_update() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        app.start().unwrap();

        // Should not error
        let result = app.update();
        assert!(result.is_ok());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_update_when_not_running() {
        let mut app = MobileApp::new_headless();

        // Update should succeed but do nothing when not running
        let result = app.update();
        assert!(result.is_ok());
    }

    #[test]
    fn test_suspend_resume() {
        let temp_dir = setup_test_game();
        let mut app = MobileApp::new_headless();

        app.load_game(&temp_dir).unwrap();
        app.start().unwrap();

        // These should not crash
        app.handle_event(PlatformEvent::Suspend);
        assert!(app.is_running()); // App should still be running

        app.handle_event(PlatformEvent::Resume);
        assert!(app.is_running());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_engine_access() {
        let mut app = MobileApp::new_headless();

        // Test immutable access
        let _engine = app.engine();

        // Test mutable access
        let engine = app.engine_mut();
        assert_eq!(engine.world().len(), 0);
    }
}
