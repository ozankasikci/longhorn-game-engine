//! TypeScript/JavaScript bindings for engine APIs
//! 
//! This module will provide JavaScript bindings for all engine APIs,
//! similar to the Lua bindings but adapted for JavaScript/TypeScript.
//! 
//! TODO: Implement JavaScript API bindings for:
//! - Input system
//! - Physics system  
//! - Event system
//! - Debugging tools
//! - Performance profiling

use engine_scripting::{ScriptError, ScriptResult};

pub mod ecs;
pub mod events;
pub mod input;
pub mod physics;

/// JavaScript API bindings manager
pub struct TypeScriptBindings {
    // TODO: Add fields for managing API bindings
}

impl TypeScriptBindings {
    /// Create new TypeScript bindings
    pub fn new() -> Self {
        Self {
            // TODO: Initialize binding state
        }
    }
    
    /// Register all engine APIs with the JavaScript context
    pub fn register_apis(&self, scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> ScriptResult<()> {
        // Register ECS API
        ecs::register_ecs_api(scope, global)?;
        
        // Register Input API
        input::register_input_api(scope, global)?;
        
        // Register Physics API
        physics::register_physics_api(scope, global)?;
        
        // Register Events API
        events::register_events_api(scope, global)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod ecs_tests;

#[cfg(test)]
mod events_tests;

#[cfg(test)]
mod input_tests;

#[cfg(test)]
mod physics_tests;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bindings_creation() {
        let _bindings = TypeScriptBindings::new();
        // Basic test that bindings can be created
        // TODO: Add more comprehensive tests when APIs are implemented
    }
}