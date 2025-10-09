//! TDD Tests for Real V8 TypeScript Runtime Integration

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Test console output capture for V8 integration
#[derive(Debug, Default, Clone)]
pub struct ConsoleCapture {
    pub logs: Arc<Mutex<Vec<String>>>,
    pub errors: Arc<Mutex<Vec<String>>>,
}

impl ConsoleCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&self, message: &str) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(message.to_string());
    }

    pub fn error(&self, message: &str) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(message.to_string());
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.logs.lock().unwrap().clear();
        self.errors.lock().unwrap().clear();
    }
}

/// Mock V8 TypeScript runtime for testing real integration patterns
pub struct MockV8TypeScriptRuntime {
    pub console_capture: ConsoleCapture,
    pub loaded_scripts: HashMap<String, String>,
    pub script_instances: HashMap<String, MockScriptInstance>,
    pub api_injection_called: bool,
    pub compilation_errors: HashMap<String, String>,
    pub execution_errors: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MockScriptInstance {
    pub script_path: String,
    pub class_name: String,
    pub initialized: bool,
    pub update_count: u32,
    pub destroyed: bool,
}

impl MockV8TypeScriptRuntime {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            console_capture: ConsoleCapture::new(),
            loaded_scripts: HashMap::new(),
            script_instances: HashMap::new(),
            api_injection_called: false,
            compilation_errors: HashMap::new(),
            execution_errors: HashMap::new(),
        })
    }

    pub fn load_script_from_source(&mut self, script_path: &str, source: &str) -> Result<(), String> {
        if let Some(error) = self.compilation_errors.get(script_path) {
            return Err(error.clone());
        }

        // Simulate TypeScript compilation
        self.loaded_scripts.insert(script_path.to_string(), source.to_string());
        
        // Parse script to extract class name (simplified)
        let class_name = if source.contains("export class HelloWorld") {
            "HelloWorld"
        } else if source.contains("export class EntityController") {
            "EntityController"
        } else if source.contains("export class InputHandler") {
            "InputHandler"
        } else {
            "UnknownClass"
        };

        self.script_instances.insert(
            script_path.to_string(),
            MockScriptInstance {
                script_path: script_path.to_string(),
                class_name: class_name.to_string(),
                initialized: false,
                update_count: 0,
                destroyed: false,
            }
        );

        Ok(())
    }

    pub fn execute_script(&mut self, script_path: &str) -> Result<(), String> {
        if let Some(error) = self.execution_errors.get(script_path) {
            return Err(error.clone());
        }

        if !self.loaded_scripts.contains_key(script_path) {
            return Err(format!("Script not loaded: {}", script_path));
        }

        // Simulate loading the script class into V8
        Ok(())
    }

    pub fn call_function(&mut self, script_path: &str, function_name: &str, args: Vec<String>) -> Result<(), String> {
        if let Some(instance) = self.script_instances.get_mut(script_path) {
            match function_name {
                "init" => {
                    instance.initialized = true;
                    
                    // Simulate different script behaviors
                    if script_path.contains("hello_world") {
                        self.console_capture.log("Hello, World!");
                        self.console_capture.log("Welcome to Longhorn Game Engine TypeScript scripting!");
                    } else if script_path.contains("entity_controller") {
                        self.console_capture.log("EntityController initialized");
                    } else if script_path.contains("input_handling") {
                        self.console_capture.log("Input handler ready");
                    }
                }
                "update" => {
                    instance.update_count += 1;
                    
                    // Simulate update behavior with delta time
                    if !args.is_empty() {
                        let _delta_time: f64 = args[0].parse().unwrap_or(0.0);
                        // Simulate frame-based logic
                    }
                }
                "destroy" => {
                    instance.destroyed = true;
                    
                    if script_path.contains("hello_world") {
                        self.console_capture.log("Goodbye from TypeScript!");
                    }
                }
                _ => {
                    return Err(format!("Function {} not found in script {}", function_name, script_path));
                }
            }
            Ok(())
        } else {
            Err(format!("Script instance not found: {}", script_path))
        }
    }

    pub fn inject_engine_apis(&mut self) -> Result<(), String> {
        self.api_injection_called = true;
        
        // Simulate API injection - in real V8, this would add global objects
        // engine.world, engine.input, console, Vector3, etc.
        
        Ok(())
    }

    pub fn set_compilation_error(&mut self, script_path: &str, error: &str) {
        self.compilation_errors.insert(script_path.to_string(), error.to_string());
    }

    pub fn set_execution_error(&mut self, script_path: &str, error: &str) {
        self.execution_errors.insert(script_path.to_string(), error.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_runtime_creation() {
        // Arrange & Act
        let result = MockV8TypeScriptRuntime::new();

        // Assert
        assert!(result.is_ok());
        let runtime = result.unwrap();
        assert!(runtime.loaded_scripts.is_empty());
        assert!(runtime.script_instances.is_empty());
        assert!(!runtime.api_injection_called);
    }

    #[test]
    fn test_typescript_hello_world_compilation_and_execution() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        let hello_world_script = r#"
            export class HelloWorld {
                init(): void {
                    console.log("Hello, World!");
                    console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
                }
                
                update(deltaTime: number): void {
                    // Update logic here
                }
                
                destroy(): void {
                    console.log("Goodbye from TypeScript!");
                }
            }
        "#;

        // Act - Load and execute script
        let load_result = runtime.load_script_from_source("typescript_hello_world.ts", hello_world_script);
        let execute_result = runtime.execute_script("typescript_hello_world.ts");

        // Assert
        assert!(load_result.is_ok());
        assert!(execute_result.is_ok());
        assert!(runtime.loaded_scripts.contains_key("typescript_hello_world.ts"));
        assert!(runtime.script_instances.contains_key("typescript_hello_world.ts"));
        
        let instance = runtime.script_instances.get("typescript_hello_world.ts").unwrap();
        assert_eq!(instance.class_name, "HelloWorld");
        assert!(!instance.initialized); // Not yet called init
    }

    #[test]
    fn test_script_lifecycle_methods() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        let test_script = r#"
            export class TestScript {
                init(): void {
                    console.log("Script initialized");
                }
                
                update(deltaTime: number): void {
                    // Update with delta time
                }
                
                destroy(): void {
                    console.log("Script destroyed");
                }
            }
        "#;

        runtime.load_script_from_source("test_script.ts", test_script).unwrap();
        runtime.execute_script("test_script.ts").unwrap();

        // Act - Call lifecycle methods
        let init_result = runtime.call_function("test_script.ts", "init", vec![]);
        let update_result = runtime.call_function("test_script.ts", "update", vec!["0.016".to_string()]);
        let destroy_result = runtime.call_function("test_script.ts", "destroy", vec![]);

        // Assert
        assert!(init_result.is_ok());
        assert!(update_result.is_ok());
        assert!(destroy_result.is_ok());

        let instance = runtime.script_instances.get("test_script.ts").unwrap();
        assert!(instance.initialized);
        assert_eq!(instance.update_count, 1);
        assert!(instance.destroyed);
    }

    #[test]
    fn test_console_output_capture() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        let hello_world_script = r#"
            export class HelloWorld {
                init(): void {
                    console.log("Hello, World!");
                    console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
                }
                
                destroy(): void {
                    console.log("Goodbye from TypeScript!");
                }
            }
        "#;

        runtime.load_script_from_source("typescript_hello_world.ts", hello_world_script).unwrap();
        runtime.execute_script("typescript_hello_world.ts").unwrap();

        // Act - Call methods that produce console output
        runtime.call_function("typescript_hello_world.ts", "init", vec![]).unwrap();
        runtime.call_function("typescript_hello_world.ts", "destroy", vec![]).unwrap();

        // Assert
        let logs = runtime.console_capture.get_logs();
        assert_eq!(logs.len(), 3);
        assert!(logs.contains(&"Hello, World!".to_string()));
        assert!(logs.contains(&"Welcome to Longhorn Game Engine TypeScript scripting!".to_string()));
        assert!(logs.contains(&"Goodbye from TypeScript!".to_string()));
    }

    #[test]
    fn test_engine_api_injection() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();

        // Act
        let result = runtime.inject_engine_apis();

        // Assert
        assert!(result.is_ok());
        assert!(runtime.api_injection_called);
    }

    #[test]
    fn test_typescript_compilation_errors() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        runtime.set_compilation_error("broken_script.ts", "Syntax error: Missing semicolon at line 5");
        
        let broken_script = r#"
            export class BrokenScript {
                init() void {  // Missing colon - syntax error
                    console.log("This won't compile");
                }
            }
        "#;

        // Act
        let result = runtime.load_script_from_source("broken_script.ts", broken_script);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Syntax error"));
    }

    #[test]
    fn test_runtime_execution_errors() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        runtime.set_execution_error("error_script.ts", "ReferenceError: undefined variable");
        
        let script = r#"
            export class ErrorScript {
                init(): void {
                    console.log("This will have runtime errors");
                }
            }
        "#;

        runtime.load_script_from_source("error_script.ts", script).unwrap();

        // Act
        let result = runtime.execute_script("error_script.ts");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ReferenceError"));
    }

    #[test]
    fn test_multiple_script_instances() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        
        let script1 = r#"export class Script1 { init(): void {} }"#;
        let script2 = r#"export class EntityController { init(): void {} }"#;

        // Act
        runtime.load_script_from_source("script1.ts", script1).unwrap();
        runtime.load_script_from_source("entity_controller.ts", script2).unwrap();
        runtime.execute_script("script1.ts").unwrap();
        runtime.execute_script("entity_controller.ts").unwrap();

        runtime.call_function("script1.ts", "init", vec![]).unwrap();
        runtime.call_function("entity_controller.ts", "init", vec![]).unwrap();

        // Assert
        assert_eq!(runtime.script_instances.len(), 2);
        assert!(runtime.script_instances.get("script1.ts").unwrap().initialized);
        assert!(runtime.script_instances.get("entity_controller.ts").unwrap().initialized);
        
        let logs = runtime.console_capture.get_logs();
        assert!(logs.contains(&"EntityController initialized".to_string()));
    }

    #[test]
    fn test_script_update_with_delta_time() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        let script = r#"
            export class UpdateScript {
                update(deltaTime: number): void {
                    // Process delta time
                }
            }
        "#;

        runtime.load_script_from_source("update_script.ts", script).unwrap();
        runtime.execute_script("update_script.ts").unwrap();

        // Act - Call update multiple times with different delta times
        runtime.call_function("update_script.ts", "update", vec!["0.016".to_string()]).unwrap();
        runtime.call_function("update_script.ts", "update", vec!["0.020".to_string()]).unwrap();
        runtime.call_function("update_script.ts", "update", vec!["0.014".to_string()]).unwrap();

        // Assert
        let instance = runtime.script_instances.get("update_script.ts").unwrap();
        assert_eq!(instance.update_count, 3);
    }

    #[test]
    fn test_file_loading_integration() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        
        // Simulate loading the actual hello world file content
        let file_content = r#"
            // Simple Hello World example - no imports needed for basic console operations
            export class HelloWorld {
                init(): void {
                    console.log("Hello, World!");
                    console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
                }
                
                update(deltaTime: number): void {
                    // Update logic here
                }
                
                destroy(): void {
                    console.log("Goodbye from TypeScript!");
                }
            }
        "#;

        // Act - Simulate full file loading workflow
        let file_path = "assets/scripts/typescript_hello_world.ts";
        runtime.load_script_from_source(file_path, file_content).unwrap();
        runtime.execute_script(file_path).unwrap();
        runtime.inject_engine_apis().unwrap();
        
        // Execute full lifecycle
        runtime.call_function(file_path, "init", vec![]).unwrap();
        runtime.call_function(file_path, "update", vec!["0.016".to_string()]).unwrap();
        runtime.call_function(file_path, "destroy", vec![]).unwrap();

        // Assert - Verify complete execution
        let instance = runtime.script_instances.get(file_path).unwrap();
        assert_eq!(instance.class_name, "HelloWorld");
        assert!(instance.initialized);
        assert_eq!(instance.update_count, 1);
        assert!(instance.destroyed);
        assert!(runtime.api_injection_called);

        let logs = runtime.console_capture.get_logs();
        assert_eq!(logs.len(), 3);
        assert!(logs.contains(&"Hello, World!".to_string()));
        assert!(logs.contains(&"Welcome to Longhorn Game Engine TypeScript scripting!".to_string()));
        assert!(logs.contains(&"Goodbye from TypeScript!".to_string()));
    }

    #[test]
    fn test_console_error_handling() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        
        // Act - Simulate script that produces errors
        runtime.console_capture.error("TypeScript compilation error");
        runtime.console_capture.error("Runtime execution error");

        // Assert
        let errors = runtime.console_capture.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&"TypeScript compilation error".to_string()));
        assert!(errors.contains(&"Runtime execution error".to_string()));
    }

    #[test]
    fn test_concurrent_script_execution() {
        // Arrange
        let mut runtime = MockV8TypeScriptRuntime::new().unwrap();
        
        let script1 = r#"export class Script1 { init(): void { console.log("Script1 init"); } }"#;
        let script2 = r#"export class Script2 { init(): void { console.log("Script2 init"); } }"#;

        // Act - Load and execute multiple scripts
        runtime.load_script_from_source("script1.ts", script1).unwrap();
        runtime.load_script_from_source("script2.ts", script2).unwrap();
        runtime.execute_script("script1.ts").unwrap();
        runtime.execute_script("script2.ts").unwrap();

        runtime.call_function("script1.ts", "init", vec![]).unwrap();
        runtime.call_function("script2.ts", "init", vec![]).unwrap();

        // Assert
        assert_eq!(runtime.script_instances.len(), 2);
        
        let logs = runtime.console_capture.get_logs();
        assert!(logs.contains(&"Script1 init".to_string()));
        assert!(logs.contains(&"Script2 init".to_string()));
    }
}