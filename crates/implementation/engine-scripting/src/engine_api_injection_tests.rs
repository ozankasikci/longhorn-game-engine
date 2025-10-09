//! TDD Tests for Engine API Injection into V8 Context

use std::collections::HashMap;

/// Mock representation of V8 global context for testing API injection
#[derive(Debug, Default)]
pub struct MockV8GlobalContext {
    pub global_objects: HashMap<String, MockV8Object>,
    pub injected_apis: Vec<String>,
    pub api_call_history: Vec<ApiCall>,
}

#[derive(Debug, Clone)]
pub struct MockV8Object {
    pub name: String,
    pub properties: HashMap<String, MockV8Value>,
    pub methods: HashMap<String, MockV8Function>,
}

#[derive(Debug, Clone)]
pub enum MockV8Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(MockV8Object),
    Undefined,
}

#[derive(Debug, Clone)]
pub struct MockV8Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub struct ApiCall {
    pub object_path: String,
    pub method_name: String,
    pub arguments: Vec<String>,
    pub timestamp: std::time::Instant,
}

impl MockV8GlobalContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inject the Engine object with all its subsystems
    pub fn inject_engine_object(&mut self) -> Result<(), String> {
        // Create the main Engine object
        let mut engine_object = MockV8Object {
            name: "Engine".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        // Add world subsystem
        let world_object = self.create_world_api();
        engine_object.properties.insert("world".to_string(), MockV8Value::Object(world_object));

        // Add input subsystem  
        let input_object = self.create_input_api();
        engine_object.properties.insert("input".to_string(), MockV8Value::Object(input_object));

        // Add physics subsystem
        let physics_object = self.create_physics_api();
        engine_object.properties.insert("physics".to_string(), MockV8Value::Object(physics_object));

        // Add events subsystem
        let events_object = self.create_events_api();
        engine_object.properties.insert("events".to_string(), MockV8Value::Object(events_object));

        // Add time subsystem
        let time_object = self.create_time_api();
        engine_object.properties.insert("time".to_string(), MockV8Value::Object(time_object));

        // Add debug subsystem
        let debug_object = self.create_debug_api();
        engine_object.properties.insert("debug".to_string(), MockV8Value::Object(debug_object));

        self.global_objects.insert("Engine".to_string(), engine_object);
        self.injected_apis.push("Engine".to_string());

        Ok(())
    }

    /// Inject console object with log, warn, error methods
    pub fn inject_console_object(&mut self) -> Result<(), String> {
        let mut console_object = MockV8Object {
            name: "console".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        console_object.methods.insert("log".to_string(), MockV8Function {
            name: "log".to_string(),
            parameters: vec!["...args".to_string()],
            return_type: "void".to_string(),
        });

        console_object.methods.insert("warn".to_string(), MockV8Function {
            name: "warn".to_string(),
            parameters: vec!["...args".to_string()],
            return_type: "void".to_string(),
        });

        console_object.methods.insert("error".to_string(), MockV8Function {
            name: "error".to_string(),
            parameters: vec!["...args".to_string()],
            return_type: "void".to_string(),
        });

        self.global_objects.insert("console".to_string(), console_object);
        self.injected_apis.push("console".to_string());

        Ok(())
    }

    /// Inject Vector3 constructor and prototype
    pub fn inject_vector3_constructor(&mut self) -> Result<(), String> {
        let mut vector3_object = MockV8Object {
            name: "Vector3".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        // Constructor
        vector3_object.methods.insert("constructor".to_string(), MockV8Function {
            name: "constructor".to_string(),
            parameters: vec!["x".to_string(), "y".to_string(), "z".to_string()],
            return_type: "Vector3".to_string(),
        });

        // Instance methods
        vector3_object.methods.insert("add".to_string(), MockV8Function {
            name: "add".to_string(),
            parameters: vec!["other".to_string()],
            return_type: "Vector3".to_string(),
        });

        vector3_object.methods.insert("subtract".to_string(), MockV8Function {
            name: "subtract".to_string(),
            parameters: vec!["other".to_string()],
            return_type: "Vector3".to_string(),
        });

        vector3_object.methods.insert("length".to_string(), MockV8Function {
            name: "length".to_string(),
            parameters: vec![],
            return_type: "number".to_string(),
        });

        vector3_object.methods.insert("normalize".to_string(), MockV8Function {
            name: "normalize".to_string(),
            parameters: vec![],
            return_type: "Vector3".to_string(),
        });

        self.global_objects.insert("Vector3".to_string(), vector3_object);
        self.injected_apis.push("Vector3".to_string());

        Ok(())
    }

    /// Inject Math object with extended game-specific functions
    pub fn inject_math_object(&mut self) -> Result<(), String> {
        let mut math_object = MockV8Object {
            name: "Math".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        // Standard Math functions
        let math_functions = vec![
            "abs", "sin", "cos", "tan", "sqrt", "pow", 
            "floor", "ceil", "round", "min", "max", "random"
        ];

        for func_name in math_functions {
            math_object.methods.insert(func_name.to_string(), MockV8Function {
                name: func_name.to_string(),
                parameters: vec!["...args".to_string()],
                return_type: "number".to_string(),
            });
        }

        // Constants
        math_object.properties.insert("PI".to_string(), MockV8Value::Number(3.14159265359));

        self.global_objects.insert("Math".to_string(), math_object);
        self.injected_apis.push("Math".to_string());

        Ok(())
    }

    /// Call a method on an injected API (for testing)
    pub fn call_api_method(&mut self, object_path: &str, method_name: &str, arguments: Vec<String>) -> Result<MockV8Value, String> {
        // Record the API call
        self.api_call_history.push(ApiCall {
            object_path: object_path.to_string(),
            method_name: method_name.to_string(),
            arguments: arguments.clone(),
            timestamp: std::time::Instant::now(),
        });

        // Parse object path (e.g., "Engine.world" or "console")
        let path_parts: Vec<&str> = object_path.split('.').collect();
        
        if path_parts.len() == 1 {
            // Direct global object method call
            if let Some(object) = self.global_objects.get(path_parts[0]) {
                if object.methods.contains_key(method_name) {
                    return self.simulate_method_call(object_path, method_name, &arguments);
                }
            }
        } else if path_parts.len() == 2 {
            // Nested object method call (e.g., Engine.world.getCurrentEntity)
            if let Some(parent_object) = self.global_objects.get(path_parts[0]) {
                if let Some(MockV8Value::Object(child_object)) = parent_object.properties.get(path_parts[1]) {
                    if child_object.methods.contains_key(method_name) {
                        return self.simulate_method_call(object_path, method_name, &arguments);
                    }
                }
            }
        }

        Err(format!("Method {} not found on {}", method_name, object_path))
    }

    fn simulate_method_call(&self, object_path: &str, method_name: &str, arguments: &[String]) -> Result<MockV8Value, String> {
        // Simulate different API behaviors for testing
        match (object_path, method_name) {
            ("Engine.world", "getCurrentEntity") => Ok(MockV8Value::Object(MockV8Object {
                name: "Entity".to_string(),
                properties: HashMap::new(),
                methods: HashMap::new(),
            })),
            ("Engine.input", "isKeyDown") => Ok(MockV8Value::Boolean(true)),
            ("Engine.input", "getMousePosition") => Ok(MockV8Value::Object(MockV8Object {
                name: "MousePosition".to_string(),
                properties: HashMap::new(),
                methods: HashMap::new(),
            })),
            ("console", "log") => Ok(MockV8Value::Undefined),
            ("Vector3", "constructor") => Ok(MockV8Value::Object(MockV8Object {
                name: "Vector3Instance".to_string(),
                properties: HashMap::new(),
                methods: HashMap::new(),
            })),
            ("Math", "sin") => Ok(MockV8Value::Number(0.5)),
            _ => Ok(MockV8Value::Undefined),
        }
    }

    /// Check if a specific API has been injected
    pub fn is_api_injected(&self, api_name: &str) -> bool {
        self.injected_apis.contains(&api_name.to_string())
    }

    /// Get all available API methods for an object path
    pub fn get_available_methods(&self, object_path: &str) -> Vec<String> {
        let path_parts: Vec<&str> = object_path.split('.').collect();
        
        if path_parts.len() == 1 {
            if let Some(object) = self.global_objects.get(path_parts[0]) {
                return object.methods.keys().cloned().collect();
            }
        } else if path_parts.len() == 2 {
            if let Some(parent_object) = self.global_objects.get(path_parts[0]) {
                if let Some(MockV8Value::Object(child_object)) = parent_object.properties.get(path_parts[1]) {
                    return child_object.methods.keys().cloned().collect();
                }
            }
        }

        Vec::new()
    }

    /// Get API call history for a specific object
    pub fn get_api_calls_for_object(&self, object_path: &str) -> Vec<&ApiCall> {
        self.api_call_history
            .iter()
            .filter(|call| call.object_path == object_path)
            .collect()
    }

    fn create_world_api(&self) -> MockV8Object {
        let mut world_object = MockV8Object {
            name: "world".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        world_object.methods.insert("getCurrentEntity".to_string(), MockV8Function {
            name: "getCurrentEntity".to_string(),
            parameters: vec![],
            return_type: "Entity".to_string(),
        });

        world_object.methods.insert("createEntity".to_string(), MockV8Function {
            name: "createEntity".to_string(),
            parameters: vec![],
            return_type: "Entity".to_string(),
        });

        world_object.methods.insert("destroyEntity".to_string(), MockV8Function {
            name: "destroyEntity".to_string(),
            parameters: vec!["entity".to_string()],
            return_type: "void".to_string(),
        });

        world_object
    }

    fn create_input_api(&self) -> MockV8Object {
        let mut input_object = MockV8Object {
            name: "input".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        input_object.methods.insert("isKeyDown".to_string(), MockV8Function {
            name: "isKeyDown".to_string(),
            parameters: vec!["key".to_string()],
            return_type: "boolean".to_string(),
        });

        input_object.methods.insert("getMousePosition".to_string(), MockV8Function {
            name: "getMousePosition".to_string(),
            parameters: vec![],
            return_type: "MousePosition".to_string(),
        });

        input_object
    }

    fn create_physics_api(&self) -> MockV8Object {
        let mut physics_object = MockV8Object {
            name: "physics".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        physics_object.methods.insert("addRigidBody".to_string(), MockV8Function {
            name: "addRigidBody".to_string(),
            parameters: vec!["entity".to_string(), "bodyType".to_string(), "mass".to_string()],
            return_type: "void".to_string(),
        });

        physics_object.methods.insert("raycast".to_string(), MockV8Function {
            name: "raycast".to_string(),
            parameters: vec!["origin".to_string(), "direction".to_string(), "maxDistance".to_string()],
            return_type: "RaycastHit".to_string(),
        });

        physics_object
    }

    fn create_events_api(&self) -> MockV8Object {
        let mut events_object = MockV8Object {
            name: "events".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        events_object.methods.insert("addEventListener".to_string(), MockV8Function {
            name: "addEventListener".to_string(),
            parameters: vec!["eventType".to_string(), "callback".to_string()],
            return_type: "void".to_string(),
        });

        events_object.methods.insert("dispatchEvent".to_string(), MockV8Function {
            name: "dispatchEvent".to_string(),
            parameters: vec!["eventType".to_string(), "data".to_string()],
            return_type: "void".to_string(),
        });

        events_object
    }

    fn create_time_api(&self) -> MockV8Object {
        let mut time_object = MockV8Object {
            name: "time".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        time_object.methods.insert("getElapsedTime".to_string(), MockV8Function {
            name: "getElapsedTime".to_string(),
            parameters: vec![],
            return_type: "number".to_string(),
        });

        time_object.methods.insert("getDeltaTime".to_string(), MockV8Function {
            name: "getDeltaTime".to_string(),
            parameters: vec![],
            return_type: "number".to_string(),
        });

        time_object
    }

    fn create_debug_api(&self) -> MockV8Object {
        let mut debug_object = MockV8Object {
            name: "debug".to_string(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        };

        debug_object.methods.insert("drawLine".to_string(), MockV8Function {
            name: "drawLine".to_string(),
            parameters: vec!["start".to_string(), "end".to_string(), "color".to_string()],
            return_type: "void".to_string(),
        });

        debug_object.methods.insert("log".to_string(), MockV8Function {
            name: "log".to_string(),
            parameters: vec!["message".to_string()],
            return_type: "void".to_string(),
        });

        debug_object
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_context_creation() {
        // Arrange & Act
        let context = MockV8GlobalContext::new();

        // Assert
        assert!(context.global_objects.is_empty());
        assert!(context.injected_apis.is_empty());
        assert!(context.api_call_history.is_empty());
    }

    #[test]
    fn test_engine_object_injection() {
        // Arrange
        let mut context = MockV8GlobalContext::new();

        // Act
        let result = context.inject_engine_object();

        // Assert
        assert!(result.is_ok());
        assert!(context.is_api_injected("Engine"));
        assert!(context.global_objects.contains_key("Engine"));

        let engine_object = context.global_objects.get("Engine").unwrap();
        assert!(engine_object.properties.contains_key("world"));
        assert!(engine_object.properties.contains_key("input"));
        assert!(engine_object.properties.contains_key("physics"));
        assert!(engine_object.properties.contains_key("events"));
        assert!(engine_object.properties.contains_key("time"));
        assert!(engine_object.properties.contains_key("debug"));
    }

    #[test]
    fn test_console_object_injection() {
        // Arrange
        let mut context = MockV8GlobalContext::new();

        // Act
        let result = context.inject_console_object();

        // Assert
        assert!(result.is_ok());
        assert!(context.is_api_injected("console"));
        
        let console_object = context.global_objects.get("console").unwrap();
        assert!(console_object.methods.contains_key("log"));
        assert!(console_object.methods.contains_key("warn"));
        assert!(console_object.methods.contains_key("error"));
    }

    #[test]
    fn test_vector3_constructor_injection() {
        // Arrange
        let mut context = MockV8GlobalContext::new();

        // Act
        let result = context.inject_vector3_constructor();

        // Assert
        assert!(result.is_ok());
        assert!(context.is_api_injected("Vector3"));
        
        let vector3_object = context.global_objects.get("Vector3").unwrap();
        assert!(vector3_object.methods.contains_key("constructor"));
        assert!(vector3_object.methods.contains_key("add"));
        assert!(vector3_object.methods.contains_key("subtract"));
        assert!(vector3_object.methods.contains_key("length"));
        assert!(vector3_object.methods.contains_key("normalize"));
    }

    #[test]
    fn test_math_object_injection() {
        // Arrange
        let mut context = MockV8GlobalContext::new();

        // Act
        let result = context.inject_math_object();

        // Assert
        assert!(result.is_ok());
        assert!(context.is_api_injected("Math"));
        
        let math_object = context.global_objects.get("Math").unwrap();
        assert!(math_object.methods.contains_key("sin"));
        assert!(math_object.methods.contains_key("cos"));
        assert!(math_object.methods.contains_key("random"));
        assert!(math_object.properties.contains_key("PI"));
    }

    #[test]
    fn test_complete_api_injection() {
        // Arrange
        let mut context = MockV8GlobalContext::new();

        // Act - Inject all APIs
        context.inject_engine_object().unwrap();
        context.inject_console_object().unwrap();
        context.inject_vector3_constructor().unwrap();
        context.inject_math_object().unwrap();

        // Assert
        assert_eq!(context.injected_apis.len(), 4);
        assert!(context.is_api_injected("Engine"));
        assert!(context.is_api_injected("console"));
        assert!(context.is_api_injected("Vector3"));
        assert!(context.is_api_injected("Math"));
    }

    #[test]
    fn test_engine_world_api_methods() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let world_methods = context.get_available_methods("Engine.world");

        // Assert
        assert!(world_methods.contains(&"getCurrentEntity".to_string()));
        assert!(world_methods.contains(&"createEntity".to_string()));
        assert!(world_methods.contains(&"destroyEntity".to_string()));
    }

    #[test]
    fn test_engine_input_api_methods() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let input_methods = context.get_available_methods("Engine.input");

        // Assert
        assert!(input_methods.contains(&"isKeyDown".to_string()));
        assert!(input_methods.contains(&"getMousePosition".to_string()));
    }

    #[test]
    fn test_api_method_calling() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();
        context.inject_console_object().unwrap();

        // Act
        let result1 = context.call_api_method("Engine.world", "getCurrentEntity", vec![]);
        let result2 = context.call_api_method("console", "log", vec!["Hello World".to_string()]);
        let result3 = context.call_api_method("Engine.input", "isKeyDown", vec!["Space".to_string()]);

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Check call history
        assert_eq!(context.api_call_history.len(), 3);
        assert_eq!(context.api_call_history[0].object_path, "Engine.world");
        assert_eq!(context.api_call_history[0].method_name, "getCurrentEntity");
        assert_eq!(context.api_call_history[1].object_path, "console");
        assert_eq!(context.api_call_history[1].method_name, "log");
    }

    #[test]
    fn test_typescript_script_api_usage_simulation() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();
        context.inject_console_object().unwrap();
        context.inject_vector3_constructor().unwrap();

        // Act - Simulate TypeScript script using all APIs
        context.call_api_method("console", "log", vec!["Script initialized".to_string()]).unwrap();
        context.call_api_method("Engine.world", "getCurrentEntity", vec![]).unwrap();
        context.call_api_method("Vector3", "constructor", vec!["1".to_string(), "2".to_string(), "3".to_string()]).unwrap();
        context.call_api_method("Engine.input", "isKeyDown", vec!["W".to_string()]).unwrap();

        // Assert
        assert_eq!(context.api_call_history.len(), 4);
        
        let console_calls = context.get_api_calls_for_object("console");
        let world_calls = context.get_api_calls_for_object("Engine.world");
        let vector3_calls = context.get_api_calls_for_object("Vector3");
        let input_calls = context.get_api_calls_for_object("Engine.input");

        assert_eq!(console_calls.len(), 1);
        assert_eq!(world_calls.len(), 1);
        assert_eq!(vector3_calls.len(), 1);
        assert_eq!(input_calls.len(), 1);
    }

    #[test]
    fn test_invalid_api_method_call() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let result = context.call_api_method("Engine.world", "nonexistentMethod", vec![]);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Method nonexistentMethod not found"));
    }

    #[test]
    fn test_api_call_argument_tracking() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        context.call_api_method("Engine.input", "isKeyDown", vec!["Space".to_string()]).unwrap();
        context.call_api_method("Engine.input", "isKeyDown", vec!["W".to_string()]).unwrap();

        // Assert
        let input_calls = context.get_api_calls_for_object("Engine.input");
        assert_eq!(input_calls.len(), 2);
        assert_eq!(input_calls[0].arguments[0], "Space");
        assert_eq!(input_calls[1].arguments[0], "W");
    }

    #[test]
    fn test_physics_api_availability() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let physics_methods = context.get_available_methods("Engine.physics");

        // Assert
        assert!(physics_methods.contains(&"addRigidBody".to_string()));
        assert!(physics_methods.contains(&"raycast".to_string()));
    }

    #[test]
    fn test_events_api_availability() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let events_methods = context.get_available_methods("Engine.events");

        // Assert
        assert!(events_methods.contains(&"addEventListener".to_string()));
        assert!(events_methods.contains(&"dispatchEvent".to_string()));
    }

    #[test]
    fn test_time_api_availability() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let time_methods = context.get_available_methods("Engine.time");

        // Assert
        assert!(time_methods.contains(&"getElapsedTime".to_string()));
        assert!(time_methods.contains(&"getDeltaTime".to_string()));
    }

    #[test]
    fn test_debug_api_availability() {
        // Arrange
        let mut context = MockV8GlobalContext::new();
        context.inject_engine_object().unwrap();

        // Act
        let debug_methods = context.get_available_methods("Engine.debug");

        // Assert
        assert!(debug_methods.contains(&"drawLine".to_string()));
        assert!(debug_methods.contains(&"log".to_string()));
    }
}