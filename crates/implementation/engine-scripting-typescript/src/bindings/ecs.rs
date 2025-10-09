//! ECS (Entity Component System) bindings for TypeScript
//! 
//! This module provides JavaScript/TypeScript bindings for the engine's ECS system,
//! allowing scripts to create entities, manage components, and query the world.

use crate::engine::TypeScriptEngine;
use engine_scripting::{ScriptError, ScriptResult};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::cell::RefCell;

/// Global entity ID counter for creating unique entity IDs
static ENTITY_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Entity data storage for queries
type EntityData = HashMap<String, v8::Global<v8::Value>>;

/// Global entity registry for tracking all created entities and their components
thread_local! {
    static ENTITY_REGISTRY: RefCell<HashMap<u32, EntityData>> = RefCell::new(HashMap::new());
}

/// Register ECS API bindings with the V8 context
pub fn register_ecs_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
    // Create engine object
    let engine_key = v8::String::new(scope, "engine")
        .ok_or_else(|| ScriptError::runtime("Failed to create engine string".to_string()))?;
    let engine_obj = v8::Object::new(scope);
    
    // Create world object
    let world_key = v8::String::new(scope, "world")
        .ok_or_else(|| ScriptError::runtime("Failed to create world string".to_string()))?;
    let world_obj = v8::Object::new(scope);
    
    // Register createEntity function
    let create_entity_key = v8::String::new(scope, "createEntity")
        .ok_or_else(|| ScriptError::runtime("Failed to create createEntity string".to_string()))?;
    
    let create_entity_fn = v8::Function::new(scope, create_entity_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create createEntity function".to_string()))?;
    
    world_obj.set(scope, create_entity_key.into(), create_entity_fn.into());
    
    // Register query function
    let query_key = v8::String::new(scope, "query")
        .ok_or_else(|| ScriptError::runtime("Failed to create query string".to_string()))?;
    
    let query_fn = v8::Function::new(scope, query_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create query function".to_string()))?;
    
    world_obj.set(scope, query_key.into(), query_fn.into());
    
    // Set world on engine
    engine_obj.set(scope, world_key.into(), world_obj.into());
    
    // Set engine on global
    global.set(scope, engine_key.into(), engine_obj.into());
    
    Ok(())
}

/// JavaScript callback for engine.world.createEntity()
fn create_entity_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Generate a unique entity ID
    let entity_id = ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Create entity object
    let entity_obj = v8::Object::new(scope);
    
    // Set entity ID
    let id_key = v8::String::new(scope, "id").unwrap();
    let id_value = v8::Number::new(scope, entity_id as f64);
    entity_obj.set(scope, id_key.into(), id_value.into());
    
    // Add component storage (simple in-memory map for now)
    let components_key = v8::String::new(scope, "__components").unwrap();
    let components_obj = v8::Object::new(scope);
    entity_obj.set(scope, components_key.into(), components_obj.into());
    
    // Register entity methods
    register_entity_methods(scope, entity_obj);
    
    rv.set(entity_obj.into());
}

/// Register methods on an entity object
fn register_entity_methods(scope: &mut v8::HandleScope, entity_obj: v8::Local<v8::Object>) {
    // addComponent method
    let add_component_key = v8::String::new(scope, "addComponent").unwrap();
    let add_component_fn = v8::Function::new(scope, add_component_callback).unwrap();
    entity_obj.set(scope, add_component_key.into(), add_component_fn.into());
    
    // getComponent method
    let get_component_key = v8::String::new(scope, "getComponent").unwrap();
    let get_component_fn = v8::Function::new(scope, get_component_callback).unwrap();
    entity_obj.set(scope, get_component_key.into(), get_component_fn.into());
    
    // hasComponent method
    let has_component_key = v8::String::new(scope, "hasComponent").unwrap();
    let has_component_fn = v8::Function::new(scope, has_component_callback).unwrap();
    entity_obj.set(scope, has_component_key.into(), has_component_fn.into());
    
    // removeComponent method
    let remove_component_key = v8::String::new(scope, "removeComponent").unwrap();
    let remove_component_fn = v8::Function::new(scope, remove_component_callback).unwrap();
    entity_obj.set(scope, remove_component_key.into(), remove_component_fn.into());
}

/// JavaScript callback for entity.addComponent(type, data)
fn add_component_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }
    
    let this = args.this();
    let component_type = args.get(0);
    let component_data = args.get(1);
    
    // Get entity ID
    let id_key = v8::String::new(scope, "id").unwrap();
    let entity_id = if let Some(id_value) = this.get(scope, id_key.into()) {
        if let Some(id_number) = id_value.number_value(scope) {
            id_number as u32
        } else {
            return;
        }
    } else {
        return;
    };
    
    // Get component type as string
    let component_type_str = if let Some(type_str) = component_type.to_string(scope) {
        type_str.to_rust_string_lossy(scope)
    } else {
        return;
    };
    
    // Get the components storage object
    let components_key = v8::String::new(scope, "__components").unwrap();
    if let Some(components_obj) = this.get(scope, components_key.into())
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok()) {
        
        // Store the component data locally
        components_obj.set(scope, component_type, component_data);
        
        // Also store in global registry for queries
        let global_data = v8::Global::new(scope, component_data);
        ENTITY_REGISTRY.with(|registry| {
            let mut reg = registry.borrow_mut();
            let entity_data = reg.entry(entity_id).or_insert_with(HashMap::new);
            entity_data.insert(component_type_str, global_data);
        });
    }
}

/// JavaScript callback for entity.getComponent(type)
fn get_component_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        rv.set(v8::undefined(scope).into());
        return;
    }
    
    let this = args.this();
    let component_type = args.get(0);
    
    // Get the components storage object
    let components_key = v8::String::new(scope, "__components").unwrap();
    if let Some(components_obj) = this.get(scope, components_key.into())
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok()) {
        
        // Get the component data
        if let Some(component_data) = components_obj.get(scope, component_type) {
            rv.set(component_data);
            return;
        }
    }
    
    rv.set(v8::undefined(scope).into());
}

/// JavaScript callback for entity.hasComponent(type)
fn has_component_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    let this = args.this();
    let component_type = args.get(0);
    
    // Get the components storage object
    let components_key = v8::String::new(scope, "__components").unwrap();
    if let Some(components_obj) = this.get(scope, components_key.into())
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok()) {
        
        // Check if component exists
        let has_component = components_obj.has(scope, component_type).unwrap_or(false);
        rv.set(v8::Boolean::new(scope, has_component).into());
        return;
    }
    
    rv.set(v8::Boolean::new(scope, false).into());
}

/// JavaScript callback for entity.removeComponent(type)
fn remove_component_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let this = args.this();
    let component_type = args.get(0);
    
    // Get the components storage object
    let components_key = v8::String::new(scope, "__components").unwrap();
    if let Some(components_obj) = this.get(scope, components_key.into())
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok()) {
        
        // Remove the component
        components_obj.delete(scope, component_type);
    }
}

/// JavaScript callback for engine.world.query(componentType)
fn query_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let array = v8::Array::new(scope, 0);
        rv.set(array.into());
        return;
    }
    
    // Get component type string
    let component_type_str = if let Some(type_str) = args.get(0).to_string(scope) {
        type_str.to_rust_string_lossy(scope)
    } else {
        let array = v8::Array::new(scope, 0);
        rv.set(array.into());
        return;
    };
    
    // Query entities with the specified component type
    let matching_entities = ENTITY_REGISTRY.with(|registry| {
        let reg = registry.borrow();
        let mut matches = Vec::new();
        
        for (&entity_id, entity_data) in reg.iter() {
            if entity_data.contains_key(&component_type_str) {
                matches.push(entity_id);
            }
        }
        
        matches
    });
    
    // Create array of entity objects
    let result_array = v8::Array::new(scope, matching_entities.len() as i32);
    
    for (index, &entity_id) in matching_entities.iter().enumerate() {
        // Create entity object with same structure as createEntity
        let entity_obj = v8::Object::new(scope);
        
        // Set entity ID
        let id_key = v8::String::new(scope, "id").unwrap();
        let id_value = v8::Number::new(scope, entity_id as f64);
        entity_obj.set(scope, id_key.into(), id_value.into());
        
        // Create components storage object
        let components_key = v8::String::new(scope, "__components").unwrap();
        let components_obj = v8::Object::new(scope);
        
        // Populate with stored component data
        ENTITY_REGISTRY.with(|registry| {
            let reg = registry.borrow();
            if let Some(entity_data) = reg.get(&entity_id) {
                for (comp_type, comp_data) in entity_data.iter() {
                    let comp_key = v8::String::new(scope, comp_type).unwrap();
                    let local_data = v8::Local::new(scope, comp_data);
                    components_obj.set(scope, comp_key.into(), local_data);
                }
            }
        });
        
        entity_obj.set(scope, components_key.into(), components_obj.into());
        
        // Register entity methods
        register_entity_methods(scope, entity_obj);
        
        // Add to result array
        let index_value = v8::Number::new(scope, index as f64);
        result_array.set_index(scope, index as u32, entity_obj.into());
    }
    
    rv.set(result_array.into());
}