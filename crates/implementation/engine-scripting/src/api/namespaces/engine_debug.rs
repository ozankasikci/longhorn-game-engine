//! Engine.Debug namespace implementation
//! 
//! Provides debugging and logging utilities for TypeScript scripts

use crate::api::registry::{
    ApiRegistry, ApiError, NamespaceDescriptor,
    MethodBindingBuilder, MethodContext, 
    Value, TypeDescriptor
};

/// Register the Engine.Debug namespace with the API registry
pub fn register_engine_debug(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    // Create namespace descriptor
    let namespace = NamespaceDescriptor::new("Engine.Debug".to_string())
        .with_methods(vec![
            "log".to_string(),
            "warn".to_string(),
            "error".to_string(),
            "info".to_string(),
            "trace".to_string(),
        ])
        .with_documentation("Debugging and logging utilities".to_string());

    // Register the namespace
    registry.register_namespace(namespace)?;

    // Register method implementations
    register_log(registry)?;
    register_warn(registry)?;
    register_error(registry)?;
    register_info(registry)?;
    register_trace(registry)?;

    Ok(())
}

/// Register log method
fn register_log(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Debug", "log")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Log a message to the console")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let message = args[0].as_string().unwrap_or("").to_string();
            log::info!("[Script] {}", message);
            Ok(Value::Void)
        });

    registry.register_method(binding)
}

/// Register warn method
fn register_warn(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Debug", "warn")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Log a warning message to the console")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let message = args[0].as_string().unwrap_or("").to_string();
            log::warn!("[Script] {}", message);
            Ok(Value::Void)
        });

    registry.register_method(binding)
}

/// Register error method
fn register_error(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Debug", "error")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Log an error message to the console")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let message = args[0].as_string().unwrap_or("").to_string();
            log::error!("[Script] {}", message);
            Ok(Value::Void)
        });

    registry.register_method(binding)
}

/// Register info method
fn register_info(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Debug", "info")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Log an info message to the console")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let message = args[0].as_string().unwrap_or("").to_string();
            log::info!("[Script] {}", message);
            Ok(Value::Void)
        });

    registry.register_method(binding)
}

/// Register trace method
fn register_trace(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    let binding = MethodBindingBuilder::new("Engine.Debug", "trace")
        .with_parameter(TypeDescriptor::String)
        .with_return_type(TypeDescriptor::Void)
        .with_documentation("Log a trace message to the console")
        .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
            let message = args[0].as_string().unwrap_or("").to_string();
            log::trace!("[Script] {}", message);
            Ok(Value::Void)
        });

    registry.register_method(binding)
}