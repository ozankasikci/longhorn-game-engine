//! Event system bindings for TypeScript
//! 
//! This module provides JavaScript/TypeScript bindings for the engine's event system,
//! allowing scripts to register event listeners, emit events, and manage event handlers.

use engine_scripting::{ScriptError, ScriptResult};
use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

/// Global listener ID counter for creating unique event listener handles
static LISTENER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Global event listener storage
static EVENT_LISTENERS: Mutex<Option<HashMap<u32, EventListener>>> = Mutex::new(None);

/// Represents an event listener with its callback function
struct EventListener {
    event_type: String,
    callback_name: String, // For simplicity, store the function name
}

/// Initialize the event listener storage
fn ensure_listeners_initialized() {
    let mut listeners = EVENT_LISTENERS.lock().unwrap();
    if listeners.is_none() {
        *listeners = Some(HashMap::new());
    }
}

/// Register Event API bindings with the V8 context
pub fn register_events_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
    ensure_listeners_initialized();
    
    // Get or create engine object
    let engine_key = v8::String::new(scope, "engine")
        .ok_or_else(|| ScriptError::runtime("Failed to create engine string".to_string()))?;
    
    let engine_obj = if let Some(existing_engine) = global.get(scope, engine_key.into())
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok()) {
        existing_engine
    } else {
        let new_engine = v8::Object::new(scope);
        global.set(scope, engine_key.into(), new_engine.into());
        new_engine
    };
    
    // Create events object
    let events_key = v8::String::new(scope, "events")
        .ok_or_else(|| ScriptError::runtime("Failed to create events string".to_string()))?;
    let events_obj = v8::Object::new(scope);
    
    // Register event listener functions
    register_listener_functions(scope, events_obj)?;
    
    // Register event emission functions
    register_emission_functions(scope, events_obj)?;
    
    // Set events on engine
    engine_obj.set(scope, events_key.into(), events_obj.into());
    
    Ok(())
}

/// Register event listener management functions
fn register_listener_functions(scope: &mut v8::HandleScope, events_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // onEvent function
    let on_event_key = v8::String::new(scope, "onEvent")
        .ok_or_else(|| ScriptError::runtime("Failed to create onEvent string".to_string()))?;
    
    let on_event_fn = v8::Function::new(scope, on_event_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create onEvent function".to_string()))?;
    
    events_obj.set(scope, on_event_key.into(), on_event_fn.into());
    
    // removeListener function
    let remove_listener_key = v8::String::new(scope, "removeListener")
        .ok_or_else(|| ScriptError::runtime("Failed to create removeListener string".to_string()))?;
    
    let remove_listener_fn = v8::Function::new(scope, remove_listener_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create removeListener function".to_string()))?;
    
    events_obj.set(scope, remove_listener_key.into(), remove_listener_fn.into());
    
    Ok(())
}

/// Register event emission functions
fn register_emission_functions(scope: &mut v8::HandleScope, events_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // emitEvent function
    let emit_event_key = v8::String::new(scope, "emitEvent")
        .ok_or_else(|| ScriptError::runtime("Failed to create emitEvent string".to_string()))?;
    
    let emit_event_fn = v8::Function::new(scope, emit_event_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create emitEvent function".to_string()))?;
    
    events_obj.set(scope, emit_event_key.into(), emit_event_fn.into());
    
    Ok(())
}

/// JavaScript callback for engine.events.onEvent(eventType, callback)
fn on_event_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    let event_type = args.get(0);
    let callback = args.get(1);
    
    // Validate event type
    let event_type_str = if let Some(type_str) = event_type.to_string(scope) {
        let type_string = type_str.to_rust_string_lossy(scope);
        if type_string.is_empty() {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
        type_string
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    };
    
    // Validate callback function
    if callback.is_null_or_undefined() || !callback.is_function() {
        let error_msg = v8::String::new(scope, "Callback must be a function").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }
    
    // Generate a listener ID
    let listener_id = LISTENER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Store the listener (simplified - just store event type for now)
    {
        let mut listeners = EVENT_LISTENERS.lock().unwrap();
        if let Some(ref mut listener_map) = *listeners {
            listener_map.insert(listener_id, EventListener {
                event_type: event_type_str,
                callback_name: "callback".to_string(), // Simplified
            });
        }
    }
    
    rv.set(v8::Number::new(scope, listener_id as f64).into());
}

/// JavaScript callback for engine.events.removeListener(listenerId)
fn remove_listener_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let listener_id = args.get(0);
    
    // Validate listener ID
    if let Some(id_val) = listener_id.number_value(scope) {
        let id = id_val as u32;
        
        // Remove the listener
        let mut listeners = EVENT_LISTENERS.lock().unwrap();
        if let Some(ref mut listener_map) = *listeners {
            listener_map.remove(&id);
        }
    }
}

/// JavaScript callback for engine.events.emitEvent(eventType, data)
fn emit_event_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }
    
    let event_type = args.get(0);
    let _data = args.get(1);
    
    // Check for null or undefined event type first
    if event_type.is_null_or_undefined() {
        let error_msg = v8::String::new(scope, "Event type cannot be null").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }
    
    // Validate event type string
    if let Some(type_str) = event_type.to_string(scope) {
        let type_string = type_str.to_rust_string_lossy(scope);
        if type_string.is_empty() {
            // Throw an error for empty event type
            let error_msg = v8::String::new(scope, "Event type cannot be empty").unwrap();
            let error = v8::Exception::error(scope, error_msg);
            scope.throw_exception(error);
            return;
        }
    } else {
        // If can't convert to string, it's invalid
        let error_msg = v8::String::new(scope, "Event type must be a string").unwrap();
        let error = v8::Exception::error(scope, error_msg);
        scope.throw_exception(error);
        return;
    }
    
    // For now, just validate the event emission (no actual event dispatch)
    // In a real implementation, this would trigger the actual event listeners
}