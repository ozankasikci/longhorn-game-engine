//! Physics system bindings for TypeScript
//! 
//! This module provides JavaScript/TypeScript bindings for the engine's physics system,
//! allowing scripts to create rigid bodies, apply forces, perform raycasts, and manage gravity.

use engine_scripting::{ScriptError, ScriptResult};
use std::sync::atomic::{AtomicU32, Ordering};

/// Global handle ID counter for creating unique physics object handles
static HANDLE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Register Physics API bindings with the V8 context
pub fn register_physics_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
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
    
    // Create physics object
    let physics_key = v8::String::new(scope, "physics")
        .ok_or_else(|| ScriptError::runtime("Failed to create physics string".to_string()))?;
    let physics_obj = v8::Object::new(scope);
    
    // Register rigid body functions
    register_rigid_body_functions(scope, physics_obj)?;
    
    // Register force functions
    register_force_functions(scope, physics_obj)?;
    
    // Register raycast functions
    register_raycast_functions(scope, physics_obj)?;
    
    // Register gravity functions
    register_gravity_functions(scope, physics_obj)?;
    
    // Register collider functions
    register_collider_functions(scope, physics_obj)?;
    
    // Set physics on engine
    engine_obj.set(scope, physics_key.into(), physics_obj.into());
    
    Ok(())
}

/// Register rigid body management functions
fn register_rigid_body_functions(scope: &mut v8::HandleScope, physics_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // addRigidBody function
    let add_rigid_body_key = v8::String::new(scope, "addRigidBody")
        .ok_or_else(|| ScriptError::runtime("Failed to create addRigidBody string".to_string()))?;
    
    let add_rigid_body_fn = v8::Function::new(scope, add_rigid_body_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create addRigidBody function".to_string()))?;
    
    physics_obj.set(scope, add_rigid_body_key.into(), add_rigid_body_fn.into());
    
    // removeRigidBody function
    let remove_rigid_body_key = v8::String::new(scope, "removeRigidBody")
        .ok_or_else(|| ScriptError::runtime("Failed to create removeRigidBody string".to_string()))?;
    
    let remove_rigid_body_fn = v8::Function::new(scope, remove_rigid_body_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create removeRigidBody function".to_string()))?;
    
    physics_obj.set(scope, remove_rigid_body_key.into(), remove_rigid_body_fn.into());
    
    Ok(())
}

/// Register force application functions
fn register_force_functions(scope: &mut v8::HandleScope, physics_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // applyForce function
    let apply_force_key = v8::String::new(scope, "applyForce")
        .ok_or_else(|| ScriptError::runtime("Failed to create applyForce string".to_string()))?;
    
    let apply_force_fn = v8::Function::new(scope, apply_force_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create applyForce function".to_string()))?;
    
    physics_obj.set(scope, apply_force_key.into(), apply_force_fn.into());
    
    // applyImpulse function
    let apply_impulse_key = v8::String::new(scope, "applyImpulse")
        .ok_or_else(|| ScriptError::runtime("Failed to create applyImpulse string".to_string()))?;
    
    let apply_impulse_fn = v8::Function::new(scope, apply_impulse_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create applyImpulse function".to_string()))?;
    
    physics_obj.set(scope, apply_impulse_key.into(), apply_impulse_fn.into());
    
    Ok(())
}

/// Register raycast functions
fn register_raycast_functions(scope: &mut v8::HandleScope, physics_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // raycast function
    let raycast_key = v8::String::new(scope, "raycast")
        .ok_or_else(|| ScriptError::runtime("Failed to create raycast string".to_string()))?;
    
    let raycast_fn = v8::Function::new(scope, raycast_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create raycast function".to_string()))?;
    
    physics_obj.set(scope, raycast_key.into(), raycast_fn.into());
    
    Ok(())
}

/// Register gravity functions
fn register_gravity_functions(scope: &mut v8::HandleScope, physics_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // getGravity function
    let get_gravity_key = v8::String::new(scope, "getGravity")
        .ok_or_else(|| ScriptError::runtime("Failed to create getGravity string".to_string()))?;
    
    let get_gravity_fn = v8::Function::new(scope, get_gravity_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create getGravity function".to_string()))?;
    
    physics_obj.set(scope, get_gravity_key.into(), get_gravity_fn.into());
    
    // setGravity function
    let set_gravity_key = v8::String::new(scope, "setGravity")
        .ok_or_else(|| ScriptError::runtime("Failed to create setGravity string".to_string()))?;
    
    let set_gravity_fn = v8::Function::new(scope, set_gravity_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create setGravity function".to_string()))?;
    
    physics_obj.set(scope, set_gravity_key.into(), set_gravity_fn.into());
    
    Ok(())
}

/// Register collider shape functions
fn register_collider_functions(scope: &mut v8::HandleScope, physics_obj: v8::Local<v8::Object>) -> ScriptResult<()> {
    // addBoxCollider function
    let add_box_key = v8::String::new(scope, "addBoxCollider")
        .ok_or_else(|| ScriptError::runtime("Failed to create addBoxCollider string".to_string()))?;
    
    let add_box_fn = v8::Function::new(scope, add_box_collider_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create addBoxCollider function".to_string()))?;
    
    physics_obj.set(scope, add_box_key.into(), add_box_fn.into());
    
    // addSphereCollider function
    let add_sphere_key = v8::String::new(scope, "addSphereCollider")
        .ok_or_else(|| ScriptError::runtime("Failed to create addSphereCollider string".to_string()))?;
    
    let add_sphere_fn = v8::Function::new(scope, add_sphere_collider_callback)
        .ok_or_else(|| ScriptError::runtime("Failed to create addSphereCollider function".to_string()))?;
    
    physics_obj.set(scope, add_sphere_key.into(), add_sphere_fn.into());
    
    Ok(())
}

/// Helper function to extract Vector3 from V8 object
fn extract_vector3(scope: &mut v8::HandleScope, obj: v8::Local<v8::Value>) -> Option<(f64, f64, f64)> {
    if let Ok(vec_obj) = v8::Local::<v8::Object>::try_from(obj) {
        let x_key = v8::String::new(scope, "x")?;
        let y_key = v8::String::new(scope, "y")?;
        let z_key = v8::String::new(scope, "z")?;
        
        let x = vec_obj.get(scope, x_key.into())?.number_value(scope)?;
        let y = vec_obj.get(scope, y_key.into())?.number_value(scope)?;
        let z = vec_obj.get(scope, z_key.into())?.number_value(scope)?;
        
        Some((x, y, z))
    } else {
        None
    }
}

/// Helper function to create Vector3 V8 object
fn create_vector3<'a>(scope: &'a mut v8::HandleScope, x: f64, y: f64, z: f64) -> v8::Local<'a, v8::Object> {
    let vec_obj = v8::Object::new(scope);
    
    let x_key = v8::String::new(scope, "x").unwrap();
    let y_key = v8::String::new(scope, "y").unwrap();
    let z_key = v8::String::new(scope, "z").unwrap();
    
    let x_value = v8::Number::new(scope, x);
    let y_value = v8::Number::new(scope, y);
    let z_value = v8::Number::new(scope, z);
    
    vec_obj.set(scope, x_key.into(), x_value.into());
    vec_obj.set(scope, y_key.into(), y_value.into());
    vec_obj.set(scope, z_key.into(), z_value.into());
    
    vec_obj
}

/// JavaScript callback for engine.physics.addRigidBody(position, bodyType, mass)
fn add_rigid_body_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 3 {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    let position = args.get(0);
    let body_type = args.get(1);
    let mass = args.get(2);
    
    // Validate position
    if extract_vector3(scope, position).is_none() {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Validate body type
    if let Some(type_str) = body_type.to_string(scope) {
        let type_string = type_str.to_rust_string_lossy(scope);
        if !["Static", "Dynamic", "Kinematic"].contains(&type_string.as_str()) {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Validate mass
    if let Some(mass_val) = mass.number_value(scope) {
        if mass_val < 0.0 {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Generate a handle ID
    let handle_id = HANDLE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // For now, just return the handle ID (no actual physics integration)
    // In a real implementation, this would create an actual rigid body
    rv.set(v8::Number::new(scope, handle_id as f64).into());
}

/// JavaScript callback for engine.physics.removeRigidBody(handle)
fn remove_rigid_body_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let _handle = args.get(0);
    
    // For now, do nothing (no actual physics integration)
    // In a real implementation, this would remove the rigid body from the physics world
}

/// JavaScript callback for engine.physics.applyForce(handle, force)
fn apply_force_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }
    
    let _handle = args.get(0);
    let force = args.get(1);
    
    // Validate force vector
    if extract_vector3(scope, force).is_none() {
        return;
    }
    
    // For now, do nothing (no actual physics integration)
    // In a real implementation, this would apply force to the rigid body
}

/// JavaScript callback for engine.physics.applyImpulse(handle, impulse)
fn apply_impulse_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        return;
    }
    
    let _handle = args.get(0);
    let impulse = args.get(1);
    
    // Validate impulse vector
    if extract_vector3(scope, impulse).is_none() {
        return;
    }
    
    // For now, do nothing (no actual physics integration)
    // In a real implementation, this would apply impulse to the rigid body
}

/// JavaScript callback for engine.physics.raycast(origin, direction, maxDistance)
fn raycast_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 3 {
        rv.set(v8::null(scope).into());
        return;
    }
    
    let origin = args.get(0);
    let direction = args.get(1);
    let max_distance = args.get(2);
    
    // Validate parameters
    if extract_vector3(scope, origin).is_none() || 
       extract_vector3(scope, direction).is_none() ||
       max_distance.number_value(scope).is_none() {
        rv.set(v8::null(scope).into());
        return;
    }
    
    // For now, return null (no hit)
    // In a real implementation, this would perform actual raycasting
    rv.set(v8::null(scope).into());
}

/// JavaScript callback for engine.physics.getGravity()
fn get_gravity_callback(
    scope: &mut v8::HandleScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // For now, return default gravity
    // In a real implementation, this would get the actual gravity from the physics world
    let gravity = create_vector3(scope, 0.0, -9.81, 0.0);
    rv.set(gravity.into());
}

/// JavaScript callback for engine.physics.setGravity(gravity)
fn set_gravity_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }
    
    let gravity = args.get(0);
    
    // Validate gravity vector
    if extract_vector3(scope, gravity).is_none() {
        return;
    }
    
    // For now, do nothing (no actual physics integration)
    // In a real implementation, this would set the gravity in the physics world
}

/// JavaScript callback for engine.physics.addBoxCollider(position, size, bodyType)
fn add_box_collider_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 3 {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    let position = args.get(0);
    let size = args.get(1);
    let body_type = args.get(2);
    
    // Validate parameters
    if extract_vector3(scope, position).is_none() || 
       extract_vector3(scope, size).is_none() {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Validate body type
    if let Some(type_str) = body_type.to_string(scope) {
        let type_string = type_str.to_rust_string_lossy(scope);
        if !["Static", "Dynamic", "Kinematic"].contains(&type_string.as_str()) {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Generate a handle ID
    let handle_id = HANDLE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // For now, just return the handle ID
    rv.set(v8::Number::new(scope, handle_id as f64).into());
}

/// JavaScript callback for engine.physics.addSphereCollider(position, radius, bodyType)
fn add_sphere_collider_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 3 {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    let position = args.get(0);
    let radius = args.get(1);
    let body_type = args.get(2);
    
    // Validate parameters
    if extract_vector3(scope, position).is_none() || 
       radius.number_value(scope).is_none() {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Validate body type
    if let Some(type_str) = body_type.to_string(scope) {
        let type_string = type_str.to_rust_string_lossy(scope);
        if !["Static", "Dynamic", "Kinematic"].contains(&type_string.as_str()) {
            rv.set(v8::Number::new(scope, -1.0).into());
            return;
        }
    } else {
        rv.set(v8::Number::new(scope, -1.0).into());
        return;
    }
    
    // Generate a handle ID
    let handle_id = HANDLE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // For now, just return the handle ID
    rv.set(v8::Number::new(scope, handle_id as f64).into());
}