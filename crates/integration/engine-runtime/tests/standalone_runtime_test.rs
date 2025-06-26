//! Tests for standalone runtime functionality

use engine_runtime::{StandaloneRuntime, StandaloneConfig};
use engine_runtime_core::{Application, GameContext, RuntimeError};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

/// Test application for standalone runtime
struct TestGameApp {
    initialized: Arc<AtomicBool>,
    updated: Arc<AtomicBool>,
    rendered: Arc<AtomicBool>,
    frame_count: Arc<Mutex<u32>>,
}

impl TestGameApp {
    fn new() -> Self {
        Self {
            initialized: Arc::new(AtomicBool::new(false)),
            updated: Arc::new(AtomicBool::new(false)),
            rendered: Arc::new(AtomicBool::new(false)),
            frame_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl Application for TestGameApp {
    fn initialize(&mut self) -> engine_runtime_core::Result<()> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }
    
    fn update(&mut self, _delta_time: std::time::Duration, _input: &engine_input::InputManager) -> engine_runtime_core::Result<()> {
        self.updated.store(true, Ordering::SeqCst);
        let mut count = self.frame_count.lock().unwrap();
        *count += 1;
        Ok(())
    }
    
    fn render(&mut self, _interpolation: f32) -> engine_runtime_core::Result<()> {
        self.rendered.store(true, Ordering::SeqCst);
        Ok(())
    }
    
    fn should_exit(&self) -> bool {
        // Quit after 5 frames for testing
        let count = self.frame_count.lock().unwrap();
        *count >= 5
    }
}

#[test]
fn test_standalone_runtime_creation() {
    // Use headless mode for tests to avoid winit main thread requirement
    let config = StandaloneConfig::builder()
        .headless(true)
        .build();
    let runtime = StandaloneRuntime::new(config);
    
    assert!(runtime.is_ok(), "Should create standalone runtime successfully");
}

#[test]
fn test_standalone_config_defaults() {
    let config = StandaloneConfig::default();
    
    assert_eq!(config.window_title, "Longhorn Game Engine");
    assert_eq!(config.window_width, 1280);
    assert_eq!(config.window_height, 720);
    assert_eq!(config.vsync, true);
    assert_eq!(config.target_fps, 60.0);
}

#[test]
fn test_standalone_config_builder() {
    let config = StandaloneConfig::builder()
        .title("Test Game")
        .resolution(1920, 1080)
        .vsync(false)
        .target_fps(120.0)
        .build();
    
    assert_eq!(config.window_title, "Test Game");
    assert_eq!(config.window_width, 1920);
    assert_eq!(config.window_height, 1080);
    assert_eq!(config.vsync, false);
    assert_eq!(config.target_fps, 120.0);
}

#[test]
fn test_standalone_runtime_with_app() {
    let config = StandaloneConfig::builder()
        .headless(true)
        .build();
    let mut runtime = StandaloneRuntime::new(config).unwrap();
    
    let app = TestGameApp::new();
    let initialized = Arc::clone(&app.initialized);
    let updated = Arc::clone(&app.updated);
    let rendered = Arc::clone(&app.rendered);
    let frame_count = Arc::clone(&app.frame_count);
    
    // Set the application
    runtime.set_application(Box::new(app));
    
    // Application should be initialized
    assert!(runtime.initialize().is_ok());
    assert!(initialized.load(Ordering::SeqCst), "App should be initialized");
}

#[test]
fn test_standalone_runtime_headless() {
    // Test headless mode for CI/testing
    let config = StandaloneConfig::builder()
        .headless(true)
        .build();
    
    let runtime = StandaloneRuntime::new(config);
    assert!(runtime.is_ok(), "Should create headless runtime");
}

#[test]
fn test_standalone_runtime_from_project_path() {
    use std::path::PathBuf;
    
    let project_path = PathBuf::from("test_project");
    let runtime = StandaloneRuntime::from_project(project_path);
    
    // Should handle missing project gracefully
    assert!(matches!(runtime, Err(engine_runtime::RuntimeError::ProjectLoadError(_))));
}

#[test]
fn test_standalone_runtime_with_systems() {
    use engine_runtime_core::{System, SystemError};
    
    struct TestSystem {
        executed: Arc<AtomicBool>,
    }
    
    impl System for TestSystem {
        fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
            self.executed.store(true, Ordering::SeqCst);
            Ok(())
        }
        
        fn name(&self) -> &str {
            "TestSystem"
        }
        
        fn is_fixed_timestep(&self) -> bool {
            true
        }
    }
    
    impl std::fmt::Debug for TestSystem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestSystem").finish()
        }
    }
    
    let config = StandaloneConfig::builder()
        .headless(true)
        .build();
    let mut runtime = StandaloneRuntime::new(config).unwrap();
    
    let executed = Arc::new(AtomicBool::new(false));
    let system = TestSystem {
        executed: Arc::clone(&executed),
    };
    
    runtime.add_system(Box::new(system));
    
    // Initialize and run one frame
    runtime.initialize().unwrap();
    runtime.update_frame();
    
    assert!(executed.load(Ordering::SeqCst), "System should have executed");
}