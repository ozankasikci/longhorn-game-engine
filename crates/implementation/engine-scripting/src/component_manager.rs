//! LuaScript component manager for entity script handling

use crate::{
    ScriptResult, ScriptFileManager, ScriptValidation, 
    components::LuaScript, lua::engine::LuaScriptEngine
};
use engine_ecs_core::World;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages LuaScript components and their execution
pub struct LuaScriptComponentManager {
    /// File manager for script validation and creation
    file_manager: ScriptFileManager,
    /// Lua script engine for execution
    script_engine: LuaScriptEngine,
    /// Cache of loaded scripts by path
    loaded_scripts: HashMap<String, crate::ScriptId>,
    /// Entity-specific script instances
    entity_scripts: HashMap<u64, EntityScriptInfo>,
}

/// Information about a script attached to an entity
#[derive(Debug, Clone)]
pub struct EntityScriptInfo {
    /// Entity ID
    pub entity_id: u64,
    /// Script ID in the engine
    pub script_id: Option<crate::ScriptId>,
    /// Current validation status
    pub validation_status: ScriptValidation,
    /// Last error message if any
    pub error_message: Option<String>,
    /// Whether the script is currently enabled
    pub enabled: bool,
    /// Execution order
    pub execution_order: i32,
}

/// Status of a script component
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptComponentStatus {
    /// Script is valid and loaded
    Ready,
    /// Script file not found
    FileNotFound,
    /// Script has syntax errors
    SyntaxError(String),
    /// Script is disabled
    Disabled,
    /// Script failed to load
    LoadError(String),
}

impl LuaScriptComponentManager {
    /// Create a new script component manager
    pub fn new(script_directory: PathBuf) -> ScriptResult<Self> {
        Ok(Self {
            file_manager: ScriptFileManager::new(script_directory)?,
            script_engine: LuaScriptEngine::new()?,
            loaded_scripts: HashMap::new(),
            entity_scripts: HashMap::new(),
        })
    }

    /// Get the script directory
    pub fn script_directory(&self) -> &std::path::Path {
        self.file_manager.script_directory()
    }

    /// Validate a script path for a LuaScript component
    pub fn validate_script_path(&mut self, script_path: &str) -> ScriptValidation {
        self.file_manager.validate_script(script_path)
    }

    /// Get the status of a script component
    pub fn get_script_status(&mut self, entity_id: u64, script_path: &str) -> ScriptComponentStatus {
        // Check if entity script info exists
        if let Some(info) = self.entity_scripts.get(&entity_id) {
            if !info.enabled {
                return ScriptComponentStatus::Disabled;
            }
            
            if let Some(error) = &info.error_message {
                return ScriptComponentStatus::LoadError(error.clone());
            }
            
            match &info.validation_status {
                ScriptValidation::Valid => ScriptComponentStatus::Ready,
                ScriptValidation::NotFound => ScriptComponentStatus::FileNotFound,
                ScriptValidation::SyntaxError(err) => ScriptComponentStatus::SyntaxError(err.clone()),
                ScriptValidation::Empty => ScriptComponentStatus::SyntaxError("Script file is empty".to_string()),
                ScriptValidation::InvalidExtension => ScriptComponentStatus::SyntaxError("Invalid file extension".to_string()),
                ScriptValidation::TooLarge(size) => ScriptComponentStatus::SyntaxError(format!("File too large: {} bytes", size)),
            }
        } else {
            // No entity info, validate script path
            match self.validate_script_path(script_path) {
                ScriptValidation::Valid => ScriptComponentStatus::Ready,
                ScriptValidation::NotFound => ScriptComponentStatus::FileNotFound,
                ScriptValidation::SyntaxError(err) => ScriptComponentStatus::SyntaxError(err),
                ScriptValidation::Empty => ScriptComponentStatus::SyntaxError("Script file is empty".to_string()),
                ScriptValidation::InvalidExtension => ScriptComponentStatus::SyntaxError("Invalid file extension".to_string()),
                ScriptValidation::TooLarge(size) => ScriptComponentStatus::SyntaxError(format!("File too large: {} bytes", size)),
            }
        }
    }

    /// Register a LuaScript component with an entity
    pub fn register_script_component(&mut self, entity_id: u64, lua_script: &LuaScript) -> ScriptResult<()> {
        let validation_status = self.validate_script_path(&lua_script.script_path);
        let is_valid = matches!(validation_status, ScriptValidation::Valid);
        
        let info = EntityScriptInfo {
            entity_id,
            script_id: None,
            validation_status,
            error_message: None,
            enabled: lua_script.enabled,
            execution_order: lua_script.execution_order,
        };
        
        self.entity_scripts.insert(entity_id, info);
        
        // Try to load the script if it's valid
        if is_valid && lua_script.enabled {
            self.load_script_for_entity(entity_id, &lua_script.script_path)?;
        }
        
        Ok(())
    }

    /// Unregister a script component from an entity
    pub fn unregister_script_component(&mut self, entity_id: u64) -> ScriptResult<()> {
        if let Some(info) = self.entity_scripts.remove(&entity_id) {
            // Clean up loaded script if it was the only user
            if let Some(script_id) = info.script_id {
                // Check if any other entities are using this script
                let other_users = self.entity_scripts.values()
                    .any(|other_info| other_info.script_id == Some(script_id));
                
                if !other_users {
                    // Remove from engine (would need method in LuaScriptEngine)
                    // self.script_engine.unload_script(script_id)?;
                }
            }
        }
        
        Ok(())
    }

    /// Load a script for a specific entity
    pub fn load_script_for_entity(&mut self, entity_id: u64, script_path: &str) -> ScriptResult<()> {
        // Check if script is already loaded
        let script_id = if let Some(&existing_id) = self.loaded_scripts.get(script_path) {
            existing_id
        } else {
            // Load the script
            let full_path = self.file_manager.script_directory().join(script_path);
            let new_id = self.script_engine.load_script_from_file(&full_path)?;
            self.loaded_scripts.insert(script_path.to_string(), new_id);
            new_id
        };
        
        // Update entity info
        if let Some(info) = self.entity_scripts.get_mut(&entity_id) {
            info.script_id = Some(script_id);
            info.error_message = None;
        }
        
        Ok(())
    }

    /// Create a new script file with template
    pub fn create_script_file(&self, relative_path: &str, template_name: Option<&str>) -> ScriptResult<PathBuf> {
        self.file_manager.create_script_file(relative_path, template_name)
    }

    /// List all available script files
    pub fn list_available_scripts(&self) -> ScriptResult<Vec<String>> {
        self.file_manager.list_script_files()
    }

    /// Update all script components (call this from game loop)
    pub fn update_scripts(&mut self, world: Arc<Mutex<World>>, delta_time: f32) -> ScriptResult<()> {
        // Get sorted list of entities by execution order
        let mut sorted_entities: Vec<_> = self.entity_scripts.iter()
            .filter(|(_, info)| info.enabled && info.script_id.is_some())
            .map(|(&entity_id, info)| (entity_id, info.execution_order, info.script_id.unwrap()))
            .collect();
        
        sorted_entities.sort_by_key(|(_, order, _)| *order);
        
        // Execute scripts in order
        for (_entity_id, _, _script_id) in sorted_entities {
            // Use the engine's update_script_systems method instead
            // This will properly handle entity context setup
        }
        
        // Use the engine's built-in update system
        self.script_engine.update_script_systems_ordered(world, delta_time)?;
        
        Ok(())
    }

    /// Initialize script for an entity (call init lifecycle method)
    pub fn initialize_script(&mut self, _entity_id: u64, _world: Arc<Mutex<World>>) -> ScriptResult<()> {
        // Script initialization is handled automatically by the engine when scripts are loaded
        // The engine calls init() when a script is first executed
        Ok(())
    }

    /// Update script component (when component properties change)
    pub fn update_script_component(&mut self, entity_id: u64, lua_script: &LuaScript) -> ScriptResult<()> {
        // First validate the script path
        let validation_status = self.validate_script_path(&lua_script.script_path);
        let is_valid = matches!(validation_status, ScriptValidation::Valid);
        
        if let Some(info) = self.entity_scripts.get_mut(&entity_id) {
            let old_enabled = info.enabled;
            
            // Update properties
            info.enabled = lua_script.enabled;
            info.execution_order = lua_script.execution_order;
            info.validation_status = validation_status;
            
            // Handle enable/disable state changes
            if lua_script.enabled && !old_enabled && is_valid {
                // Script was enabled, try to load it
                self.load_script_for_entity(entity_id, &lua_script.script_path)?;
            } else if !lua_script.enabled && old_enabled {
                // Script was disabled, clean up
                info.script_id = None;
                info.error_message = None;
            }
        } else {
            // No existing info, register as new
            self.register_script_component(entity_id, lua_script)?;
        }
        
        Ok(())
    }

    /// Get detailed information about an entity's script
    pub fn get_entity_script_info(&self, entity_id: u64) -> Option<&EntityScriptInfo> {
        self.entity_scripts.get(&entity_id)
    }

    /// Clear all cached data
    pub fn clear_cache(&mut self) {
        self.file_manager.clear_cache();
    }

    /// Check for script file changes and reload if necessary
    pub fn check_for_script_changes(&mut self) -> ScriptResult<()> {
        self.script_engine.check_and_reload_scripts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_manager() -> (LuaScriptComponentManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = LuaScriptComponentManager::new(temp_dir.path().to_path_buf())
            .expect("Failed to create component manager");
        (manager, temp_dir)
    }

    #[test]
    fn test_component_manager_creation() {
        let (manager, _temp_dir) = create_test_manager();
        assert!(manager.entity_scripts.is_empty());
        assert!(manager.loaded_scripts.is_empty());
    }

    #[test]
    fn test_validate_script_path() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Test nonexistent script
        let result = manager.validate_script_path("nonexistent.lua");
        assert_eq!(result, ScriptValidation::NotFound);
        
        // Create a valid script
        let script_path = temp_dir.path().join("test_script.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function update() end").expect("Failed to write file");
        
        // Test valid script
        let result = manager.validate_script_path("test_script.lua");
        assert_eq!(result, ScriptValidation::Valid);
    }

    #[test]
    fn test_get_script_status_file_not_found() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let status = manager.get_script_status(1, "missing.lua");
        assert_eq!(status, ScriptComponentStatus::FileNotFound);
    }

    #[test]
    fn test_get_script_status_valid_file() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create valid script
        let script_path = temp_dir.path().join("valid.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function update() print('test') end").expect("Failed to write file");
        
        let status = manager.get_script_status(1, "valid.lua");
        assert_eq!(status, ScriptComponentStatus::Ready);
    }

    #[test]
    fn test_register_script_component() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create valid script
        let script_path = temp_dir.path().join("component_test.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function init() print('initialized') end").expect("Failed to write file");
        
        let lua_script = LuaScript {
            script_path: "component_test.lua".to_string(),
            enabled: true,
            instance_id: None,
            execution_order: 0,
        };
        
        let result = manager.register_script_component(1, &lua_script);
        assert!(result.is_ok());
        
        // Check that entity info was created
        let info = manager.get_entity_script_info(1);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.entity_id, 1);
        assert!(info.enabled);
        assert_eq!(info.execution_order, 0);
    }

    #[test]
    fn test_register_script_component_disabled() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let lua_script = LuaScript {
            script_path: "disabled_script.lua".to_string(),
            enabled: false,
            instance_id: None,
            execution_order: 0,
        };
        
        let result = manager.register_script_component(1, &lua_script);
        assert!(result.is_ok());
        
        let status = manager.get_script_status(1, "disabled_script.lua");
        assert_eq!(status, ScriptComponentStatus::Disabled);
    }

    #[test]
    fn test_unregister_script_component() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create and register script
        let script_path = temp_dir.path().join("unregister_test.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function init() end").expect("Failed to write file");
        
        let lua_script = LuaScript {
            script_path: "unregister_test.lua".to_string(),
            enabled: true,
            instance_id: None,
            execution_order: 0,
        };
        
        manager.register_script_component(1, &lua_script).expect("Failed to register");
        assert!(manager.get_entity_script_info(1).is_some());
        
        // Unregister
        let result = manager.unregister_script_component(1);
        assert!(result.is_ok());
        assert!(manager.get_entity_script_info(1).is_none());
    }

    #[test]
    fn test_create_script_file() {
        let (manager, _temp_dir) = create_test_manager();
        
        let result = manager.create_script_file("new_test_script.lua", Some("entity"));
        assert!(result.is_ok());
        
        let created_path = result.unwrap();
        assert!(created_path.exists());
        
        // Verify it's a valid Lua file
        let content = std::fs::read_to_string(&created_path).expect("Failed to read created file");
        assert!(content.contains("function"));
    }

    #[test]
    fn test_list_available_scripts() {
        let (manager, temp_dir) = create_test_manager();
        
        // Create some test scripts
        let script1_path = temp_dir.path().join("script1.lua");
        File::create(&script1_path).expect("Failed to create file");
        
        let script2_path = temp_dir.path().join("script2.lua");
        File::create(&script2_path).expect("Failed to create file");
        
        let scripts = manager.list_available_scripts().expect("Failed to list scripts");
        assert_eq!(scripts.len(), 2);
        assert!(scripts.contains(&"script1.lua".to_string()));
        assert!(scripts.contains(&"script2.lua".to_string()));
    }

    #[test]
    fn test_update_script_component() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create valid script
        let script_path = temp_dir.path().join("update_test.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function update() end").expect("Failed to write file");
        
        // Register initial component
        let lua_script = LuaScript {
            script_path: "update_test.lua".to_string(),
            enabled: false,
            instance_id: None,
            execution_order: 0,
        };
        
        manager.register_script_component(1, &lua_script).expect("Failed to register");
        
        // Update to enable the script
        let updated_script = LuaScript {
            script_path: "update_test.lua".to_string(),
            enabled: true,
            instance_id: None,
            execution_order: 5,
        };
        
        let result = manager.update_script_component(1, &updated_script);
        assert!(result.is_ok());
        
        let info = manager.get_entity_script_info(1).unwrap();
        assert!(info.enabled);
        assert_eq!(info.execution_order, 5);
    }

    #[test]
    fn test_script_status_syntax_error() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create script with syntax error
        let script_path = temp_dir.path().join("syntax_error.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function broken_syntax(").expect("Failed to write file");
        
        let status = manager.get_script_status(1, "syntax_error.lua");
        match status {
            ScriptComponentStatus::SyntaxError(_) => (),
            _ => panic!("Expected syntax error, got: {:?}", status),
        }
    }
}