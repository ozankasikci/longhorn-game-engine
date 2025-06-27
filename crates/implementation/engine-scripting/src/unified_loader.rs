//! Unified script loader implementation

use crate::{
    api::{ScriptLoader, ScriptLoadRequest, ScriptSource, ScriptHandle, ExecutionContext, AccessPermissions},
    manager::ScriptManager,
    ScriptError, ScriptId,
};
use std::collections::HashMap;
use std::path::PathBuf;
use engine_ecs_core::Entity;

/// Unified script loader that integrates with the existing ScriptManager
pub struct UnifiedScriptLoader {
    /// Internal script manager
    manager: ScriptManager,
    /// Map from handles to script IDs
    handle_to_id: HashMap<ScriptHandle, ScriptId>,
    /// Map from script IDs to handles
    id_to_handle: HashMap<ScriptId, ScriptHandle>,
    /// Next handle ID
    next_handle: u64,
    /// Script metadata
    script_metadata: HashMap<ScriptHandle, ScriptLoadRequest>,
}

impl UnifiedScriptLoader {
    /// Create a new unified script loader
    pub fn new() -> Result<Self, ScriptError> {
        Ok(Self {
            manager: ScriptManager::new()?,
            handle_to_id: HashMap::new(),
            id_to_handle: HashMap::new(),
            next_handle: 1,
            script_metadata: HashMap::new(),
        })
    }

    /// Get the internal script manager
    pub fn manager(&self) -> &ScriptManager {
        &self.manager
    }

    /// Get the internal script manager mutably
    pub fn manager_mut(&mut self) -> &mut ScriptManager {
        &mut self.manager
    }

    /// Get script ID from handle
    pub fn get_script_id(&self, handle: ScriptHandle) -> Option<ScriptId> {
        self.handle_to_id.get(&handle).copied()
    }

    /// Get handle from script ID
    pub fn get_handle(&self, script_id: ScriptId) -> Option<ScriptHandle> {
        self.id_to_handle.get(&script_id).copied()
    }
}

impl ScriptLoader for UnifiedScriptLoader {
    fn load_script(&mut self, request: ScriptLoadRequest) -> Result<ScriptHandle, ScriptError> {
        // Extract source content and path
        let (content, path) = match &request.source {
            ScriptSource::File(path) => {
                // Read file content
                let content = std::fs::read_to_string(path)
                    .map_err(|e| ScriptError::NotFound(format!("Failed to read file: {}", e)))?;
                (content, path.to_string_lossy().to_string())
            }
            ScriptSource::String { content, name } => {
                (content.clone(), format!("inline:{}", name))
            }
            ScriptSource::Bytecode(_) => {
                return Err(ScriptError::InvalidScript {
                    script_name: "bytecode".to_string(),
                    reason: "Bytecode loading not yet implemented".to_string(),
                });
            }
        };

        // Load script with capabilities based on execution context
        let capabilities = crate::api::ScriptCapabilities::from_permissions(&request.execution_context);
        let script_ref = self.manager.load_script_with_capabilities(&path, &content, capabilities)?;
        
        // Create handle
        let handle = ScriptHandle(self.next_handle);
        self.next_handle += 1;
        
        // Store mappings
        self.handle_to_id.insert(handle, script_ref.id);
        self.id_to_handle.insert(script_ref.id, handle);
        self.script_metadata.insert(handle, request);
        
        Ok(handle)
    }

    fn unload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError> {
        let script_id = *self.handle_to_id.get(&handle)
            .ok_or(ScriptError::InvalidHandle { handle })?;
        
        // Unload from manager
        self.manager.unload_script(script_id)?;
        
        // Clean up mappings
        self.handle_to_id.remove(&handle);
        self.id_to_handle.remove(&script_id);
        self.script_metadata.remove(&handle);
        
        Ok(())
    }

    fn reload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError> {
        let script_id = self.handle_to_id.get(&handle)
            .ok_or(ScriptError::InvalidHandle { handle })?;
        
        let request = self.script_metadata.get(&handle)
            .ok_or(ScriptError::InvalidHandle { handle })?;
        
        // Re-read source
        let new_content = match &request.source {
            ScriptSource::File(path) => {
                std::fs::read_to_string(path)
                    .map_err(|e| ScriptError::NotFound(format!("Failed to read file: {}", e)))?
            }
            ScriptSource::String { content, .. } => content.clone(),
            ScriptSource::Bytecode(_) => {
                return Err(ScriptError::InvalidScript {
                    script_name: "bytecode".to_string(),
                    reason: "Bytecode reloading not yet implemented".to_string(),
                });
            }
        };
        
        // Reload in manager
        self.manager.reload_script(*script_id, &new_content)?;
        
        Ok(())
    }
}

// Extension to create capabilities from execution context
impl crate::api::ScriptCapabilities {
    /// Create capabilities from execution context permissions
    pub fn from_permissions(context: &ExecutionContext) -> Self {
        let mut capabilities = Self::new();
        
        // Map common permissions
        if context.has_permission("file_read") {
            capabilities.add_capability("file_read");
        }
        if context.has_permission("file_write") {
            capabilities.add_capability("file_write");
        }
        if context.has_permission("entity_read") {
            capabilities.add_capability("entity_read");
        }
        if context.has_permission("entity_write") {
            capabilities.add_capability("entity_write");
        }
        if context.has_permission("console_write") {
            capabilities.add_capability("console_write");
        }
        
        capabilities
    }
}