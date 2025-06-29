//! Engine.Math namespace implementation
//! 
//! Provides mathematical utilities for TypeScript scripts

use crate::api::registry::{
    ApiRegistry, ApiError, NamespaceDescriptor, ClassDescriptor,
    MethodBindingBuilder, MethodContext, 
    Value, ObjectValue, TypeDescriptor
};

/// Register the Engine.Math namespace with the API registry
pub fn register_engine_math(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    // Create Vector3 class descriptor
    let vector3_class = ClassDescriptor::new("Vector3".to_string())
        .with_methods(vec![
            "magnitude".to_string(),
            "normalize".to_string(),
            "dot".to_string(),
            "cross".to_string(),
        ])
        .with_documentation("3D vector mathematics".to_string());

    // Create namespace descriptor
    let namespace = NamespaceDescriptor::new("Engine.Math".to_string())
        .with_methods(vec![
            "lerp".to_string(),
            "clamp".to_string(),
            "abs".to_string(),
            "sin".to_string(),
            "cos".to_string(),
            "sqrt".to_string(),
            "distance".to_string(),
        ])
        .with_classes(vec![vector3_class])
        .with_documentation("Mathematical utilities and vector operations".to_string());

    // Register the namespace
    registry.register_namespace(namespace)?;

    // Register method implementations
    register_lerp(registry)?;
    register_clamp(registry)?;
    register_abs(registry)?;
    register_sin(registry)?;
    register_cos(registry)?;
    register_sqrt(registry)?;
    register_distance(registry)?;

    Ok(())
}

/// Register lerp method
fn register_lerp(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "lerp")
        .with_parameter(TypeDescriptor::Number)
        .with_parameter(TypeDescriptor::Number)
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Linear interpolation between two values")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let a = args[0].as_number().unwrap_or(0.0);
            let b = args[1].as_number().unwrap_or(0.0);
            let t = args[2].as_number().unwrap_or(0.0);

            let result = a + (b - a) * t.clamp(0.0, 1.0);
            Ok(Value::Number(result))
        });

    registry.register_method(binding)
}

/// Register clamp method
fn register_clamp(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "clamp")
        .with_parameter(TypeDescriptor::Number)
        .with_parameter(TypeDescriptor::Number)
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Clamp a value between min and max")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let value = args[0].as_number().unwrap_or(0.0);
            let min = args[1].as_number().unwrap_or(0.0);
            let max = args[2].as_number().unwrap_or(1.0);

            let result = value.clamp(min, max);
            Ok(Value::Number(result))
        });

    registry.register_method(binding)
}

/// Register abs method
fn register_abs(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "abs")
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Absolute value of a number")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let value = args[0].as_number().unwrap_or(0.0);
            Ok(Value::Number(value.abs()))
        });

    registry.register_method(binding)
}

/// Register sin method
fn register_sin(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "sin")
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Sine of an angle in radians")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let angle = args[0].as_number().unwrap_or(0.0);
            Ok(Value::Number(angle.sin()))
        });

    registry.register_method(binding)
}

/// Register cos method
fn register_cos(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "cos")
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Cosine of an angle in radians")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let angle = args[0].as_number().unwrap_or(0.0);
            Ok(Value::Number(angle.cos()))
        });

    registry.register_method(binding)
}

/// Register sqrt method
fn register_sqrt(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "sqrt")
        .with_parameter(TypeDescriptor::Number)
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Square root of a number")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let value = args[0].as_number().unwrap_or(0.0);
            if value < 0.0 {
                Ok(Value::Number(f64::NAN))
            } else {
                Ok(Value::Number(value.sqrt()))
            }
        });

    registry.register_method(binding)
}

/// Register distance method
fn register_distance(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Math", "distance")
        .with_parameter(TypeDescriptor::Object("Vector3".to_string()))
        .with_parameter(TypeDescriptor::Object("Vector3".to_string()))
        .with_return_type(TypeDescriptor::Number)
        .with_documentation("Distance between two 3D points")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let vec1 = args[0].as_object()
                .ok_or_else(|| ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedObject("Vector3".to_string())))?;
            let vec2 = args[1].as_object()
                .ok_or_else(|| ApiError::TypeError(crate::api::registry::type_system::TypeError::ExpectedObject("Vector3".to_string())))?;

            // Extract coordinates from Vector3 objects
            let x1 = vec1.get_property("x").and_then(|v| v.as_number()).unwrap_or(0.0);
            let y1 = vec1.get_property("y").and_then(|v| v.as_number()).unwrap_or(0.0);
            let z1 = vec1.get_property("z").and_then(|v| v.as_number()).unwrap_or(0.0);

            let x2 = vec2.get_property("x").and_then(|v| v.as_number()).unwrap_or(0.0);
            let y2 = vec2.get_property("y").and_then(|v| v.as_number()).unwrap_or(0.0);
            let z2 = vec2.get_property("z").and_then(|v| v.as_number()).unwrap_or(0.0);

            // Calculate distance
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dz = z2 - z1;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

            Ok(Value::Number(distance))
        });

    registry.register_method(binding)
}

/// Helper to create Vector3 Value objects
pub fn create_vector3_value(x: f32, y: f32, z: f32) -> Value {
    let mut vector_obj = ObjectValue::new("Vector3".to_string());
    vector_obj.set_property("x".to_string(), Value::Number(x as f64));
    vector_obj.set_property("y".to_string(), Value::Number(y as f64));
    vector_obj.set_property("z".to_string(), Value::Number(z as f64));
    Value::Object(vector_obj)
}