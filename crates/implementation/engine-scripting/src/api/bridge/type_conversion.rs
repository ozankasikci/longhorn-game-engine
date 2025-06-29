//! Type conversion between Rust Value types and V8 JavaScript values

use crate::api::registry::type_system::{Value, ObjectValue, TypeError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use engine_components_3d::Transform;

// Global context for V8 callbacks to access ECS data
pub struct EcsContext {
    pub world_ptr: Option<*mut engine_ecs_core::World>,
    pub current_entity_id: Option<u32>,
}

unsafe impl Send for EcsContext {}
unsafe impl Sync for EcsContext {}

pub static ECS_CONTEXT: Lazy<Arc<Mutex<EcsContext>>> = Lazy::new(|| {
    Arc::new(Mutex::new(EcsContext {
        world_ptr: None,
        current_entity_id: None,
    }))
});

pub fn set_ecs_context(world_ptr: *mut engine_ecs_core::World, entity_id: u32) {
    if let Ok(mut context) = ECS_CONTEXT.lock() {
        context.world_ptr = Some(world_ptr);
        context.current_entity_id = Some(entity_id);
    }
}

/// Convert a V8 value to our internal Value type
pub fn value_from_v8(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<Value, TypeError> {
    if value.is_number() {
        Ok(Value::Number(value.number_value(scope).unwrap_or(0.0)))
    } else if value.is_string() {
        let string_val = value.to_rust_string_lossy(scope);
        Ok(Value::String(string_val))
    } else if value.is_boolean() {
        Ok(Value::Boolean(value.boolean_value(scope)))
    } else if value.is_null() || value.is_undefined() {
        Ok(Value::Null)
    } else if value.is_array() {
        let array = v8::Local::<v8::Array>::try_from(value)
            .map_err(|_| TypeError::ConversionError)?;
        let length = array.length();
        let mut result = Vec::new();

        for i in 0..length {
            if let Some(element) = array.get_index(scope, i) {
                result.push(value_from_v8(scope, element)?);
            }
        }

        Ok(Value::Array(result))
    } else if value.is_object() {
        // Handle custom objects
        let obj = value.to_object(scope)
            .ok_or(TypeError::ConversionError)?;

        // Extract class name from constructor or prototype
        let class_name = extract_class_name(scope, obj)?;

        Ok(Value::Object(ObjectValue::new(class_name)))
    } else {
        Err(TypeError::UnsupportedType)
    }
}

/// Convert our internal Value type to a V8 value
pub fn value_to_v8<'a>(scope: &'a mut v8::HandleScope, value: &Value) -> v8::Local<'a, v8::Value> {
    match value {
        Value::Void => v8::undefined(scope).into(),
        Value::Number(n) => v8::Number::new(scope, *n).into(),
        Value::String(s) => v8::String::new(scope, s).unwrap().into(),
        Value::Boolean(b) => v8::Boolean::new(scope, *b).into(),
        Value::Null => v8::null(scope).into(),
        Value::Array(_arr) => {
            // TODO: Fix V8 borrow checker issues with array conversion
            v8::null(scope).into()
        }
        Value::Object(obj) => {
            // Check if this object needs special V8 method binding
            if obj.needs_v8_methods() {
                match obj.class_name() {
                    "Entity" => {
                        // Extract entity ID from properties
                        let entity_id = obj.get_property("id")
                            .and_then(|v| v.as_number())
                            .unwrap_or(0.0) as u32;
                        
                        // Create Entity object with V8 methods using the bridge
                        create_entity_object_with_methods(scope, entity_id)
                    }
                    _ => {
                        // For other classes that need methods, create regular object for now
                        create_object_with_properties(scope, obj)
                    }
                }
            } else {
                // Create regular V8 object with properties
                create_object_with_properties(scope, obj)
            }
        }
    }
}

/// Extract class name from a V8 object
fn extract_class_name(scope: &mut v8::HandleScope, obj: v8::Local<v8::Object>) -> Result<String, TypeError> {
    // Try to get constructor.name
    if let Some(constructor_key) = v8::String::new(scope, "constructor") {
        if let Some(constructor) = obj.get(scope, constructor_key.into()) {
            if let Ok(constructor_obj) = constructor.try_into() {
                let constructor_obj: v8::Local<v8::Object> = constructor_obj;
                if let Some(name_key) = v8::String::new(scope, "name") {
                    if let Some(name_value) = constructor_obj.get(scope, name_key.into()) {
                        if name_value.is_string() {
                            return Ok(name_value.to_rust_string_lossy(scope));
                        }
                    }
                }
            }
        }
    }

    // Fallback to "Object"
    Ok("Object".to_string())
}

/// Convert a V8 array to a Vec<Value>
pub fn array_from_v8(scope: &mut v8::HandleScope, array: v8::Local<v8::Array>) -> Result<Vec<Value>, TypeError> {
    let length = array.length();
    let mut result = Vec::with_capacity(length as usize);

    for i in 0..length {
        if let Some(element) = array.get_index(scope, i) {
            result.push(value_from_v8(scope, element)?);
        } else {
            result.push(Value::Null);
        }
    }

    Ok(result)
}

/// Convert a Vec<Value> to a V8 array
pub fn array_to_v8<'a>(scope: &'a mut v8::HandleScope, values: &[Value]) -> v8::Local<'a, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);

    // TODO: Fix V8 borrow checker issues with array conversion
    let _values = values; // Avoid unused warning

    array
}

/// Convert a V8 object to a HashMap<String, Value>
pub fn object_from_v8(scope: &mut v8::HandleScope, obj: v8::Local<v8::Object>) -> Result<HashMap<String, Value>, TypeError> {
    let mut result = HashMap::new();

    if let Some(prop_names) = obj.get_own_property_names(scope, v8::GetPropertyNamesArgs::default()) {
        let length = prop_names.length();

        for i in 0..length {
            if let Some(key) = prop_names.get_index(scope, i) {
                if key.is_string() {
                    let key_str = key.to_rust_string_lossy(scope);
                    if let Some(value) = obj.get(scope, key) {
                        result.insert(key_str, value_from_v8(scope, value)?);
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Convert a HashMap<String, Value> to a V8 object
pub fn object_to_v8<'a>(scope: &'a mut v8::HandleScope, map: &HashMap<String, Value>) -> v8::Local<'a, v8::Object> {
    let obj = v8::Object::new(scope);

    // TODO: Fix V8 borrow checker issues with object conversion  
    let _map = map; // Avoid unused warning

    obj
}

/// Helper function to create a V8 object with specific class name
pub fn create_v8_object_with_class<'a>(scope: &'a mut v8::HandleScope, class_name: &str) -> v8::Local<'a, v8::Object> {
    let obj = v8::Object::new(scope);

    // Set constructor.name
    if let (Some(constructor_key), Some(name_key), Some(class_name_str)) = (
        v8::String::new(scope, "constructor"),
        v8::String::new(scope, "name"),
        v8::String::new(scope, class_name)
    ) {
        let constructor_obj = v8::Object::new(scope);
        constructor_obj.set(scope, name_key.into(), class_name_str.into());
        obj.set(scope, constructor_key.into(), constructor_obj.into());
    }

    obj
}

/// Create a regular V8 object with properties from ObjectValue (simplified)
fn create_object_with_properties<'a>(scope: &'a mut v8::HandleScope, _obj: &ObjectValue) -> v8::Local<'a, v8::Value> {
    // TODO: Implement proper object creation with class name and properties
    // For now, return a simple object to avoid V8 borrow checker issues
    let v8_obj = v8::Object::new(scope);
    v8_obj.into()
}

/// Create Entity object with V8 methods (id(), getComponent()) - fixed version
pub fn create_entity_object_with_methods<'a>(scope: &'a mut v8::HandleScope, entity_id: u32) -> v8::Local<'a, v8::Value> {
    // Create entity object first
    let entity_obj = v8::Object::new(scope);
    
    // Set class name as constructor
    set_object_class_name(scope, entity_obj, "Entity");
    
    // Add id property with the entity ID (simpler approach)
    let id_key = v8::String::new(scope, "id").unwrap();
    let id_value = v8::Number::new(scope, entity_id as f64);
    entity_obj.set(scope, id_key.into(), id_value.into());
    
    // Add a simple id() method that returns the stored id
    add_id_method(scope, entity_obj);
    
    // Add direct ECS methods (Unity-style approach) 
    add_direct_ecs_methods(scope, entity_obj);
    
    entity_obj.into()
}

/// Helper to set object class name
fn set_object_class_name<'a>(scope: &'a mut v8::HandleScope, obj: v8::Local<'a, v8::Object>, class_name: &str) {
    if let (Some(constructor_key), Some(name_key), Some(class_name_str)) = (
        v8::String::new(scope, "constructor"),
        v8::String::new(scope, "name"),
        v8::String::new(scope, class_name)
    ) {
        let constructor_obj = v8::Object::new(scope);
        constructor_obj.set(scope, name_key.into(), class_name_str.into());
        obj.set(scope, constructor_key.into(), constructor_obj.into());
    }
}

/// Add id() method that reads from the id property
fn add_id_method<'a>(scope: &'a mut v8::HandleScope, obj: v8::Local<'a, v8::Object>) {
    // Create a simple function that reads the id property
    let id_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        // Get 'this' object
        let this = args.this();
        let id_key = v8::String::new(scope, "id").unwrap();
        if let Some(id_value) = this.get(scope, id_key.into()) {
            rv.set(id_value);
        } else {
            rv.set(v8::Number::new(scope, 0.0).into());
        }
    }).unwrap();
    
    let method_key = v8::String::new(scope, "id").unwrap();
    obj.set(scope, method_key.into(), id_fn.into());
}

/// Add direct ECS methods (Unity-style approach)
fn add_direct_ecs_methods<'a>(scope: &'a mut v8::HandleScope, obj: v8::Local<'a, v8::Object>) {
    // Add setPosition(x, y, z) method - direct Rust ECS call
    let set_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        if args.length() < 3 {
            if let Some(error) = v8::String::new(scope, "setPosition requires 3 arguments: x, y, z") {
                scope.throw_exception(error.into());
            }
            return;
        }

        // Extract x, y, z arguments
        let x = args.get(0).number_value(scope).unwrap_or(0.0) as f32;
        let y = args.get(1).number_value(scope).unwrap_or(0.0) as f32;
        let z = args.get(2).number_value(scope).unwrap_or(0.0) as f32;

        // Get entity ID from 'this' object
        let entity_id = {
            let this = args.this();
            let id_key = v8::String::new(scope, "id").unwrap();
            this.get(scope, id_key.into())
                .and_then(|v| v.number_value(scope))
                .unwrap_or(0.0) as u32
        };

        // Access ECS context and modify the transform
        if let Ok(context) = ECS_CONTEXT.lock() {
            if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                unsafe {
                    let world = &mut *world_ptr;
                    
                    // Get entity from ID (assuming generation 1)
                    let entity = engine_ecs_core::Entity::new(entity_id, 1);
                    
                    // Try to get and update the transform component
                    if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
                        transform.position = [x, y, z];
                        println!("✅ ACTUAL ECS UPDATE: Set position to [{}, {}, {}] for entity {}", x, y, z, entity_id);
                    } else {
                        println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                    }
                }
            } else {
                println!("❌ ECS ERROR: No world context available");
            }
        }
        
        rv.set(v8::undefined(scope).into());
    }).unwrap();
    
    let set_position_key = v8::String::new(scope, "setPosition").unwrap();
    obj.set(scope, set_position_key.into(), set_position_fn.into());

    // Add getPosition() method - direct Rust ECS call
    let get_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
        println!("🚀 DIRECT ECS CALL: getPosition()");

        // Get entity ID from 'this' object
        let entity_id = {
            let this = args.this();
            let id_key = v8::String::new(scope, "id").unwrap();
            this.get(scope, id_key.into())
                .and_then(|v| v.number_value(scope))
                .unwrap_or(0.0) as u32
        };

        // Get actual position from ECS
        let (pos_x, pos_y, pos_z) = if let Ok(context) = ECS_CONTEXT.lock() {
            if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                unsafe {
                    let world = &*world_ptr;
                    let entity = engine_ecs_core::Entity::new(entity_id, 1);
                    
                    if let Some(transform) = world.get_component::<Transform>(entity) {
                        let pos = transform.position;
                        println!("✅ ACTUAL ECS READ: Got position [{}, {}, {}] for entity {}", pos[0], pos[1], pos[2], entity_id);
                        (pos[0], pos[1], pos[2])
                    } else {
                        println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                        (0.0, 0.0, 0.0)
                    }
                }
            } else {
                println!("❌ ECS ERROR: No world context available");
                (0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0)
        };
        
        // Create Vector3-like object with actual position values
        let position_obj = v8::Object::new(scope);
        if let (Some(x_key), Some(y_key), Some(z_key)) = (
            v8::String::new(scope, "x"),
            v8::String::new(scope, "y"), 
            v8::String::new(scope, "z")
        ) {
            let x_val = v8::Number::new(scope, pos_x as f64);
            position_obj.set(scope, x_key.into(), x_val.into());
            
            let y_val = v8::Number::new(scope, pos_y as f64);
            position_obj.set(scope, y_key.into(), y_val.into());
            
            let z_val = v8::Number::new(scope, pos_z as f64);
            position_obj.set(scope, z_key.into(), z_val.into());
        }
        
        rv.set(position_obj.into());
    }).unwrap();
    
    let get_position_key = v8::String::new(scope, "getPosition").unwrap();
    obj.set(scope, get_position_key.into(), get_position_fn.into());
}