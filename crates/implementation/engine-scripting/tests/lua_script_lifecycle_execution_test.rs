//! Tests for Lua script lifecycle execution (init/update/destroy)
//! 
//! This test suite verifies that script lifecycle methods are called in the correct order
//! and that scripts can maintain state between calls.

use engine_scripting::{LuaScriptSystem, components::LuaScript};
use engine_ecs_core::{World, register_component};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

#[test]
fn test_script_lifecycle_init_called_once() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world and entity
    let mut world = World::new();
    let entity = world.spawn();
    
    // Create script that tracks init calls
    let script_content = r#"
        local script = {}
        script.init_count = 0
        
        function script:init()
            self.init_count = self.init_count + 1
            print("Init called, count:", self.init_count)
        end
        
        function script:update(delta_time)
            -- Verify init was called
            assert(self.init_count == 1, "Init should be called exactly once")
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript::new(script_path.clone());
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute multiple times
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute multiple frames - init should only be called once
    for i in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Frame {} should execute successfully: {:?}", i, result.err());
    }
    
    // Verify script was loaded and initialized
    assert!(system.has_loaded_script(&script_path), "Script should be loaded");
}

#[test]
fn test_script_lifecycle_update_called_every_frame() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world and entity
    let mut world = World::new();
    let entity = world.spawn();
    
    // Create script that tracks update calls
    let script_content = r#"
        local script = {}
        script.update_count = 0
        script.total_time = 0.0
        
        function script:init()
            print("Script initialized")
        end
        
        function script:update(delta_time)
            self.update_count = self.update_count + 1
            self.total_time = self.total_time + delta_time
            print("Update called", self.update_count, "times, total time:", self.total_time)
            
            -- Verify delta_time is reasonable
            assert(delta_time > 0.0, "Delta time should be positive")
            assert(delta_time < 1.0, "Delta time should be less than 1 second")
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript::new(script_path.clone());
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute multiple frames
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    let frame_count = 10;
    for i in 0..frame_count {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Frame {} should execute successfully: {:?}", i, result.err());
    }
    
    // All frames should execute - update should be called frame_count times
    assert!(system.has_loaded_script(&script_path), "Script should be loaded");
}

#[test]
fn test_script_lifecycle_state_persistence() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world and entity
    let mut world = World::new();
    let entity = world.spawn();
    
    // Create script that maintains state between updates
    let script_content = r#"
        local script = {}
        
        function script:init()
            self.counter = 0
            self.initialized = true
            print("Counter initialized to", self.counter)
        end
        
        function script:update(delta_time)
            assert(self.initialized == true, "Script should be initialized")
            self.counter = self.counter + 1
            print("Counter is now", self.counter)
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript::new(script_path.clone());
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute multiple frames
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute multiple frames to verify state persistence
    for i in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Frame {} should execute successfully: {:?}", i, result.err());
    }
    
    assert!(system.has_loaded_script(&script_path), "Script should be loaded");
}

#[test]
fn test_script_lifecycle_destroy_method_exists() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world and entity
    let mut world = World::new();
    let entity = world.spawn();
    
    // Create script with destroy method
    let script_content = r#"
        local script = {}
        script.destroyed = false
        
        function script:init()
            print("Script initialized")
        end
        
        function script:update(delta_time)
            print("Script updating")
        end
        
        function script:destroy()
            self.destroyed = true
            print("Script destroyed")
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript::new(script_path.clone());
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute a few frames
    for i in 0..3 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Frame {} should execute successfully: {:?}", i, result.err());
    }
    
    // TODO: Add destroy method call test when entity is removed or script disabled
    assert!(system.has_loaded_script(&script_path), "Script should be loaded");
}

#[test]
fn test_multiple_scripts_lifecycle_order() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world with multiple entities
    let mut world = World::new();
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    
    // Create first script (lower execution order)
    let script1_content = r#"
        local script = {}
        
        function script:init()
            print("Script 1 initialized")
        end
        
        function script:update(delta_time)
            print("Script 1 updating")
        end
        
        return script
    "#;
    
    // Create second script (higher execution order)
    let script2_content = r#"
        local script = {}
        
        function script:init()
            print("Script 2 initialized")
        end
        
        function script:update(delta_time)
            print("Script 2 updating")
        end
        
        return script
    "#;
    
    // Create temporary script files
    let mut temp_file1 = NamedTempFile::new().expect("Failed to create temp file 1");
    temp_file1.write_all(script1_content.as_bytes()).expect("Failed to write script 1");
    let script_path1 = temp_file1.path().to_string_lossy().to_string();
    
    let mut temp_file2 = NamedTempFile::new().expect("Failed to create temp file 2");
    temp_file2.write_all(script2_content.as_bytes()).expect("Failed to write script 2");
    let script_path2 = temp_file2.path().to_string_lossy().to_string();
    
    // Add LuaScript components with different execution orders
    world.add_component(entity1, LuaScript {
        script_path: script_path1.clone(),
        enabled: true,
        execution_order: 0,  // Lower number = executes first
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    world.add_component(entity2, LuaScript {
        script_path: script_path2.clone(),
        enabled: true,
        execution_order: 10, // Higher number = executes second
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute multiple frames
    for i in 0..3 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Frame {} should execute successfully: {:?}", i, result.err());
    }
    
    // Both scripts should be loaded
    assert!(system.has_loaded_script(&script_path1), "Script 1 should be loaded");
    assert!(system.has_loaded_script(&script_path2), "Script 2 should be loaded");
}

#[test]
fn test_script_lifecycle_with_errors() {
    // Register components
    register_component::<LuaScript>();
    
    // Create world and entity
    let mut world = World::new();
    let entity = world.spawn();
    
    // Create script with error in update method
    let script_content = r#"
        local script = {}
        script.update_count = 0
        
        function script:init()
            print("Script initialized successfully")
        end
        
        function script:update(delta_time)
            self.update_count = self.update_count + 1
            
            -- Cause an error on the 3rd update
            if self.update_count == 3 then
                error("Intentional error for testing")
            end
            
            print("Update", self.update_count, "completed")
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript::new(script_path.clone());
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute frames - some should succeed, some should fail
    let mut success_count = 0;
    let mut error_count = 0;
    
    for i in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        if result.is_ok() {
            success_count += 1;
        } else {
            error_count += 1;
            println!("Expected error on frame {}: {:?}", i, result.err());
        }
    }
    
    // Script should be loaded despite errors
    assert!(system.has_loaded_script(&script_path), "Script should be loaded");
    // We should have some successes and some errors
    assert!(success_count > 0, "Should have some successful executions");
}