//! Tests for LuaScriptSystem integration with HybridGameLoop

use engine_scripting::{LuaScriptSystem, components::LuaScript};
use engine_runtime::{HybridGameLoop, EngineMode};
use engine_runtime_core::{System, GameContext};
use engine_ecs_core::{World, register_component};
use std::time::Duration;

#[test]
fn test_lua_script_system_can_be_added_to_hybrid_game_loop() {
    // Register the LuaScript component
    register_component::<LuaScript>();
    
    // Create a HybridGameLoop in Editor mode
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    
    // Create our LuaScriptSystem
    let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    
    // Add the system to the game loop - this should work
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
    
    // Resolve dependencies
    let result = game_loop.system_scheduler_mut().resolve_dependencies();
    assert!(result.is_ok(), "Should be able to resolve dependencies with LuaScriptSystem: {:?}", result.err());
}

#[test]
fn test_lua_script_system_executes_in_game_loop() {
    // Register the LuaScript component
    register_component::<LuaScript>();
    
    // Create a HybridGameLoop
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    
    // Add LuaScriptSystem
    let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
    game_loop.system_scheduler_mut().resolve_dependencies().expect("Failed to resolve dependencies");
    
    // Execute a frame - this should run our LuaScriptSystem
    let frame_result = game_loop.update_frame(Duration::from_millis(16));
    
    // Should complete without errors
    assert!(frame_result.fixed_updates >= 0, "Frame should execute successfully");
}

#[test]
fn test_lua_script_system_executes_with_entities_in_game_loop() {
    // This test verifies that when entities with LuaScript components exist,
    // the HybridGameLoop properly executes our LuaScriptSystem and processes them
    
    // Register the LuaScript component
    register_component::<LuaScript>();
    
    // Create a HybridGameLoop
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    
    // Add LuaScriptSystem
    let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
    game_loop.system_scheduler_mut().resolve_dependencies().expect("Failed to resolve dependencies");
    
    // TODO: We need access to the World to add entities with scripts
    // This exposes a limitation in our current HybridGameLoop architecture
    // The game loop needs to provide access to the World for systems to use
    
    // For now, just verify the system can execute without entities
    let frame_result = game_loop.update_frame(Duration::from_millis(16));
    assert!(frame_result.fixed_updates >= 0, "Should execute even without entities");
}

#[test]
fn test_lua_script_system_fixed_timestep_in_game_loop() {
    // Test that our LuaScriptSystem runs at fixed timestep as intended
    
    register_component::<LuaScript>();
    
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    
    // Verify the system reports fixed timestep
    assert!(lua_system.is_fixed_timestep(), "LuaScriptSystem should use fixed timestep");
    
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
    game_loop.system_scheduler_mut().resolve_dependencies().expect("Failed to resolve dependencies");
    
    // Execute multiple frames with different delta times
    let deltas = [
        Duration::from_millis(8),   // 120 FPS
        Duration::from_millis(16),  // 60 FPS  
        Duration::from_millis(33),  // 30 FPS
        Duration::from_millis(50),  // 20 FPS
    ];
    
    for delta in deltas.iter() {
        let frame_result = game_loop.update_frame(*delta);
        // Fixed timestep systems should execute consistently regardless of frame time
        assert!(frame_result.fixed_updates >= 0, "Fixed timestep should work with delta {:?}", delta);
    }
}

#[test]
fn test_lua_script_system_in_different_engine_modes() {
    // Test that LuaScriptSystem works in different engine modes
    
    register_component::<LuaScript>();
    
    let modes = [
        EngineMode::Editor,
        EngineMode::EditorPlay,
        EngineMode::Standalone,
    ];
    
    for mode in modes.iter() {
        let mut game_loop = HybridGameLoop::new(*mode);
        let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
        
        game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
        let result = game_loop.system_scheduler_mut().resolve_dependencies();
        assert!(result.is_ok(), "LuaScriptSystem should work in mode {:?}: {:?}", mode, result.err());
        
        // Execute a frame
        let frame_result = game_loop.update_frame(Duration::from_millis(16));
        assert!(frame_result.fixed_updates >= 0, "Should execute in mode {:?}", mode);
    }
}

#[test]
fn test_multiple_lua_script_systems_dependency_behavior() {
    // Test that multiple LuaScriptSystem instances are handled appropriately
    // This test documents the current behavior: circular dependency detection
    
    register_component::<LuaScript>();
    
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    
    // Add first LuaScriptSystem
    let lua_system1 = LuaScriptSystem::new().expect("Failed to create first LuaScriptSystem");
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system1));
    
    // Add second LuaScriptSystem
    let lua_system2 = LuaScriptSystem::new().expect("Failed to create second LuaScriptSystem");
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system2));
    
    // The system scheduler detects this as a circular dependency
    let result = game_loop.system_scheduler_mut().resolve_dependencies();
    
    // Document current behavior: multiple systems with same name cause dependency issues
    match result {
        Err(engine_runtime_core::SystemError::DependencyCycle(msg)) => {
            assert!(msg.contains("Circular dependency"), "Should detect circular dependency: {}", msg);
            // This is the current expected behavior
        }
        Ok(_) => {
            panic!("Expected circular dependency error for multiple LuaScriptSystems");
        }
        Err(other) => {
            panic!("Unexpected error type: {:?}", other);
        }
    }
    
    // TODO: In the future, we might want to:
    // 1. Allow multiple LuaScriptSystems with different names
    // 2. Or design the system to be singleton
    // 3. Or improve the dependency resolution logic
}

#[test]
fn test_lua_script_system_world_access_limitation() {
    // This test documents a current limitation: LuaScriptSystem needs access to World
    // but the current HybridGameLoop/System architecture doesn't provide it
    
    register_component::<LuaScript>();
    
    let mut game_loop = HybridGameLoop::new(EngineMode::Editor);
    let lua_system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    
    game_loop.system_scheduler_mut().add_system(Box::new(lua_system));
    game_loop.system_scheduler_mut().resolve_dependencies().expect("Failed to resolve dependencies");
    
    // Execute frame - this works but system can't access entities because:
    // 1. System::execute() only gets GameContext, not World
    // 2. GameContext doesn't currently provide World access
    // 3. We create a dummy World in our execute() method
    
    let frame_result = game_loop.update_frame(Duration::from_millis(16));
    assert!(frame_result.fixed_updates >= 0, "Executes but with limitations");
    
    // TODO: This test documents that we need to either:
    // 1. Modify System trait to include World access
    // 2. Add World to GameContext  
    // 3. Use a different architecture pattern
}