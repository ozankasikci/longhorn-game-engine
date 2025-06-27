//! Tests for additional Lua component bindings beyond Transform
//! 
//! This test suite verifies that Lua scripts can access and modify various
//! component types including Health, Velocity, and custom components.

use engine_scripting::{LuaScriptSystem, components::{LuaScript, Health, Transform, Velocity}};
use engine_ecs_core::{World, register_component};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_lua_script_can_read_health_component() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Health>();
    
    // Create world and entity with Health
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Health { current: 75, max: 100 }).unwrap();
    
    // Create Lua script that reads health
    let script_content = r#"
        local script = {}
        
        function script:init()
            local health = self.entity:get_component("Health")
            if health then
                print("Initial health:", health.current, "/", health.max)
                assert(health.current == 75, "Current health should be 75")
                assert(health.max == 100, "Max health should be 100")
            else
                error("Health component not found")
            end
        end
        
        function script:update(delta_time)
            local health = self.entity:get_component("Health")
            if health then
                print("Current health:", health.current, "/", health.max)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // This should work - script should be able to read Health component
    let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
    assert!(result.is_ok(), "Script should be able to read Health component: {:?}", result.err());
}

#[test]
fn test_lua_script_can_modify_health_component() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Health>();
    
    // Create world and entity with Health
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Health { current: 100, max: 100 }).unwrap();
    
    // Create Lua script that damages the entity
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            local health = self.entity:get_component("Health")
            if health then
                -- Apply damage over time
                health.current = health.current - math.floor(10 * delta_time)
                if health.current < 0 then
                    health.current = 0
                end
                self.entity:set_component("Health", health)
                print("Applied damage, health now:", health.current)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute multiple frames
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for several frames
    for _ in 0..5 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 1.0); // 1 second per frame for easy math
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Check that health was reduced
    let world_lock = world_arc.lock().unwrap();
    let health = world_lock.get_component::<Health>(entity).expect("Health component should exist");
    assert!(health.current < 100, "Health should be reduced from damage");
    assert!(health.current >= 50, "Health should not be reduced too much (5 seconds * 10 damage/sec = 50 damage)");
}

#[test]
fn test_lua_script_can_access_velocity_component() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Velocity>();
    register_component::<Transform>();
    
    // Create world and entity with Velocity and Transform
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Velocity::new([10.0, 0.0, 0.0], [0.0, 0.0, 0.0])).unwrap();
    let initial_transform = Transform::identity();
    world.add_component(entity, initial_transform.clone()).unwrap();
    
    // Initialize shared state with the Transform
    engine_scripting::shared_state::clear_shared_state();
    engine_scripting::shared_state::init_entity_transform(entity, initial_transform);
    
    // Create Lua script that uses velocity to move
    let script_content = r#"
        local script = {}
        
        function script:init()
            print("Script initialized!")
        end
        
        function script:update(delta_time)
            print("Script update called with delta_time:", delta_time)
            local velocity = self.entity:get_component("Velocity")
            local transform = self.entity:get_component("Transform")
            
            print("Got components - velocity:", velocity ~= nil, "transform:", transform ~= nil)
            
            if velocity and transform then
                print("Velocity linear:", velocity.linear.x, velocity.linear.y, velocity.linear.z)
                print("Transform position before:", transform.position.x, transform.position.y, transform.position.z)
                
                -- Apply velocity to position
                transform.position.x = transform.position.x + velocity.linear.x * delta_time
                transform.position.y = transform.position.y + velocity.linear.y * delta_time
                transform.position.z = transform.position.z + velocity.linear.z * delta_time
                
                print("Transform position after:", transform.position.x, transform.position.y, transform.position.z)
                self.entity:set_component("Transform", transform)
                print("Position after velocity:", transform.position.x, transform.position.y, transform.position.z)
            else
                if not velocity then
                    print("ERROR: Velocity component is nil")
                end
                if not transform then
                    print("ERROR: Transform component is nil")
                end
                error("Missing required components")
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    let lua_script = LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    };
    world.add_component(entity, lua_script).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute for several frames
    for _ in 0..10 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.1); // 0.1 seconds per frame
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Verify position changed based on velocity (check shared state)
    let transform = engine_scripting::shared_state::get_entity_transform(entity)
        .expect("Transform should exist in shared state");
    // After 10 frames * 0.1 sec/frame * 10 units/sec = 10 units moved
    assert!((transform.position[0] - 10.0).abs() < 0.01, "X position should have moved by velocity");
}

#[test]
fn test_lua_script_multiple_component_interactions() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Health>();
    register_component::<Transform>();
    register_component::<Velocity>();
    
    // Create world with entity having multiple components
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Health { current: 100, max: 100 }).unwrap();
    let initial_transform = Transform::identity();
    world.add_component(entity, initial_transform.clone()).unwrap();
    world.add_component(entity, Velocity::new([5.0, 0.0, 0.0], [0.0, 0.0, 0.0])).unwrap();
    
    // Initialize shared state with the Transform
    engine_scripting::shared_state::clear_shared_state();
    engine_scripting::shared_state::init_entity_transform(entity, initial_transform);
    
    // Create script that uses multiple components together
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            local health = self.entity:get_component("Health")
            local transform = self.entity:get_component("Transform")
            local velocity = self.entity:get_component("Velocity")
            
            if health and transform and velocity then
                -- Move based on velocity
                transform.position.x = transform.position.x + velocity.linear.x * delta_time
                self.entity:set_component("Transform", transform)
                
                -- Take damage if moving too fast
                local speed = math.sqrt(velocity.linear.x^2 + velocity.linear.y^2 + velocity.linear.z^2)
                if speed > 4.0 then
                    health.current = health.current - 1
                    self.entity:set_component("Health", health)
                end
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    world.add_component(entity, LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute frames
    for _ in 0..10 {
        let result = system.execute_scripts_from_world(world_arc.clone(), 0.1);
        assert!(result.is_ok(), "Script execution should succeed: {:?}", result.err());
    }
    
    // Verify both movement and damage occurred
    let health_current = {
        let world_lock = world_arc.lock().unwrap();
        let health = world_lock.get_component::<Health>(entity).expect("Health should exist");
        health.current
    };
    
    let transform = engine_scripting::shared_state::get_entity_transform(entity)
        .expect("Transform should exist in shared state");
    
    assert!(transform.position[0] > 0.0, "Entity should have moved");
    assert!(health_current < 100, "Entity should have taken damage from high speed");
}

#[test]
fn test_lua_script_handles_missing_components_gracefully() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Transform>();
    
    // Create world and entity with only Transform (no Health or Velocity)
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Transform::identity()).unwrap();
    
    // Create script that tries to access missing components
    let script_content = r#"
        local script = {}
        
        function script:update(delta_time)
            local health = self.entity:get_component("Health")
            local velocity = self.entity:get_component("Velocity")
            local transform = self.entity:get_component("Transform")
            
            -- Should handle nil components gracefully
            if not health then
                print("No health component found - that's ok")
            end
            
            if not velocity then
                print("No velocity component found - using default")
                -- Use a default velocity
                if transform then
                    transform.position.x = transform.position.x + 1.0 * delta_time
                    self.entity:set_component("Transform", transform)
                end
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    world.add_component(entity, LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Should not crash when components are missing
    let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
    assert!(result.is_ok(), "Script should handle missing components gracefully: {:?}", result.err());
}

#[test]
fn test_lua_script_can_add_components_to_entity() {
    // Register components
    register_component::<LuaScript>();
    register_component::<Health>();
    register_component::<Transform>();
    
    // Create world and entity with only Transform
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Transform::identity()).unwrap();
    
    // Create script that adds Health component
    let script_content = r#"
        local script = {}
        
        function script:init()
            -- Check if entity has health
            local health = self.entity:get_component("Health")
            if not health then
                -- Add health component
                self.entity:add_component("Health", {
                    current = 50,
                    max = 50
                })
                print("Added Health component to entity")
            end
        end
        
        function script:update(delta_time)
            local health = self.entity:get_component("Health")
            if health then
                print("Entity has health:", health.current, "/", health.max)
            end
        end
        
        return script
    "#;
    
    // Create temporary script file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(script_content.as_bytes()).expect("Failed to write script");
    let script_path = temp_file.path().to_string_lossy().to_string();
    
    // Add LuaScript component
    world.add_component(entity, LuaScript {
        script_path: script_path.clone(),
        enabled: true,
        execution_order: 0,
        instance_id: None,
        additional_scripts: Vec::new(),
    }).unwrap();
    
    // Create system and execute
    let mut system = LuaScriptSystem::new().expect("Failed to create LuaScriptSystem");
    let world_arc = Arc::new(Mutex::new(world));
    
    // Execute to trigger init
    let result = system.execute_scripts_from_world(world_arc.clone(), 0.016);
    assert!(result.is_ok(), "Script should be able to add components: {:?}", result.err());
    
    // Verify Health component was added
    let world_lock = world_arc.lock().unwrap();
    let health = world_lock.get_component::<Health>(entity);
    assert!(health.is_some(), "Health component should have been added");
    assert_eq!(health.unwrap().current, 50, "Health should be initialized to 50");
}