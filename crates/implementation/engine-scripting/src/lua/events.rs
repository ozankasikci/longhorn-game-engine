//! Event system integration for Lua

use mlua::{Lua, Table, Value, Function};
use crate::ScriptResult;
use std::collections::HashMap;

/// Event dispatcher for Lua scripts
pub struct LuaEventDispatcher {
    /// Event listeners by event type
    listeners: HashMap<String, Vec<Function>>,
}

impl LuaEventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Subscribe to an event
    pub fn subscribe(&mut self, event_type: String, callback: Function) {
        self.listeners
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Dispatch an event to all listeners
    pub fn dispatch(&self, event_type: &str, event_data: Value) -> ScriptResult<()> {
        if let Some(listeners) = self.listeners.get(event_type) {
            for listener in listeners {
                listener.call::<()>(event_data.clone())
                    .map_err(|e| crate::ScriptError::RuntimeError(
                        format!("Event handler error: {}", e)
                    ))?;
            }
        }
        Ok(())
    }
}

/// Set up event system bindings
pub fn setup_event_bindings(lua: &Lua, engine: &Table) -> ScriptResult<()> {
    let events = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create events table: {}", e)))?;

    // Subscribe function
    let subscribe_fn = lua.create_function(|_, (_event_type, _callback): (String, Function)| {
        // TODO: Connect to actual event dispatcher
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create subscribe function: {}", e)))?;

    events.set("subscribe", subscribe_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set subscribe function: {}", e)))?;

    // Emit function
    let emit_fn = lua.create_function(|_, (_event_type, _data): (String, Value)| {
        // TODO: Connect to actual event dispatcher
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create emit function: {}", e)))?;

    events.set("emit", emit_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set emit function: {}", e)))?;

    engine.set("events", events)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set events table: {}", e)))?;

    Ok(())
}