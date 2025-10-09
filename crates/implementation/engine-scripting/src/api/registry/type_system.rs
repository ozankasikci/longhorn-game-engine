//! Type system for API registry with V8 conversion support

use std::collections::HashMap;
use std::any::Any;

/// Core value type that bridges Rust and V8 JavaScript
#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectValue),
    Array(Vec<Value>),
    Null,
}

/// Object value that wraps Rust data for JavaScript access
#[derive(Debug)]
pub struct ObjectValue {
    class_name: String,
    pub properties: HashMap<String, Value>,  // Made public for V8 conversion
    rust_handle: Option<Box<dyn Any + Send + Sync>>,
    /// Special marker for objects that need V8 method binding
    needs_v8_methods: bool,
}

impl Clone for ObjectValue {
    fn clone(&self) -> Self {
        Self {
            class_name: self.class_name.clone(),
            properties: self.properties.clone(),
            // Cannot clone Box<dyn Any>, so we set to None
            rust_handle: None,
            needs_v8_methods: self.needs_v8_methods,
        }
    }
}

impl ObjectValue {
    pub fn new(class_name: String) -> Self {
        Self {
            class_name,
            properties: HashMap::new(),
            rust_handle: None,
            needs_v8_methods: false,
        }
    }

    /// Create an object that needs V8 method binding (like Entity)
    pub fn new_with_methods(class_name: String) -> Self {
        Self {
            class_name,
            properties: HashMap::new(),
            rust_handle: None,
            needs_v8_methods: true,
        }
    }

    pub fn with_rust_handle<T: Any + Send + Sync>(mut self, handle: Box<T>) -> Self {
        self.rust_handle = Some(handle);
        self
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn get_property(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    pub fn set_property(&mut self, name: String, value: Value) {
        self.properties.insert(name, value);
    }

    pub fn get_rust_handle<T: 'static>(&self) -> Option<&T> {
        self.rust_handle
            .as_ref()
            .and_then(|h| h.downcast_ref::<T>())
    }

    pub fn needs_v8_methods(&self) -> bool {
        self.needs_v8_methods
    }
}

/// Type descriptors for runtime validation and TypeScript generation
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDescriptor {
    Void,
    Number,
    String,
    Boolean,
    Object(String),  // Class name
    Array(Box<TypeDescriptor>),
    Optional(Box<TypeDescriptor>),
    Union(Vec<TypeDescriptor>),
}

impl TypeDescriptor {
    pub fn to_typescript(&self) -> String {
        match self {
            TypeDescriptor::Void => "void".to_string(),
            TypeDescriptor::Number => "number".to_string(),
            TypeDescriptor::String => "string".to_string(),
            TypeDescriptor::Boolean => "boolean".to_string(),
            TypeDescriptor::Object(class_name) => class_name.clone(),
            TypeDescriptor::Array(inner) => format!("{}[]", inner.to_typescript()),
            TypeDescriptor::Optional(inner) => format!("{} | null", inner.to_typescript()),
            TypeDescriptor::Union(types) => {
                types.iter()
                    .map(|t| t.to_typescript())
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        }
    }
}

impl Value {
    pub fn get_type(&self) -> TypeDescriptor {
        match self {
            Value::Void => TypeDescriptor::Void,
            Value::Number(_) => TypeDescriptor::Number,
            Value::String(_) => TypeDescriptor::String,
            Value::Boolean(_) => TypeDescriptor::Boolean,
            Value::Object(obj) => TypeDescriptor::Object(obj.class_name.clone()),
            Value::Array(arr) => {
                let inner_type = arr.first()
                    .map(|v| v.get_type())
                    .unwrap_or(TypeDescriptor::Void);
                TypeDescriptor::Array(Box::new(inner_type))
            }
            Value::Null => TypeDescriptor::Optional(Box::new(TypeDescriptor::Void)),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&ObjectValue> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut ObjectValue> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

// Conversion traits for common Rust types
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(values: Vec<T>) -> Self {
        Value::Array(values.into_iter().map(|v| v.into()).collect())
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}

// Try conversion traits for extracting Rust types from Values
impl TryFrom<&Value> for f64 {
    type Error = TypeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(*n),
            _ => Err(TypeError::ExpectedNumber),
        }
    }
}

impl TryFrom<&Value> for String {
    type Error = TypeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s.clone()),
            _ => Err(TypeError::ExpectedString),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = TypeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(*b),
            _ => Err(TypeError::ExpectedBoolean),
        }
    }
}

/// Type conversion errors
#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("Expected number")]
    ExpectedNumber,
    
    #[error("Expected string")]
    ExpectedString,
    
    #[error("Expected boolean")]
    ExpectedBoolean,
    
    #[error("Expected object of type {0}")]
    ExpectedObject(String),
    
    #[error("Expected array")]
    ExpectedArray,
    
    #[error("Invalid object")]
    InvalidObject,
    
    #[error("Conversion error")]
    ConversionError,
    
    #[error("Unsupported type")]
    UnsupportedType,
    
    #[error("Missing parameter {0}")]
    MissingParameter(String),
}