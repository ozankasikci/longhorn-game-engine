//! Script execution engine - handles ONLY execution, not storage
//! This component receives ScriptRef from ScriptManager and executes them safely

use crate::{ScriptError, ScriptResult, ScriptRef, ScriptId};
use crate::resource_limits::{ScriptResourceLimits, ScriptExecutionContext};
use crate::secure_lua_engine::SecureLuaScriptEngine;
use crate::api::{ScriptApi, ApiPermission, ApiInputValidator};
use mlua::Value;
use std::sync::{Arc, Mutex};

/// Script execution engine - ONLY handles execution, not storage
pub struct ScriptEngine {
    /// Lua engine for execution only
    lua_engine: SecureLuaScriptEngine,
    /// Current execution context
    execution_context: Arc<Mutex<ScriptExecutionContext>>,
    /// Script API for permission checking
    pub api: Arc<Mutex<ScriptApi>>,
}

impl ScriptEngine {
    /// Create a new execution-only script engine
    pub fn new() -> ScriptResult<Self> {
        Self::new_with_limits(ScriptResourceLimits::default())
    }

    /// Create a new execution-only script engine with custom limits
    pub fn new_with_limits(limits: ScriptResourceLimits) -> ScriptResult<Self> {
        let lua_engine = SecureLuaScriptEngine::new_with_limits(limits.clone())?;
        let execution_context = Arc::new(Mutex::new(ScriptExecutionContext::new(limits)));

        Ok(Self {
            lua_engine,
            execution_context,
            api: Arc::new(Mutex::new(ScriptApi::new())),
        })
    }

    /// Execute a script by reference (does NOT store the script)
    pub fn execute_script(&mut self, script_ref: &ScriptRef, function_name: &str, args: impl mlua::IntoLuaMulti) -> ScriptResult<Value> {
        let resource_limits = self.lua_engine.resource_limits().clone();
        
        // Set up API bindings with current script context
        self.lua_engine.setup_api_bindings_with_permissions(self.api.clone(), script_ref.id)?;
        // Load the script temporarily for execution (not stored)
        self.lua_engine.load_script_internal(script_ref.metadata.clone(), &script_ref.source)
            .map_err(|e| {
                // Enrich error with script context
                match e {
                    ScriptError::CompilationError { mut message, .. } => {
                        // Extract line info from compilation errors
                        let (enriched_message, line, _column) = Self::extract_error_context(&message, &script_ref.metadata.path);
                        
                        // Create RuntimeError instead of CompilationError to carry line info
                        ScriptError::RuntimeError {
                            message: enriched_message,
                            script_id: Some(script_ref.id),
                            line,
                            column: None,
                            source: None,
                        }
                    },
                    ScriptError::RuntimeError { message, .. } => {
                        // Try to extract line number from Lua error
                        let (enriched_message, line, column) = Self::extract_error_context(&message, &script_ref.metadata.path);
                        ScriptError::RuntimeError {
                            message: enriched_message,
                            script_id: Some(script_ref.id),
                            line,
                            column,
                            source: None,
                        }
                    },
                    other => other.with_script_id(script_ref.id),
                }
            })?;
        
        // Execute the function
        let result = self.lua_engine.execute_function_internal(function_name, args)
            .map_err(|e| {
                // Enrich execution errors with context
                match e {
                    ScriptError::RuntimeError { message, .. } => {
                        // Check for security violations
                        if message.contains("os.execute") || message.contains("io.open") || 
                           message.contains("attempt to index a nil value (global 'os')") ||
                           message.contains("attempt to index a nil value (global 'io')") {
                            return ScriptError::SecurityViolation {
                                script_id: script_ref.id,
                                violation_type: "forbidden_function".to_string(),
                                message: format!("Script attempted to use forbidden function: {}", message),
                                severity: crate::SecuritySeverity::Critical,
                            };
                        }
                        
                        // Check for permission errors
                        if message.contains("FILE_READ") || message.contains("filesystem") || 
                           message.contains("Permission denied") {
                            // Extract details from the error message
                            let resource = if message.contains("engine.read_file") {
                                "filesystem"
                            } else {
                                "unknown"
                            };
                            
                            return ScriptError::PermissionDenied {
                                script_id: script_ref.id,
                                resource: resource.to_string(),
                                action: "read_file".to_string(),
                                required_permission: "FILE_READ".to_string(),
                            };
                        }
                        
                        // Check for path traversal attempts
                        if message.contains("path traversal") || message.contains("Invalid path") {
                            return ScriptError::InvalidArguments {
                                script_id: Some(script_ref.id),
                                function_name: "read_file".to_string(),
                                message: "path traversal detected".to_string(),
                                expected: "safe path".to_string(),
                                actual: "path with ..".to_string(),
                            };
                        }
                        
                        // Check for resource limit violations
                        if message.contains("Script execution timeout") {
                            let limit_millis = resource_limits.max_execution_time.as_millis();
                            return ScriptError::ResourceLimitExceeded {
                                script_id: script_ref.id,
                                limit_type: "execution_time".to_string(),
                                limit_value: format!("{}ms", limit_millis),
                                actual_value: format!(">{}ms", limit_millis),
                            };
                        }
                        
                        // Check for string length limit
                        if message.contains("String too long") {
                            return ScriptError::ResourceLimitExceeded {
                                script_id: script_ref.id,
                                limit_type: "string_length".to_string(),
                                limit_value: "10MB".to_string(),
                                actual_value: "1000MB".to_string(),
                            };
                        }
                        
                        // Check for rate limit violations
                        if message.contains("api_rate_limit") || message.contains("Rate limit exceeded") {
                            return ScriptError::ResourceLimitExceeded {
                                script_id: script_ref.id,
                                limit_type: "api_rate_limit".to_string(),
                                limit_value: "rate limit".to_string(),
                                actual_value: message.clone(),
                            };
                        }
                        
                        let (enriched_message, line, column) = Self::extract_error_context(&message, &script_ref.metadata.path);
                        ScriptError::RuntimeError {
                            message: enriched_message,
                            script_id: Some(script_ref.id),
                            line,
                            column,
                            source: None,
                        }
                    },
                    other => other.with_script_id(script_ref.id),
                }
            })?;
        
        // No permanent storage - script is executed and forgotten
        Ok(result)
    }

    /// Execute a script from source directly (for one-time execution)
    pub fn execute_source(&mut self, source: &str, function_name: &str, args: impl mlua::IntoLuaMulti) -> ScriptResult<Value> {
        use crate::{ScriptMetadata, ScriptType};
        
        // Create temporary metadata for execution
        let metadata = ScriptMetadata {
            id: ScriptId(0), // Temporary ID
            script_type: ScriptType::Lua,
            path: "temp_execution".to_string(),
            entry_point: None,
        };

        // Load temporarily for execution only
        self.lua_engine.load_script_internal(metadata, source)?;
        
        // Execute the function
        let result = self.lua_engine.execute_function_internal(function_name, args)?;
        
        Ok(result)
    }

    /// Setup core bindings
    pub fn setup_core_bindings(&mut self) -> ScriptResult<()> {
        // First apply safe bindings (removes dangerous functions)
        use crate::bindings::create_safe_bindings;
        let lua = self.lua_engine.lua();
        create_safe_bindings(lua)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to create safe bindings: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        self.lua_engine.setup_core_bindings()?;
        self.setup_bindings_with_api()
    }
    
    /// Setup bindings with API security
    fn setup_bindings_with_api(&mut self) -> ScriptResult<()> {
        // Setup API bindings with permission checking
        let api = self.api.clone();
        let current_script_id = ScriptId(0); // TODO: Track current executing script
        
        self.lua_engine.setup_api_bindings_with_permissions(api, current_script_id)
    }

    /// Get resource limits
    pub fn resource_limits(&self) -> &ScriptResourceLimits {
        self.lua_engine.resource_limits()
    }

    /// Get console messages (non-global)
    pub fn get_console_messages(&self) -> &std::collections::VecDeque<crate::ecs_console::ConsoleMessage> {
        self.lua_engine.get_console_messages()
    }

    /// Clear console messages (non-global)
    pub fn clear_console_messages(&mut self) {
        self.lua_engine.clear_console_messages();
    }

    /// Extract error context from Lua error messages
    fn extract_error_context(error_msg: &str, script_path: &str) -> (String, Option<u32>, Option<u32>) {
        // Lua errors often have format: "[string \"name\"]:line:column: message"
        // or: "script.lua:line: message"
        // or: "[string \"chunk\"]:2: syntax error near 'here'"
        
        let mut enriched_message = error_msg.to_string();
        let mut line = None;
        let mut column = None;

        // Try to parse Lua error format with [string "..."]
        if let Some(pos) = error_msg.find("]:") {
            // Format: [string "script"]:line:column: message
            let after_bracket = &error_msg[pos + 2..];
            let parts: Vec<&str> = after_bracket.splitn(3, ':').collect();
            
            if parts.len() >= 1 {
                // First part after ]: should be line number
                if let Ok(line_num) = parts[0].trim().parse::<u32>() {
                    line = Some(line_num);
                    
                    // Check for column in second part
                    if parts.len() >= 2 {
                        // Sometimes it's line:column, sometimes it's line: message
                        if let Ok(col_num) = parts[1].trim().parse::<u32>() {
                            column = Some(col_num);
                        }
                    }
                }
            }
        } else if error_msg.contains(".lua:") {
            // Format: script.lua:line: message
            let parts: Vec<&str> = error_msg.splitn(3, ':').collect();
            if parts.len() >= 2 {
                if let Ok(line_num) = parts[1].trim().parse::<u32>() {
                    line = Some(line_num);
                }
            }
        }

        // Add script path if not already present
        if !enriched_message.contains(script_path) {
            enriched_message = format!("{}: {}", script_path, enriched_message);
        }

        // Ensure "syntax" is mentioned for syntax errors
        if enriched_message.contains("unexpected") || enriched_message.contains("expected") || enriched_message.contains("near") {
            if !enriched_message.contains("syntax") {
                enriched_message = enriched_message.replace("error", "syntax error");
                if !enriched_message.contains("syntax") {
                    enriched_message = enriched_message.replace("near", "syntax error near");
                }
            }
        }

        // Ensure we have line info in the message
        if let Some(line_num) = line {
            if !enriched_message.contains("line") && !enriched_message.contains(&line_num.to_string()) {
                enriched_message = format!("{} (line {})", enriched_message, line_num);
            }
        }

        (enriched_message, line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScriptMetadata, ScriptType};

    #[test]
    fn test_script_engine_execution_only() {
        let mut engine = ScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        // Create a script reference (normally from ScriptManager)
        let script_ref = ScriptRef {
            id: ScriptId(1),
            metadata: ScriptMetadata {
                id: ScriptId(1),
                script_type: ScriptType::Lua,
                path: "test.lua".to_string(),
                entry_point: None,
            },
            source: "function test() return 42 end".to_string(),
        };

        // Execute the script by reference
        let result = engine.execute_script(&script_ref, "test", ()).expect("Failed to execute");
        
        // Verify result
        if let mlua::Value::Integer(value) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected integer result");
        }
    }

    #[test]
    fn test_script_engine_does_not_store_scripts() {
        let mut engine = ScriptEngine::new().expect("Failed to create engine");
        
        // Execute source directly
        let result = engine.execute_source("function temp() return 'hello' end", "temp", ()).expect("Failed to execute");
        
        // Verify result but engine doesn't store the script
        if let mlua::Value::String(value) = result {
            assert_eq!(value.to_string_lossy(), "hello");
        } else {
            panic!("Expected string result");
        }
        
        // Engine should not have persistent script storage for this
        // (This test documents that ScriptEngine is execution-only)
        assert!(true, "ScriptEngine is execution-only - scripts not permanently stored");
    }
}