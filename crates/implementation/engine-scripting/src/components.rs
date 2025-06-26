//! Common components for scripting tests and examples

use engine_component_traits::impl_component;

/// Transform component for position, rotation, and scale
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3], // Changed to 3-component for simpler testing
    pub scale: [f32; 3],
}

impl Transform {
    pub fn new(pos: [f32; 3], rot: [f32; 3], scale: [f32; 3]) -> Self {
        Self {
            position: pos,
            rotation: rot,
            scale: scale,
        }
    }
    
    pub fn identity() -> Self {
        Self::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
    }
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
/// Note: Due to ECS limitations, only one LuaScript component per entity is supported.
/// To use multiple scripts, store multiple script paths in the scripts field.
#[derive(Debug, Clone, PartialEq)]
pub struct LuaScript {
    /// Path to the Lua script file (primary script for backward compatibility)
    pub script_path: String,
    /// Whether the script is enabled and should receive updates
    pub enabled: bool,
    /// Script instance ID (for tracking multiple instances of the same script)
    pub instance_id: Option<u64>,
    /// Execution order priority (lower numbers execute first)
    pub execution_order: i32,
    /// Additional script paths for multiple script support
    pub additional_scripts: Vec<String>,
}

impl LuaScript {
    pub fn new(script_path: String) -> Self {
        Self {
            script_path,
            enabled: true,
            instance_id: None,
            execution_order: 0,
            additional_scripts: Vec::new(),
        }
    }
    
    pub fn with_execution_order(script_path: String, execution_order: i32) -> Self {
        Self {
            script_path,
            enabled: true,
            instance_id: None,
            execution_order,
            additional_scripts: Vec::new(),
        }
    }
    
    /// Add an additional script to this component
    pub fn add_script(&mut self, script_path: String) {
        if !self.additional_scripts.contains(&script_path) && script_path != self.script_path {
            self.additional_scripts.push(script_path);
        }
    }
    
    /// Remove a script from this component
    pub fn remove_script(&mut self, script_path: &str) -> bool {
        if self.script_path == script_path {
            // Can't remove the primary script - just clear it if this is the only script
            if self.additional_scripts.is_empty() {
                return false; // Indicate that the component should be removed
            } else {
                // Replace primary with first additional script
                self.script_path = self.additional_scripts.remove(0);
                return true;
            }
        } else {
            // Remove from additional scripts
            if let Some(pos) = self.additional_scripts.iter().position(|s| s == script_path) {
                self.additional_scripts.remove(pos);
                return true;
            }
        }
        false
    }
    
    /// Get all script paths in this component
    pub fn get_all_scripts(&self) -> Vec<&String> {
        let mut scripts = vec![&self.script_path];
        scripts.extend(self.additional_scripts.iter());
        scripts
    }
    
    /// Get the number of scripts in this component
    pub fn script_count(&self) -> usize {
        1 + self.additional_scripts.len()
    }
}

impl_component!(LuaScript);

/// Velocity component for linear and angular movement
#[derive(Debug, Clone, PartialEq)]
pub struct Velocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

impl Velocity {
    pub fn new(linear: [f32; 3], angular: [f32; 3]) -> Self {
        Self { linear, angular }
    }
    
    pub fn zero() -> Self {
        Self::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
    }
}

impl_component!(Velocity);

