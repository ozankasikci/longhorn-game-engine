//! Tests for Lua Transform component bindings
//! 
//! This test suite verifies that Lua scripts can access and modify Transform components
//! on entities through the ECS system integration.

use engine_scripting::{LuaScriptSystem, components::{LuaScript, Transform}};
use engine_ecs_core::{World, register_component, Entity};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_lua_script_can_read_transform_position() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity with Transform
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Transform::new([10.0, 20.0, 30.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])).unwrap();
    
    // Create Lua script that reads position
    let script_content = r#"
        local script = {}
        
        function script:init()
            -- Store initial position for verification
            local transform = self.entity:get_component("Transform")
            if transform then
                self.initial_x = transform.position.x
                self.initial_y = transform.position.y
                self.initial_z = transform.position.z
            end
        end
        
        function script:update(delta_time)
            -- Verify we can still read the position
            local transform = self.entity:get_component("Transform")
            if transform then
                print("Position: ", transform.position.x, transform.position.y, transform.position.z)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component to entity
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create LuaScriptSystem and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // This should work - script should be able to read Transform position
    let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
    assert!(result.is_ok(), "Script should be able to read Transform position: {:?}", result.err());
    
    // Verify script was executed by checking it was loaded
    assert!(system.has_loaded_script(&script_path), "Script should have been loaded");
}

#[test]
fn test_lua_script_can_modify_transform_position() {
    // Clear shared state for clean test
    engine_scripting::shared_state::clear_shared_state();
    
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity with Transform
    let mut world = World::new();
    let entity = world.spawn();
    let initial_transform = Transform::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    world.add_component(entity, initial_transform.clone()).unwrap();
    
    // Initialize shared state with the Transform
    engine_scripting::shared_state::init_entity_transform(entity, initial_transform);
    
    // Create Lua script that modifies position
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            -- Move the entity
            local transform = self.entity:get_component("Transform")
            if transform then
                transform.position.x = transform.position.x + 1.0 * delta_time
                transform.position.y = transform.position.y + 2.0 * delta_time
                transform.position.z = transform.position.z + 3.0 * delta_time
                self.entity:set_component("Transform", transform)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component to entity
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create LuaScriptSystem and execute multiple frames
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for 5 frames
    for _ in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Check that Transform was modified in shared state
    let updated_transform = engine_scripting::shared_state::get_entity_transform(entity)
        .expect("Transform should exist in shared state");
    
    // After 5 frames at 0.016 delta each, position should have moved
    let expected_delta = 5.0 * 0.016;
    assert!((updated_transform.position[0] - expected_delta).abs() < 0.001, "X position should be updated");
    assert!((updated_transform.position[1] - 2.0 * expected_delta).abs() < 0.001, "Y position should be updated");
    assert!((updated_transform.position[2] - 3.0 * expected_delta).abs() < 0.001, "Z position should be updated");
}

#[test]
fn test_lua_script_can_modify_transform_rotation() {
    // Clear shared state for clean test
    engine_scripting::shared_state::clear_shared_state();
    
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity with Transform
    let mut world = World::new();
    let entity = world.spawn();
    let initial_transform = Transform::identity();
    world.add_component(entity, initial_transform.clone()).unwrap();
    
    // Initialize shared state with the Transform
    engine_scripting::shared_state::init_entity_transform(entity, initial_transform);
    
    // Create Lua script that modifies rotation
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            -- Rotate the entity
            local transform = self.entity:get_component("Transform")
            if transform then
                transform.rotation.y = transform.rotation.y + 90.0 * delta_time
                self.entity:set_component("Transform", transform)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component to entity
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create LuaScriptSystem and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for multiple frames
    for _ in 0..10 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Check that Transform rotation was modified in shared state
    let updated_transform = engine_scripting::shared_state::get_entity_transform(entity)
        .expect("Transform should exist in shared state");
    
    // After 10 frames, rotation should have changed
    let expected_rotation = 10.0 * 0.016 * 90.0;
    assert!((updated_transform.rotation[1] - expected_rotation).abs() < 0.001, "Y rotation should be updated");
}

#[test]
fn test_lua_script_can_modify_transform_scale() {
    // Clear shared state for clean test
    engine_scripting::shared_state::clear_shared_state();
    
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity with Transform
    let mut world = World::new();
    let entity = world.spawn();
    let initial_transform = Transform::identity();
    world.add_component(entity, initial_transform.clone()).unwrap();
    
    // Initialize shared state with the Transform
    engine_scripting::shared_state::init_entity_transform(entity, initial_transform);
    
    // Create Lua script that modifies scale
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            -- Scale the entity over time
            local transform = self.entity:get_component("Transform")
            if transform then
                local scale_factor = 1.0 + 0.5 * delta_time
                transform.scale.x = transform.scale.x * scale_factor
                transform.scale.y = transform.scale.y * scale_factor
                transform.scale.z = transform.scale.z * scale_factor
                self.entity:set_component("Transform", transform)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component to entity
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create LuaScriptSystem and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for multiple frames
    for _ in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Check that Transform scale was modified in shared state
    let updated_transform = engine_scripting::shared_state::get_entity_transform(entity)
        .expect("Transform should exist in shared state");
    
    // Scale should be larger than 1.0 after multiple frames
    assert!(updated_transform.scale[0] > 1.0, "X scale should have grown");
    assert!(updated_transform.scale[1] > 1.0, "Y scale should have grown");
    assert!(updated_transform.scale[2] > 1.0, "Z scale should have grown");
}

#[test]
fn test_multiple_entities_with_transform_scripts() {
    // Clear shared state for clean test
    engine_scripting::shared_state::clear_shared_state();
    
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world with multiple entities
    let mut world = World::new();
    
    // Entity 1: moves in X direction
    let entity1 = world.spawn();
    let transform1 = Transform::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    world.add_component(entity1, transform1.clone()).unwrap();
    engine_scripting::shared_state::init_entity_transform(entity1, transform1);
    
    // Entity 2: moves in Y direction  
    let entity2 = world.spawn();
    let transform2 = Transform::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    world.add_component(entity2, transform2.clone()).unwrap();
    engine_scripting::shared_state::init_entity_transform(entity2, transform2);
    
    // Create scripts for each entity
    let script1_content = r#"
        local script = {}
        function script:update(delta_time)
            local transform = self.entity:get_component("Transform")
            if transform then
                transform.position.x = transform.position.x + 1.0 * delta_time
                self.entity:set_component("Transform", transform)
            end
        end
        return script
    "#;
    
    let script2_content = r#"
        local script = {}
        function script:update(delta_time)
            local transform = self.entity:get_component("Transform")
            if transform then
                transform.position.y = transform.position.y + 2.0 * delta_time
                self.entity:set_component("Transform", transform)
            end
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
    
    // Add LuaScript components
    world.add_component(entity1, LuaScript {
        script_path: script_path1.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    world.add_component(entity2, LuaScript {
        script_path: script_path2.clone(),
        enabled: true,
        execution_order: 1,
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    // Create LuaScriptSystem and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for multiple frames
    for _ in 0..10 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Check that both entities moved in their respective directions from shared state
    let transform1 = engine_scripting::shared_state::get_entity_transform(entity1)
        .expect("Transform1 should exist in shared state");
    let transform2 = engine_scripting::shared_state::get_entity_transform(entity2)
        .expect("Transform2 should exist in shared state");
    
    let expected_delta = 10.0 * 0.016;
    
    // Entity 1 should have moved in X
    assert!((transform1.position[0] - expected_delta).abs() < 0.001, "Entity1 X should be updated");
    assert!(transform1.position[1].abs() < 0.001, "Entity1 Y should be unchanged");
    
    // Entity 2 should have moved in Y
    assert!(transform2.position[0].abs() < 0.001, "Entity2 X should be unchanged");
    assert!((transform2.position[1] - 2.0 * expected_delta).abs() < 0.001, "Entity2 Y should be updated");
}

#[test]
fn test_script_without_transform_component_handles_gracefully() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity WITHOUT Transform
    let mut world = World::new();
    let entity = world.spawn();
    // Don't add Transform component
    
    // Create Lua script that tries to access Transform
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            -- Try to access Transform that doesn't exist
            local transform = self.entity:get_component("Transform")
            if transform then
                transform.position.x = 10.0
                self.entity:set_component("Transform", transform)
            else
                print("No Transform component found - this is expected")
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component to entity
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create LuaScriptSystem and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // This should work without crashing - script should handle missing component gracefully
    let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
    assert!(result.is_ok(), "Script should handle missing Transform gracefully: {:?}", result.err());
    
    // Verify entity still exists and has no Transform
    let world_lock = world_arc.lock().unwrap();
    assert!(world_lock.contains(entity), "Entity should still exist");
    assert!(!world_lock.has_component::<Transform>(entity), "Entity should still have no Transform");
}