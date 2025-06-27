//! TypeScript scripting support for Longhorn Game Engine
//! 
//! This crate provides TypeScript execution capabilities using V8 JavaScript engine
//! with SWC for TypeScript compilation. It maintains the same security and performance
//! characteristics as the Lua implementation while providing access to the vast
//! JavaScript/TypeScript ecosystem.

pub mod engine;
pub mod compiler;
pub mod runtime;
pub mod bindings;
pub mod security;

// Re-export main types
pub use engine::{TypeScriptEngine, TypeScriptEngineConfig};
pub use compiler::{TypeScriptCompiler, CompilationResult, CompilerOptions};
pub use runtime::TypeScriptRuntime;

use engine_scripting::{ScriptError, ScriptResult};

/// Initialize V8 platform (should be called once at application startup)
pub fn initialize_v8_platform() -> ScriptResult<()> {
    static INIT: std::sync::Once = std::sync::Once::new();
    static mut INIT_RESULT: Option<ScriptResult<()>> = None;
    
    unsafe {
        INIT.call_once(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            INIT_RESULT = Some(Ok(()));
        });
        
        match INIT_RESULT.as_ref() {
            Some(result) => match result {
                Ok(()) => Ok(()),
                Err(_) => Err(ScriptError::InitializationError {
                    message: "V8 platform initialization failed".to_string(),
                    component: "TypeScript".to_string(),
                    source: None,
                }),
            },
            None => Err(ScriptError::InitializationError {
                message: "V8 platform not initialized".to_string(),
                component: "TypeScript".to_string(),
                source: None,
            }),
        }
    }
}

/// Shutdown V8 platform (should be called once at application shutdown)
pub fn shutdown_v8_platform() {
    unsafe {
        v8::V8::dispose();
    }
    v8::V8::dispose_platform();
}

#[cfg(test)]
mod hot_reload_tests;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_v8_initialization() {
        let result = initialize_v8_platform();
        assert!(result.is_ok(), "V8 platform should initialize successfully");
    }
}