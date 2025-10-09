//! Method binding system for registering Rust functions with the API registry

use super::type_system::{Value, TypeDescriptor, TypeError};
use super::ApiError;
use std::any::Any;

/// Context provided to method calls with engine state
pub struct MethodContext {
    pub entity_context: Option<u32>, // Entity ID
    pub world: Option<*mut u8>, // Raw pointer to World data - for future use
    pub script_id: u32,
}

impl MethodContext {
    pub fn new(script_id: u32) -> Self {
        Self {
            entity_context: None,
            world: None,
            script_id,
        }
    }

    pub fn with_entity(mut self, entity_id: u32) -> Self {
        self.entity_context = Some(entity_id);
        self
    }

    pub fn with_world(mut self, world_ptr: *mut u8) -> Self {
        self.world = Some(world_ptr);
        self
    }
}

/// Function type for method implementations
pub type MethodFunction = Box<dyn Fn(&MethodContext, &[Value]) -> Result<Value, ApiError> + Send + Sync>;

/// Binding that connects a namespace method to a Rust function
pub struct MethodBinding {
    pub namespace: String,
    pub method_name: String,
    pub rust_function: MethodFunction,
    pub parameter_types: Vec<TypeDescriptor>,
    pub return_type: TypeDescriptor,
    pub documentation: String,
    pub is_static: bool,
}

impl MethodBinding {
    pub fn new(
        namespace: String,
        method_name: String,
        rust_function: MethodFunction,
        parameter_types: Vec<TypeDescriptor>,
        return_type: TypeDescriptor,
    ) -> Self {
        Self {
            namespace,
            method_name,
            rust_function,
            parameter_types,
            return_type,
            documentation: String::new(),
            is_static: true,
        }
    }

    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = documentation;
        self
    }

    pub fn non_static(mut self) -> Self {
        self.is_static = false;
        self
    }

    /// Call the method with validation
    pub fn call(&self, context: &MethodContext, args: &[Value]) -> Result<Value, ApiError> {
        // Validate parameter count
        if args.len() != self.parameter_types.len() {
            return Err(ApiError::InvalidParameterCount {
                expected: self.parameter_types.len(),
                actual: args.len(),
            });
        }

        // Validate parameter types
        for (i, (arg, expected_type)) in args.iter().zip(&self.parameter_types).enumerate() {
            if !self.validate_type(arg, expected_type) {
                return Err(ApiError::TypeMismatch {
                    parameter_index: i,
                    expected: expected_type.clone(),
                    actual: arg.get_type(),
                });
            }
        }

        // Call the Rust function
        (self.rust_function)(context, args)
    }

    /// Validate that a value matches the expected type
    fn validate_type(&self, value: &Value, expected: &TypeDescriptor) -> bool {
        match (value, expected) {
            (Value::Number(_), TypeDescriptor::Number) => true,
            (Value::String(_), TypeDescriptor::String) => true,
            (Value::Boolean(_), TypeDescriptor::Boolean) => true,
            (Value::Object(obj), TypeDescriptor::Object(class_name)) => {
                obj.class_name() == class_name
            }
            (Value::Array(arr), TypeDescriptor::Array(inner_type)) => {
                arr.iter().all(|v| self.validate_type(v, inner_type))
            }
            (_, TypeDescriptor::Optional(inner_type)) => {
                value.is_null() || self.validate_type(value, inner_type)
            }
            (_, TypeDescriptor::Union(types)) => {
                types.iter().any(|t| self.validate_type(value, t))
            }
            (Value::Null, TypeDescriptor::Optional(_)) => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for MethodBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MethodBinding")
            .field("namespace", &self.namespace)
            .field("method_name", &self.method_name)
            .field("parameter_types", &self.parameter_types)
            .field("return_type", &self.return_type)
            .field("documentation", &self.documentation)
            .field("is_static", &self.is_static)
            .finish()
    }
}

/// Convenience macro for creating method bindings
#[macro_export]
macro_rules! api_method {
    (
        fn $method_name:ident($context:ident: &MethodContext $(, $param:ident: $param_type:ty)*) -> $return_type:ty {
            $($body:tt)*
        }
    ) => {
        {
            let method_fn: MethodFunction = Box::new(|$context: &MethodContext, args: &[Value]| -> Result<Value, ApiError> {
                // Extract parameters from args
                let mut arg_iter = args.iter();
                $(
                    let $param: $param_type = arg_iter.next()
                        .ok_or_else(|| ApiError::TypeError(TypeError::MissingParameter(stringify!($param).to_string())))?
                        .try_into()
                        .map_err(ApiError::TypeError)?;
                )*

                // Call the actual implementation
                let result: $return_type = {
                    $($body)*
                };

                // Convert result to Value
                Ok(result.into())
            });
            
            method_fn
        }
    };
}

/// Convenience function for creating simple method bindings
pub fn create_method_binding<F, R>(
    namespace: &str,
    method_name: &str,
    parameter_types: Vec<TypeDescriptor>,
    return_type: TypeDescriptor,
    implementation: F,
) -> MethodBinding
where
    F: Fn(&MethodContext, &[Value]) -> Result<R, ApiError> + Send + Sync + 'static,
    R: Into<Value>,
{
    let rust_function: MethodFunction = Box::new(move |context, args| {
        implementation(context, args).map(|r| r.into())
    });

    MethodBinding::new(
        namespace.to_string(),
        method_name.to_string(),
        rust_function,
        parameter_types,
        return_type,
    )
}

/// Builder for creating method bindings with a fluent interface
pub struct MethodBindingBuilder {
    namespace: String,
    method_name: String,
    parameter_types: Vec<TypeDescriptor>,
    return_type: TypeDescriptor,
    documentation: String,
    is_static: bool,
}

impl MethodBindingBuilder {
    pub fn new(namespace: &str, method_name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            method_name: method_name.to_string(),
            parameter_types: Vec::new(),
            return_type: TypeDescriptor::Void,
            documentation: String::new(),
            is_static: true,
        }
    }

    pub fn with_parameter(mut self, param_type: TypeDescriptor) -> Self {
        self.parameter_types.push(param_type);
        self
    }

    pub fn with_return_type(mut self, return_type: TypeDescriptor) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn with_documentation(mut self, documentation: &str) -> Self {
        self.documentation = documentation.to_string();
        self
    }

    pub fn non_static(mut self) -> Self {
        self.is_static = false;
        self
    }

    pub fn build<F, R>(self, implementation: F) -> MethodBinding
    where
        F: Fn(&MethodContext, &[Value]) -> Result<R, ApiError> + Send + Sync + 'static,
        R: Into<Value>,
    {
        let rust_function: MethodFunction = Box::new(move |context, args| {
            implementation(context, args).map(|r| r.into())
        });

        MethodBinding {
            namespace: self.namespace,
            method_name: self.method_name,
            rust_function,
            parameter_types: self.parameter_types,
            return_type: self.return_type,
            documentation: self.documentation,
            is_static: self.is_static,
        }
    }
}