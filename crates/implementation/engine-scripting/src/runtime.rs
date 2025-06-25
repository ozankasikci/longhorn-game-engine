//! Script runtime trait and implementations

use crate::{ScriptError, ScriptResult, ScriptId, ScriptMetadata, ScriptType};

/// Trait for script runtime implementations
pub trait ScriptRuntime {
    /// Initialize the runtime
    fn initialize(&mut self) -> ScriptResult<()>;
    
    /// Load a script into the runtime
    fn load_script(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()>;
    
    /// Execute a loaded script by ID
    fn execute_script(&mut self, id: ScriptId) -> ScriptResult<()>;
    
    /// Execute a specific function in the runtime
    fn execute_function(&mut self, function_name: &str, args: Vec<String>) -> ScriptResult<String>;
    
    /// Check if runtime supports a script type
    fn supports_type(&self, script_type: &ScriptType) -> bool;
    
    /// Update runtime state (called each frame)
    fn update(&mut self, delta_time: f32) -> ScriptResult<()>;
}

/// Factory for creating script runtimes
pub fn create_runtime(script_type: ScriptType) -> ScriptResult<Box<dyn ScriptRuntime>> {
    match script_type {
        ScriptType::Lua => {
            let mut engine = crate::lua::LuaScriptEngine::new()?;
            engine.initialize()?;
            Ok(Box::new(engine))
        }
        _ => Err(ScriptError::RuntimeError(format!(
            "Script type {:?} not yet implemented",
            script_type
        ))),
    }
}