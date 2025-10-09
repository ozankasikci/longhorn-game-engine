//! Script API for engine integration

mod security;
pub use security::{ScriptApiConfig, ApiPermission, ScriptCapabilities, ApiRateLimiter, ApiInputValidator};

pub mod script_loader;
pub use script_loader::{ScriptLoader, ScriptLoadRequest, ScriptSource, ScriptHandle, ExecutionContext, AccessPermissions};

#[cfg(test)]
mod script_loader_tests;

#[cfg(test)]
mod registry_tests;

// New TypeScript API Registry System
pub mod registry;
pub mod bridge;
pub mod namespaces;
pub mod codegen;

pub use registry::*;
pub use bridge::*;

use crate::{ScriptError, ScriptId, ScriptResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Script API for exposing engine functionality to scripts
pub struct ScriptApi {
    config: ScriptApiConfig,
    rate_limiter: ApiRateLimiter,
    script_capabilities: HashMap<ScriptId, ScriptCapabilities>,
}

/// Main TypeScript API system that coordinates all components
pub struct TypeScriptApiSystem {
    registry: Arc<ApiRegistry>,
    bridge: V8ApiBridge,
}

impl TypeScriptApiSystem {
    pub fn new() -> Result<Self, ApiError> {
        let mut registry = ApiRegistry::new();

        // Register core namespaces
        namespaces::register_engine_world(&mut registry)?;
        namespaces::register_engine_math(&mut registry)?;
        namespaces::register_engine_debug(&mut registry)?;

        // Finalize registry
        registry.finalize()?;

        let registry = Arc::new(registry);
        let bridge = V8ApiBridge::new(registry.clone())?;

        Ok(Self { registry, bridge })
    }

    pub fn execute_script(&mut self, script_source: &str, entity_context: Option<u32>) -> Result<String, ApiError> {
        self.bridge.execute_script(script_source, entity_context)
    }

    pub fn generate_type_definitions(&self) -> String {
        let generator = codegen::TypeScriptGenerator::new(self.registry.clone());
        generator.generate_definitions()
    }

    pub fn get_registry(&self) -> &Arc<ApiRegistry> {
        &self.registry
    }

    pub fn get_bridge(&self) -> &V8ApiBridge {
        &self.bridge
    }
}

impl Default for ScriptApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptApi {
    /// Create a new script API
    pub fn new() -> Self {
        Self {
            config: ScriptApiConfig::new(),
            rate_limiter: ApiRateLimiter::new(),
            script_capabilities: HashMap::new(),
        }
    }
    
    /// Check if a script has permission for an API call
    pub fn check_permission(&self, script_id: ScriptId, api_name: &str, permission: ApiPermission) -> ScriptResult<()> {
        // Check if function is allowed
        if !self.config.is_function_allowed(api_name) {
            return Err(ScriptError::PermissionDenied {
                script_id,
                resource: api_name.split('.').next().unwrap_or("unknown").to_string(),
                action: api_name.to_string(),
                required_permission: format!("{:?}", permission),
            });
        }
        
        // Check if script has the required permission
        if let Some(capabilities) = self.script_capabilities.get(&script_id) {
            let capability_name = match permission {
                ApiPermission::FileRead => "file_read",
                ApiPermission::FileWrite => "file_write",
                ApiPermission::ConsoleWrite => "console_write",
                ApiPermission::EntityRead => "entity_read",
                ApiPermission::EntityWrite => "entity_write",
                _ => "unknown",
            };
            
            if !capabilities.has_capability(capability_name) {
                return Err(ScriptError::PermissionDenied {
                    script_id,
                    resource: api_name.split('.').next().unwrap_or("unknown").to_string(),
                    action: api_name.to_string(),
                    required_permission: capability_name.to_string(),
                });
            }
        } else {
            // No capabilities registered means no permissions
            return Err(ScriptError::PermissionDenied {
                script_id,
                resource: api_name.split('.').next().unwrap_or("unknown").to_string(),
                action: api_name.to_string(),
                required_permission: format!("{:?}", permission),
            });
        }
        
        // Check rate limits
        if let Err(e) = self.rate_limiter.check_rate_limit(api_name) {
            return Err(ScriptError::ResourceLimitExceeded {
                script_id,
                limit_type: "api_rate_limit".to_string(),
                limit_value: "rate limit".to_string(),
                actual_value: e,
            });
        }
        
        Ok(())
    }
    
    /// Register script capabilities
    pub fn register_script_capabilities(&mut self, script_id: ScriptId, capabilities: ScriptCapabilities) {
        self.script_capabilities.insert(script_id, capabilities);
    }
}
