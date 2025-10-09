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


/// TypeScriptScript component for attaching TypeScript scripts to entities
/// Similar to LuaScript but for TypeScript/JavaScript execution
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeScriptScript {
    /// Path to the TypeScript script file (primary script for backward compatibility)
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

impl TypeScriptScript {
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
    
    /// Get the path of the primary script
    pub fn get_path(&self) -> &str {
        &self.script_path
    }
    
    /// Check if the script is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get the execution order
    pub fn get_execution_order(&self) -> i32 {
        self.execution_order
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
    
    /// Get the file extension for TypeScript scripts
    pub fn get_file_extension(&self) -> &str {
        "ts"
    }
}

impl_component!(TypeScriptScript);

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

