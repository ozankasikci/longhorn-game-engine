//! Common components for scripting tests and examples

use engine_component_traits::impl_component;

/// Transform component for position, rotation, and scale
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl_component!(Transform);

/// Health component
#[derive(Debug, Clone, PartialEq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl_component!(Health);

/// LuaScript component for attaching Lua scripts to entities
#[derive(Debug, Clone, PartialEq)]
pub struct LuaScript {
    /// Path to the Lua script file
    pub script_path: String,
    /// Whether the script is enabled and should receive updates
    pub enabled: bool,
    /// Script instance ID (for tracking multiple instances of the same script)
    pub instance_id: Option<u64>,
    /// Execution order priority (lower numbers execute first)
    pub execution_order: i32,
}

impl LuaScript {
    pub fn new(script_path: String) -> Self {
        Self {
            script_path,
            enabled: true,
            instance_id: None,
            execution_order: 0,
        }
    }
    
    pub fn with_execution_order(script_path: String, execution_order: i32) -> Self {
        Self {
            script_path,
            enabled: true,
            instance_id: None,
            execution_order,
        }
    }
}

impl_component!(LuaScript);