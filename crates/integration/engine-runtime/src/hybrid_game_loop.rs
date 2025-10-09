//! Hybrid game loop that can run in both editor and standalone modes
//!
//! This module provides a unified game loop that integrates with both
//! eframe (for editor mode) and winit (for standalone mode).

use engine_runtime_core::{SystemScheduler, GameContext, TimeManager, Application};
use engine_input::InputManager;
use std::time::{Duration, Instant};

/// Mode of operation for the hybrid game loop
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineMode {
    /// Editor mode - runs within eframe with EGUI overlay
    Editor,
    /// Standalone mode - runs directly with winit
    Standalone,
    /// Play mode - editor is simulating game runtime
    EditorPlay,
}

/// Hybrid game loop that supports both editor and standalone execution
pub struct HybridGameLoop {
    /// Current engine mode
    mode: EngineMode,
    /// Core game systems scheduler
    system_scheduler: SystemScheduler,
    /// Game context with resources
    game_context: GameContext,
    /// Time management for fixed timestep
    time_manager: TimeManager,
    /// Input management
    input_manager: InputManager,
    /// Whether game systems should update
    should_update_systems: bool,
    /// Accumulator for fixed timestep
    accumulator: Duration,
    /// Last frame time
    last_frame_time: Instant,
}

impl HybridGameLoop {
    /// Create a new hybrid game loop
    pub fn new(mode: EngineMode) -> Self {
        Self {
            mode,
            system_scheduler: SystemScheduler::new(),
            game_context: GameContext::with_target_fps(60.0),
            time_manager: TimeManager::new(),
            input_manager: InputManager::new().expect("Failed to create InputManager"),
            should_update_systems: mode == EngineMode::Standalone,
            accumulator: Duration::ZERO,
            last_frame_time: Instant::now(),
        }
    }
    
    /// Update the game loop (called from eframe in editor mode)
    pub fn update_frame(&mut self, delta_time: Duration) -> HybridFrameResult {
        // Update time tracking
        let now = Instant::now();
        let frame_time = if delta_time == Duration::ZERO {
            now.duration_since(self.last_frame_time)
        } else {
            delta_time
        };
        self.last_frame_time = now;
        
        // Only update systems if we should (playing or standalone)
        if !self.should_update_systems {
            return HybridFrameResult {
                fixed_updates: 0,
                interpolation: 0.0,
                should_render: true,
            };
        }
        
        // Fixed timestep with accumulator
        self.accumulator += frame_time;
        let fixed_timestep = self.time_manager.fixed_timestep();
        let max_updates = 10; // Prevent death spiral
        let mut updates = 0;
        
        // Fixed timestep updates
        while self.accumulator >= fixed_timestep && updates < max_updates {
            self.fixed_update(fixed_timestep);
            self.accumulator -= fixed_timestep;
            updates += 1;
        }
        
        // Calculate interpolation factor for smooth rendering
        let interpolation = self.accumulator.as_secs_f32() / fixed_timestep.as_secs_f32();
        
        HybridFrameResult {
            fixed_updates: updates,
            interpolation,
            should_render: true,
        }
    }
    
    /// Perform a fixed timestep update
    fn fixed_update(&mut self, delta_time: Duration) {
        eprintln!("🚨 GAME LOOP: fixed_update() called with delta_time={}ms", delta_time.as_millis());
        
        // Update game context
        self.game_context.update(delta_time.as_secs_f32())
            .expect("Failed to update game context");
        
        // Update input
        self.input_manager.update();
        
        // Execute fixed timestep systems
        eprintln!("🚨 GAME LOOP: About to execute_fixed_systems()");
        self.system_scheduler
            .execute_fixed_systems(&mut self.game_context, delta_time.as_secs_f32())
            .expect("Failed to execute fixed systems");
        eprintln!("🚨 GAME LOOP: execute_fixed_systems() completed");
    }
    
    /// Render with interpolation
    pub fn render(&mut self, interpolation: f32) {
        // Execute variable timestep systems (rendering, effects, etc.)
        self.system_scheduler
            .execute_variable_systems(&mut self.game_context, interpolation)
            .expect("Failed to execute variable systems");
    }
    
    /// Set engine mode
    pub fn set_mode(&mut self, mode: EngineMode) {
        eprintln!("🚨 GAME LOOP: Setting mode to {:?}, should_update_systems will be {}", 
                 mode, matches!(mode, EngineMode::Standalone | EngineMode::EditorPlay));
        self.mode = mode;
        self.should_update_systems = matches!(mode, EngineMode::Standalone | EngineMode::EditorPlay);
        
        // Reset accumulator when changing modes
        if self.should_update_systems {
            eprintln!("🚨 GAME LOOP: Systems ENABLED - resetting accumulator");
            self.accumulator = Duration::ZERO;
            self.last_frame_time = Instant::now();
        } else {
            eprintln!("🚨 GAME LOOP: Systems DISABLED");
        }
    }
    
    /// Get current engine mode
    pub fn mode(&self) -> EngineMode {
        self.mode
    }
    
    /// Check if systems should be updating
    pub fn is_running(&self) -> bool {
        self.should_update_systems
    }
    
    /// Access the system scheduler
    pub fn system_scheduler_mut(&mut self) -> &mut SystemScheduler {
        &mut self.system_scheduler
    }
    
    /// Access the game context
    pub fn game_context(&self) -> &GameContext {
        &self.game_context
    }
    
    /// Access the game context mutably
    pub fn game_context_mut(&mut self) -> &mut GameContext {
        &mut self.game_context
    }
    
    /// Access the input manager
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }
    
    /// Process input events (for editor integration)
    pub fn process_input_event(&mut self, event: &winit::event::WindowEvent) {
        self.input_manager.process_window_event(event);
    }
}

/// Result of a frame update
#[derive(Debug)]
pub struct HybridFrameResult {
    /// Number of fixed timestep updates performed
    pub fixed_updates: usize,
    /// Interpolation factor for smooth rendering
    pub interpolation: f32,
    /// Whether rendering should occur
    pub should_render: bool,
}

/// Trait for applications that can run in the hybrid game loop
pub trait HybridApplication {
    /// Called when entering play mode (editor only)
    fn on_play_mode_enter(&mut self, game_loop: &mut HybridGameLoop);
    
    /// Called when exiting play mode (editor only)
    fn on_play_mode_exit(&mut self, game_loop: &mut HybridGameLoop);
    
    /// Called before fixed update
    fn pre_update(&mut self, game_loop: &mut HybridGameLoop);
    
    /// Called after fixed update
    fn post_update(&mut self, game_loop: &mut HybridGameLoop);
    
    /// Called before rendering
    fn pre_render(&mut self, game_loop: &mut HybridGameLoop, interpolation: f32);
    
    /// Called after rendering
    fn post_render(&mut self, game_loop: &mut HybridGameLoop, interpolation: f32);
}

#[cfg(feature = "editor")]
pub mod editor_integration {
    use super::*;
    
    /// Bridge between eframe and the hybrid game loop
    pub struct EditorGameLoopBridge {
        game_loop: HybridGameLoop,
        last_update: Instant,
    }
    
    impl EditorGameLoopBridge {
        pub fn new() -> Self {
            Self {
                game_loop: HybridGameLoop::new(EngineMode::Editor),
                last_update: Instant::now(),
            }
        }
        
        /// Update from eframe's update() method
        pub fn update_from_eframe(&mut self, ctx: &egui::Context) -> HybridFrameResult {
            let now = Instant::now();
            let delta = now.duration_since(self.last_update);
            self.last_update = now;
            
            // Request continuous updates when playing
            if self.game_loop.is_running() {
                ctx.request_repaint();
            }
            
            self.game_loop.update_frame(delta)
        }
        
        /// Enter play mode
        pub fn enter_play_mode(&mut self) {
            self.game_loop.set_mode(EngineMode::EditorPlay);
        }
        
        /// Exit play mode
        pub fn exit_play_mode(&mut self) {
            self.game_loop.set_mode(EngineMode::Editor);
        }
        
        /// Check if in play mode
        pub fn is_playing(&self) -> bool {
            self.game_loop.mode() == EngineMode::EditorPlay
        }
        
        /// Access the game loop
        pub fn game_loop_mut(&mut self) -> &mut HybridGameLoop {
            &mut self.game_loop
        }
    }
}

#[cfg(not(feature = "editor"))]
pub mod standalone {
    use super::*;
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;
    
    /// Run the game loop in standalone mode
    pub fn run_standalone<A: Application + 'static>(
        mut app: A,
        mut game_loop: HybridGameLoop,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Longhorn Game Engine")
            .build(&event_loop)?;
        
        app.initialize()?;
        
        event_loop.run(move |event, elwt| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => {
                    game_loop.process_input_event(&event);
                    
                    if let winit::event::WindowEvent::CloseRequested = event {
                        elwt.exit();
                    }
                }
                winit::event::Event::AboutToWait => {
                    let result = game_loop.update_frame(Duration::ZERO);
                    
                    if result.should_render {
                        game_loop.render(result.interpolation);
                        window.request_redraw();
                    }
                    
                    if app.should_exit() {
                        elwt.exit();
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }
}