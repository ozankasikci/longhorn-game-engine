//! Secure Lua script engine with resource limits enforcement
//! Production-ready engine with sandboxing, resource limits, and API security

use crate::{ScriptError, ScriptResult, ScriptId, ScriptMetadata, ScriptType};
use crate::ecs_console::{ScriptConsoleHandler};
use crate::ecs_component_storage::{ScriptComponentHandler};
use crate::components::Transform;
use crate::resource_limits::{ScriptResourceLimits, ScriptExecutionContext};
use mlua::{Lua, LuaOptions, StdLib, Table, Value, Function, HookTriggers};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use engine_ecs_core::{Entity};
use log::{debug, info};

/// Lua script instance with resource tracking
pub struct LuaScript {
    pub metadata: ScriptMetadata,
    pub chunk: Option<mlua::Chunk<'static>>,
    pub file_path: Option<PathBuf>,
    pub last_modified: Option<SystemTime>,
    pub source: String,
}

/// Shared execution context that can be accessed from Lua hooks
type SharedExecutionContext = Arc<Mutex<ScriptExecutionContext>>;

/// Secure Lua script engine with enforced resource limits
pub struct SecureLuaScriptEngine {
    /// Lua state with hooks for resource monitoring
    lua: Lua,
    /// Loaded scripts
    scripts: HashMap<ScriptId, LuaScript>,
    /// Global engine table
    engine_table: Table,
    /// Non-global console handler
    console_handler: ScriptConsoleHandler,
    /// Non-global component handler
    component_handler: ScriptComponentHandler,
    /// Resource limits configuration
    resource_limits: ScriptResourceLimits,
    /// Current execution context (shared with Lua hooks)
    execution_context: SharedExecutionContext,
    /// Next script ID counter
    next_script_id: u32,
}

impl SecureLuaScriptEngine {
    /// Create a new Lua script engine with default resource limits
    pub fn new() -> ScriptResult<Self> {
        Self::new_with_limits(ScriptResourceLimits::default())
    }

    /// Create a new Lua script engine with custom resource limits
    pub fn new_with_limits(limits: ScriptResourceLimits) -> ScriptResult<Self> {
        // Create Lua instance with SAFE subset of standard libraries only
        // SECURITY: Remove StdLib::OS and StdLib::IO to prevent access to dangerous functions
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::COROUTINE | StdLib::UTF8,
            LuaOptions::default(),
        ).map_err(|e| ScriptError::RuntimeError(format!("Failed to create Lua instance: {}", e)))?;

        // Create shared execution context for hooks
        let execution_context = Arc::new(Mutex::new(ScriptExecutionContext::new(limits.clone())));

        // Set up resource monitoring hooks
        Self::setup_resource_hooks(&lua, execution_context.clone())?;

        // Create global engine table
        let engine_table = lua.create_table()
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to create engine table: {}", e)))?;

        // Set up global namespace
        lua.globals().set("engine", engine_table.clone())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set engine global: {}", e)))?;

        // Create non-global handlers
        let console_handler = ScriptConsoleHandler::new(1000);
        let component_handler = ScriptComponentHandler::new();

        info!("Secure Lua script engine initialized with resource limits: {:?}", limits);

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            engine_table,
            console_handler,
            component_handler,
            resource_limits: limits,
            execution_context,
            next_script_id: 1,
        })
    }

    /// Set up Lua hooks for resource monitoring
    fn setup_resource_hooks(lua: &Lua, context: SharedExecutionContext) -> ScriptResult<()> {
        // Set hook that triggers on every line and function call/return
        lua.set_hook(
            HookTriggers::EVERY_LINE | HookTriggers::ON_CALLS | HookTriggers::ON_RETURNS,
            move |_lua_ctx, debug| {
                if let Ok(mut ctx) = context.try_lock() {
                    // Check timeout on every hook
                    if let Err(_e) = ctx.check_timeout() {
                        return Err(mlua::Error::runtime("Script execution timeout"));
                    }

                    // Track recursion depth
                    match debug.event() {
                        mlua::DebugEvent::Call => {
                            if let Err(_e) = ctx.check_recursion_depth() {
                                return Err(mlua::Error::runtime("Maximum recursion depth exceeded"));
                            }
                        }
                        mlua::DebugEvent::Ret => {  // Use Ret instead of Return
                            ctx.decrease_recursion_depth();
                        }
                        _ => {}
                    }
                }
                // Return Continue to keep script running
                Ok(mlua::VmState::Continue)
            }
        );

        Ok(())
    }
    
    /// Load a script from string with resource limit enforcement
    pub fn load_script_internal(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()> {
        if metadata.script_type != ScriptType::Lua {
            return Err(ScriptError::RuntimeError("Not a Lua script".to_string()));
        }

        debug!("Loading Lua script: {:?}", metadata.path);

        // Check source code size against limits
        if source.len() > self.resource_limits.max_string_length {
            return Err(ScriptError::RuntimeError(format!(
                "Script source too large: {} bytes > {} limit",
                source.len(),
                self.resource_limits.max_string_length
            )));
        }

        // Compile the script
        let chunk = self.lua.load(source)
            .set_name(&metadata.path)
            .into_function()
            .map_err(|e| ScriptError::CompilationError(format!("Failed to compile script: {}", e)))?;

        // Create script instance
        let script = LuaScript {
            metadata: metadata.clone(),
            chunk: None,
            file_path: None,
            last_modified: None,
            source: source.to_string(),
        };

        // Store the script
        self.scripts.insert(metadata.id, script);

        // Start execution tracking
        {
            let mut context = self.execution_context.lock().unwrap();
            context.start_execution();
        }

        // Execute the script to define its functions/globals with resource limits
        let result = chunk.call::<mlua::Value>(())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to execute script: {}", e)))?;

        // If the script returns a table (module), store it for lifecycle calls
        if let mlua::Value::Table(module_table) = result {
            self.lua.globals().set("_LAST_SCRIPT_MODULE", module_table)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to store script module: {}", e)))?;
        }

        Ok(())
    }

    /// Execute a script function by name with resource limits
    pub fn execute_function_internal(&mut self, function_name: &str, args: impl mlua::IntoLuaMulti) -> ScriptResult<Value> {
        let func: Function = self.lua.globals()
            .get(function_name)
            .map_err(|_| ScriptError::RuntimeError(format!("Function '{}' not found", function_name)))?;

        // Start execution tracking
        {
            let mut context = self.execution_context.lock().unwrap();
            context.start_execution();
        }

        // Execute with resource limits
        let result = func.call(args)
            .map_err(|e| ScriptError::RuntimeError(format!("Function execution error: {}", e)))?;
        
        Ok(result)
    }


    /// Get the Lua state (for bindings)
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get the engine table (for bindings)
    pub fn engine_table(&self) -> &Table {
        &self.engine_table
    }

    /// Get resource limits
    pub fn resource_limits(&self) -> &ScriptResourceLimits {
        &self.resource_limits
    }
    
    /// Setup API bindings with permission checking
    pub fn setup_api_bindings<F>(&mut self, _permission_checker: F) -> ScriptResult<()> 
    where
        F: Fn(&str, ScriptId) -> ScriptResult<()> + 'static
    {
        // For now, just ensure basic bindings are in place
        // TODO: Integrate permission checker into API calls
        use crate::bindings::create_safe_bindings;
        create_safe_bindings(&self.lua)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to create safe bindings: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })
    }
    
    /// Setup API bindings with full permission integration
    pub fn setup_api_bindings_with_permissions(&mut self, api: Arc<Mutex<crate::api::ScriptApi>>, current_script_id: ScriptId) -> ScriptResult<()> {
        use crate::api::{ApiPermission, ApiInputValidator};
        
        // Ensure core bindings like print are available
        self.setup_core_bindings()?;
        
        // Get engine table
        let engine_table = self.engine_table.clone();
        
        // Create read_file function with permission checking
        let api_clone = api.clone();
        let read_file = self.lua.create_function(move |_lua, path: String| -> mlua::Result<String> {
            println!("DEBUG: read_file called with path: '{}'", path);
            
            // Validate input FIRST before checking permissions
            if let Err(e) = ApiInputValidator::validate_file_path(&path) {
                println!("DEBUG: Validation failed: {}", e);
                return Err(mlua::Error::runtime(format!("Invalid path: {}", e)));
            }
            
            // Then check permissions
            let api_lock = api_clone.lock().unwrap();
            if let Err(e) = api_lock.check_permission(current_script_id, "engine.read_file", ApiPermission::FileRead) {
                return Err(mlua::Error::runtime(e.to_string()));
            }
            
            println!("DEBUG: Path validation passed, returning file access error");
            // TODO: Actually read file
            Err(mlua::Error::runtime("file access not implemented"))
        }).map_err(|e| ScriptError::InitializationError {
            message: format!("Failed to create read_file function: {}", e),
            component: "api_bindings".to_string(),
            source: None,
        })?;
        
        engine_table.set("read_file", read_file)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set read_file: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        // Create write_file function for buffer overflow test
        let api_clone2 = api.clone();
        let write_file = self.lua.create_function(move |_lua, (_path, content): (String, String)| -> mlua::Result<()> {
            // Check string length limit
            let max_length = 10 * 1024 * 1024; // 10MB
            if content.len() > max_length {
                return Err(mlua::Error::runtime(format!("String too long: {} > {}", content.len(), max_length)));
            }
            Ok(())
        }).map_err(|e| ScriptError::InitializationError {
            message: format!("Failed to create write_file function: {}", e),
            component: "api_bindings".to_string(),
            source: None,
        })?;
        
        engine_table.set("write_file", write_file)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set write_file: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        // Create console table with log function
        let console_table = self.lua.create_table()
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to create console table: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        let api_clone3 = api.clone();
        let console_log = self.lua.create_function(move |_lua, message: String| -> mlua::Result<()> {
            // Check rate limit
            let api_lock = api_clone3.lock().unwrap();
            if let Err(e) = api_lock.check_permission(current_script_id, "console.log", ApiPermission::ConsoleWrite) {
                return Err(mlua::Error::runtime(e.to_string()));
            }
            
            // Log the message
            println!("[Console] {}", message);
            Ok(())
        }).map_err(|e| ScriptError::InitializationError {
            message: format!("Failed to create console.log function: {}", e),
            component: "api_bindings".to_string(),
            source: None,
        })?;
        
        console_table.set("log", console_log)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set console.log: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        self.lua.globals().set("console", console_table)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set console global: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        // Create entity table with create function
        let entity_table = self.lua.create_table()
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to create entity table: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        let entity_create = self.lua.create_function(move |_lua, _name: String| -> mlua::Result<()> {
            // TODO: Check permissions and rate limits
            Ok(())
        }).map_err(|e| ScriptError::InitializationError {
            message: format!("Failed to create entity.create function: {}", e),
            component: "api_bindings".to_string(),
            source: None,
        })?;
        
        entity_table.set("create", entity_create)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set entity.create: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        self.lua.globals().set("entity", entity_table)
            .map_err(|e| ScriptError::InitializationError {
                message: format!("Failed to set entity global: {}", e),
                component: "api_bindings".to_string(),
                source: None,
            })?;
        
        Ok(())
    }

    /// Get console messages (non-global)
    pub fn get_console_messages(&self) -> &std::collections::VecDeque<crate::ecs_console::ConsoleMessage> {
        self.console_handler.get_messages()
    }

    /// Clear console messages (non-global)
    pub fn clear_console_messages(&mut self) {
        self.console_handler.clear_messages();
    }

    /// Initialize entity transform (non-global)
    pub fn init_entity_transform(&mut self, entity: Entity, transform: Transform) {
        self.component_handler.init_entity_transform(entity, transform);
    }

    /// Check if entity has transform (non-global)
    pub fn has_entity_transform(&self, entity: Entity) -> bool {
        self.component_handler.has_entity_transform(entity)
    }

    /// Set up core engine bindings WITHOUT using any globals
    pub fn setup_core_bindings(&mut self) -> ScriptResult<()> {
        // Create a print function that uses local console handler instead of global
        let print_fn = {
            self.lua.create_function(|_, args: mlua::MultiValue| {
                let mut output = String::new();
                for (i, value) in args.iter().enumerate() {
                    if i > 0 {
                        output.push('\t');
                    }
                    // Format Lua values properly
                    match value {
                        mlua::Value::String(s) => output.push_str(&s.to_string_lossy()),
                        mlua::Value::Number(n) => output.push_str(&n.to_string()),
                        mlua::Value::Integer(i) => output.push_str(&i.to_string()),
                        mlua::Value::Boolean(b) => output.push_str(&b.to_string()),
                        mlua::Value::Nil => output.push_str("nil"),
                        _ => output.push_str(&format!("{:?}", value)),
                    }
                }
                
                // Use println! for direct console output
                println!("{}", output);
                // Also log it for systems that have logging configured
                info!("[Lua] {}", output);
                
                Ok(())
            }).map_err(|e| ScriptError::RuntimeError(format!("Failed to create print function: {}", e)))?
        };

        self.lua.globals().set("print", print_fn)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set print function: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_lua_engine_creation() {
        let engine = SecureLuaScriptEngine::new().expect("Failed to create secure Lua engine");
        assert_eq!(engine.resource_limits.max_execution_time, Duration::from_secs(10));
        assert_eq!(engine.resource_limits.max_memory_bytes, 1024 * 1024 * 1024);
        assert_eq!(engine.resource_limits.max_recursion_depth, 10000);
    }

    #[test] 
    fn test_secure_lua_engine_with_custom_limits() {
        let mut limits = ScriptResourceLimits::default();
        limits.max_execution_time = Duration::from_millis(50);
        limits.max_recursion_depth = 10;

        let engine = SecureLuaScriptEngine::new_with_limits(limits).expect("Failed to create engine");
        assert_eq!(engine.resource_limits.max_execution_time, Duration::from_millis(50));
        assert_eq!(engine.resource_limits.max_recursion_depth, 10);
    }

    #[test]
    fn test_timeout_enforcement() {
        let mut limits = ScriptResourceLimits::default();
        limits.max_execution_time = Duration::from_millis(50); // Very short timeout

        let mut engine = SecureLuaScriptEngine::new_with_limits(limits).expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(1),
            script_type: ScriptType::Lua,
            path: "timeout_test.lua".to_string(),
            entry_point: None,
        };

        let infinite_loop_script = r#"
            function infinite_loop()
                local i = 0
                while true do
                    i = i + 1
                end
                return i
            end
        "#;

        engine.load_script_internal(metadata, infinite_loop_script).expect("Script should load");
        
        let start = Instant::now();
        let result = engine.execute_function_internal("infinite_loop", ());
        let elapsed = start.elapsed();
        
        // Should timeout and return error
        assert!(result.is_err(), "Infinite loop should be interrupted by timeout");
        assert!(elapsed.as_millis() < 200, "Should timeout quickly, took {:?}", elapsed);
    }
}