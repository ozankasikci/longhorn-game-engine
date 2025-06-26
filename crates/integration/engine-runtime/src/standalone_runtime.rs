//! Standalone runtime implementation for running games without editor
//!
//! This module provides a complete runtime environment for games to run
//! independently of the editor, using winit for windowing and the hybrid
//! game loop for execution.

use crate::{HybridGameLoop, EngineMode, RuntimeError};
use engine_runtime_core::{Application, GameContext, System, HotReloadManager, AssetType};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use std::path::PathBuf;
use std::time::Instant;

/// Configuration for standalone runtime
#[derive(Debug, Clone)]
pub struct StandaloneConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub vsync: bool,
    pub target_fps: f32,
    pub headless: bool,
    pub enable_hot_reload: bool,
}

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            window_title: "Longhorn Game Engine".to_string(),
            window_width: 1280,
            window_height: 720,
            vsync: true,
            target_fps: 60.0,
            headless: false,
            enable_hot_reload: true,
        }
    }
}

impl StandaloneConfig {
    /// Create a new config builder
    pub fn builder() -> StandaloneConfigBuilder {
        StandaloneConfigBuilder::default()
    }
}

/// Builder for StandaloneConfig
#[derive(Default)]
pub struct StandaloneConfigBuilder {
    config: StandaloneConfig,
}

impl StandaloneConfigBuilder {
    pub fn title(mut self, title: &str) -> Self {
        self.config.window_title = title.to_string();
        self
    }
    
    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.config.window_width = width;
        self.config.window_height = height;
        self
    }
    
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.vsync = vsync;
        self
    }
    
    pub fn target_fps(mut self, fps: f32) -> Self {
        self.config.target_fps = fps;
        self
    }
    
    pub fn headless(mut self, headless: bool) -> Self {
        self.config.headless = headless;
        self
    }
    
    pub fn hot_reload(mut self, enable: bool) -> Self {
        self.config.enable_hot_reload = enable;
        self
    }
    
    pub fn build(self) -> StandaloneConfig {
        self.config
    }
}

/// Standalone runtime for running games without editor
pub struct StandaloneRuntime {
    config: StandaloneConfig,
    game_loop: HybridGameLoop,
    event_loop: Option<EventLoop<()>>,
    window: Option<Window>,
    application: Option<Box<dyn Application>>,
    hot_reload_manager: Option<HotReloadManager>,
}

impl StandaloneRuntime {
    /// Create a new standalone runtime
    pub fn new(config: StandaloneConfig) -> Result<Self, RuntimeError> {
        // Create game loop in standalone mode
        let mut game_loop = HybridGameLoop::new(EngineMode::Standalone);
        
        // Target FPS is set in GameContext constructor
        
        // Create window and event loop if not headless
        let (event_loop, window) = if !config.headless {
            let event_loop = EventLoop::new()
                .map_err(|e| RuntimeError::InitializationError(format!("Failed to create event loop: {}", e)))?;
            let window = WindowBuilder::new()
                .with_title(&config.window_title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    config.window_width,
                    config.window_height,
                ))
                .build(&event_loop)
                .map_err(|e| RuntimeError::InitializationError(format!("Failed to create window: {}", e)))?;
            
            (Some(event_loop), Some(window))
        } else {
            (None, None)
        };
        
        // Create hot reload manager if enabled
        let hot_reload_manager = if config.enable_hot_reload {
            Some(HotReloadManager::new())
        } else {
            None
        };
        
        Ok(Self {
            config,
            game_loop,
            event_loop,
            window,
            application: None,
            hot_reload_manager,
        })
    }
    
    /// Create runtime from a project path
    pub fn from_project(project_path: PathBuf) -> Result<Self, RuntimeError> {
        // TODO: Load project configuration
        if !project_path.exists() {
            return Err(RuntimeError::ProjectLoadError(
                format!("Project path does not exist: {}", project_path.display())
            ));
        }
        
        // For now, return error as project loading is not implemented
        Err(RuntimeError::ProjectLoadError(
            "Project loading not yet implemented".to_string()
        ))
    }
    
    /// Set the application to run
    pub fn set_application(&mut self, app: Box<dyn Application>) {
        self.application = Some(app);
    }
    
    /// Add a system to the runtime
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.game_loop.system_scheduler_mut().add_system(system);
    }
    
    /// Initialize the runtime
    pub fn initialize(&mut self) -> Result<(), RuntimeError> {
        // Initialize application if set
        if let Some(app) = &mut self.application {
            app.initialize()
                .map_err(|e| RuntimeError::InitializationError(format!("App init failed: {}", e)))?;
        }
        
        // Resolve system dependencies
        self.game_loop.system_scheduler_mut()
            .resolve_dependencies()
            .map_err(|e| RuntimeError::SystemError(format!("Failed to resolve dependencies: {}", e)))?;
        
        Ok(())
    }
    
    /// Update one frame (for testing)
    pub fn update_frame(&mut self) {
        let delta = std::time::Duration::from_secs_f32(1.0 / self.config.target_fps);
        let result = self.game_loop.update_frame(delta);
        
        if result.should_render {
            self.game_loop.render(result.interpolation);
        }
    }
    
    /// Run the main loop
    pub fn run(mut self) -> Result<(), RuntimeError> {
        // Initialize first
        self.initialize()?;
        
        if let Some(event_loop) = self.event_loop {
            // Run with winit event loop
            let mut last_frame = Instant::now();
            
            event_loop.run(move |event, event_loop_window_target| {
                event_loop_window_target.set_control_flow(ControlFlow::Poll);
                
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        event_loop_window_target.exit();
                    }
                    Event::WindowEvent { event, .. } => {
                        // Process input
                        self.game_loop.process_input_event(&event);
                    }
                    Event::AboutToWait => {
                        // Update game loop
                        let now = Instant::now();
                        let delta = now.duration_since(last_frame);
                        last_frame = now;
                        
                        // Update application
                        if let Some(app) = &mut self.application {
                            let input = self.game_loop.input_manager();
                            if let Err(e) = app.update(delta, input) {
                                log::error!("Application update error: {}", e);
                            }
                            
                            if app.should_exit() {
                                event_loop_window_target.exit();
                            }
                        }
                        
                        // Update game loop
                        let result = self.game_loop.update_frame(delta);
                        
                        // Render
                        if result.should_render {
                            if let Some(app) = &mut self.application {
                                if let Err(e) = app.render(result.interpolation) {
                                    log::error!("Application render error: {}", e);
                                }
                            }
                            
                            self.game_loop.render(result.interpolation);
                        }
                        
                        // Request redraw
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
            });
        } else {
            // Headless mode - run for a fixed number of frames or until quit
            let mut frame_count = 0;
            let max_frames = 1000; // Safety limit for tests
            
            loop {
                let delta = std::time::Duration::from_secs_f32(1.0 / self.config.target_fps);
                
                // Update application
                if let Some(app) = &mut self.application {
                    let input = self.game_loop.input_manager();
                    app.update(delta, input)
                        .map_err(|e| RuntimeError::SystemError(format!("Update failed: {}", e)))?;
                    
                    if app.should_exit() {
                        break;
                    }
                }
                
                // Update game loop
                let result = self.game_loop.update_frame(delta);
                
                // Render
                if result.should_render {
                    if let Some(app) = &mut self.application {
                        app.render(result.interpolation)
                            .map_err(|e| RuntimeError::SystemError(format!("Render failed: {}", e)))?;
                    }
                    self.game_loop.render(result.interpolation);
                }
                
                frame_count += 1;
                if frame_count >= max_frames {
                    break;
                }
            }
        }
        
        Ok(())
    }
}

/// Error types for standalone runtime
#[derive(Debug, thiserror::Error)]
pub enum ProjectLoadError {
    #[error("Project not found: {0}")]
    NotFound(String),
    #[error("Invalid project format: {0}")]
    InvalidFormat(String),
    #[error("Missing configuration: {0}")]
    MissingConfig(String),
}