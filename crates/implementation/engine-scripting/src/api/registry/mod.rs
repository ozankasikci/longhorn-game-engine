//! Core API Registry System
//! 
//! This module implements the registry pattern for managing TypeScript API namespaces,
//! methods, and types. It provides a foundation for Unity-style namespace organization.

use std::collections::HashMap;
use std::sync::Arc;

pub mod namespace;
pub mod method_binding;
pub mod type_system;

pub use namespace::*;
pub use method_binding::*;
pub use type_system::*;

/// Core registry that manages all API namespaces, methods, and types
pub struct ApiRegistry {
    namespaces: HashMap<String, NamespaceDescriptor>,
    methods: HashMap<String, Arc<MethodBinding>>,
    types: HashMap<String, TypeDescriptor>,
    initialized: bool,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            methods: HashMap::new(),
            types: HashMap::new(),
            initialized: false,
        }
    }

    /// Register a new namespace in the API
    pub fn register_namespace(&mut self, descriptor: NamespaceDescriptor) -> Result<(), ApiError> {
        if self.initialized {
            return Err(ApiError::RegistrationAfterInit);
        }

        let name = descriptor.name.clone();
        
        // Validate namespace hierarchy
        if let Some(parent) = &descriptor.parent {
            if !self.namespaces.contains_key(parent) {
                return Err(ApiError::ParentNamespaceNotFound(parent.clone()));
            }
        }

        // Register methods
        for method_name in &descriptor.methods {
            let full_name = format!("{}.{}", name, method_name);
            if self.methods.contains_key(&full_name) {
                return Err(ApiError::DuplicateMethod(full_name));
            }
        }

        self.namespaces.insert(name, descriptor);
        Ok(())
    }

    /// Register a method implementation
    pub fn register_method(&mut self, binding: MethodBinding) -> Result<(), ApiError> {
        let full_name = format!("{}.{}", binding.namespace, binding.method_name);
        
        if self.methods.contains_key(&full_name) {
            return Err(ApiError::DuplicateMethod(full_name));
        }

        self.methods.insert(full_name, Arc::new(binding));
        Ok(())
    }

    /// Register a type descriptor
    pub fn register_type(&mut self, name: String, descriptor: TypeDescriptor) -> Result<(), ApiError> {
        if self.initialized {
            return Err(ApiError::RegistrationAfterInit);
        }

        self.types.insert(name, descriptor);
        Ok(())
    }

    /// Finalize the registry and validate all registrations
    pub fn finalize(&mut self) -> Result<(), ApiError> {
        // Validate all registered methods have implementations
        for (namespace_name, namespace) in &self.namespaces {
            for method_name in &namespace.methods {
                let full_name = format!("{}.{}", namespace_name, method_name);
                if !self.methods.contains_key(&full_name) {
                    return Err(ApiError::MissingMethodImplementation(full_name));
                }
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Get a method binding by namespace and method name
    pub fn get_method(&self, namespace: &str, method: &str) -> Option<&Arc<MethodBinding>> {
        let full_name = format!("{}.{}", namespace, method);
        self.methods.get(&full_name)
    }

    /// Get all namespaces
    pub fn get_namespaces(&self) -> &HashMap<String, NamespaceDescriptor> {
        &self.namespaces
    }

    /// Get all methods
    pub fn get_methods(&self) -> &HashMap<String, Arc<MethodBinding>> {
        &self.methods
    }

    /// Get all types
    pub fn get_types(&self) -> &HashMap<String, TypeDescriptor> {
        &self.types
    }

    /// Check if the registry is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Errors that can occur during API registry operations
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Type error: {0}")]
    TypeError(#[from] TypeError),
    
    #[error("Method not found: {namespace}.{method}")]
    MethodNotFound { namespace: String, method: String },
    
    #[error("Invalid parameter count: expected {expected}, got {actual}")]
    InvalidParameterCount { expected: usize, actual: usize },
    
    #[error("Type mismatch at parameter {parameter_index}: expected {expected:?}, got {actual:?}")]
    TypeMismatch { 
        parameter_index: usize, 
        expected: TypeDescriptor, 
        actual: TypeDescriptor 
    },
    
    #[error("Namespace conflict: {0}")]
    NamespaceConflict(String),
    
    #[error("Registration after initialization")]
    RegistrationAfterInit,
    
    #[error("Parent namespace not found: {0}")]
    ParentNamespaceNotFound(String),
    
    #[error("Duplicate method: {0}")]
    DuplicateMethod(String),
    
    #[error("Missing method implementation: {0}")]
    MissingMethodImplementation(String),
    
    #[error("No entity context available")]
    NoEntityContext,
    
    #[error("V8 error: {0}")]
    V8Error(String),
    
    #[error("Engine error: {0}")]
    EngineError(String),
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}