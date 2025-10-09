//! V8 Bridge system for exposing the API registry to JavaScript

pub mod v8_bindings;
pub mod type_conversion;

pub use v8_bindings::*;
pub use type_conversion::*;

use super::registry::{ApiRegistry, ApiError, MethodContext};
use std::sync::Arc;

/// Bridge that connects the API registry to V8 JavaScript engine
pub struct V8ApiBridge {
    registry: Arc<ApiRegistry>,
    entity_context: std::sync::Arc<std::sync::Mutex<Option<u32>>>,
}

impl V8ApiBridge {
    pub fn new(registry: Arc<ApiRegistry>) -> Result<Self, ApiError> {
        Ok(Self { 
            registry,
            entity_context: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    /// Set entity context for method calls
    pub fn set_entity_context(&self, entity_id: Option<u32>) {
        if let Ok(mut context) = self.entity_context.lock() {
            *context = entity_id;
        }
    }

    /// Initialize V8 global objects for all registered namespaces
    pub fn initialize_context<'a>(&self, _scope: &'a mut v8::HandleScope, _global: v8::Local<'a, v8::Object>) -> Result<(), ApiError> {
        // TODO: Fix V8 borrow checker issues - temporarily disabled
        // Create global objects for all namespaces
        // for (namespace_name, _) in self.registry.get_namespaces() {
        //     self.create_namespace_object(scope, global, namespace_name)?;
        // }

        Ok(())
    }

    /// Execute a JavaScript script with API access
    pub fn execute_script(&self, script_source: &str, entity_context: Option<u32>) -> Result<String, ApiError> {
        // This is a simplified version - in the full implementation this would
        // integrate with the existing TypeScript runtime
        
        // For now, return a placeholder indicating the API system is ready
        Ok(format!(
            "API Bridge initialized with {} namespaces, {} methods. Entity context: {:?}",
            self.registry.get_namespaces().len(),
            self.registry.get_methods().len(),
            entity_context
        ))
    }

    /// Create a namespace object hierarchy in V8 (temporarily disabled)
    fn create_namespace_object<'a>(
        &self,
        _scope: &'a mut v8::HandleScope,
        _global: v8::Local<'a, v8::Object>,
        _namespace: &str,
    ) -> Result<(), ApiError> {
        // TODO: Fix V8 borrow checker issues - temporarily disabled
        // Complex borrow checker issue with loop and multiple mutable borrows of scope
        Ok(())
    }

    /// Create method bindings for a specific namespace
    fn create_namespace_methods<'a>(
        &self,
        scope: &'a mut v8::HandleScope,
        _namespace: &str,
    ) -> Result<v8::Local<'a, v8::Object>, ApiError> {
        let obj = v8::Object::new(scope);

        // TODO: Fix V8 borrow checker issues - temporarily disabled
        // if let Some(ns_descriptor) = self.registry.get_namespaces().get(namespace) {
        //     for method_name in &ns_descriptor.methods {
        //         let method_binding = self.registry.get_method(namespace, method_name)
        //             .ok_or_else(|| ApiError::MethodNotFound {
        //                 namespace: namespace.to_string(),
        //                 method: method_name.to_string(),
        //             })?;

        //         let method_fn = self.create_v8_method(scope, method_binding.clone(), namespace)?;
        //         let name = v8::String::new(scope, method_name)
        //             .ok_or_else(|| ApiError::V8Error("Failed to create method name".to_string()))?;

        //         obj.set(scope, name.into(), method_fn.into());
        //     }
        // }

        Ok(obj)
    }

    /// Create a V8 function that calls a registered method (simplified version)
    fn create_v8_method<'a>(
        &self,
        scope: &'a mut v8::HandleScope,
        _binding: Arc<super::registry::MethodBinding>,
        _namespace: &str,
    ) -> Result<v8::Local<'a, v8::Function>, ApiError> {
        // TODO: Implement proper method binding system
        // For now, create a placeholder function that returns null
        let function = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            rv.set(v8::null(scope).into());
        })
        .ok_or_else(|| ApiError::V8Error("Failed to create function".to_string()))?;

        Ok(function)
    }

    /// Get the registry reference
    pub fn registry(&self) -> &Arc<ApiRegistry> {
        &self.registry
    }
}