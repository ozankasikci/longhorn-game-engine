//! Lua script engine implementation

use crate::{ScriptError, ScriptResult, ScriptId, ScriptMetadata, ScriptType, runtime::ScriptRuntime};
use mlua::{Lua, LuaOptions, StdLib, Table, Value, Function};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use engine_ecs_core::World;
use log::{debug, error, info};

/// Lua script instance
pub struct LuaScript {
    /// Script metadata
    pub metadata: ScriptMetadata,
    /// Compiled chunk (if any)
    pub chunk: Option<mlua::Chunk<'static>>,
    /// File path if loaded from file
    pub file_path: Option<PathBuf>,
    /// Last modification time
    pub last_modified: Option<SystemTime>,
    /// Script source code for reloading
    pub source: String,
}

/// Lua script engine for executing Lua scripts
pub struct LuaScriptEngine {
    /// Lua state
    lua: Lua,
    /// Loaded scripts
    scripts: HashMap<ScriptId, LuaScript>,
    /// Global engine table
    engine_table: Table,
    /// Watched directories for hot-reload
    watched_directories: Vec<PathBuf>,
    /// Next script ID counter
    next_script_id: u32,
}

impl LuaScriptEngine {
    /// Create a new Lua script engine
    pub fn new() -> ScriptResult<Self> {
        // Create Lua instance with safe subset of standard libraries
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::COROUTINE | StdLib::OS,
            LuaOptions::default(),
        ).map_err(|e| ScriptError::RuntimeError(format!("Failed to create Lua instance: {}", e)))?;

        // Create global engine table
        let engine_table = lua.create_table()
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to create engine table: {}", e)))?;

        // Set up global namespace
        lua.globals().set("engine", engine_table.clone())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set engine global: {}", e)))?;


        info!("Lua script engine initialized");

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            engine_table,
            watched_directories: Vec::new(),
            next_script_id: 1,
        })
    }

    /// Load a script from string
    pub fn load_script_internal(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()> {
        if metadata.script_type != ScriptType::Lua {
            return Err(ScriptError::RuntimeError("Not a Lua script".to_string()));
        }

        debug!("Loading Lua script: {:?}", metadata.path);

        // Compile the script
        let chunk = self.lua.load(source)
            .set_name(&metadata.path)
            .into_function()
            .map_err(|e| ScriptError::CompilationError(format!("Failed to compile script: {}", e)))?;

        // Create script instance
        let script = LuaScript {
            metadata: metadata.clone(),
            chunk: None, // We'll execute immediately for now
            file_path: None,
            last_modified: None,
            source: source.to_string(),
        };

        // Store the script
        self.scripts.insert(metadata.id, script);

        // Execute the script to define its functions/globals
        let result = chunk.call::<mlua::Value>(())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to execute script: {}", e)))?;

        // If the script returns a table (module), store it for lifecycle calls
        if let mlua::Value::Table(module_table) = result {
            self.lua.globals().set("_LAST_SCRIPT_MODULE", module_table)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to store script module: {}", e)))?;
        }

        Ok(())
    }

    /// Execute a loaded script by ID
    pub fn execute_script_internal(&mut self, id: ScriptId) -> ScriptResult<()> {
        let script = self.scripts.get(&id)
            .ok_or_else(|| ScriptError::NotFound(format!("Script {:?} not found", id)))?;

        debug!("Executing script: {:?}", script.metadata.path);

        // Scripts have already been executed during load, so this is a no-op
        // The load_script_internal method already executes the script to define functions
        
        Ok(())
    }
    
    /// Run a specific function if it exists
    pub fn run_if_exists(&mut self, function_name: &str) -> ScriptResult<()> {
        if let Ok(func) = self.lua.globals().get::<Function>(function_name) {
            func.call::<()>(())
                .map_err(|e| ScriptError::RuntimeError(format!("Function execution error: {}", e)))?;
        }
        Ok(())
    }

    /// Execute a script function by name
    pub fn execute_function_internal(&mut self, function_name: &str, args: impl mlua::IntoLuaMulti) -> ScriptResult<Value> {
        let func: Function = self.lua.globals()
            .get(function_name)
            .map_err(|_| ScriptError::RuntimeError(format!("Function '{}' not found", function_name)))?;

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

    /// Set up core engine bindings
    pub fn setup_core_bindings(&mut self) -> ScriptResult<()> {
        // Add print function
        let print_fn = self.lua.create_function(|_, args: mlua::MultiValue| {
            let mut output = String::new();
            for (i, value) in args.iter().enumerate() {
                if i > 0 {
                    output.push('\t');
                }
                output.push_str(&format!("{:?}", value));
            }
            info!("[Lua] {}", output);
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError(format!("Failed to create print function: {}", e)))?;

        self.lua.globals().set("print", print_fn)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set print function: {}", e)))?;

        Ok(())
    }

    /// Enable file watching for a directory
    pub fn watch_directory(&mut self, path: &Path) -> ScriptResult<()> {
        let canonical_path = path.canonicalize()
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to canonicalize path: {}", e)))?;
        
        if !self.watched_directories.contains(&canonical_path) {
            self.watched_directories.push(canonical_path);
            info!("Watching directory for script changes: {:?}", path);
        }
        
        Ok(())
    }

    /// Load a script from file
    pub fn load_script_from_file(&mut self, path: &Path) -> ScriptResult<ScriptId> {
        let source = fs::read_to_string(path)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to read script file: {}", e)))?;
        
        let metadata = fs::metadata(path)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get file metadata: {}", e)))?;
        
        let last_modified = metadata.modified()
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get modification time: {}", e)))?;
        
        let script_id = ScriptId(self.next_script_id.into());
        self.next_script_id += 1;
        
        let script_metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::Lua,
            path: path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        // Load and compile script
        let chunk = self.lua.load(&source)
            .set_name(&script_metadata.path)
            .into_function()
            .map_err(|e| ScriptError::CompilationError(format!("Failed to compile script: {}", e)))?;
        
        // Create script instance with file info
        let script = LuaScript {
            metadata: script_metadata,
            chunk: None,
            file_path: Some(path.to_path_buf()),
            last_modified: Some(last_modified),
            source: source.clone(),
        };
        
        // Store the script
        self.scripts.insert(script_id, script);
        
        // Execute the script to define its functions/globals
        let result = chunk.call::<mlua::Value>(())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to execute script: {}", e)))?;

        // If the script returns a table (module), store it for lifecycle calls
        if let mlua::Value::Table(module_table) = result {
            self.lua.globals().set("_LAST_SCRIPT_MODULE", module_table)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to store script module: {}", e)))?;
        }
        
        info!("Loaded script from file: {:?}", path);
        Ok(script_id)
    }

    /// Check for script file changes and reload if necessary
    pub fn check_and_reload_scripts(&mut self) -> ScriptResult<()> {
        let mut scripts_to_reload = Vec::new();
        
        // Check each script for file changes
        for (script_id, script) in &self.scripts {
            if let Some(file_path) = &script.file_path {
                if let Ok(metadata) = fs::metadata(file_path) {
                    if let Ok(current_modified) = metadata.modified() {
                        if let Some(last_modified) = script.last_modified {
                            debug!("Checking script {:?}: current={:?}, last={:?}", 
                                   script_id, current_modified, last_modified);
                            // Use >= instead of > to handle filesystem timestamp granularity issues
                            if current_modified >= last_modified {
                                // Also check if file contents have actually changed
                                if let Ok(current_source) = fs::read_to_string(file_path) {
                                    if current_source != script.source {
                                        debug!("Script {:?} has changed content, marking for reload", script_id);
                                        scripts_to_reload.push((*script_id, file_path.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        debug!("Found {} scripts to reload", scripts_to_reload.len());
        
        // Reload changed scripts
        for (script_id, file_path) in scripts_to_reload {
            info!("Reloading changed script: {:?}", file_path);
            
            // Read new source
            let new_source = fs::read_to_string(&file_path)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to read script file: {}", e)))?;
            
            let metadata = fs::metadata(&file_path)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to get file metadata: {}", e)))?;
            
            let new_modified = metadata.modified()
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to get modification time: {}", e)))?;
            
            // Preserve Lua state (globals) by capturing important values
            let preserved_globals = self.capture_script_state()?;
            
            // Restore preserved state BEFORE executing new script
            self.restore_script_state(preserved_globals)?;
            
            // Compile and execute new script
            let chunk = self.lua.load(&new_source)
                .set_name(&*file_path.to_string_lossy())
                .into_function()
                .map_err(|e| ScriptError::CompilationError(format!("Failed to compile reloaded script: {}", e)))?;
            
            chunk.call::<()>(())
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to execute reloaded script: {}", e)))?;
            
            // Update script info
            if let Some(script) = self.scripts.get_mut(&script_id) {
                script.source = new_source;
                script.last_modified = Some(new_modified);
            }
        }
        
        Ok(())
    }

    /// Capture important global state for preservation during reload
    fn capture_script_state(&self) -> ScriptResult<HashMap<String, Value>> {
        let mut preserved = HashMap::new();
        
        // Capture persistent_data if it exists
        if let Ok(persistent_data) = self.lua.globals().get::<Value>("persistent_data") {
            preserved.insert("persistent_data".to_string(), persistent_data);
        }
        
        // Capture reload_count if it exists
        if let Ok(reload_count) = self.lua.globals().get::<Value>("reload_count") {
            preserved.insert("reload_count".to_string(), reload_count);
        }
        
        Ok(preserved)
    }

    /// Restore preserved global state after reload
    fn restore_script_state(&self, preserved: HashMap<String, Value>) -> ScriptResult<()> {
        for (key, value) in preserved {
            self.lua.globals().set(key, value)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to restore state: {}", e)))?;
        }
        Ok(())
    }

    /// Execute a specific lifecycle method on a script
    pub fn execute_script_lifecycle_method(&mut self, script_id: ScriptId, method_name: &str, args: Vec<String>) -> ScriptResult<()> {
        // Check if script exists
        if !self.scripts.contains_key(&script_id) {
            return Err(ScriptError::NotFound(format!("Script {:?} not found", script_id)));
        }

        // Convert string args to Lua values
        let lua_args: Vec<mlua::Value> = args.into_iter()
            .map(|arg| {
                // Try to parse as number first, fallback to string
                if let Ok(num) = arg.parse::<f64>() {
                    mlua::Value::Number(num)
                } else {
                    mlua::Value::String(self.lua.create_string(&arg).unwrap())
                }
            })
            .collect();

        // Look for a module/table that was returned by the script
        // In typical Lua patterns, scripts return a table with methods
        let script_module: mlua::Table = self.lua.globals().get("_LAST_SCRIPT_MODULE")
            .or_else(|_| {
                // Fallback: try to find the method as a global function
                if let Ok(func) = self.lua.globals().get::<mlua::Function>(method_name) {
                    // Create a temporary module table
                    let temp_module = self.lua.create_table()?;
                    temp_module.set(method_name, func)?;
                    Ok(temp_module)
                } else {
                    Err(mlua::Error::RuntimeError(format!("No script module or global function '{}' found", method_name)))
                }
            })
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get script module: {}", e)))?;

        // Get the method from the module
        if let Ok(method) = script_module.get::<mlua::Function>(method_name) {
            // Create script instance context (self parameter)
            let instance = self.lua.create_table()
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
            
            // Call the method with instance as first parameter
            let mut call_args = vec![mlua::Value::Table(instance)];
            call_args.extend(lua_args);
            
            method.call::<mlua::MultiValue>(mlua::MultiValue::from_vec(call_args))
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to call {}: {}", method_name, e)))?;
        } else {
            // Method doesn't exist - this is OK for optional lifecycle methods
            debug!("Lifecycle method '{}' not found in script {:?}", method_name, script_id);
        }

        Ok(())
    }

    /// Execute update lifecycle for all entities with LuaScript components
    pub fn update_script_systems(&mut self, world: Arc<Mutex<World>>, delta_time: f32) -> ScriptResult<()> {
        // Get all entities with LuaScript components
        let entities_with_scripts = {
            let world_lock = world.lock().unwrap();
            world_lock.query_legacy::<crate::components::LuaScript>()
                .filter(|(_, script)| script.enabled)
                .map(|(entity, script)| (entity, script.clone()))
                .collect::<Vec<_>>()
        };

        // Execute update for each script
        for (entity, lua_script) in entities_with_scripts {
            // Load script if not already loaded
            if !self.scripts.values().any(|s| s.metadata.path == lua_script.script_path) {
                if let Ok(script_id) = self.load_script_from_file(std::path::Path::new(&lua_script.script_path)) {
                    // Set up entity context for the script
                    self.setup_script_entity_context(script_id, entity, world.clone())?;
                    
                    // Call init if it's the first time loading
                    self.execute_script_lifecycle_method(script_id, "init", vec![]).ok();
                }
            }

            // Find the script ID for this path
            let script_id = self.scripts.iter()
                .find(|(_, s)| s.metadata.path == lua_script.script_path)
                .map(|(id, _)| *id);
            
            if let Some(script_id) = script_id {
                // Set up entity context
                self.setup_script_entity_context(script_id, entity, world.clone())?;
                
                // Call update method
                self.execute_script_lifecycle_method(script_id, "update", vec![delta_time.to_string()]).ok();
            }
        }

        Ok(())
    }

    /// Execute update lifecycle for all entities with LuaScript components in execution order
    pub fn update_script_systems_ordered(&mut self, world: Arc<Mutex<World>>, delta_time: f32) -> ScriptResult<()> {
        // Get all entities with LuaScript components and sort by execution order
        let mut entities_with_scripts = {
            let world_lock = world.lock().unwrap();
            world_lock.query_legacy::<crate::components::LuaScript>()
                .filter(|(_, script)| script.enabled)
                .map(|(entity, script)| (entity, script.clone()))
                .collect::<Vec<_>>()
        };

        // Sort by execution order (lower numbers execute first), then by entity ID for deterministic ordering
        entities_with_scripts.sort_by(|(entity_a, script_a), (entity_b, script_b)| {
            script_a.execution_order.cmp(&script_b.execution_order)
                .then_with(|| entity_a.id().cmp(&entity_b.id()))
        });

        // Execute update for each script in order
        for (entity, lua_script) in entities_with_scripts {
            // Load script if not already loaded
            if !self.scripts.values().any(|s| s.metadata.path == lua_script.script_path) {
                if let Ok(script_id) = self.load_script_from_file(std::path::Path::new(&lua_script.script_path)) {
                    // Set up entity context for the script
                    self.setup_script_entity_context(script_id, entity, world.clone())?;
                    
                    // Call init if it's the first time loading
                    self.execute_script_lifecycle_method(script_id, "init", vec![]).ok();
                }
            }

            // Find the script ID for this path
            let script_id = self.scripts.iter()
                .find(|(_, s)| s.metadata.path == lua_script.script_path)
                .map(|(id, _)| *id);
            
            if let Some(script_id) = script_id {
                // Set up entity context
                self.setup_script_entity_context(script_id, entity, world.clone())?;
                
                // Call update method
                self.execute_script_lifecycle_method(script_id, "update", vec![delta_time.to_string()]).ok();
            }
        }

        Ok(())
    }

    /// Set up entity context for a script instance
    fn setup_script_entity_context(&mut self, _script_id: ScriptId, entity: engine_ecs_core::Entity, world: Arc<Mutex<World>>) -> ScriptResult<()> {
        // Create entity wrapper for the script
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world.clone(),
        };

        // Store in a global that the script can access as `self.entity`
        self.lua.globals().set("_CURRENT_SCRIPT_ENTITY", lua_entity)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set script entity context: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_engine_creation() {
        let engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
        assert!(engine.scripts.is_empty());
    }

    #[test]
    fn test_basic_script_execution() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create Lua engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(1),
            script_type: ScriptType::Lua,
            path: "test.lua".to_string(),
            entry_point: None,
        };

        let script = r#"
            function update()
                print("Hello from Lua!")
            end
        "#;

        engine.load_script_internal(metadata, script).expect("Failed to load script");
        engine.execute_script_internal(ScriptId(1)).expect("Failed to execute script");
    }
}

impl ScriptRuntime for LuaScriptEngine {
    fn initialize(&mut self) -> ScriptResult<()> {
        // Set up core bindings
        self.setup_core_bindings()?;
        
        // Set up additional bindings
        crate::lua::bindings::setup_math_bindings(&self.lua, &self.engine_table)?;
        crate::lua::bindings::setup_time_bindings(&self.lua, &self.engine_table)?;
        crate::lua::bindings::setup_input_bindings(&self.lua, &self.engine_table)?;
        crate::lua::bindings::setup_debug_bindings(&self.lua, &self.engine_table)?;
        crate::lua::events::setup_event_bindings(&self.lua, &self.engine_table)?;
        crate::lua::assets::setup_asset_bindings(&self.lua, &self.engine_table)?;
        
        info!("Lua runtime initialized with all bindings");
        Ok(())
    }
    
    fn load_script(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()> {
        self.load_script_internal(metadata, source)
    }
    
    fn execute_script(&mut self, id: ScriptId) -> ScriptResult<()> {
        self.execute_script_internal(id)
    }
    
    fn execute_function(&mut self, function_name: &str, args: Vec<String>) -> ScriptResult<String> {
        // Convert string args to Lua values
        let lua_args: Vec<Value> = args.into_iter()
            .map(|arg| Value::String(self.lua.create_string(&arg).unwrap()))
            .collect();
        
        let result = self.execute_function_internal(function_name, mlua::MultiValue::from_vec(lua_args))?;
        
        // Convert result to string
        Ok(format!("{:?}", result))
    }
    
    fn supports_type(&self, script_type: &ScriptType) -> bool {
        matches!(script_type, ScriptType::Lua)
    }
    
    fn update(&mut self, delta_time: f32) -> ScriptResult<()> {
        // Update time values
        if let Ok(time_table) = self.engine_table.get::<Table>("time") {
            time_table.set("delta_time", delta_time).ok();
            
            // Update total time
            if let Ok(total) = time_table.get::<f32>("total_time") {
                time_table.set("total_time", total + delta_time).ok();
            }
        }
        
        // Call update functions in all loaded scripts
        for (id, _) in self.scripts.iter() {
            // Try to call update function if it exists
            if let Ok(update_fn) = self.lua.globals().get::<Function>("update") {
                if let Err(e) = update_fn.call::<()>(delta_time) {
                    error!("Error in script {:?} update: {}", id, e);
                }
            }
        }
        
        Ok(())
    }
}