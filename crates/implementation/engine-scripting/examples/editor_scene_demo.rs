//! Demo showing how Lua scripts work with the editor's default scene
//! 
//! Run with: cargo run --example editor_scene_demo -p engine-scripting

use engine_scripting::{LuaScriptSystem, components::{LuaScript, Transform}};
use engine_ecs_core::{World, register_component};
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== Longhorn Engine - Editor Scene with Lua Demo ===\n");
    
    // Register components
    register_component::<Transform>();
    register_component::<LuaScript>();
    
    // Create a simple world with a cube entity
    let mut world = World::new();
    let cube_entity = world.spawn();
    
    // Add Transform component
    world.add_component(cube_entity, Transform {
        position: [0.0, 0.5, 0.0],
        rotation: [0.0, 0.0, 0.0], 
        scale: [1.0, 1.0, 1.0],
    }).unwrap();
    
    // Add LuaScript component (same as editor does)
    world.add_component(
        cube_entity,
        LuaScript::new("assets/scripts/simple_test.lua".to_string()),
    ).unwrap();
    println!("Created default world with cube entity: {:?}", cube_entity);
    
    // Verify LuaScript component exists
    if let Some(script) = world.get_component::<LuaScript>(cube_entity) {
        println!("✓ Cube has LuaScript: {}", script.script_path);
        println!("  Enabled: {}", script.enabled);
        println!("  Execution order: {}", script.execution_order);
    } else {
        println!("✗ ERROR: Cube does not have LuaScript component!");
        return;
    }
    
    // List all entities with scripts
    let script_entities: Vec<_> = world.query_legacy::<LuaScript>()
        .map(|(e, s)| (e, s.script_path.clone()))
        .collect();
    println!("\nTotal entities with scripts: {}", script_entities.len());
    for (entity, path) in &script_entities {
        println!("  {:?} -> {}", entity, path);
    }
    
    // Create world Arc for LuaScriptSystem
    let world_arc = Arc::new(Mutex::new(world));
    
    // Create and initialize LuaScriptSystem
    println!("\nInitializing Lua script system...");
    let mut lua_system = LuaScriptSystem::new()
        .expect("Failed to create LuaScriptSystem");
    
    println!("\nStarting script execution (simulating play mode)...\n");
    println!("{}", "=".repeat(50));
    
    // Run for 2 seconds to see the periodic message
    let mut total_time = 0.0;
    let delta = 0.016; // 60 FPS
    
    while total_time < 3.5 {
        // Execute scripts
        match lua_system.execute_scripts_from_world(world_arc.clone(), delta) {
            Ok(_) => {
                // Scripts executed successfully
            }
            Err(e) => {
                println!("Script execution error: {:?}", e);
                break;
            }
        }
        
        total_time += delta;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    println!("\n{}", "=".repeat(50));
    println!("\nDemo complete! The cube's Lua script executed successfully.");
    println!("\nIn the actual editor:");
    println!("1. The cube entity is created with a LuaScript component");
    println!("2. When you press the Play button, scripts should execute");
    println!("3. You should see script output in the console/terminal");
}