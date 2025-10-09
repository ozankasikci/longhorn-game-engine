//! Standardized script loading interface

use crate::ScriptError;
use std::path::PathBuf;
use engine_ecs_core::Entity;

/// Handle to a loaded script
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptHandle(pub u64);

/// Source of a script
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptSource {
    /// Load from file
    File(PathBuf),
    /// Load from string with a name
    String { 
        content: String, 
        name: String 
    },
    /// Load from compiled bytecode
    Bytecode(Vec<u8>),
}

/// Execution context for scripts
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Permissions granted to the script
    permissions: Vec<String>,
    /// Environment variables available to the script
    environment: std::collections::HashMap<String, String>,
    /// Maximum execution time in milliseconds
    timeout_ms: Option<u64>,
}

impl ExecutionContext {
    /// Create a new execution context with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific permissions
    pub fn with_permissions(permissions: Vec<String>) -> Self {
        Self {
            permissions,
            ..Default::default()
        }
    }

    /// Add a permission
    pub fn add_permission(&mut self, permission: &str) {
        if !self.permissions.contains(&permission.to_string()) {
            self.permissions.push(permission.to_string());
        }
    }

    /// Check if a permission is granted
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Set execution timeout
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = Some(timeout_ms);
    }

    /// Get execution timeout
    pub fn timeout(&self) -> Option<u64> {
        self.timeout_ms
    }

    /// Set an environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }

    /// Get an environment variable
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(|s| s.as_str())
    }
}

/// Request to load a script
#[derive(Debug, Clone)]
pub struct ScriptLoadRequest {
    /// Source of the script
    pub source: ScriptSource,
    /// Optional entity to bind the script to
    pub entity_binding: Option<Entity>,
    /// Execution context for the script
    pub execution_context: ExecutionContext,
}

/// Trait for script loading
pub trait ScriptLoader {
    /// Load a script with the given request
    fn load_script(&mut self, request: ScriptLoadRequest) -> Result<ScriptHandle, ScriptError>;
    
    /// Unload a script by handle
    fn unload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError>;
    
    /// Reload a script by handle
    fn reload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError>;
}

/// Access permissions for scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessPermissions {
    pub can_read_components: bool,
    pub can_write_components: bool,
    pub can_create_entities: bool,
    pub can_destroy_entities: bool,
    pub can_access_resources: bool,
    pub can_emit_events: bool,
}

impl AccessPermissions {
    /// Create with all permissions
    pub fn all() -> Self {
        Self {
            can_read_components: true,
            can_write_components: true,
            can_create_entities: true,
            can_destroy_entities: true,
            can_access_resources: true,
            can_emit_events: true,
        }
    }

    /// Create with read-only permissions
    pub fn read_only() -> Self {
        Self {
            can_read_components: true,
            can_write_components: false,
            can_create_entities: false,
            can_destroy_entities: false,
            can_access_resources: false,
            can_emit_events: false,
        }
    }

    /// Create with no permissions
    pub fn none() -> Self {
        Self {
            can_read_components: false,
            can_write_components: false,
            can_create_entities: false,
            can_destroy_entities: false,
            can_access_resources: false,
            can_emit_events: false,
        }
    }

    /// Check if can read a specific component type
    pub fn can_read<T>(&self) -> bool {
        self.can_read_components
    }

    /// Check if can write a specific component type
    pub fn can_write<T>(&self) -> bool {
        self.can_write_components
    }
}