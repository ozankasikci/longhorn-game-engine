# TypeScript API Implementation Guide
## Practical Implementation Steps for Longhorn Game Engine

### Overview

This guide provides concrete implementation steps for building the TypeScript API system outlined in the architecture plan. It includes code examples, file structures, and step-by-step instructions.

## 1. Project Structure

```
crates/implementation/engine-scripting/
├── src/
│   ├── api/
│   │   ├── mod.rs                 # API module root
│   │   ├── registry/
│   │   │   ├── mod.rs             # Registry system
│   │   │   ├── namespace.rs       # Namespace management
│   │   │   ├── method_binding.rs  # Method registration
│   │   │   ├── type_system.rs     # Type descriptors
│   │   │   └── macros.rs          # Registration macros
│   │   ├── bridge/
│   │   │   ├── mod.rs             # V8 bridge
│   │   │   ├── v8_bindings.rs     # V8 integration
│   │   │   ├── type_conversion.rs # Rust ↔ V8 conversion
│   │   │   └── error_handling.rs  # Error management
│   │   ├── namespaces/
│   │   │   ├── mod.rs             # Namespace implementations
│   │   │   ├── engine_world.rs    # Engine.World
│   │   │   ├── engine_math.rs     # Engine.Math
│   │   │   ├── engine_physics.rs  # Engine.Physics
│   │   │   ├── engine_input.rs    # Engine.Input
│   │   │   └── engine_debug.rs    # Engine.Debug
│   │   └── codegen/
│   │       ├── mod.rs             # Code generation
│   │       ├── typescript_gen.rs  # .d.ts generation
│   │       └── docs_gen.rs        # Documentation generation
│   └── typescript_api_system.rs   # Main API system
├── typescript/
│   ├── engine.d.ts                # Generated type definitions
│   └── examples/                  # Example scripts
└── tests/
    ├── api_tests.rs               # API functionality tests
    └── integration_tests.rs       # Full integration tests
```

## 2. Core Registry Implementation

### 2.1 Registry Data Structures

```rust
// src/api/registry/mod.rs
use std::collections::HashMap;
use std::sync::Arc;

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

    pub fn register_method(&mut self, binding: MethodBinding) -> Result<(), ApiError> {
        let full_name = format!("{}.{}", binding.namespace, binding.method_name);
        
        if self.methods.contains_key(&full_name) {
            return Err(ApiError::DuplicateMethod(full_name));
        }

        self.methods.insert(full_name, Arc::new(binding));
        Ok(())
    }

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

    pub fn get_method(&self, namespace: &str, method: &str) -> Option<&Arc<MethodBinding>> {
        let full_name = format!("{}.{}", namespace, method);
        self.methods.get(&full_name)
    }
}
```

### 2.2 Namespace Descriptor

```rust
// src/api/registry/namespace.rs
#[derive(Debug, Clone)]
pub struct NamespaceDescriptor {
    pub name: String,
    pub parent: Option<String>,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub classes: Vec<ClassDescriptor>,
    pub child_namespaces: Vec<String>,
    pub documentation: String,
}

#[derive(Debug, Clone)]
pub struct ClassDescriptor {
    pub name: String,
    pub methods: Vec<String>,
    pub properties: Vec<PropertyDescriptor>,
    pub constructor: Option<String>,
    pub documentation: String,
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: String,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub readonly: bool,
    pub property_type: TypeDescriptor,
    pub documentation: String,
}
```

### 2.3 Method Binding System

```rust
// src/api/registry/method_binding.rs
use std::any::Any;

pub struct MethodBinding {
    pub namespace: String,
    pub method_name: String,
    pub rust_function: MethodFunction,
    pub parameter_types: Vec<TypeDescriptor>,
    pub return_type: TypeDescriptor,
    pub documentation: String,
    pub is_static: bool,
}

pub type MethodFunction = Box<dyn Fn(&MethodContext, &[Value]) -> Result<Value, ApiError> + Send + Sync>;

pub struct MethodContext {
    pub entity_context: Option<Entity>,
    pub world: *mut World, // Careful with lifetime management
    pub script_id: u32,
}

// Safe wrapper for method calls
impl MethodBinding {
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
            _ => false,
        }
    }
}
```

## 3. Registration Macros

### 3.1 Namespace Registration Macro

```rust
// src/api/registry/macros.rs

#[macro_export]
macro_rules! api_namespace {
    ($name:expr => {
        methods: [$($method_name:expr => $method_fn:expr),* $(,)?],
        $(classes: [$($class_name:expr => $class_def:expr),* $(,)?],)?
        $(properties: [$($prop_name:expr => $prop_def:expr),* $(,)?],)?
        $(documentation: $doc:expr,)?
    }) => {
        {
            let mut descriptor = NamespaceDescriptor {
                name: $name.to_string(),
                parent: None, // TODO: Extract from name
                methods: vec![$($method_name.to_string()),*],
                properties: vec![], // TODO: Handle properties
                classes: vec![], // TODO: Handle classes
                child_namespaces: vec![],
                documentation: stringify!($($doc)?).to_string(),
            };

            // Register methods
            $(
                let method_binding = MethodBinding {
                    namespace: $name.to_string(),
                    method_name: $method_name.to_string(),
                    rust_function: Box::new($method_fn),
                    parameter_types: vec![], // TODO: Extract from function signature
                    return_type: TypeDescriptor::Void, // TODO: Extract from function signature
                    documentation: String::new(),
                    is_static: true,
                };
                registry.register_method(method_binding)?;
            )*

            registry.register_namespace(descriptor)?;
        }
    };
}

// Usage example:
api_namespace! {
    "Engine.World" => {
        methods: [
            "getCurrentEntity" => get_current_entity,
            "createEntity" => create_entity,
            "destroyEntity" => destroy_entity,
        ],
        classes: [
            "Entity" => EntityClass,
        ],
        documentation: "Core world and entity management functionality",
    }
}
```

### 3.2 Method Implementation Macro

```rust
#[macro_export]
macro_rules! api_method {
    (
        fn $method_name:ident($context:ident: &MethodContext $(, $param:ident: $param_type:ty)*) -> $return_type:ty {
            $($body:tt)*
        }
    ) => {
        fn $method_name($context: &MethodContext, args: &[Value]) -> Result<Value, ApiError> {
            // Extract parameters from args
            let mut arg_iter = args.iter();
            $(
                let $param: $param_type = arg_iter.next()
                    .ok_or(ApiError::MissingParameter(stringify!($param).to_string()))?
                    .try_into()?;
            )*

            // Call the actual implementation
            let result: $return_type = {
                $($body)*
            };

            // Convert result to Value
            Ok(result.into())
        }
    };
}

// Usage example:
api_method! {
    fn get_current_entity(context: &MethodContext) -> EntityHandle {
        match context.entity_context {
            Some(entity) => EntityHandle::new(entity),
            None => return Err(ApiError::NoEntityContext),
        }
    }
}
```

## 4. V8 Bridge Implementation

### 4.1 V8 Integration Layer

```rust
// src/api/bridge/v8_bindings.rs
use v8;
use crate::api::registry::ApiRegistry;

pub struct V8ApiBridge {
    isolate: v8::OwnedIsolate,
    registry: Arc<ApiRegistry>,
    global_context: v8::Global<v8::Context>,
}

impl V8ApiBridge {
    pub fn new(registry: Arc<ApiRegistry>) -> Result<Self, ApiError> {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let isolate = v8::Isolate::new(Default::default());
        let mut bridge = Self {
            isolate,
            registry,
            global_context: v8::Global::empty(&mut isolate),
        };

        bridge.initialize_context()?;
        Ok(bridge)
    }

    fn initialize_context(&mut self) -> Result<(), ApiError> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        // Create global objects for all namespaces
        for (namespace_name, _) in &self.registry.namespaces {
            self.create_namespace_object(scope, namespace_name)?;
        }

        self.global_context.set(scope, context);
        Ok(())
    }

    fn create_namespace_object(&self, scope: &mut v8::ContextScope, namespace: &str) -> Result<(), ApiError> {
        let parts: Vec<&str> = namespace.split('.').collect();
        let global = scope.get_current_context().global(scope);
        
        let mut current_obj = global;
        
        for (i, part) in parts.iter().enumerate() {
            let name = v8::String::new(scope, part)
                .ok_or_else(|| ApiError::V8Error("Failed to create string".to_string()))?;

            let is_leaf = i == parts.len() - 1;
            
            if let Some(existing) = current_obj.get(scope, name.into()) {
                if existing.is_object() {
                    current_obj = existing.to_object(scope)
                        .ok_or_else(|| ApiError::V8Error("Failed to convert to object".to_string()))?;
                } else {
                    return Err(ApiError::NamespaceConflict(namespace.to_string()));
                }
            } else {
                let new_obj = if is_leaf {
                    self.create_namespace_methods(scope, namespace)?
                } else {
                    v8::Object::new(scope)
                };
                
                current_obj.set(scope, name.into(), new_obj.into());
                current_obj = new_obj;
            }
        }

        Ok(())
    }

    fn create_namespace_methods(&self, scope: &mut v8::ContextScope, namespace: &str) -> Result<v8::Local<v8::Object>, ApiError> {
        let obj = v8::Object::new(scope);
        
        if let Some(ns_descriptor) = self.registry.namespaces.get(namespace) {
            for method_name in &ns_descriptor.methods {
                let method_binding = self.registry.get_method(namespace, method_name)
                    .ok_or_else(|| ApiError::MethodNotFound {
                        namespace: namespace.to_string(),
                        method: method_name.to_string(),
                    })?;

                let method_fn = self.create_v8_method(scope, method_binding.clone())?;
                let name = v8::String::new(scope, method_name)
                    .ok_or_else(|| ApiError::V8Error("Failed to create method name".to_string()))?;
                
                obj.set(scope, name.into(), method_fn.into());
            }
        }

        Ok(obj)
    }

    fn create_v8_method(&self, scope: &mut v8::ContextScope, binding: Arc<MethodBinding>) -> Result<v8::Local<v8::Function>, ApiError> {
        let registry = self.registry.clone();
        
        let function = v8::Function::new(scope, move |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Convert V8 arguments to our Value type
            let mut converted_args = Vec::new();
            for i in 0..args.length() {
                let arg = args.get(i);
                match Value::from_v8(scope, arg) {
                    Ok(value) => converted_args.push(value),
                    Err(e) => {
                        let error_msg = format!("Argument conversion error: {}", e);
                        let error = v8::String::new(scope, &error_msg).unwrap();
                        scope.throw_exception(error.into());
                        return;
                    }
                }
            }

            // Create method context
            let context = MethodContext {
                entity_context: None, // TODO: Extract from execution context
                world: std::ptr::null_mut(), // TODO: Get from context
                script_id: 0, // TODO: Extract from context
            };

            // Call the method
            match binding.call(&context, &converted_args) {
                Ok(result) => {
                    let v8_result = result.to_v8(scope);
                    rv.set(v8_result);
                }
                Err(e) => {
                    let error_msg = format!("Method error: {}", e);
                    let error = v8::String::new(scope, &error_msg).unwrap();
                    scope.throw_exception(error.into());
                }
            }
        })
        .ok_or_else(|| ApiError::V8Error("Failed to create function".to_string()))?;

        Ok(function)
    }
}
```

## 5. Type System Implementation

### 5.1 Value Type System

```rust
// src/api/registry/type_system.rs

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

#[derive(Debug, Clone)]
pub struct ObjectValue {
    class_name: String,
    properties: HashMap<String, Value>,
    rust_handle: Option<Box<dyn Any + Send + Sync>>,
}

impl ObjectValue {
    pub fn class_name(&self) -> &str {
        &self.class_name
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
}

// Conversion traits
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

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
```

### 5.2 V8 Conversion Layer

```rust
// src/api/bridge/type_conversion.rs

impl Value {
    pub fn from_v8(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<Self, TypeError> {
        if value.is_number() {
            Ok(Value::Number(value.number_value(scope).unwrap()))
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
                let element = array.get_index(scope, i)
                    .ok_or(TypeError::ConversionError)?;
                result.push(Self::from_v8(scope, element)?);
            }
            
            Ok(Value::Array(result))
        } else if value.is_object() {
            // Handle custom objects
            let obj = value.to_object(scope)
                .ok_or(TypeError::ConversionError)?;
            
            // Extract class name from constructor or prototype
            let class_name = self.extract_class_name(scope, obj)?;
            
            Ok(Value::Object(ObjectValue {
                class_name,
                properties: HashMap::new(), // TODO: Extract properties
                rust_handle: None,
            }))
        } else {
            Err(TypeError::UnsupportedType)
        }
    }

    pub fn to_v8(&self, scope: &mut v8::HandleScope) -> v8::Local<v8::Value> {
        match self {
            Value::Void => v8::undefined(scope).into(),
            Value::Number(n) => v8::Number::new(scope, *n).into(),
            Value::String(s) => v8::String::new(scope, s).unwrap().into(),
            Value::Boolean(b) => v8::Boolean::new(scope, *b).into(),
            Value::Null => v8::null(scope).into(),
            Value::Array(arr) => {
                let array = v8::Array::new(scope, arr.len() as i32);
                for (i, item) in arr.iter().enumerate() {
                    let v8_item = item.to_v8(scope);
                    array.set_index(scope, i as u32, v8_item);
                }
                array.into()
            }
            Value::Object(obj) => {
                // Create V8 object with properties
                let v8_obj = v8::Object::new(scope);
                for (key, value) in &obj.properties {
                    let v8_key = v8::String::new(scope, key).unwrap();
                    let v8_value = value.to_v8(scope);
                    v8_obj.set(scope, v8_key.into(), v8_value);
                }
                v8_obj.into()
            }
        }
    }
}
```

## 6. Example Namespace Implementation

### 6.1 Engine.World Namespace

```rust
// src/api/namespaces/engine_world.rs

use crate::api::registry::*;
use engine_ecs_core::{Entity, World};

pub fn register_engine_world(registry: &mut ApiRegistry) -> Result<(), ApiError> {
    api_namespace! {
        "Engine.World" => {
            methods: [
                "getCurrentEntity" => get_current_entity,
                "createEntity" => create_entity,
                "destroyEntity" => destroy_entity,
                "findEntitiesByTag" => find_entities_by_tag,
            ],
            classes: [
                "Entity" => EntityClass,
            ],
            documentation: "Core world and entity management functionality",
        }
    }

    Ok(())
}

// Method implementations
api_method! {
    fn get_current_entity(context: &MethodContext) -> EntityHandle {
        match context.entity_context {
            Some(entity) => EntityHandle::new(entity),
            None => return Err(ApiError::NoEntityContext),
        }
    }
}

api_method! {
    fn create_entity(context: &MethodContext, name: String) -> EntityHandle {
        unsafe {
            let world = &mut *context.world;
            let entity = world.create_entity();
            // TODO: Set entity name component
            EntityHandle::new(entity)
        }
    }
}

api_method! {
    fn destroy_entity(context: &MethodContext, entity: EntityHandle) {
        unsafe {
            let world = &mut *context.world;
            world.destroy_entity(entity.entity);
        }
    }
}

// Entity wrapper for scripts
#[derive(Debug, Clone)]
pub struct EntityHandle {
    pub entity: Entity,
}

impl EntityHandle {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn get_component<T: Component>(&self, world: &World) -> Option<&T> {
        world.get_component::<T>(self.entity)
    }
}

// Convert EntityHandle to/from Value
impl From<EntityHandle> for Value {
    fn from(handle: EntityHandle) -> Self {
        Value::Object(ObjectValue {
            class_name: "Entity".to_string(),
            properties: HashMap::new(),
            rust_handle: Some(Box::new(handle)),
        })
    }
}

impl TryFrom<&Value> for EntityHandle {
    type Error = TypeError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(obj) if obj.class_name == "Entity" => {
                obj.rust_handle
                    .as_ref()
                    .and_then(|h| h.downcast_ref::<EntityHandle>())
                    .map(|h| h.clone())
                    .ok_or(TypeError::InvalidObject)
            }
            _ => Err(TypeError::ExpectedObject("Entity".to_string())),
        }
    }
}
```

## 7. TypeScript Definition Generation

### 7.1 Code Generator

```rust
// src/api/codegen/typescript_gen.rs

pub struct TypeScriptGenerator {
    registry: Arc<ApiRegistry>,
}

impl TypeScriptGenerator {
    pub fn new(registry: Arc<ApiRegistry>) -> Self {
        Self { registry }
    }

    pub fn generate_definitions(&self) -> String {
        let mut output = String::new();
        
        output.push_str("// Auto-generated TypeScript definitions for Longhorn Engine API\n");
        output.push_str("// Do not edit this file manually\n\n");

        // Generate namespace declarations
        for (namespace_name, descriptor) in &self.registry.namespaces {
            output.push_str(&self.generate_namespace(namespace_name, descriptor));
            output.push('\n');
        }

        output
    }

    fn generate_namespace(&self, name: &str, descriptor: &NamespaceDescriptor) -> String {
        let mut output = String::new();
        
        if !descriptor.documentation.is_empty() {
            output.push_str(&format!("/**\n * {}\n */\n", descriptor.documentation));
        }

        output.push_str(&format!("declare namespace {} {{\n", name));

        // Generate methods
        for method_name in &descriptor.methods {
            if let Some(method_binding) = self.registry.get_method(name, method_name) {
                output.push_str(&self.generate_method_signature(method_binding));
            }
        }

        // Generate classes
        for class in &descriptor.classes {
            output.push_str(&self.generate_class(class));
        }

        output.push_str("}\n");
        output
    }

    fn generate_method_signature(&self, binding: &MethodBinding) -> String {
        let params = binding.parameter_types
            .iter()
            .enumerate()
            .map(|(i, param_type)| format!("param{}: {}", i, self.type_to_typescript(param_type)))
            .collect::<Vec<_>>()
            .join(", ");

        let return_type = self.type_to_typescript(&binding.return_type);

        format!("    export function {}({}): {};\n", 
                binding.method_name, params, return_type)
    }

    fn generate_class(&self, class: &ClassDescriptor) -> String {
        let mut output = String::new();
        
        if !class.documentation.is_empty() {
            output.push_str(&format!("    /**\n     * {}\n     */\n", class.documentation));
        }

        output.push_str(&format!("    export class {} {{\n", class.name));

        // Generate properties
        for property in &class.properties {
            let prop_type = self.type_to_typescript(&property.property_type);
            let readonly = if property.readonly { "readonly " } else { "" };
            output.push_str(&format!("        {}{}: {};\n", readonly, property.name, prop_type));
        }

        // Generate methods
        for method_name in &class.methods {
            // TODO: Generate method signatures for class methods
        }

        output.push_str("    }\n");
        output
    }

    fn type_to_typescript(&self, type_desc: &TypeDescriptor) -> String {
        match type_desc {
            TypeDescriptor::Void => "void".to_string(),
            TypeDescriptor::Number => "number".to_string(),
            TypeDescriptor::String => "string".to_string(),
            TypeDescriptor::Boolean => "boolean".to_string(),
            TypeDescriptor::Object(class_name) => class_name.clone(),
            TypeDescriptor::Array(inner) => format!("{}[]", self.type_to_typescript(inner)),
            TypeDescriptor::Optional(inner) => format!("{} | null", self.type_to_typescript(inner)),
            TypeDescriptor::Union(types) => {
                types.iter()
                    .map(|t| self.type_to_typescript(t))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        }
    }
}
```

## 8. Integration with Existing System

### 8.1 Main API System

```rust
// src/typescript_api_system.rs

use crate::api::registry::ApiRegistry;
use crate::api::bridge::V8ApiBridge;
use crate::api::namespaces::*;

pub struct TypeScriptApiSystem {
    registry: Arc<ApiRegistry>,
    bridge: V8ApiBridge,
}

impl TypeScriptApiSystem {
    pub fn new() -> Result<Self, ApiError> {
        let mut registry = ApiRegistry::new();

        // Register core namespaces
        engine_world::register_engine_world(&mut registry)?;
        engine_math::register_engine_math(&mut registry)?;
        engine_physics::register_engine_physics(&mut registry)?;
        engine_input::register_engine_input(&mut registry)?;
        engine_debug::register_engine_debug(&mut registry)?;

        // Finalize registry
        registry.finalize()?;

        let registry = Arc::new(registry);
        let bridge = V8ApiBridge::new(registry.clone())?;

        Ok(Self { registry, bridge })
    }

    pub fn execute_script(&mut self, script_source: &str, entity_context: Option<Entity>) -> Result<(), ApiError> {
        self.bridge.execute_script(script_source, entity_context)
    }

    pub fn generate_type_definitions(&self) -> String {
        let generator = TypeScriptGenerator::new(self.registry.clone());
        generator.generate_definitions()
    }
}
```

### 8.2 Integration Tests

```rust
// tests/api_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_world_api() {
        let mut api_system = TypeScriptApiSystem::new().unwrap();
        
        let script = r#"
            const entity = Engine.World.getCurrentEntity();
            console.log("Entity ID:", entity.id());
            
            const newEntity = Engine.World.createEntity("TestEntity");
            Engine.World.destroyEntity(newEntity);
        "#;

        api_system.execute_script(script, Some(Entity::new(42))).unwrap();
    }

    #[test]
    fn test_typescript_generation() {
        let api_system = TypeScriptApiSystem::new().unwrap();
        let definitions = api_system.generate_type_definitions();
        
        assert!(definitions.contains("declare namespace Engine.World"));
        assert!(definitions.contains("function getCurrentEntity"));
        assert!(definitions.contains("class Entity"));
    }
}
```

## 9. Next Steps

1. **Implement Core Registry**: Start with the registry system and basic V8 integration
2. **Add Engine.World**: Implement the first namespace as a proof of concept
3. **Create Type System**: Build the Value type and conversion system
4. **Generate TypeScript Definitions**: Implement automatic .d.ts generation
5. **Add More Namespaces**: Expand to Physics, Math, Input, etc.
6. **Performance Optimization**: Optimize method calls and memory usage
7. **Documentation**: Generate comprehensive API documentation
8. **Testing**: Build comprehensive test suite

This implementation provides a solid foundation for a Unity-style TypeScript API system while maintaining flexibility for future extensions and optimizations.