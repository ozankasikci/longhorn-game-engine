//! Asset system integration for Lua

use mlua::{Lua, Table};
use crate::ScriptResult;

/// Set up asset system bindings
pub fn setup_asset_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let assets = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create assets table: {}", e)))?;

    // Load texture function
    let load_texture = lua.create_function(|_, path: String| {
        // TODO: Connect to actual asset system
        log::debug!("Loading texture: {}", path);
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create load_texture: {}", e)))?;

    assets.set("load_texture", load_texture)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set load_texture: {}", e)))?;

    // Load sound function
    let load_sound = lua.create_function(|_, path: String| {
        // TODO: Connect to actual asset system
        log::debug!("Loading sound: {}", path);
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create load_sound: {}", e)))?;

    assets.set("load_sound", load_sound)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set load_sound: {}", e)))?;

    // Load model function
    let load_model = lua.create_function(|_, path: String| {
        // TODO: Connect to actual asset system
        log::debug!("Loading model: {}", path);
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create load_model: {}", e)))?;

    assets.set("load_model", load_model)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set load_model: {}", e)))?;

    engine.set("assets", assets)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set assets table: {}", e)))?;

    Ok(())
}