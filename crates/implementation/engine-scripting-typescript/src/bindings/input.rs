//! Input system bindings for TypeScript
//! 
//! This module provides JavaScript/TypeScript bindings for the engine's input system,
//! allowing scripts to query key/mouse states, bind callbacks, and handle input events.

use engine_scripting::{ScriptError, ScriptResult};
use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;

/// Global binding ID counter for creating unique binding IDs
static BINDING_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Register Input API bindings with the V8 context
pub fn register_input_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
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
    
    // Create input object
    let input_key = v8::String::new(scope, "input")
        .ok_or_else(|| ScriptError::runtime("Failed to create input string".to_string()))?;
    let input_obj = v8::Object::new(scope);
    
    // Register key state functions
    register_key_state_functions(scope, input_obj)?;
    
    // Register mouse functions
    register_mouse_functions(scope, input_obj)?;
    
    // Register key binding functions
    register_binding_functions(scope, input_obj)?;
    
    // Register event listener functions
    register_event_functions(scope, input_obj)?;
    
    // Register key codes
    register_key_codes(scope, input_obj)?;
    
    // Set input on engine
    engine_obj.set(scope, input_key.into(), input_obj.into());
    
    Ok(())
}

/// Register key state query functions
fn register_key_state_functions(scope: &mut v8::HandleScope, input_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // isKeyPressed function
    let is_key_pressed_key = v8::String::new(scope, "isKeyPressed")
        .ok_or_else(|| ScriptError::runtime("Failed to create isKeyPressed string".to_string()))?;
    
    let is_key_pressed_fn = v8::Function::new(scope, is_key_pressed_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create isKeyPressed function".to_string()))?;
    
    input_obj.set(scope, is_key_pressed_key.into(), is_key_pressed_fn.into());
    
    // isMouseButtonPressed function
    let is_mouse_pressed_key = v8::String::new(scope, "isMouseButtonPressed")
        .ok_or_else(|| ScriptError::runtime("Failed to create isMouseButtonPressed string".to_string()))?;
    
    let is_mouse_pressed_fn = v8::Function::new(scope, is_mouse_button_pressed_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create isMouseButtonPressed function".to_string()))?;
    
    input_obj.set(scope, is_mouse_pressed_key.into(), is_mouse_pressed_fn.into());
    
    Ok(())
}

/// Register mouse-related functions
fn register_mouse_functions(scope: &mut v8::HandleScope, input_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // getMousePosition function
    let get_mouse_pos_key = v8::String::new(scope, "getMousePosition")
        .ok_or_else(|| ScriptError::runtime("Failed to create getMousePosition string".to_string()))?;
    
    let get_mouse_pos_fn = v8::Function::new(scope, get_mouse_position_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create getMousePosition function".to_string()))?;
    
    input_obj.set(scope, get_mouse_pos_key.into(), get_mouse_pos_fn.into());
    
    Ok(())
}

/// Register key binding functions
fn register_binding_functions(scope: &mut v8::HandleScope, input_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // bindKey function
    let bind_key_key = v8::String::new(scope, "bindKey")
        .ok_or_else(|| ScriptError::runtime("Failed to create bindKey string".to_string()))?;
    
    let bind_key_fn = v8::Function::new(scope, bind_key_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create bindKey function".to_string()))?;
    
    input_obj.set(scope, bind_key_key.into(), bind_key_fn.into());
    
    // unbindKey function
    let unbind_key_key = v8::String::new(scope, "unbindKey")
        .ok_or_else(|| ScriptError::runtime("Failed to create unbindKey string".to_string()))?;
    
    let unbind_key_fn = v8::Function::new(scope, unbind_key_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create unbindKey function".to_string()))?;
    
    input_obj.set(scope, unbind_key_key.into(), unbind_key_fn.into());
    
    Ok(())
}

/// Register event listener functions
fn register_event_functions(scope: &mut v8::HandleScope, input_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // onKeyDown function
    let on_key_down_key = v8::String::new(scope, "onKeyDown")
        .ok_or_else(|| ScriptError::runtime("Failed to create onKeyDown string".to_string()))?;
    
    let on_key_down_fn = v8::Function::new(scope, on_key_down_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create onKeyDown function".to_string()))?;
    
    input_obj.set(scope, on_key_down_key.into(), on_key_down_fn.into());
    
    // onKeyUp function
    let on_key_up_key = v8::String::new(scope, "onKeyUp")
        .ok_or_else(|| ScriptError::runtime("Failed to create onKeyUp string".to_string()))?;
    
    let on_key_up_fn = v8::Function::new(scope, on_key_up_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create onKeyUp function".to_string()))?;
    
    input_obj.set(scope, on_key_up_key.into(), on_key_up_fn.into());
    
    // onMouseDown function
    let on_mouse_down_key = v8::String::new(scope, "onMouseDown")
        .ok_or_else(|| ScriptError::runtime("Failed to create onMouseDown string".to_string()))?;
    
    let on_mouse_down_fn = v8::Function::new(scope, on_mouse_down_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create onMouseDown function".to_string()))?;
    
    input_obj.set(scope, on_mouse_down_key.into(), on_mouse_down_fn.into());
    
    // onMouseUp function
    let on_mouse_up_key = v8::String::new(scope, "onMouseUp")
        .ok_or_else(|| ScriptError::runtime("Failed to create onMouseUp string".to_string()))?;
    
    let on_mouse_up_fn = v8::Function::new(scope, on_mouse_up_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create onMouseUp function".to_string()))?;
    
    input_obj.set(scope, on_mouse_up_key.into(), on_mouse_up_fn.into());
    
    Ok(())
}

/// Register key code constants
fn register_key_codes(scope: &mut v8::HandleScope, input_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    let keys_key = v8::String::new(scope, "keys")
        .ok_or_else(|| ScriptError::runtime("Failed to create keys string".to_string()))?;
    let keys_obj = v8::Object::new(scope);
    
    // Add common key codes (simplified for now)
    let key_codes = [
        ("A", 65), ("B", 66), ("C", 67), ("D", 68), ("E", 69), ("F", 70), ("G", 71), ("H", 72),
        ("I", 73), ("J", 74), ("K", 75), ("L", 76), ("M", 77), ("N", 78), ("O", 79), ("P", 80),
        ("Q", 81), ("R", 82), ("S", 83), ("T", 84), ("U", 85), ("V", 86), ("W", 87), ("X", 88),
        ("Y", 89), ("Z", 90),
        ("0", 48), ("1", 49), ("2", 50), ("3", 51), ("4", 52), ("5", 53), ("6", 54), ("7", 55),
        ("8", 56), ("9", 57),
        ("Space", 32), ("Enter", 13), ("Tab", 9), ("Escape", 27),
        ("ArrowUp", 38), ("ArrowDown", 40), ("ArrowLeft", 37), ("ArrowRight", 39),
    ];
    
    for (key_name, key_code) in &key_codes {
        let name_key = v8::String::new(scope, key_name).unwrap();
        let code_value = v8::Number::new(scope, *key_code as f64);
        keys_obj.set(scope, name_key.into(), code_value.into());
    }
    
    input_obj.set(scope, keys_key.into(), keys_obj.into());
    
    Ok(())
}

/// JavaScript callback for engine.input.isKeyPressed(key)
fn is_key_pressed_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    let _key = args.get(0);
    // For now, always return false (no actual input system integration)
    // In a real implementation, this would query the actual input system
    rv.set(v8::Boolean::new(scope, false).into());
}

/// JavaScript callback for engine.input.isMouseButtonPressed(button)
fn is_mouse_button_pressed_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    let _button = args.get(0);
    // For now, always return false (no actual input system integration)
    rv.set(v8::Boolean::new(scope, false).into());
}

/// JavaScript callback for engine.input.getMousePosition()
fn get_mouse_position_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Create mouse position object
    let pos_obj = v8::Object::new(scope);
    
    let x_key = v8::String::new(scope, "x").unwrap();
    let y_key = v8::String::new(scope, "y").unwrap();
    
    // For now, return (0, 0) - in a real implementation this would query the actual mouse position
    let x_value = v8::Number::new(scope, 0.0);
    let y_value = v8::Number::new(scope, 0.0);
    
    pos_obj.set(scope, x_key.into(), x_value.into());
    pos_obj.set(scope, y_key.into(), y_value.into());
    
    rv.set(pos_obj.into());
}

/// JavaScript callback for engine.input.bindKey(key, callback)
fn bind_key_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    let key = args.get(0);
    let _callback = args.get(1);
    
    // Check if key is empty or invalid
    if let Some(key_str) = key.to_string(scope) {
        let key_string = key_str.to_rust_string_lossy(scope);
        if key_string.is_empty() {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Generate a binding ID
    let binding_id = BINDING_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // For now, just return the binding ID (no actual binding implementation)
    // In a real implementation, this would store the binding and set up input handling
    rv.set(v8::Number::new(scope, binding_id as f64).into());
}

/// JavaScript callback for engine.input.unbindKey(bindingId)
fn unbind_key_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _binding_id = args.get(0);
    
    // For now, do nothing (no actual unbinding implementation)
    // In a real implementation, this would remove the stored binding
}

/// JavaScript callback for engine.input.onKeyDown(callback)
fn on_key_down_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _callback = args.get(0);
    
    // For now, do nothing (no actual event listener implementation)
    // In a real implementation, this would register the callback for key down events
}

/// JavaScript callback for engine.input.onKeyUp(callback)
fn on_key_up_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _callback = args.get(0);
    
    // For now, do nothing (no actual event listener implementation)
}

/// JavaScript callback for engine.input.onMouseDown(callback)
fn on_mouse_down_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _callback = args.get(0);
    
    // For now, do nothing (no actual event listener implementation)
}

/// JavaScript callback for engine.input.onMouseUp(callback)
fn on_mouse_up_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _callback = args.get(0);
    
    // For now, do nothing (no actual event listener implementation)
}