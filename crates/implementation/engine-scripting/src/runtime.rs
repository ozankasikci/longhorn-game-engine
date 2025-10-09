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
            Err(ScriptError::runtime(
                "Lua support has been removed from the engine".to_string()
            ))
        }
        ScriptType::TypeScript => {
            Err(ScriptError::runtime(
                "TypeScript support is available in the engine-scripting-typescript crate".to_string()
            ))
        }
        _ => Err(ScriptError::runtime(format!(
            "Script type {:?} not yet implemented",
            script_type
        ))),
    }
}