# TypeScript Scripting API Architecture Plan
## Longhorn Game Engine - Unity-Style API System

### Executive Summary

This document outlines a comprehensive plan for creating a robust, extensible TypeScript scripting API system for the Longhorn Game Engine. The goal is to build a Unity-style namespace system that provides clean, maintainable access to engine functionality through TypeScript scripts.

## 1. Architecture Overview

### 1.1 Core Design Principles

1. **Namespace-Based Organization**: Like Unity's `UnityEngine.*` namespaces
2. **Registry Pattern**: Dynamic API registration system for extensibility
3. **Type Safety**: Full TypeScript type definitions with compile-time checking
4. **Runtime Safety**: Rust-based validation and error handling
5. **Hot Reload Support**: Live script reloading during development
6. **Performance**: Minimal overhead for frequent operations

### 1.2 High-Level Architecture

```
┌─────────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│   TypeScript        │    │   API Bridge         │    │   Rust Engine       │
│   Scripts           │◄──►│   (V8 + Registry)    │◄──►│   Core Systems      │
│                     │    │                      │    │                     │
│ • Entity Scripts    │    │ • Namespace Manager  │    │ • ECS World         │
│ • Component Logic   │    │ • Type Validation    │    │ • Physics Engine    │
│ • Game Logic        │    │ • Method Dispatch    │    │ • Renderer          │
│ • UI Controllers    │    │ • Error Handling     │    │ • Audio System      │
└─────────────────────┘    └──────────────────────┘    └─────────────────────┘
```

## 2. Namespace System Design

### 2.1 Core Namespaces (Unity-Inspired)

```typescript
// Core engine functionality
Engine.World.*          // Entity/Component management
Engine.Physics.*        // Physics system access
Engine.Rendering.*      // Rendering and graphics
Engine.Audio.*          // Audio system
Engine.Input.*          // Input handling
Engine.Time.*           // Time management
Engine.Math.*           // Mathematical utilities
Engine.Debug.*          // Debugging tools

// Editor-specific (development only)
EngineEditor.*          // Editor scripting API
EngineEditor.Inspector.*  // Custom inspector scripts
EngineEditor.Tools.*    // Custom editor tools
```

### 2.2 Example API Structure

```typescript
// Engine.World namespace
namespace Engine.World {
    export function getCurrentEntity(): Entity;
    export function getEntity(id: number): Entity | null;
    export function createEntity(name?: string): Entity;
    export function destroyEntity(entity: Entity): void;
    export function findEntitiesByTag(tag: string): Entity[];
    
    export class Entity {
        readonly id: number;
        readonly name: string;
        
        getComponent<T extends Component>(type: ComponentType<T>): T | null;
        addComponent<T extends Component>(type: ComponentType<T>, data?: Partial<T>): T;
        removeComponent<T extends Component>(type: ComponentType<T>): boolean;
        hasComponent<T extends Component>(type: ComponentType<T>): boolean;
    }
}
```

## 3. API Registry System

### 3.1 Registry Architecture

```rust
// Core registry system in Rust
pub struct ApiRegistry {
    namespaces: HashMap<String, NamespaceDescriptor>,
    methods: HashMap<String, MethodBinding>,
    types: HashMap<String, TypeDescriptor>,
}

pub struct NamespaceDescriptor {
    name: String,
    parent: Option<String>,
    methods: Vec<String>,
    properties: Vec<String>,
    child_namespaces: Vec<String>,
}

pub struct MethodBinding {
    namespace: String,
    method_name: String,
    rust_function: Box<dyn Fn(&[Value]) -> Result<Value, ApiError>>,
    parameter_types: Vec<TypeDescriptor>,
    return_type: TypeDescriptor,
    documentation: String,
}
```

### 3.2 Registration Macros

```rust
// Convenient macros for API registration
api_namespace! {
    "Engine.World" => {
        methods: [
            "getCurrentEntity" => get_current_entity,
            "createEntity" => create_entity,
            "destroyEntity" => destroy_entity,
        ],
        classes: [
            "Entity" => EntityClass {
                methods: [
                    "getComponent" => entity_get_component,
                    "addComponent" => entity_add_component,
                ],
                properties: [
                    "id" => get_entity_id,
                    "name" => (get_entity_name, set_entity_name),
                ]
            }
        ]
    }
}
```

## 4. Implementation Strategy

### 4.1 Phase 1: Core Infrastructure (Week 1-2)

1. **API Registry Core**
   - Implement `ApiRegistry` struct
   - Create namespace management system
   - Build method registration macros
   - Implement V8 binding generator

2. **Type System Foundation**
   - Define core TypeScript types
   - Create Rust ↔ V8 type conversion system
   - Implement validation framework
   - Generate TypeScript definition files

### 4.2 Phase 2: Core Namespaces (Week 3-4)

1. **Engine.World Implementation**
   - Entity management API
   - Component system bindings
   - Query and search functionality

2. **Engine.Math Implementation**
   - Vector3, Vector2, Quaternion
   - Matrix operations
   - Utility functions

3. **Engine.Time Implementation**
   - Delta time access
   - Time scale control
   - Timer utilities

### 4.3 Phase 3: Advanced Systems (Week 5-6)

1. **Engine.Physics**
   - Rigidbody control
   - Collision detection
   - Force application

2. **Engine.Input**
   - Keyboard/mouse input
   - Event system
   - Input mapping

3. **Engine.Debug**
   - Console logging
   - Visual debugging
   - Performance profiling

### 4.4 Phase 4: Editor Integration (Week 7-8)

1. **EngineEditor namespace**
   - Inspector customization
   - Custom tools API
   - Asset pipeline integration

2. **Development Tools**
   - Hot reload system
   - Script validation
   - Error reporting

## 5. Technical Implementation Details

### 5.1 V8 Integration Layer

```rust
pub struct V8ApiBridge {
    isolate: v8::OwnedIsolate,
    registry: ApiRegistry,
    context_data: ContextData,
}

impl V8ApiBridge {
    pub fn register_namespace(&mut self, namespace: &str) -> Result<(), ApiError> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Create namespace object hierarchy
        self.create_namespace_hierarchy(scope, namespace)?;
        
        // Register methods and properties
        self.register_namespace_methods(scope, namespace)?;
        
        Ok(())
    }
    
    fn create_namespace_hierarchy(&mut self, scope: &mut v8::ContextScope, namespace: &str) -> Result<(), ApiError> {
        let parts: Vec<&str> = namespace.split('.').collect();
        let global = scope.get_current_context().global(scope);
        
        let mut current_obj = global;
        for part in parts {
            let name = v8::String::new(scope, part).unwrap();
            
            if let Some(existing) = current_obj.get(scope, name.into()) {
                if existing.is_object() {
                    current_obj = existing.to_object(scope).unwrap();
                } else {
                    return Err(ApiError::NamespaceConflict(namespace.to_string()));
                }
            } else {
                let new_obj = v8::Object::new(scope);
                current_obj.set(scope, name.into(), new_obj.into());
                current_obj = new_obj;
            }
        }
        
        Ok(())
    }
}
```

### 5.2 Type Safety System

```rust
// Type descriptor for runtime validation
#[derive(Debug, Clone)]
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

// Type conversion traits
pub trait FromV8: Sized {
    fn from_v8(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<Self, TypeError>;
}

pub trait ToV8 {
    fn to_v8(&self, scope: &mut v8::HandleScope) -> v8::Local<v8::Value>;
}

// Automatic implementations for common types
impl FromV8 for f64 {
    fn from_v8(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<Self, TypeError> {
        value.number_value(scope).ok_or(TypeError::ExpectedNumber)
    }
}

impl ToV8 for f64 {
    fn to_v8(&self, scope: &mut v8::HandleScope) -> v8::Local<v8::Value> {
        v8::Number::new(scope, *self).into()
    }
}
```

### 5.3 Error Handling System

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Type error: {0}")]
    TypeError(#[from] TypeError),
    
    #[error("Method not found: {namespace}.{method}")]
    MethodNotFound { namespace: String, method: String },
    
    #[error("Invalid parameter count: expected {expected}, got {actual}")]
    InvalidParameterCount { expected: usize, actual: usize },
    
    #[error("Namespace conflict: {0}")]
    NamespaceConflict(String),
    
    #[error("Engine error: {0}")]
    EngineError(String),
}
```

## 6. Code Generation System

### 6.1 TypeScript Definition Generation

```rust
// Automatic TypeScript definition generation
pub struct TypeScriptGenerator {
    registry: &ApiRegistry,
}

impl TypeScriptGenerator {
    pub fn generate_definitions(&self) -> String {
        let mut output = String::new();
        
        // Generate namespace declarations
        for (namespace, descriptor) in &self.registry.namespaces {
            output.push_str(&self.generate_namespace(namespace, descriptor));
        }
        
        output
    }
    
    fn generate_namespace(&self, name: &str, descriptor: &NamespaceDescriptor) -> String {
        format!(
            "declare namespace {} {{\n{}\n}}\n",
            name,
            self.generate_namespace_content(descriptor)
        )
    }
}
```

### 6.2 Documentation Generation

```rust
// Automatic documentation generation from code
pub struct DocumentationGenerator {
    registry: &ApiRegistry,
}

impl DocumentationGenerator {
    pub fn generate_markdown(&self) -> String {
        // Generate comprehensive API documentation
        // Similar to Unity's scripting reference
    }
}
```

## 7. Performance Considerations

### 7.1 Optimization Strategies

1. **Method Caching**: Cache frequently used method lookups
2. **Object Pooling**: Reuse V8 objects for common types
3. **Lazy Loading**: Load namespaces on first access
4. **Native Optimization**: Use V8's fast property access for hot paths

### 7.2 Memory Management

1. **Weak References**: Avoid circular references between V8 and Rust
2. **Garbage Collection**: Proper cleanup of V8 handles
3. **Object Lifetime**: Clear separation between engine and script object lifetimes

## 8. Development Workflow

### 8.1 API Development Process

1. **Define Interface**: Write TypeScript definitions first
2. **Implement Registry**: Register the API in Rust
3. **Create Bindings**: Implement the V8 bridge
4. **Test Coverage**: Write comprehensive tests
5. **Documentation**: Generate and review docs

### 8.2 Quality Assurance

1. **Type Safety Tests**: Verify TypeScript compilation
2. **Runtime Tests**: Test all API methods
3. **Performance Tests**: Benchmark critical paths
4. **Integration Tests**: Test with real game scenarios

## 9. Migration Strategy

### 9.1 From Current System

1. **Parallel Implementation**: Build new API alongside existing V8 bindings
2. **Incremental Migration**: Move one namespace at a time
3. **Compatibility Layer**: Maintain backward compatibility during transition
4. **Testing**: Ensure existing scripts continue to work

### 9.2 Future Extensions

1. **Plugin System**: Allow third-party API extensions
2. **Custom Components**: User-defined component types
3. **Visual Scripting**: Potential integration with node-based editors
4. **Multi-Language**: Support for other scripting languages

## 10. Success Metrics

### 10.1 Technical Metrics

- API method call overhead: < 100ns for simple calls
- Memory usage: < 10MB baseline for API system
- Type safety: 100% TypeScript compilation with no errors
- Test coverage: > 95% for all API methods

### 10.2 Developer Experience Metrics

- Time to implement new API: < 1 hour for simple methods
- Documentation completeness: 100% of public APIs documented
- Error message quality: Clear, actionable error messages
- Hot reload time: < 1 second for script changes

## Conclusion

This architecture plan provides a solid foundation for building a Unity-style TypeScript scripting API for the Longhorn Game Engine. The registry-based approach ensures extensibility, while the namespace system provides familiar organization for developers. The phased implementation strategy allows for incremental development and testing, ensuring a robust and maintainable scripting system.

The key to success will be maintaining a balance between flexibility and performance, while providing excellent developer experience through comprehensive tooling and documentation.