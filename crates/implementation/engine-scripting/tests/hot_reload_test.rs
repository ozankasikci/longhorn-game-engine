//! Hot-reload tests for Lua scripting

use engine_scripting::{
    lua::LuaScriptEngine,
    runtime::ScriptRuntime,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_script_file_watching_setup() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    // Create a temporary directory for test scripts
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test_script.lua");
    
    // Write initial script
    let initial_script = r#"
        local test_value = 42
        function get_test_value()
            return test_value
        end
    "#;
    
    fs::write(&script_path, initial_script).expect("Failed to write script");
    
    // Test that we can enable file watching for a directory
    let result = engine.watch_directory(temp_dir.path());
    assert!(result.is_ok(), "Should be able to enable file watching");
    
    // Test that we can load and execute the script
    let script_id = engine.load_script_from_file(&script_path).expect("Failed to load script");
    engine.execute_script(script_id).expect("Failed to execute script");
    
    // Verify initial state
    let result: i32 = engine.lua().globals().get("get_test_value")
        .and_then(|f: mlua::Function| f.call(()))
        .expect("Failed to get test value");
    assert_eq!(result, 42);
}

#[test]  
fn test_hot_reload_script_change() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("reload_test.lua");
    
    // Write initial script
    let initial_script = r#"
        function get_message()
            return "original"
        end
        
        reload_count = (reload_count or 0) + 1
    "#;
    
    fs::write(&script_path, initial_script).expect("Failed to write initial script");
    
    // Enable watching and load script
    engine.watch_directory(temp_dir.path()).expect("Failed to enable watching");
    let script_id = engine.load_script_from_file(&script_path).expect("Failed to load script");
    engine.execute_script(script_id).expect("Failed to execute script");
    
    // Verify initial state  
    let message: String = engine.lua().globals().get("get_message")
        .and_then(|f: mlua::Function| f.call(()))
        .expect("Failed to get message");
    assert_eq!(message, "original");
    
    let reload_count: i32 = engine.lua().globals().get("reload_count").expect("Failed to get reload_count");
    assert_eq!(reload_count, 1);
    
    // Modify the script
    let modified_script = r#"
        function get_message()
            return "modified"
        end
        
        reload_count = (reload_count or 0) + 1
    "#;
    
    // Wait a bit to ensure different modification time
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    fs::write(&script_path, modified_script).expect("Failed to write modified script");
    
    // Trigger hot reload manually (in real implementation this would be automatic)
    engine.check_and_reload_scripts().expect("Failed to check for changes");
    
    // Verify the script was reloaded
    let new_message: String = engine.lua().globals().get("get_message")
        .and_then(|f: mlua::Function| f.call(()))
        .expect("Failed to get new message");
    assert_eq!(new_message, "modified");
    
    let new_reload_count: i32 = engine.lua().globals().get("reload_count").expect("Failed to get new reload_count");
    assert_eq!(new_reload_count, 2);
}

#[test]
fn test_hot_reload_with_state_preservation() {
    let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
    engine.initialize().expect("Failed to initialize");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("state_test.lua");
    
    // Write script that maintains state
    let script_with_state = r#"
        -- Preserve existing state or initialize
        persistent_data = persistent_data or {counter = 0, name = "test"}
        
        function increment_counter()
            persistent_data.counter = persistent_data.counter + 1
            return persistent_data.counter
        end
        
        function get_state()
            return persistent_data
        end
        
        function get_version()
            return "v1"
        end
    "#;
    
    fs::write(&script_path, script_with_state).expect("Failed to write script");
    
    engine.watch_directory(temp_dir.path()).expect("Failed to enable watching");
    let script_id = engine.load_script_from_file(&script_path).expect("Failed to load script");
    engine.execute_script(script_id).expect("Failed to execute script");
    
    // Interact with the script to build up state
    let inc_fn: mlua::Function = engine.lua().globals().get("increment_counter").expect("Failed to get increment function");
    inc_fn.call::<()>(()).expect("Failed to increment");
    inc_fn.call::<()>(()).expect("Failed to increment");
    inc_fn.call::<()>(()).expect("Failed to increment");
    
    let counter: i32 = engine.lua().globals().get("get_state")
        .and_then(|f: mlua::Function| f.call::<mlua::Table>(()))
        .and_then(|t| t.get("counter"))
        .expect("Failed to get counter");
    assert_eq!(counter, 3);
    
    // Modify script but preserve state mechanism
    let modified_script = r#"
        -- Preserve existing state or initialize  
        persistent_data = persistent_data or {counter = 0, name = "test"}
        
        function increment_counter()
            persistent_data.counter = persistent_data.counter + 1
            return persistent_data.counter
        end
        
        function get_state()
            return persistent_data
        end
        
        function get_version()
            return "v2"  -- Changed this
        end
        
        function double_counter()  -- New function
            persistent_data.counter = persistent_data.counter * 2
            return persistent_data.counter
        end
    "#;
    
    // Wait a bit to ensure different modification time
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    fs::write(&script_path, modified_script).expect("Failed to write modified script");
    
    // Reload with state preservation
    engine.check_and_reload_scripts().expect("Failed to reload");
    
    // Verify state was preserved
    let preserved_counter: i32 = engine.lua().globals().get("get_state")
        .and_then(|f: mlua::Function| f.call::<mlua::Table>(()))
        .and_then(|t| t.get("counter"))
        .expect("Failed to get preserved counter");
    assert_eq!(preserved_counter, 3);
    
    // Verify new functionality works
    let version: String = engine.lua().globals().get("get_version")
        .and_then(|f: mlua::Function| f.call(()))
        .expect("Failed to get version");
    assert_eq!(version, "v2");
    
    // Test new function
    let double_fn: mlua::Function = engine.lua().globals().get("double_counter").expect("Failed to get double function");
    let doubled: i32 = double_fn.call(()).expect("Failed to call double");
    assert_eq!(doubled, 6);
}