//! Event system integration for Lua scripting

use crate::ScriptError;
use mlua::{Lua, Function as LuaFunction, Value as LuaValue, Table};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

/// Event types that can be handled by the scripting system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// System events
    System(SystemEventType),
    /// Game events  
    Game(GameEventType),
    /// Custom events with string identifier
    Custom(String),
}

/// System-level events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SystemEventType {
    Start,
    Stop,
    Pause,
    Resume,
    Update,
    FixedUpdate,
}

/// Game-specific events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameEventType {
    EntitySpawned,
    EntityDestroyed,
    ComponentAdded,
    ComponentRemoved,
    CollisionStart,
    CollisionEnd,
}

/// Event data passed to scripts
#[derive(Debug, Clone)]
pub struct ScriptEvent {
    pub event_type: EventType,
    pub data: LuaValue,
    pub source: Option<String>,
}

/// Unique identifier for event listeners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

static NEXT_LISTENER_ID: AtomicU64 = AtomicU64::new(1);

impl ListenerId {
    fn new() -> Self {
        ListenerId(NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Lua event system for handling events in scripts (simplified for now)
pub struct LuaEventSystem {
    event_queue: VecDeque<ScriptEvent>,
}

impl LuaEventSystem {
    /// Create a new event system
    pub fn new(_lua: &Lua) -> Result<Self, ScriptError> {
        Ok(Self {
            event_queue: VecDeque::new(),
        })
    }
    
    /// Check if the event system has no listeners
    pub fn is_empty(&self) -> bool {
        true // Simplified implementation
    }
    
    /// Get the number of pending events
    pub fn pending_event_count(&self) -> usize {
        self.event_queue.len()
    }
    
    /// Check if there are listeners for a specific event type
    pub fn has_listeners(&self, _event_type: &EventType) -> bool {
        false // Simplified implementation
    }
    
    /// Register an event listener (simplified - returns dummy ID)
    pub fn register_event_listener(
        &mut self, 
        _event_type: EventType, 
        _callback: LuaFunction
    ) -> Result<ListenerId, ScriptError> {
        Ok(ListenerId::new())
    }
    
    /// Remove an event listener by ID
    pub fn remove_event_listener(&mut self, _listener_id: ListenerId) -> Result<(), ScriptError> {
        Ok(()) // Simplified implementation
    }
    
    /// Emit an event to be processed
    pub fn emit_event(&mut self, event: ScriptEvent) {
        self.event_queue.push_back(event);
    }
    
    /// Process all pending events
    pub fn process_events(&mut self, _lua: &Lua) -> Result<(), ScriptError> {
        self.event_queue.clear(); // Just clear for now
        Ok(())
    }
    
    /// Clear all listeners and pending events
    pub fn clear(&mut self) {
        self.event_queue.clear();
    }
    
    /// Register the event API in Lua globals
    pub fn register_api(&self, lua: &Lua) -> Result<(), ScriptError> {
        let globals = lua.globals();
        
        // on_event function
        let on_event = lua.create_function(|_, (_event_type, _callback): (String, LuaFunction)| {
            // TODO: Get event system instance and register listener
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create on_event function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("on_event", on_event).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set on_event function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // emit_event function
        let emit_event = lua.create_function(|_, (_event_type, _data): (String, LuaValue)| {
            // TODO: Get event system instance and emit event
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create emit_event function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("emit_event", emit_event).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set emit_event function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // remove_listener function
        let remove_listener = lua.create_function(|_, _listener_id: u64| {
            // TODO: Get event system instance and remove listener
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create remove_listener function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("remove_listener", remove_listener).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set remove_listener function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        Ok(())
    }
}

// Backward compatibility - keep the old API
pub struct LuaEventDispatcher {
    event_queue: VecDeque<(String, LuaValue)>,
}

impl LuaEventDispatcher {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
        }
    }

    pub fn subscribe(&mut self, _event_type: String, _callback: LuaFunction) {
        // Simplified implementation
    }

    pub fn dispatch(&self, _event_type: &str, _event_data: LuaValue) -> crate::ScriptResult<()> {
        // Simplified implementation
        Ok(())
    }
}

/// Set up event system bindings (backward compatibility)
pub fn setup_event_bindings(lua: &Lua, engine: &Table) -> crate::ScriptResult<()> {
    let events = lua.create_table()
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create events table: {}", e)))?;

    // Subscribe function
    let subscribe_fn = lua.create_function(|_, (_event_type, _callback): (String, LuaFunction)| {
        // TODO: Connect to actual event dispatcher
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create subscribe function: {}", e)))?;

    events.set("subscribe", subscribe_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set subscribe function: {}", e)))?;

    // Emit function
    let emit_fn = lua.create_function(|_, (_event_type, _data): (String, LuaValue)| {
        // TODO: Connect to actual event dispatcher
        Ok(())
    }).map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to create emit function: {}", e)))?;

    events.set("emit", emit_fn)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set emit function: {}", e)))?;
    engine.set("events", events)
        .map_err(|e| crate::ScriptError::RuntimeError(format!("Failed to set events table: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_system_creation() {
        let lua = Lua::new();
        let event_system = LuaEventSystem::new(&lua).unwrap();
        
        assert!(event_system.is_empty());
        assert_eq!(event_system.pending_event_count(), 0);
    }
    
    #[test]
    fn test_emit_and_process_events() {
        let lua = Lua::new();
        let mut event_system = LuaEventSystem::new(&lua).unwrap();
        
        // Create a simple event
        let event = ScriptEvent {
            event_type: EventType::Custom("test_event".to_string()),
            data: LuaValue::String(lua.create_string("test_data").unwrap()),
            source: None,
        };
        
        event_system.emit_event(event);
        assert_eq!(event_system.pending_event_count(), 1);
        
        // Process events
        event_system.process_events(&lua).unwrap();
        assert_eq!(event_system.pending_event_count(), 0);
    }
    
    #[test]
    fn test_register_listener() {
        let lua = Lua::new();
        let mut event_system = LuaEventSystem::new(&lua).unwrap();
        
        // Create a dummy callback
        let callback: LuaFunction = lua.create_function(|_, _: LuaValue| {
            Ok(())
        }).unwrap();
        
        // Register listener
        let listener_id = event_system.register_event_listener(
            EventType::Custom("test_event".to_string()), 
            callback
        ).unwrap();
        
        // Remove listener
        event_system.remove_event_listener(listener_id).unwrap();
    }
    
    #[test]
    fn test_api_registration() {
        let lua = Lua::new();
        let event_system = LuaEventSystem::new(&lua).unwrap();
        
        // Register API
        event_system.register_api(&lua).unwrap();
        
        // Check that functions are available
        let globals = lua.globals();
        assert!(globals.get::<LuaFunction>("on_event").is_ok());
        assert!(globals.get::<LuaFunction>("emit_event").is_ok());
        assert!(globals.get::<LuaFunction>("remove_listener").is_ok());
    }
}