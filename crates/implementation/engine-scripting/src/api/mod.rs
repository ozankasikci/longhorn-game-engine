//! Script API for engine integration

mod security;
pub use security::{ScriptApiConfig, ApiPermission, ScriptCapabilities, ApiRateLimiter, ApiInputValidator};

pub mod script_loader;
pub use script_loader::{ScriptLoader, ScriptLoadRequest, ScriptSource, ScriptHandle, ExecutionContext, AccessPermissions};

#[cfg(test)]
mod script_loader_tests;

use crate::{ScriptError, ScriptId, ScriptResult};
use std::collections::HashMap;

/// Script API for exposing engine functionality to scripts
pub struct ScriptApi {
    config: ScriptApiConfig,
    rate_limiter: ApiRateLimiter,
    script_capabilities: HashMap<ScriptId, ScriptCapabilities>,
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
