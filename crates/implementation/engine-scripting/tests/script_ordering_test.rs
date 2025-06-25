//! Script execution ordering system tests

use engine_scripting::{
    lua::LuaScriptEngine,
    runtime::ScriptRuntime,
    components::{Transform, LuaScript},
};
use engine_ecs_core::World;
use std::sync::{Arc, Mutex};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_script_execution_order_by_priority() {
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<LuaScript>();
    
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let world = Arc::new(Mutex::new(World::new()));
    engine_scripting::lua::ecs::setup_ecs_bindings(engine.lua(), engine.engine_table(), world.clone())
        .expect("Failed to setup ECS bindings");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create scripts with different priorities
    let high_priority_script = temp_dir.path().join("high_priority.lua");
    let medium_priority_script = temp_dir.path().join("medium_priority.lua");
    let low_priority_script = temp_dir.path().join("low_priority.lua");
    
    // High priority script (should execute first)
    let high_script_content = r#"
        local HighPriorityScript = {}
        
        execution_order = execution_order or {}
        
        function HighPriorityScript:update(dt)
            table.insert(execution_order, "high")
        end
        
        return HighPriorityScript
    "#;
    
    // Medium priority script
    let medium_script_content = r#"
        local MediumPriorityScript = {}
        
        execution_order = execution_order or {}
        
        function MediumPriorityScript:update(dt)
            table.insert(execution_order, "medium")
        end
        
        return MediumPriorityScript
    "#;
    
    // Low priority script (should execute last)
    let low_script_content = r#"
        local LowPriorityScript = {}
        
        execution_order = execution_order or {}
        
        function LowPriorityScript:update(dt)
            table.insert(execution_order, "low")
        end
        
        return LowPriorityScript
    "#;
    
    fs::write(&high_priority_script, high_script_content).expect("Failed to write high priority script");
    fs::write(&medium_priority_script, medium_script_content).expect("Failed to write medium priority script");
    fs::write(&low_priority_script, low_script_content).expect("Failed to write low priority script");
    
    // Create entities with scripts in reverse order (low, medium, high)
    // This tests that execution order is determined by priority, not creation order
    let create_entities_script = format!(r#"
        -- Create low priority entity first
        local low_entity = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 100  -- Lower number = higher priority
            }}
        }})
        
        -- Create medium priority entity
        local medium_entity = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 50   -- Medium priority
            }}
        }})
        
        -- Create high priority entity last
        local high_entity = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 10   -- Higher priority (executes first)
            }}
        }})
        
        test_entities = {{low_entity, medium_entity, high_entity}}
    "#, 
        low_priority_script.to_string_lossy(),
        medium_priority_script.to_string_lossy(),
        high_priority_script.to_string_lossy()
    );
    
    engine.lua().load(&create_entities_script).exec().expect("Failed to create entities");
    
    // Execute scripts using the ordering system
    engine.update_script_systems_ordered(world.clone(), 0.016).expect("Failed to update script systems");
    
    // Verify execution order: high, medium, low
    let execution_order: Vec<String> = engine.lua().globals().get("execution_order").expect("Failed to get execution order");
    assert_eq!(execution_order, vec!["high", "medium", "low"]);
}

#[test]
fn test_script_execution_order_same_priority() {
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<LuaScript>();
    
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let world = Arc::new(Mutex::new(World::new()));
    engine_scripting::lua::ecs::setup_ecs_bindings(engine.lua(), engine.engine_table(), world.clone())
        .expect("Failed to setup ECS bindings");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create scripts with same priority
    let script1_path = temp_dir.path().join("script1.lua");
    let script2_path = temp_dir.path().join("script2.lua");
    
    let script1_content = r#"
        local Script1 = {}
        
        execution_order = execution_order or {}
        
        function Script1:update(dt)
            table.insert(execution_order, "script1")
        end
        
        return Script1
    "#;
    
    let script2_content = r#"
        local Script2 = {}
        
        execution_order = execution_order or {}
        
        function Script2:update(dt)
            table.insert(execution_order, "script2")
        end
        
        return Script2
    "#;
    
    fs::write(&script1_path, script1_content).expect("Failed to write script1");
    fs::write(&script2_path, script2_content).expect("Failed to write script2");
    
    // Create entities with same priority - should execute in creation order
    let create_entities_script = format!(r#"
        local entity1 = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 0
            }}
        }})
        
        local entity2 = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 0
            }}
        }})
        
        test_entities = {{entity1, entity2}}
    "#, 
        script1_path.to_string_lossy(),
        script2_path.to_string_lossy()
    );
    
    engine.lua().load(&create_entities_script).exec().expect("Failed to create entities");
    
    // Execute scripts
    engine.update_script_systems_ordered(world.clone(), 0.016).expect("Failed to update script systems");
    
    // Verify execution order follows creation order when priorities are same
    let execution_order: Vec<String> = engine.lua().globals().get("execution_order").expect("Failed to get execution order");
    assert_eq!(execution_order, vec!["script1", "script2"]);
}

#[test]
fn test_script_execution_order_disabled_scripts() {
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<LuaScript>();
    
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let world = Arc::new(Mutex::new(World::new()));
    engine_scripting::lua::ecs::setup_ecs_bindings(engine.lua(), engine.engine_table(), world.clone())
        .expect("Failed to setup ECS bindings");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    let enabled_script = temp_dir.path().join("enabled.lua");
    let disabled_script = temp_dir.path().join("disabled.lua");
    
    let enabled_content = r#"
        local EnabledScript = {}
        
        execution_order = execution_order or {}
        
        function EnabledScript:update(dt)
            table.insert(execution_order, "enabled")
        end
        
        return EnabledScript
    "#;
    
    let disabled_content = r#"
        local DisabledScript = {}
        
        execution_order = execution_order or {}
        
        function DisabledScript:update(dt)
            table.insert(execution_order, "disabled")
        end
        
        return DisabledScript
    "#;
    
    fs::write(&enabled_script, enabled_content).expect("Failed to write enabled script");
    fs::write(&disabled_script, disabled_content).expect("Failed to write disabled script");
    
    // Create entities with one disabled script
    let create_entities_script = format!(r#"
        local enabled_entity = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = true,
                execution_order = 0
            }}
        }})
        
        local disabled_entity = engine.world:create_entity({{
            Transform = {{
                position = {{x = 0, y = 0, z = 0}},
                rotation = {{x = 0, y = 0, z = 0, w = 1}},
                scale = {{x = 1, y = 1, z = 1}}
            }},
            LuaScript = {{
                script_path = "{}",
                enabled = false,  -- Disabled script
                execution_order = 0
            }}
        }})
        
        test_entities = {{enabled_entity, disabled_entity}}
    "#, 
        enabled_script.to_string_lossy(),
        disabled_script.to_string_lossy()
    );
    
    engine.lua().load(&create_entities_script).exec().expect("Failed to create entities");
    
    // Execute scripts
    engine.update_script_systems_ordered(world.clone(), 0.016).expect("Failed to update script systems");
    
    // Verify only enabled script executed
    let execution_order: Vec<String> = engine.lua().globals().get("execution_order").expect("Failed to get execution order");
    assert_eq!(execution_order, vec!["enabled"]);
}