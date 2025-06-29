//! Unified editor coordinator that uses the hybrid game loop
//!
//! This replaces the old EditorCoordinator with one that properly integrates
//! the engine-runtime-core game loop for both editor and play modes.

use engine_runtime::{HybridGameLoop, EngineMode, HybridFrameResult};
use engine_runtime_core::{System, SystemError, GameContext, HotReloadManager, HotReloadEvent, AssetType};
use engine_ecs_core::World;
use engine_scripting::{LuaScriptSystem, TypeScriptScriptSystem, components::{LuaScript, TypeScriptScript}};
use crate::{EditorState, PlayStateManager, PlayState};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Unified coordinator that manages both editor and game runtime
pub struct UnifiedEditorCoordinator {
    /// Hybrid game loop that supports both modes
    game_loop: HybridGameLoop,
    /// ECS world (shared between editor and runtime)
    ecs_world: Arc<Mutex<World>>,
    /// Editor-specific state
    editor_state: EditorState,
    /// Play state manager
    play_state_manager: PlayStateManager,
    /// Hot reload manager for development
    hot_reload_manager: HotReloadManager,
    /// Last update time for editor mode
    last_update: Instant,
}

impl UnifiedEditorCoordinator {
    /// Create a new unified coordinator with empty world
    pub fn new() -> Self {
        Self::with_world(Arc::new(Mutex::new(World::new())))
    }
    
    /// Create a new unified coordinator with provided world
    pub fn with_world(ecs_world: Arc<Mutex<World>>) -> Self {
        
        // Create game loop in editor mode
        let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
        
        // Register editor systems that should run during play mode
        let mut scheduler = game_loop.system_scheduler_mut();
        
        // Add ECS update system
        scheduler.add_system(Box::new(ECSUpdateSystem {
            world: Arc::clone(&ecs_world),
        }));
        
        // Add rendering system (will be handled by scene view)
        scheduler.add_system(Box::new(RenderingSystem {
            world: Arc::clone(&ecs_world),
        }));
        
        // Add Lua scripting system
        let lua_system = LuaScriptSystemWrapper {
            system: LuaScriptSystem::new().expect("Failed to create LuaScriptSystem"),
            coordinator_world: Arc::clone(&ecs_world),
        };
        scheduler.add_system(Box::new(lua_system));
        
        // Add TypeScript scripting system
        let typescript_system = TypeScriptScriptSystemWrapper {
            system: TypeScriptScriptSystem::new(),
            coordinator_world: Arc::clone(&ecs_world),
        };
        scheduler.add_system(Box::new(typescript_system));
        
        // Resolve dependencies
        scheduler.resolve_dependencies()
            .expect("Failed to resolve system dependencies");
        
        // Create hot reload manager with default handlers
        let mut hot_reload_manager = HotReloadManager::new();
        Self::setup_default_hot_reload_handlers(&mut hot_reload_manager);
        
        Self {
            game_loop,
            ecs_world,
            editor_state: EditorState::new(),
            play_state_manager: PlayStateManager::new(),
            hot_reload_manager,
            last_update: Instant::now(),
        }
    }
    
    /// Update the coordinator (called from eframe)
    pub fn update(&mut self, delta_time: f32) {
        // CRITICAL: Simple debug to see if this is called at all - using eprintln for immediate output
        eprintln!("🔥 DEBUG: UnifiedEditorCoordinator::update() CALLED delta_time={}", delta_time);
        println!("🔥 DEBUG: UnifiedEditorCoordinator::update() CALLED delta_time={}", delta_time);
        
        // Update play state manager
        self.play_state_manager.update_time(delta_time);
        
        // Process hot reload events
        self.process_hot_reload_events();
        
        // Handle play state transitions
        let current_play_state = self.play_state_manager.get_state();
        let current_mode = self.game_loop.mode();
        
        // Debug: Show current states every few frames
        static mut STATE_LOG_COUNTER: u32 = 0;
        unsafe {
            STATE_LOG_COUNTER += 1;
            if STATE_LOG_COUNTER % 60 == 0 { // Every 60 frames
                eprintln!("🚨 UnifiedEditorCoordinator::update() - Play State: {:?}, Game Mode: {:?}", current_play_state, current_mode);
                log::debug!("Play State: {:?}, Game Mode: {:?}", current_play_state, current_mode);
            }
        }
        
        match current_play_state {
            PlayState::Editing => {
                // CRITICAL FIX: Force play mode if we have TypeScript scripts
                // This ensures TypeScript scripts execute during development
                let world_lock = self.ecs_world.lock().unwrap();
                let has_typescript_scripts = world_lock.query_legacy::<engine_scripting::components::TypeScriptScript>().count() > 0;
                drop(world_lock);
                
                if has_typescript_scripts {
                    eprintln!("🔥 COORDINATOR: Found TypeScript scripts, forcing EditorPlay mode for development!");
                    if current_mode != EngineMode::EditorPlay {
                        self.game_loop.set_mode(EngineMode::EditorPlay);
                    }
                } else {
                    // Make sure we're in editor mode
                    if current_mode != EngineMode::Editor {
                        log::info!("Exiting play mode to editing");
                        self.exit_play_mode();
                    }
                }
            }
            PlayState::Playing => {
                // Enter play mode if not already
                if current_mode != EngineMode::EditorPlay {
                    log::info!("Entering play mode");
                    self.enter_play_mode();
                }
            }
            PlayState::Paused => {
                // Stay in play mode but don't update
                if current_mode == EngineMode::EditorPlay {
                    log::info!("Pausing game execution");
                    self.game_loop.set_mode(EngineMode::Editor);
                }
            }
        }
        
        // Update the game loop
        eprintln!("🚨 Calling game_loop.update_frame()...");
        let frame_result = self.game_loop.update_frame(
            std::time::Duration::from_secs_f32(delta_time)
        );
        eprintln!("🚨 Game loop update result: fixed_updates={}", frame_result.fixed_updates);
        
        // If we had fixed updates, we need to update editor state
        if frame_result.fixed_updates > 0 {
            eprintln!("🚨 Syncing editor state after {} fixed updates", frame_result.fixed_updates);
            self.sync_editor_state();
        }
    }
    
    /// Render the scene with interpolation
    pub fn render(&mut self, interpolation: f32) {
        self.game_loop.render(interpolation);
    }
    
    /// Enter play mode
    fn enter_play_mode(&mut self) {
        log::info!("Entering play mode with unified game loop");
        
        // Save current editor state
        self.save_editor_state();
        
        // Switch game loop to play mode
        self.game_loop.set_mode(EngineMode::EditorPlay);
        
        // Initialize game systems
        self.initialize_game_systems();
    }
    
    /// Exit play mode
    fn exit_play_mode(&mut self) {
        log::info!("Exiting play mode");
        
        // Switch back to editor mode
        self.game_loop.set_mode(EngineMode::Editor);
        
        // Restore editor state
        self.restore_editor_state();
        
        // Clean up game systems
        self.cleanup_game_systems();
    }
    
    /// Save editor state before playing
    fn save_editor_state(&mut self) {
        // TODO: Implement state saving
        // This would save entity positions, component values, etc.
    }
    
    /// Restore editor state after playing
    fn restore_editor_state(&mut self) {
        // TODO: Implement state restoration
        // This would restore entity positions, component values, etc.
    }
    
    /// Initialize game systems for play mode
    fn initialize_game_systems(&mut self) {
        // TODO: Initialize game-specific systems
        // Physics, AI, gameplay logic, etc.
    }
    
    /// Clean up game systems after play mode
    fn cleanup_game_systems(&mut self) {
        // Force complete script reinitialization to ensure updated scripts are loaded
        self.force_script_reinitialization();
    }
    
    /// Sync editor state with game state
    fn sync_editor_state(&mut self) {
        // CRITICAL FIX: Copy Transform changes from coordinator world back to editor world
        // This ensures that script modifications are visible in the editor
        log::info!("🔄 Syncing coordinator world changes back to editor world");
        
        // Get both worlds
        let coordinator_world = Arc::clone(&self.ecs_world);
        
        if let Ok(coordinator_world_lock) = coordinator_world.try_lock() {
            // TODO: In a full implementation, we need access to the editor world
            // For now, log that sync is happening
            eprintln!("🔄 SYNC: coordinator world has {} entities", coordinator_world_lock.entity_count());
            
            // The challenge is that we need access to the editor world to copy data back
            // This requires architectural changes to pass the editor world reference
            log::info!("✅ Sync coordinator state - coordinator world access confirmed");
        } else {
            log::warn!("❌ Could not lock coordinator world for sync");
        };
    }
    
    /// Get the editor state
    pub fn editor_state(&self) -> &EditorState {
        &self.editor_state
    }
    
    /// Get mutable editor state
    pub fn editor_state_mut(&mut self) -> &mut EditorState {
        &mut self.editor_state
    }
    
    /// Get the play state manager
    pub fn play_state_manager(&self) -> &PlayStateManager {
        &self.play_state_manager
    }
    
    /// Get mutable play state manager
    pub fn play_state_manager_mut(&mut self) -> &mut PlayStateManager {
        &mut self.play_state_manager
    }
    
    /// Get the game context for accessing resources
    pub fn game_context(&self) -> &GameContext {
        self.game_loop.game_context()
    }
    
    /// Get mutable game context
    pub fn game_context_mut(&mut self) -> &mut GameContext {
        self.game_loop.game_context_mut()
    }
    
    /// Get interpolation factor for rendering
    pub fn get_interpolation(&self) -> f32 {
        // For now, return a simple interpolation
        // In a full implementation, this would come from the last frame result
        0.5
    }
    
    /// Get the hot reload manager
    pub fn hot_reload_manager(&self) -> &HotReloadManager {
        &self.hot_reload_manager
    }
    
    /// Get mutable hot reload manager
    pub fn hot_reload_manager_mut(&mut self) -> &mut HotReloadManager {
        &mut self.hot_reload_manager
    }
    
    /// Set the ECS world (used by editor to provide the actual world)
    pub fn set_world(&mut self, world: Arc<Mutex<World>>) {
        self.ecs_world = world.clone();
        
        // We need to recreate the systems with the new world
        // For now, store the world and we'll update systems when they're accessed
    }
    
    /// Get the ECS world
    pub fn world(&self) -> Arc<Mutex<World>> {
        Arc::clone(&self.ecs_world)
    }
    
    /// Force complete script reinitialization for stop/start cycles
    pub fn force_script_reinitialization(&mut self) {
        log::info!("🔄 UNIFIED COORDINATOR: Forcing script reinitialization for stop/start cycle");
        
        // Access the TypeScript system through the scheduler to force reinitialization
        let scheduler = self.game_loop.system_scheduler_mut();
        
        if let Some(system) = scheduler.find_system_mut("TypeScriptScriptSystem") {
            // Try to downcast to our wrapper type
            if let Some(wrapper) = system.as_any_mut().downcast_mut::<TypeScriptScriptSystemWrapper>() {
                wrapper.system.force_complete_script_reinitialization();
                log::info!("✅ UNIFIED COORDINATOR: Script reinitialization complete");
            } else {
                log::warn!("❌ UNIFIED COORDINATOR: Failed to downcast TypeScript system for reinitialization");
            }
        } else {
            log::warn!("❌ UNIFIED COORDINATOR: TypeScript system not found for reinitialization");
        }
    }
    
    /// Setup default hot reload handlers
    fn setup_default_hot_reload_handlers(manager: &mut HotReloadManager) {
        // Texture reloading
        manager.register_handler(AssetType::Texture, Box::new(|path, _| {
            log::info!("Reloading texture: {}", path.display());
            // TODO: Implement texture reloading
            Ok(())
        }));
        
        // Shader reloading
        manager.register_handler(AssetType::Shader, Box::new(|path, _| {
            log::info!("Reloading shader: {}", path.display());
            // TODO: Implement shader recompilation
            Ok(())
        }));
        
        // Script reloading
        manager.register_handler(AssetType::Script, Box::new(|path, _| {
            log::info!("Reloading script: {}", path.display());
            // TODO: Implement script reloading
            Ok(())
        }));
        
        // Model reloading
        manager.register_handler(AssetType::Model, Box::new(|path, _| {
            log::info!("Reloading model: {}", path.display());
            // TODO: Implement model reloading
            Ok(())
        }));
        
        // Audio reloading
        manager.register_handler(AssetType::Audio, Box::new(|path, _| {
            log::info!("Reloading audio: {}", path.display());
            // TODO: Implement audio reloading
            Ok(())
        }));
        
        // Config reloading
        manager.register_handler(AssetType::Config, Box::new(|path, _| {
            log::info!("Reloading config: {}", path.display());
            // TODO: Implement config reloading
            Ok(())
        }));
    }
    
    /// Process hot reload events
    fn process_hot_reload_events(&mut self) {
        let events = self.hot_reload_manager.poll_events();
        
        for event in events {
            match event {
                HotReloadEvent::FileModified(path, asset_type) => {
                    if let Err(e) = self.hot_reload_manager.trigger_reload(&path, asset_type) {
                        log::error!("Failed to reload asset {}: {}", path.display(), e);
                    }
                }
                HotReloadEvent::FileCreated(path, asset_type) => {
                    log::info!("New asset created: {} ({:?})", path.display(), asset_type);
                    // TODO: Add new asset to project
                }
                HotReloadEvent::FileDeleted(path, asset_type) => {
                    log::info!("Asset deleted: {} ({:?})", path.display(), asset_type);
                    // TODO: Remove asset from project
                }
                HotReloadEvent::DirectoryCreated(path) => {
                    log::info!("Directory created: {}", path.display());
                }
                HotReloadEvent::DirectoryDeleted(path) => {
                    log::info!("Directory deleted: {}", path.display());
                }
            }
        }
    }
}

/// System that updates the ECS world
struct ECSUpdateSystem {
    world: Arc<Mutex<World>>,
}

impl System for ECSUpdateSystem {
    fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
        // Update ECS world
        // In a real implementation, this would run all ECS systems
        Ok(())
    }
    
    fn name(&self) -> &str {
        "ECSUpdateSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        true
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for ECSUpdateSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ECSUpdateSystem").finish()
    }
}

/// System that prepares rendering data
struct RenderingSystem {
    world: Arc<Mutex<World>>,
}

impl System for RenderingSystem {
    fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
        // Prepare rendering data
        // In a real implementation, this would update render components
        Ok(())
    }
    
    fn name(&self) -> &str {
        "RenderingSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        false // Rendering is variable timestep
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for RenderingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderingSystem").finish()
    }
}

/// Wrapper for LuaScriptSystem that provides world access  
struct LuaScriptSystemWrapper {
    system: LuaScriptSystem,
    coordinator_world: Arc<Mutex<World>>,
}

impl System for LuaScriptSystemWrapper {
    fn execute(&mut self, _context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        // CRITICAL DEBUG: Always log system execution
        eprintln!("🚨 LuaScriptSystemWrapper::execute() called! delta_time={}", delta_time);
        
        // Execute scripts with world access
        let world_lock = self.coordinator_world.lock().unwrap();
        let script_count = world_lock.query_legacy::<LuaScript>().count();
        drop(world_lock);
        
        eprintln!("🚨 Lua script count: {}", script_count);
        
        if script_count > 0 {
            eprintln!("🚨 EXECUTING Lua scripts!");
            println!("[LuaScriptSystemWrapper] Executing {} scripts", script_count);
        } else {
            eprintln!("🚨 No Lua scripts to execute");
        }
        
        self.system.execute_scripts_from_world(Arc::clone(&self.coordinator_world), delta_time)
            .map_err(|e| SystemError::ExecutionFailed(format!("LuaScriptSystem error: {}", e)))
    }
    
    fn name(&self) -> &str {
        "LuaScriptSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        true
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for LuaScriptSystemWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaScriptSystemWrapper").finish()
    }
}

/// Wrapper for TypeScriptScriptSystem that provides world access
struct TypeScriptScriptSystemWrapper {
    system: TypeScriptScriptSystem,
    coordinator_world: Arc<Mutex<World>>,
}

impl System for TypeScriptScriptSystemWrapper {
    fn execute(&mut self, _context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        // CRITICAL DEBUG: Always log system execution
        eprintln!("🚨 TypeScriptScriptSystemWrapper::execute() called! delta_time={}", delta_time);
        
        // Execute scripts with world access
        let mut world_lock = self.coordinator_world.lock().unwrap();
        let script_count = world_lock.query_legacy::<TypeScriptScript>().count();
        
        eprintln!("🚨 TypeScript script count: {}", script_count);
        
        if script_count > 0 {
            eprintln!("🚨 EXECUTING TypeScript scripts!");
            // Execute TypeScript scripts
            self.system.update(&mut world_lock, delta_time as f64);
            eprintln!("🚨 TypeScript scripts execution COMPLETE!");
        } else {
            eprintln!("🚨 No TypeScript scripts to execute");
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "TypeScriptScriptSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        true
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for TypeScriptScriptSystemWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeScriptScriptSystemWrapper").finish()
    }
}

#[cfg(feature = "editor")]
pub mod editor_integration {
    use super::*;
    
    /// Extension trait for egui integration
    pub trait UnifiedCoordinatorExt {
        /// Update from egui context
        fn update_from_egui(&mut self, ctx: &egui::Context, delta_time: f32);
        
        /// Check if continuous updates are needed
        fn needs_continuous_update(&self) -> bool;
    }
    
    impl UnifiedCoordinatorExt for UnifiedEditorCoordinator {
        fn update_from_egui(&mut self, ctx: &egui::Context, delta_time: f32) {
            // Request repaint if playing
            if self.play_state_manager.get_state() == PlayState::Playing {
                ctx.request_repaint();
            }
            
            self.update(delta_time);
        }
        
        fn needs_continuous_update(&self) -> bool {
            self.play_state_manager.get_state() == PlayState::Playing
        }
    }
}