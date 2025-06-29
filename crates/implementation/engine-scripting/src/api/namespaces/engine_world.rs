//! Engine.World namespace implementation
//! 
//! Provides entity and world management functionality to TypeScript scripts

use crate::api::registry::{
    ApiRegistry, ApiError, NamespaceDescriptor, ClassDescriptor, PropertyDescriptor,
    MethodBinding, MethodBindingBuilder, MethodContext, 
    Value, ObjectValue, TypeDescriptor
};

/// Register the Engine.World namespace with the API registry
pub fn register_engine_world(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    // Create Entity class descriptor
    let entity_class = ClassDescriptor::new("Entity".to_string())
        .with_methods(vec![
            "id".to_string(),
            "getComponent".to_string(),
            "addComponent".to_string(),
            "removeComponent".to_string(),
            "hasComponent".to_string(),
        ])
        .with_properties(vec![
            PropertyDescriptor::new("id".to_string(), TypeDescriptor::Number).readonly(),
        ])
        .with_documentation("Represents an entity in the game world".to_string());

    // Create namespace descriptor
    let namespace = NamespaceDescriptor::new("Engine.World".to_string())
        .with_methods(vec![
            "getCurrentEntity".to_string(),
            "getEntity".to_string(),
            "createEntity".to_string(),
            "destroyEntity".to_string(),
            "findEntitiesByTag".to_string(),
        ])
        .with_classes(vec![entity_class])
        .with_documentation("Core world and entity management functionality".to_string());

    // Register the namespace
    registry.register_namespace(namespace)?;

    // Register method implementations
    register_get_current_entity(registry)?;
    register_get_entity(registry)?;
    register_create_entity(registry)?;
    register_destroy_entity(registry)?;
    register_find_entities_by_tag(registry)?;

    Ok(())
}

/// Register getCurrentEntity method
fn register_get_current_entity(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.World", "getCurrentEntity")
        .with_return_type(TypeDescriptor::Object("Entity".to_string()))
        .with_documentation("Get the entity that this script is attached to")
        .build(|context: &MethodContext, _args: &[Value]| -> Result<Value, ApiError> {
            match context.entity_context {
                Some(entity_id) => {
                    // Create Entity object that needs V8 method binding
                    let mut entity_obj = ObjectValue::new_with_methods("Entity".to_string());
                    entity_obj.set_property("id".to_string(), Value::Number(entity_id as f64));
                    
                    Ok(Value::Object(entity_obj))
                }
                None => Err(ApiError::NoEntityContext),
            }
        });

    registry.register_method(binding)
}

/// Register getEntity method
fn register_get_entity(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.World", "getEntity")
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Optional(Box::new(TypeDescriptor::Object("Entity".to_string()))))
        .with_documentation("Get an entity by its ID")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let entity_id: f64 = args[0].as_number()
                .ok_or_else(|| ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedNumber))?;

            // TODO: Check if entity exists in world
            // For now, always return an entity if ID is positive
            if entity_id > 0.0 {
                let mut entity_obj = ObjectValue::new_with_methods("Entity".to_string());
                entity_obj.set_property("id".to_string(), Value::Number(entity_id));
                Ok(Value::Object(entity_obj))
            } else {
                Ok(Value::Null)
            }
        });

    registry.register_method(binding)
}

/// Register createEntity method
fn register_create_entity(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.World", "createEntity")
        .with_parameter(TypeDescriptor::Optional(Box::new(TypeDescriptor::String)))
        .with_return_type(TypeDescriptor::Object("Entity".to_string()))
        .with_documentation("Create a new entity with optional name")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let name = if args.is_empty() {
                "Entity".to_string()
            } else {
                args[0].as_string().unwrap_or("Entity").to_string()
            };

            // TODO: Create actual entity in world
            // For now, return mock entity with incremental ID
            let entity_id = 1000.0; // Mock ID
            
            let mut entity_obj = ObjectValue::new_with_methods("Entity".to_string());
            entity_obj.set_property("id".to_string(), Value::Number(entity_id));
            entity_obj.set_property("name".to_string(), Value::String(name));
            
            Ok(Value::Object(entity_obj))
        });

    registry.register_method(binding)
}

/// Register destroyEntity method
fn register_destroy_entity(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.World", "destroyEntity")
        .with_parameter(TypeDescriptor::Object("Entity".to_string()))
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Destroy an entity")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let entity = args[0].as_object()
                .ok_or_else(|| ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedObject("Entity".to_string())))?;

            if entity.class_name() != "Entity" {
                return Err(ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedObject("Entity".to_string())));
            }

            // TODO: Actually destroy entity in world
            // For now, just return success
            
            Ok(Value::Void)
        });

    registry.register_method(binding)
}

/// Register findEntitiesByTag method
fn register_find_entities_by_tag(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.World", "findEntitiesByTag")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Array(Box::new(TypeDescriptor::Object("Entity".to_string()))))
        .with_documentation("Find all entities with a specific tag")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let _tag = args[0].as_string()
                .ok_or_else(|| ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedString))?;

            // TODO: Actually search for entities by tag
            // For now, return empty array
            Ok(Value::Array(vec![]))
        });

    registry.register_method(binding)
}

/// Helper to create Entity Value objects
pub fn create_entity_value(entity_id: u32) -> Value {
    let mut entity_obj = ObjectValue::new_with_methods("Entity".to_string());
    entity_obj.set_property("id".to_string(), Value::Number(entity_id as f64));
    Value::Object(entity_obj)
}

/// Helper to create Transform component Value objects
pub fn create_transform_value(x: f32, y: f32, z: f32) -> Value {
    let mut transform_obj = ObjectValue::new("Transform".to_string());
    
    // Create position object
    let mut position_obj = ObjectValue::new("Vector3".to_string());
    position_obj.set_property("x".to_string(), Value::Number(x as f64));
    position_obj.set_property("y".to_string(), Value::Number(y as f64));
    position_obj.set_property("z".to_string(), Value::Number(z as f64));
    
    transform_obj.set_property("position".to_string(), Value::Object(position_obj));
    Value::Object(transform_obj)
}