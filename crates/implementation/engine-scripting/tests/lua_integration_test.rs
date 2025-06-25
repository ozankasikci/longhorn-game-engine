//! Integration tests for Lua scripting

use engine_scripting::{
    lua::LuaScriptEngine,
    runtime::{ScriptRuntime, create_runtime},
    ScriptId, ScriptMetadata, ScriptType,
};

#[test]
fn test_lua_runtime_creation() {
    let runtime = create_runtime(ScriptType::Lua).expect("Failed to create Lua runtime");
    assert!(runtime.supports_type(&ScriptType::Lua));
    assert!(!runtime.supports_type(&ScriptType::JavaScript));
}

#[test]
fn test_lua_math_bindings() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
    engine.initialize().expect("Failed to initialize");

    let script = r#"
        local v1 = engine.math.vec3(1, 2, 3)
        local v2 = engine.math.vec3(4, 5, 6)
        local v3 = v1:add(v2)
        
        assert(v3.x == 5)
        assert(v3.y == 7)
        assert(v3.z == 9)
        
        print("Math test passed!")
    "#;

    let metadata = ScriptMetadata {
        id: ScriptId(1),
        script_type: ScriptType::Lua,
        path: "test_math.lua".to_string(),
        entry_point: None,
    };

    engine.load_script_internal(metadata, script).expect("Failed to load script");
    engine.execute_script_internal(ScriptId(1)).expect("Failed to execute script");
}

#[test]
fn test_lua_time_and_update() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
    engine.initialize().expect("Failed to initialize");

    let script = r#"
        update_count = 0  -- Make it global
        total_dt = 0      -- Make it global
        
        function update(dt)
            update_count = update_count + 1
            total_dt = total_dt + dt
            
            print("Update called, dt: " .. dt)
            print("Total time: " .. engine.time.total_time)
        end
    "#;

    let metadata = ScriptMetadata {
        id: ScriptId(2),
        script_type: ScriptType::Lua,
        path: "test_update.lua".to_string(),
        entry_point: None,
    };

    engine.load_script_internal(metadata, script).expect("Failed to load script");
    
    // Simulate frame updates
    engine.update(0.016).expect("Failed to update");
    engine.update(0.016).expect("Failed to update");
    
    // Verify update was called by checking global variable
    let result = engine.lua().load("return update_count").eval::<i32>();
    assert!(result.is_ok(), "Failed to get update count");
    let count = result.unwrap();
    assert!(count > 0, "Update function was not called");
}

#[test]
fn test_lua_event_system() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
    engine.initialize().expect("Failed to initialize");

    let script = r#"
        local event_received = false
        
        function on_test_event(data)
            print("Event received: " .. data.message)
            event_received = true
        end
        
        engine.events.subscribe("test_event", on_test_event)
        
        -- Emit an event
        engine.events.emit("test_event", { message = "Hello from Lua!" })
        
        _G.event_received = event_received
    "#;

    let metadata = ScriptMetadata {
        id: ScriptId(3),
        script_type: ScriptType::Lua,
        path: "test_events.lua".to_string(),
        entry_point: None,
    };

    engine.load_script_internal(metadata, script).expect("Failed to load script");
    engine.execute_script_internal(ScriptId(3)).expect("Failed to execute script");
}

#[test]
fn test_lua_debug_logging() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
    engine.initialize().expect("Failed to initialize");

    let script = r#"
        engine.debug.log("info", "This is an info message")
        engine.debug.log("debug", "This is a debug message")
        engine.debug.log("warn", "This is a warning message")
        engine.debug.log("error", "This is an error message")
        
        print("Debug logging test completed")
    "#;

    let metadata = ScriptMetadata {
        id: ScriptId(4),
        script_type: ScriptType::Lua,
        path: "test_debug.lua".to_string(),
        entry_point: None,
    };

    engine.load_script_internal(metadata, script).expect("Failed to load script");
    engine.execute_script_internal(ScriptId(4)).expect("Failed to execute script");
}

#[test]
fn test_lua_error_handling() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
    engine.initialize().expect("Failed to initialize");

    // Test syntax error
    let bad_script = r#"
        function broken(
            -- Missing closing parenthesis
    "#;

    let metadata = ScriptMetadata {
        id: ScriptId(5),
        script_type: ScriptType::Lua,
        path: "test_error.lua".to_string(),
        entry_point: None,
    };

    let result = engine.load_script_internal(metadata, bad_script);
    assert!(result.is_err());
    
    // Test runtime error
    let runtime_error_script = r#"
        function update()
            local x = nil
            x.property = 5  -- This will cause a runtime error
        end
    "#;

    let metadata2 = ScriptMetadata {
        id: ScriptId(6),
        script_type: ScriptType::Lua,
        path: "test_runtime_error.lua".to_string(),
        entry_point: None,
    };

    engine.load_script_internal(metadata2, runtime_error_script).expect("Script should load");
    // Try to call the update function which has a runtime error
    let result = engine.run_if_exists("update");
    assert!(result.is_err());
}