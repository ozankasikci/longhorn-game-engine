//! Simplified event system integration for Lua scripting

use crate::ScriptError;
use mlua::{Lua, Function as LuaFunction, Value as LuaValue, Table};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

/// Event types that can be handled by the scripting system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Custom events with string identifier
    Custom(String),
}

/// Event data passed to scripts
#[derive(Debug, Clone)]
pub struct ScriptEvent {
    pub event_type: EventType,
    pub data: String, // Simplified to string for now
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

/// Event listener storage
#[derive(Clone)]
struct EventListener {
    id: ListenerId,
    callback: String, // Store as function name for now
}

/// Shared event system state
#[derive(Clone)]
struct EventSystemState {
    event_listeners: HashMap<EventType, Vec<EventListener>>,
    event_queue: VecDeque<ScriptEvent>,
}

/// Lua event system for handling events in scripts
pub struct LuaEventSystem {
    state: Arc<Mutex<EventSystemState>>,
}

impl LuaEventSystem {
    /// Create a new event system
    pub fn new(_lua: &Lua) -> Result<Self, ScriptError> {
        Ok(Self {
            state: Arc::new(Mutex::new(EventSystemState {
                event_listeners: HashMap::new(),
                event_queue: VecDeque::new(),
            })),
        })
    }
    
    /// Check if the event system has no listeners
    pub fn is_empty(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.event_listeners.is_empty()
    }
    
    /// Get the number of pending events
    pub fn pending_event_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.event_queue.len()
    }
    
    /// Check if there are listeners for a specific event type
    pub fn has_listeners(&self, event_type: &EventType) -> bool {
        let state = self.state.lock().unwrap();
        state.event_listeners.get(event_type)
            .map(|listeners| !listeners.is_empty())
            .unwrap_or(false)
    }
    
    /// Register an event listener (simplified)
    pub fn register_event_listener_simple(
        &mut self, 
        event_type: EventType, 
        callback_name: String
    ) -> Result<ListenerId, ScriptError> {
        let id = ListenerId::new();
        let listener = EventListener {
            id,
            callback: callback_name,
        };
        
        let mut state = self.state.lock().unwrap();
        state.event_listeners.entry(event_type).or_default().push(listener);
        
        Ok(id)
    }
    
    /// Emit an event to be processed
    pub fn emit_event(&mut self, event: ScriptEvent) {
        let mut state = self.state.lock().unwrap();
        state.event_queue.push_back(event);
    }
    
    /// Clear all listeners and pending events
    pub fn clear(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.event_listeners.clear();
        state.event_queue.clear();
    }
    
    /// Process events (simplified - returns event data for testing)
    pub fn process_events_simple(&mut self) -> Vec<(EventType, String)> {
        let mut state = self.state.lock().unwrap();
        let mut results = Vec::new();
        
        while let Some(event) = state.event_queue.pop_front() {
            if let Some(listeners) = state.event_listeners.get(&event.event_type) {
                for _listener in listeners {
                    results.push((event.event_type.clone(), event.data.clone()));
                }
            }
        }
        
        results
    }
}

// Tests will be simpler
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
    fn test_register_and_emit_event() {
        let lua = Lua::new();
        let mut event_system = LuaEventSystem::new(&lua).unwrap();
        
        // Register listener
        let listener_id = event_system.register_event_listener_simple(
            EventType::Custom("test_event".to_string()), 
            "callback_function".to_string()
        ).unwrap();
        
        assert!(event_system.has_listeners(&EventType::Custom("test_event".to_string())));
        
        // Emit event
        let event = ScriptEvent {
            event_type: EventType::Custom("test_event".to_string()),
            data: "test_data".to_string(),
            source: None,
        };
        event_system.emit_event(event);
        
        assert_eq!(event_system.pending_event_count(), 1);
        
        // Process events
        let results = event_system.process_events_simple();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "test_data");
    }
}