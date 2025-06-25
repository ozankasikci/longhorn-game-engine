//! Core engine bindings for Lua

use mlua::{Lua, Table};
use crate::ScriptResult;

/// Math bindings for vectors, matrices, etc.
pub fn setup_math_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let math = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create math table: {}", e)))?;

    // Vector3 constructor
    let vec3_fn = lua.create_function(|lua, (x, y, z): (f32, f32, f32)| {
        let vec = lua.create_table()?;
        vec.set("x", x)?;
        vec.set("y", y)?;
        vec.set("z", z)?;
        
        // Add vector methods
        vec.set("add", lua.create_function(|_, (this, other): (Table, Table)| {
            let x1 = this.get::<f32>("x")?;
            let y1 = this.get::<f32>("y")?;
            let z1 = this.get::<f32>("z")?;
            let x2 = other.get::<f32>("x")?;
            let y2 = other.get::<f32>("y")?;
            let z2 = other.get::<f32>("z")?;
            
            let result = this.clone();
            result.set("x", x1 + x2)?;
            result.set("y", y1 + y2)?;
            result.set("z", z1 + z2)?;
            Ok(result)
        })?)?;
        
        Ok(vec)
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create vec3 function: {}", e)))?;

    math.set("vec3", vec3_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set vec3 function: {}", e)))?;

    engine.set("math", math)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set math table: {}", e)))?;

    Ok(())
}

/// Time and frame bindings
pub fn setup_time_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let time = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create time table: {}", e)))?;

    // We'll update these values each frame
    time.set("delta_time", 0.0f32)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set delta_time: {}", e)))?;
    
    time.set("total_time", 0.0f32)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set total_time: {}", e)))?;

    engine.set("time", time)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set time table: {}", e)))?;

    Ok(())
}

/// Input system bindings
pub fn setup_input_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let input = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create input table: {}", e)))?;

    // Key codes table
    let keys = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create keys table: {}", e)))?;
    
    // Common key codes
    keys.set("W", 87).ok();
    keys.set("A", 65).ok();
    keys.set("S", 83).ok();
    keys.set("D", 68).ok();
    keys.set("SPACE", 32).ok();
    keys.set("ENTER", 13).ok();
    keys.set("ESCAPE", 27).ok();

    input.set("keys", keys)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set keys table: {}", e)))?;

    // Input query functions (will be implemented with actual input system)
    let is_key_pressed = lua.create_function(|_, _key: i32| {
        // TODO: Connect to actual input system
        Ok(false)
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create is_key_pressed: {}", e)))?;

    input.set("is_key_pressed", is_key_pressed)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set is_key_pressed: {}", e)))?;

    engine.set("input", input)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set input table: {}", e)))?;

    Ok(())
}

/// Debug and logging bindings
pub fn setup_debug_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let debug = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create debug table: {}", e)))?;

    // Log levels
    let log_fn = lua.create_function(|_, (level, message): (String, String)| {
        match level.as_str() {
            "error" => log::error!("[Lua] {}", message),
            "warn" => log::warn!("[Lua] {}", message),
            "info" => log::info!("[Lua] {}", message),
            "debug" => log::debug!("[Lua] {}", message),
            _ => log::info!("[Lua] {}", message),
        }
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create log function: {}", e)))?;

    debug.set("log", log_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set log function: {}", e)))?;

    engine.set("debug", debug)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set debug table: {}", e)))?;

    Ok(())
}