//! Script manager for coordinating script lifecycle
//! This is the SINGLE SOURCE OF TRUTH for script storage and management

use crate::{ScriptError, ScriptId, ScriptResult, ScriptMetadata, ScriptType};
use crate::api::ScriptCapabilities;
use std::collections::HashMap;

/// Script reference that other components can use
#[derive(Debug, Clone)]
pub struct ScriptRef {
    pub id: ScriptId,
    pub metadata: ScriptMetadata,
    pub source: String,
}

/// Central script manager - SINGLE SOURCE OF TRUTH for scripts
pub struct ScriptManager {
    /// All scripts stored here and ONLY here
    scripts: HashMap<ScriptId, ScriptRef>,
    /// Script path to ID mapping
    path_to_id: HashMap<String, ScriptId>,
    /// Next script ID counter
    next_script_id: u32,
    /// Quarantined scripts (after security violations)
    quarantined_scripts: HashMap<ScriptId, String>, // script_id -> reason
    /// Script capabilities
    script_capabilities: HashMap<ScriptId, ScriptCapabilities>,
}

impl ScriptManager {
    /// Create a new script manager
    pub fn new() -> ScriptResult<Self> {
        Ok(Self {
            scripts: HashMap::new(),
            path_to_id: HashMap::new(),
            next_script_id: 1,
            quarantined_scripts: HashMap::new(),
            script_capabilities: HashMap::new(),
        })
    }

    /// Load a script - ONLY place where scripts are stored
    pub fn load_script(&mut self, path: &str, source: &str) -> ScriptResult<ScriptRef> {
        // Check if already loaded
        if let Some(&existing_id) = self.path_to_id.get(path) {
            if let Some(script) = self.scripts.get(&existing_id) {
                return Ok(script.clone());
            }
        }

        let script_id = ScriptId(self.next_script_id as u64);
        self.next_script_id += 1;

        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::Lua, // For now, assume Lua
            path: path.to_string(),
            entry_point: None,
        };

        let script_ref = ScriptRef {
            id: script_id,
            metadata,
            source: source.to_string(),
        };

        // Store in SINGLE location
        self.scripts.insert(script_id, script_ref.clone());
        self.path_to_id.insert(path.to_string(), script_id);

        Ok(script_ref)
    }

    /// Get a script by ID
    pub fn get_script(&self, script_id: ScriptId) -> Option<&ScriptRef> {
        self.scripts.get(&script_id)
    }

    /// Get a script by path
    pub fn get_script_by_path(&self, path: &str) -> Option<&ScriptRef> {
        self.path_to_id.get(path)
            .and_then(|id| self.scripts.get(id))
    }

    /// Unload a script
    pub fn unload_script(&mut self, script_id: ScriptId) -> ScriptResult<()> {
        if let Some(script) = self.scripts.remove(&script_id) {
            self.path_to_id.remove(&script.metadata.path);
            Ok(())
        } else {
            Err(ScriptError::NotFound(format!("Script ID {:?} not found", script_id)))
        }
    }

    /// Reload a script
    pub fn reload_script(&mut self, script_id: ScriptId, new_source: &str) -> ScriptResult<ScriptRef> {
        if let Some(script) = self.scripts.get_mut(&script_id) {
            script.source = new_source.to_string();
            Ok(script.clone())
        } else {
            Err(ScriptError::NotFound(format!("Script ID {:?} not found", script_id)))
        }
    }

    /// Get all loaded scripts
    pub fn get_all_scripts(&self) -> Vec<&ScriptRef> {
        self.scripts.values().collect()
    }

    /// Quarantine a script after security violation
    pub fn quarantine_script(&mut self, script_id: ScriptId, reason: String) {
        self.quarantined_scripts.insert(script_id, reason);
    }

    /// Check if a script is quarantined
    pub fn is_script_quarantined(&self, script_id: ScriptId) -> bool {
        self.quarantined_scripts.contains_key(&script_id)
    }

    /// Get quarantine reason
    pub fn get_quarantine_reason(&self, script_id: ScriptId) -> Option<&str> {
        self.quarantined_scripts.get(&script_id).map(|s| s.as_str())
    }

    /// Remove script from quarantine (after remediation)
    pub fn unquarantine_script(&mut self, script_id: ScriptId) -> bool {
        self.quarantined_scripts.remove(&script_id).is_some()
    }
    
    /// Load a script with declared capabilities
    pub fn load_script_with_capabilities(&mut self, path: &str, source: &str, capabilities: ScriptCapabilities) -> ScriptResult<ScriptRef> {
        let script_ref = self.load_script(path, source)?;
        self.script_capabilities.insert(script_ref.id, capabilities);
        Ok(script_ref)
    }
    
    /// Get script capabilities
    pub fn get_script_capabilities(&self, script_id: ScriptId) -> Option<&ScriptCapabilities> {
        self.script_capabilities.get(&script_id)
    }
}
