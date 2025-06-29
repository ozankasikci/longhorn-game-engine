//! V8 bindings for the API registry system

use super::type_conversion;
use crate::api::registry::{ApiRegistry, ApiError, MethodContext, MethodBinding};
use std::sync::Arc;

/// V8 binding manager that creates JavaScript functions from method bindings
pub struct V8BindingManager {
    registry: Arc<ApiRegistry>,
}

impl V8BindingManager {
    pub fn new(registry: Arc<ApiRegistry>) -> Self {
        Self { registry }
    }

    /// Setup global Engine object with all namespace bindings
    pub fn setup_engine_api_injection<'a>(&self, scope: &'a mut v8::HandleScope, global: v8::Local<'a, v8::Object>) -> Result<(), ApiError> {
        // Create the root Engine object
        let engine_obj = v8::Object::new(scope);
        let engine_name = v8::String::new(scope, "Engine")
            .ok_or_else(|| ApiError::V8Error("Failed to create Engine string".to_string()))?;

        // Setup all Engine.* namespaces
        for (namespace_name, descriptor) in self.registry.get_namespaces() {
            if namespace_name.starts_with("Engine.") {
                self.setup_namespace_binding(scope, engine_obj, namespace_name, descriptor)?;
            }
        }

        // Set Engine on global object
        global.set(scope, engine_name.into(), engine_obj.into());
        Ok(())
    }

    /// Setup a specific namespace binding
    fn setup_namespace_binding<'a>(
        &self,
        scope: &'a mut v8::HandleScope,
        parent_obj: v8::Local<'a, v8::Object>,
        full_namespace: &str,
        descriptor: &crate::api::registry::NamespaceDescriptor,
    ) -> Result<(), ApiError> {
        // Extract the namespace part after "Engine."
        let namespace_part = full_namespace.strip_prefix("Engine.")
            .unwrap_or(full_namespace);

        // Create namespace object
        let namespace_obj = v8::Object::new(scope);
        let namespace_name = v8::String::new(scope, namespace_part)
            .ok_or_else(|| ApiError::V8Error(format!("Failed to create string for {}", namespace_part)))?;

        // TODO: Add all methods to the namespace object
        // Temporarily disabled due to V8 lifetime issues
        let _descriptor_methods = &descriptor.methods; // Avoid unused warning

        // Add namespace to parent object
        parent_obj.set(scope, namespace_name.into(), namespace_obj.into());
        Ok(())
    }

    /// Create a V8 function that calls a method binding (simplified version)
    fn create_method_function<'a>(
        &self,
        scope: &'a mut v8::HandleScope,
        _binding: Arc<MethodBinding>,
    ) -> Result<v8::Local<'a, v8::Function>, ApiError> {
        // TODO: Implement proper method binding system
        // For now, create a placeholder function that returns null
        let function = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            rv.set(v8::null(scope).into());
        })
        .ok_or_else(|| ApiError::V8Error("Failed to create V8 function".to_string()))?;

        Ok(function)
    }

    /// Create Entity object binding for scripts (simplified version)
    pub fn create_entity_object<'a>(&self, scope: &'a mut v8::HandleScope, _entity_id: u32) -> Result<v8::Local<'a, v8::Object>, ApiError> {
        // TODO: Implement proper entity object with methods
        // For now, return a simple object
        let entity_obj = type_conversion::create_v8_object_with_class(scope, "Entity");
        Ok(entity_obj)
    }

    /// Create getComponent function for Entity objects (simplified version)
    fn create_get_component_function<'a>(&self, scope: &'a mut v8::HandleScope, _entity_id: u32) -> Result<v8::Local<'a, v8::Function>, ApiError> {
        // TODO: Implement proper getComponent function
        // For now, return a placeholder function that returns null
        let function = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            rv.set(v8::null(scope).into());
        })
        .ok_or_else(|| ApiError::V8Error("Failed to create getComponent function".to_string()))?;

        Ok(function)
    }

    /// Get reference to the registry
    pub fn registry(&self) -> &Arc<ApiRegistry> {
        &self.registry
    }
}