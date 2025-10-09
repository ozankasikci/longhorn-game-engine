//! V8 JavaScript engine wrapper for TypeScript execution
//! 
//! This module provides a secure, isolated JavaScript execution environment
//! using V8 with proper resource management and security controls.

use engine_scripting::{ScriptError, ScriptResult, ScriptId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// V8 JavaScript engine configuration
#[derive(Debug, Clone)]
pub struct TypeScriptEngineConfig {
    /// Maximum heap size in bytes
    pub max_heap_size: usize,
    /// Script execution timeout
    pub execution_timeout: Duration,
    /// Enable debugging support
    pub enable_debugging: bool,
    /// Enable console API (console.log, etc.)
    pub enable_console: bool,
    /// Maximum stack depth
    pub max_stack_depth: u32,
}

impl Default for TypeScriptEngineConfig {
    fn default() -> Self {
        Self {
            max_heap_size: 64 * 1024 * 1024, // 64MB
            execution_timeout: Duration::from_secs(10),
            enable_debugging: false,
            enable_console: true,
            max_stack_depth: 1000,
        }
    }
}

/// Execution context for a script
#[derive(Debug)]
struct ExecutionContext {
    script_id: ScriptId,
    start_time: Instant,
    timeout: Duration,
}

/// V8 JavaScript engine wrapper
pub struct TypeScriptEngine {
    pub isolate: v8::OwnedIsolate,
    config: TypeScriptEngineConfig,
    contexts: Arc<Mutex<HashMap<ScriptId, ExecutionContext>>>,
    pub global_context: v8::Global<v8::Context>,
}

impl TypeScriptEngine {
    /// Create a new TypeScript engine with default configuration
    pub fn new() -> ScriptResult<Self> {
        Self::with_config(TypeScriptEngineConfig::default())
    }
    
    /// Create a new TypeScript engine with custom configuration
    pub fn with_config(config: TypeScriptEngineConfig) -> ScriptResult<Self> {
        // Create V8 isolate with resource limits
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        
        // Set up resource constraints
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
        
        let global_context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(scope);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            // Set up global objects and APIs
            let global = context.global(scope);
            
            // Add console API if enabled
            if config.enable_console {
                Self::setup_console_api(scope, global)?;
            }
            
            v8::Global::new(scope, context)
        };
        
        Ok(Self {
            isolate,
            config,
            contexts: Arc::new(Mutex::new(HashMap::new())),
            global_context,
        })
    }
    
    /// Execute JavaScript code
    pub fn execute(&mut self, script_id: ScriptId, code: &str) -> ScriptResult<v8::Global<v8::Value>> {
        // Set up execution context first
        self.start_execution(script_id)?;
        
        let result = {
            let scope = &mut v8::HandleScope::new(&mut self.isolate);
            let context = v8::Local::new(scope, &self.global_context);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            // Compile the script
            let source = v8::String::new(scope, code)
                .ok_or_else(|| ScriptError::runtime("Failed to create V8 string".to_string()))?;
            
            let script = v8::Script::compile(scope, source, None)
                .ok_or_else(|| ScriptError::CompilationError {
                    message: "Failed to compile JavaScript code".to_string(),
                    script_id: Some(script_id),
                    source: None,
                })?;
            
            // Execute the script
            let result = script.run(scope)
                .ok_or_else(|| ScriptError::RuntimeError {
                    message: "Script execution failed".to_string(),
                    script_id: Some(script_id),
                    line: None,
                    column: None,
                    source: None,
                })?;
            
            Ok(v8::Global::new(scope, result))
        };
        
        // Clean up execution context
        self.end_execution(script_id)?;
        
        result
    }
    
    /// Execute JavaScript code and return the result as a JSON string
    pub fn execute_json(&mut self, script_id: ScriptId, code: &str) -> ScriptResult<String> {
        let result = self.execute(script_id, code)?;
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        let local_result = v8::Local::new(scope, result);
        
        // Convert result to JSON
        let json_result = v8::json::stringify(scope, local_result)
            .ok_or_else(|| ScriptError::runtime("Failed to stringify result".to_string()))?;
        
        Ok(json_result.to_rust_string_lossy(scope))
    }
    
    /// Call a JavaScript function by name
    pub fn call_function(&mut self, script_id: ScriptId, function_name: &str, args: &[v8::Global<v8::Value>]) -> ScriptResult<v8::Global<v8::Value>> {
        // Set up execution context first
        self.start_execution(script_id)?;
        
        let result = {
            let scope = &mut v8::HandleScope::new(&mut self.isolate);
            let context = v8::Local::new(scope, &self.global_context);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            // Get the global object
            let global = context.global(scope);
            
            // Get the function
            let function_key = v8::String::new(scope, function_name)
                .ok_or_else(|| ScriptError::runtime("Failed to create function name string".to_string()))?;
            
            let function_value = global.get(scope, function_key.into())
                .ok_or_else(|| ScriptError::runtime(format!("Function '{}' not found", function_name)))?;
            
            let function = v8::Local::<v8::Function>::try_from(function_value)
                .map_err(|_| ScriptError::runtime(format!("'{}' is not a function", function_name)))?;
            
            // Convert arguments
            let mut local_args = Vec::new();
            for arg in args {
                local_args.push(v8::Local::new(scope, arg));
            }
            
            // Call the function
            let result = function.call(scope, global.into(), &local_args)
                .ok_or_else(|| ScriptError::RuntimeError {
                    message: format!("Failed to call function '{}'", function_name),
                    script_id: Some(script_id),
                    line: None,
                    column: None,
                    source: None,
                })?;
            
            Ok(v8::Global::new(scope, result))
        };
        
        // Clean up execution context
        self.end_execution(script_id)?;
        
        result
    }
    
    /// Set up console API (console.log, console.error, etc.)
    fn setup_console_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
        let console_name = v8::String::new(scope, "console")
            .ok_or_else(|| ScriptError::runtime("Failed to create console string".to_string()))?;
        
        let console_obj = v8::Object::new(scope);
        
        // console.log
        let log_name = v8::String::new(scope, "log")
            .ok_or_else(|| ScriptError::runtime("Failed to create log string".to_string()))?;
        
        let log_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let mut message_parts = Vec::new();
            for i in 0..args.length() {
                let arg = args.get(i);
                let string_val = arg.to_string(scope).unwrap_or_else(|| {
                    v8::String::new(scope, "[object]").unwrap()
                });
                message_parts.push(string_val.to_rust_string_lossy(scope));
            }
            log::info!("[JS Console] {}", message_parts.join(" "));
        }).unwrap();
        
        console_obj.set(scope, log_name.into(), log_fn.into());
        
        // console.error
        let error_name = v8::String::new(scope, "error")
            .ok_or_else(|| ScriptError::runtime("Failed to create error string".to_string()))?;
        
        let error_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let mut message_parts = Vec::new();
            for i in 0..args.length() {
                let arg = args.get(i);
                let string_val = arg.to_string(scope).unwrap_or_else(|| {
                    v8::String::new(scope, "[object]").unwrap()
                });
                message_parts.push(string_val.to_rust_string_lossy(scope));
            }
            log::error!("[JS Console] {}", message_parts.join(" "));
        }).unwrap();
        
        console_obj.set(scope, error_name.into(), error_fn.into());
        
        // Set console object on global
        global.set(scope, console_name.into(), console_obj.into());
        
        Ok(())
    }
    
    /// Start script execution and set up timeout monitoring
    fn start_execution(&self, script_id: ScriptId) -> ScriptResult<()> {
        let mut contexts = self.contexts.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire execution context lock".to_string())
        })?;
        
        let context = ExecutionContext {
            script_id,
            start_time: Instant::now(),
            timeout: self.config.execution_timeout,
        };
        
        contexts.insert(script_id, context);
        Ok(())
    }
    
    /// End script execution and clean up
    fn end_execution(&self, script_id: ScriptId) -> ScriptResult<()> {
        let mut contexts = self.contexts.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire execution context lock".to_string())
        })?;
        
        if let Some(context) = contexts.remove(&script_id) {
            let execution_time = context.start_time.elapsed();
            if execution_time > context.timeout {
                return Err(ScriptError::RuntimeError {
                    message: format!("Script execution timeout: {:?}", execution_time),
                    script_id: Some(script_id),
                    line: None,
                    column: None,
                    source: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Get engine statistics
    pub fn get_stats(&mut self) -> EngineStats {
        let mut heap_stats = v8::HeapStatistics::default();
        self.isolate.get_heap_statistics(&mut heap_stats);
        
        EngineStats {
            heap_size_limit: heap_stats.heap_size_limit(),
            total_heap_size: heap_stats.total_heap_size(),
            used_heap_size: heap_stats.used_heap_size(),
            active_scripts: self.contexts.lock().map(|c| c.len()).unwrap_or(0),
        }
    }
    
    /// Force garbage collection
    pub fn collect_garbage(&mut self) {
        self.isolate.low_memory_notification();
    }
}

/// Engine runtime statistics
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub heap_size_limit: usize,
    pub total_heap_size: usize,
    pub used_heap_size: usize,
    pub active_scripts: usize,
}

// V8 requires special handling for thread safety
unsafe impl Send for TypeScriptEngine {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_v8_platform;
    
    #[test]
    fn test_engine_creation() {
        initialize_v8_platform().unwrap();
        let engine = TypeScriptEngine::new();
        assert!(engine.is_ok(), "Engine should create successfully");
    }
    
    #[test]
    fn test_basic_javascript_execution() {
        initialize_v8_platform().unwrap();
        let mut engine = TypeScriptEngine::new().unwrap();
        
        let script_id = ScriptId(1);
        let code = "42 + 8";
        
        let result = engine.execute_json(script_id, code);
        assert!(result.is_ok(), "Script execution should succeed");
        assert_eq!(result.unwrap(), "50", "Result should be 50");
    }
    
    #[test]
    fn test_console_api() {
        initialize_v8_platform().unwrap();
        let mut engine = TypeScriptEngine::new().unwrap();
        
        let script_id = ScriptId(2);
        let code = r#"
            console.log("Hello from TypeScript!");
            console.error("This is an error message");
            "test complete"
        "#;
        
        let result = engine.execute_json(script_id, code);
        assert!(result.is_ok(), "Console API should work");
        assert_eq!(result.unwrap(), "\"test complete\"", "Result should be the return value");
    }
    
    #[test]
    fn test_function_calling() {
        initialize_v8_platform().unwrap();
        let mut engine = TypeScriptEngine::new().unwrap();
        
        let script_id = ScriptId(3);
        
        // Define a function
        let define_code = r#"
            function multiply(a, b) {
                return a * b;
            }
        "#;
        
        engine.execute(script_id, define_code).unwrap();
        
        // Call the function
        let args = {
            let scope = &mut v8::HandleScope::new(&mut engine.isolate);
            let context = v8::Local::new(scope, &engine.global_context);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            let arg1: v8::Local<v8::Value> = v8::Number::new(scope, 6.0).into();
            let arg2: v8::Local<v8::Value> = v8::Number::new(scope, 7.0).into();
            vec![
                v8::Global::new(scope, arg1),
                v8::Global::new(scope, arg2),
            ]
        };
        
        let result = engine.call_function(script_id, "multiply", &args);
        assert!(result.is_ok(), "Function call should succeed");
    }
}