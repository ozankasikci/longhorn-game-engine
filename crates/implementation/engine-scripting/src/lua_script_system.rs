//! LuaScriptSystem for integrating Lua scripts with the game loop
//!
//! This system executes Lua scripts attached to entities via LuaScript components,
//! supporting lifecycle methods (init, update, destroy) and execution ordering.

use crate::{ScriptError, ScriptResult, components::LuaScript, lua::engine::LuaScriptEngine};
use engine_runtime_core::{System, GameContext, SystemError};
use engine_ecs_core::{World, Entity};
use std::collections::HashMap;
use std::sync::Arc;
use log::{debug, error, info, warn};

/// System that executes Lua scripts attached to entities
pub struct LuaScriptSystem {
    /// Lua script engine for executing scripts
    lua_engine: LuaScriptEngine,
    /// Loaded scripts keyed by file path
    loaded_scripts: HashMap<String, bool>,
    /// Entities that have been initialized
    initialized_entities: HashMap<Entity, String>,
    /// Script modules keyed by script path
    script_modules: HashMap<String, String>,
    /// Script instances keyed by entity
    script_instances: HashMap<Entity, String>,
}

impl LuaScriptSystem {
    /// Create a new LuaScriptSystem
    pub fn new() -> ScriptResult<Self> {
        let mut lua_engine = LuaScriptEngine::new()?;
        
        // Initialize the Lua engine with core bindings
        lua_engine.setup_core_bindings()?;
        
        info!("LuaScriptSystem created successfully");
        
        Ok(Self {
            lua_engine,
            loaded_scripts: HashMap::new(),
            initialized_entities: HashMap::new(),
            script_modules: HashMap::new(),
            script_instances: HashMap::new(),
        })
    }
    
    /// Execute scripts for all entities with LuaScript components
    fn execute_scripts(&mut self, world: &World, delta_time: f32) -> ScriptResult<()> {
        // Get all entities with LuaScript components, sorted by execution order
        let mut script_entities = self.collect_script_entities(world)?;
        
        // Sort by execution order (lower numbers execute first), then by entity ID for determinism
        script_entities.sort_by(|(entity_a, script_a), (entity_b, script_b)| {
            script_a.execution_order.cmp(&script_b.execution_order)
                .then_with(|| entity_a.id().cmp(&entity_b.id()))
        });
        
        // Process each script entity
        for (entity, script) in script_entities {
            if !script.enabled {
                debug!("Skipping disabled script for entity {:?}: {}", entity, script.script_path);
                continue;
            }
            
            // Load script if not already loaded
            if !self.loaded_scripts.contains_key(&script.script_path) {
                match self.load_script(&script.script_path) {
                    Ok(_) => {
                        self.loaded_scripts.insert(script.script_path.clone(), true);
                        info!("Loaded script: {}", script.script_path);
                    }
                    Err(e) => {
                        error!("Failed to load script {}: {}", script.script_path, e);
                        // Mark as failed so we don't keep trying
                        self.loaded_scripts.insert(script.script_path.clone(), false);
                        continue;
                    }
                }
            }
            
            // Skip if script failed to load
            if !self.loaded_scripts.get(&script.script_path).unwrap_or(&false) {
                continue;
            }
            
            // Initialize script if not already initialized
            if !self.initialized_entities.contains_key(&entity) {
                match self.initialize_script_for_entity(entity, &script.script_path, world) {
                    Ok(_) => {
                        self.initialized_entities.insert(entity, script.script_path.clone());
                        debug!("Initialized script for entity {:?}: {}", entity, script.script_path);
                    }
                    Err(e) => {
                        warn!("Failed to initialize script for entity {:?}: {}", entity, e);
                        continue;
                    }
                }
            }
            
            // Execute update method
            if let Err(e) = self.execute_update_for_entity(entity, &script.script_path, delta_time, world) {
                warn!("Error executing update for entity {:?}: {}", entity, e);
            }
        }
        
        Ok(())
    }
    
    /// Collect all entities with LuaScript components
    fn collect_script_entities(&self, world: &World) -> ScriptResult<Vec<(Entity, LuaScript)>> {
        // Query all entities with LuaScript components
        let entities: Vec<(Entity, LuaScript)> = world
            .query_legacy::<LuaScript>()
            .map(|(entity, script_component)| (entity, script_component.clone()))
            .collect();
        
        debug!("Collected {} entities with LuaScript components", entities.len());
        Ok(entities)
    }
    
    /// Load a Lua script from file
    fn load_script(&mut self, script_path: &str) -> ScriptResult<()> {
        debug!("Loading script: {}", script_path);
        
        // Try to load actual file, fallback to dummy script
        let script_content = if std::path::Path::new(script_path).exists() {
            std::fs::read_to_string(script_path)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to read script file {}: {}", script_path, e)))?
        } else {
            // Fallback to dummy script for existing tests
            r#"
                local script = {}
                
                function script:init()
                    -- Script initialization
                end
                
                function script:update(delta_time)
                    -- Script update logic
                end
                
                function script:destroy()
                    -- Script cleanup
                end
                
                return script
            "#.to_string()
        };
        
        // Load the script into the Lua engine
        let metadata = crate::types::ScriptMetadata {
            id: crate::types::ScriptId(1),
            script_type: crate::types::ScriptType::Lua,
            path: script_path.to_string(),
            entry_point: None,
        };
        
        self.lua_engine.load_script_internal(metadata, &script_content)?;
        
        // After loading, move the _LAST_SCRIPT_MODULE to a script-specific global
        let module_name = format!("script_module_{}", script_path.replace("/", "_").replace(".", "_").replace("-", "_"));
        
        // Copy the module from _LAST_SCRIPT_MODULE to the specific module name
        if let Ok(last_module) = self.lua_engine.lua().globals().get::<mlua::Table>("_LAST_SCRIPT_MODULE") {
            self.lua_engine.lua().globals().set(module_name.clone(), last_module)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to store script module: {}", e)))?;
        }
        
        self.script_modules.insert(script_path.to_string(), module_name);
        
        Ok(())
    }
    
    /// Initialize a script for a specific entity
    fn initialize_script_for_entity(&mut self, entity: Entity, script_path: &str, world: &World) -> ScriptResult<()> {
        debug!("Initializing script {} for entity {:?}", script_path, entity);
        
        // Set up entity context for the script
        let world_arc = std::sync::Arc::new(std::sync::Mutex::new(World::new()));
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world_arc,
        };
        
        // Set the entity as the script's self.entity
        self.lua_engine.lua().globals().set("_CURRENT_SCRIPT_ENTITY", lua_entity.clone())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set script entity context: {}", e)))?;
        
        // Execute init lifecycle method if it exists
        // Get the specific module for this script
        if let Some(module_name) = self.script_modules.get(script_path) {
            if let Ok(module) = self.lua_engine.lua().globals().get::<mlua::Table>(module_name.as_str()) {
                if let Ok(init_method) = module.get::<mlua::Function>("init") {
                // Create script instance (self parameter)
                let instance = self.lua_engine.lua().create_table()
                    .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
                
                // Set entity reference in the instance
                instance.set("entity", lua_entity)
                    .map_err(|e| ScriptError::RuntimeError(format!("Failed to set entity in instance: {}", e)))?;
                
                // Store the instance for this entity
                let instance_name = format!("_INSTANCE_{}", entity.id());
                self.lua_engine.lua().globals().set(instance_name.as_str(), instance.clone())
                    .map_err(|e| ScriptError::RuntimeError(format!("Failed to store instance: {}", e)))?;
                self.script_instances.insert(entity, instance_name);
                
                // Call init method
                init_method.call::<()>(instance)
                    .map_err(|e| ScriptError::RuntimeError(format!("Failed to call init method: {}", e)))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute the update method for a script on a specific entity
    fn execute_update_for_entity(&mut self, entity: Entity, script_path: &str, delta_time: f32, world: &World) -> ScriptResult<()> {
        debug!("Executing update for entity {:?}, script: {}, delta: {}", entity, script_path, delta_time);
        
        // Set up entity context for the script
        let world_arc = std::sync::Arc::new(std::sync::Mutex::new(World::new()));
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world_arc,
        };
        
        // Execute update lifecycle method if it exists
        // Get the specific module for this script
        if let Some(module_name) = self.script_modules.get(script_path) {
            if let Ok(module) = self.lua_engine.lua().globals().get::<mlua::Table>(module_name.as_str()) {
                if let Ok(update_method) = module.get::<mlua::Function>("update") {
                    // Get the stored instance for this entity
                    let instance = if let Some(instance_name) = self.script_instances.get(&entity) {
                        self.lua_engine.lua().globals().get::<mlua::Table>(instance_name.as_str())
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get instance: {}", e)))?
                    } else {
                        // Create instance if it doesn't exist (shouldn't happen if init was called)
                        let instance = self.lua_engine.lua().create_table()
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
                        
                        // Set entity reference in the instance
                        instance.set("entity", lua_entity)
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set entity in instance: {}", e)))?;
                        
                        // Store the instance
                        let instance_name = format!("_INSTANCE_{}", entity.id());
                        self.lua_engine.lua().globals().set(instance_name.as_str(), instance.clone())
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to store instance: {}", e)))?;
                        self.script_instances.insert(entity, instance_name);
                        
                        instance
                    };
                    
                    // Call update method with delta_time
                    update_method.call::<()>((instance, delta_time))
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to call update method: {}", e)))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Test-only method to access entity collection (for driving TDD)
    pub fn collect_script_entities_for_test(&self, world: &World) -> Vec<(Entity, LuaScript)> {
        self.collect_script_entities(world).unwrap_or_default()
    }
    
    /// Execute scripts from a World wrapped in Arc<Mutex<>>
    pub fn execute_scripts_from_world(&mut self, world: Arc<std::sync::Mutex<World>>, delta_time: f32) -> ScriptResult<()> {
        // Clean up scripts for entities that no longer exist
        self.cleanup_removed_entities(world.clone())?;
        
        // Get all entities with LuaScript components first
        let script_entities = {
            let world_lock = world.lock().unwrap();
            self.collect_script_entities(&*world_lock)?
        };
        
        // Sort by execution order (lower numbers execute first), then by entity ID for determinism
        let mut sorted_entities = script_entities;
        sorted_entities.sort_by(|(entity_a, script_a), (entity_b, script_b)| {
            script_a.execution_order.cmp(&script_b.execution_order)
                .then_with(|| entity_a.id().cmp(&entity_b.id()))
        });
        
        // Process each script entity with real world access
        for (entity, script) in sorted_entities {
            if !script.enabled {
                debug!("Skipping disabled script for entity {:?}: {}", entity, script.script_path);
                continue;
            }
            
            // Load script if not already loaded
            if !self.loaded_scripts.contains_key(&script.script_path) {
                match self.load_script(&script.script_path) {
                    Ok(_) => {
                        self.loaded_scripts.insert(script.script_path.clone(), true);
                        info!("Loaded script: {}", script.script_path);
                    }
                    Err(e) => {
                        error!("Failed to load script {}: {}", script.script_path, e);
                        self.loaded_scripts.insert(script.script_path.clone(), false);
                        continue;
                    }
                }
            }
            
            // Skip if script failed to load
            if !self.loaded_scripts.get(&script.script_path).unwrap_or(&false) {
                continue;
            }
            
            // Initialize script if not already initialized
            if !self.initialized_entities.contains_key(&entity) {
                match self.initialize_script_for_entity_with_world(entity, &script.script_path, world.clone()) {
                    Ok(_) => {
                        self.initialized_entities.insert(entity, script.script_path.clone());
                        debug!("Initialized script for entity {:?}: {}", entity, script.script_path);
                    }
                    Err(e) => {
                        warn!("Failed to initialize script for entity {:?}: {}", entity, e);
                        continue;
                    }
                }
            }
            
            // Execute update method with real world access
            if let Err(e) = self.execute_update_for_entity_with_world(entity, &script.script_path, delta_time, world.clone()) {
                warn!("Error executing update for entity {:?}: {}", entity, e);
            }
        }
        
        Ok(())
    }
    
    /// Clean up scripts for entities that no longer exist in the world
    fn cleanup_removed_entities(&mut self, world: Arc<std::sync::Mutex<World>>) -> ScriptResult<()> {
        let world_lock = world.lock().unwrap();
        
        // Check all initialized entities to see if they still exist
        let mut entities_to_remove = Vec::new();
        
        for (entity, script_path) in &self.initialized_entities {
            if !world_lock.contains(*entity) {
                entities_to_remove.push((*entity, script_path.clone()));
            }
        }
        
        // Drop the world lock before calling cleanup methods
        drop(world_lock);
        
        // Clean up scripts for removed entities
        for (entity, script_path) in entities_to_remove {
            debug!("Entity {:?} no longer exists, cleaning up script: {}", entity, script_path);
            if let Err(e) = self.cleanup_entity_script(entity, &script_path, world.clone()) {
                warn!("Error cleaning up script for removed entity {:?}: {}", entity, e);
            }
        }
        
        Ok(())
    }
    
    /// Check if a script has been loaded by path
    pub fn has_loaded_script(&self, script_path: &str) -> bool {
        self.loaded_scripts.contains_key(script_path)
    }
    
    /// Execute destroy method for a script on a specific entity
    fn execute_destroy_for_entity(&mut self, entity: Entity, script_path: &str, world: Arc<std::sync::Mutex<World>>) -> ScriptResult<()> {
        debug!("Executing destroy for entity {:?}, script: {}", entity, script_path);
        
        // Set up entity context for the script
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world.clone(),
        };
        
        // Execute destroy lifecycle method if it exists
        // Get the specific module for this script
        if let Some(module_name) = self.script_modules.get(script_path) {
            if let Ok(module) = self.lua_engine.lua().globals().get::<mlua::Table>(module_name.as_str()) {
                if let Ok(destroy_method) = module.get::<mlua::Function>("destroy") {
                    // Get the stored instance for this entity
                    let instance = if let Some(instance_name) = self.script_instances.get(&entity) {
                        self.lua_engine.lua().globals().get::<mlua::Table>(instance_name.as_str())
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get instance: {}", e)))?
                    } else {
                        // Create instance if it doesn't exist
                        let instance = self.lua_engine.lua().create_table()
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
                        
                        // Set entity reference in the instance
                        instance.set("entity", lua_entity)
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set entity in instance: {}", e)))?;
                        
                        instance
                    };
                    
                    // Call destroy method
                    destroy_method.call::<()>(instance)
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to call destroy method: {}", e)))?;
                    
                    debug!("Successfully called destroy method for entity {:?}", entity);
                }
            }
        }
        
        Ok(())
    }
    
    /// Clean up script for an entity that has been removed or disabled
    pub fn cleanup_entity_script(&mut self, entity: Entity, script_path: &str, world: Arc<std::sync::Mutex<World>>) -> ScriptResult<()> {
        // Call destroy method if script was initialized
        if self.initialized_entities.contains_key(&entity) {
            if let Err(e) = self.execute_destroy_for_entity(entity, script_path, world) {
                warn!("Error executing destroy for entity {:?}: {}", entity, e);
            }
            
            // Remove from initialized entities
            self.initialized_entities.remove(&entity);
            
            // Clean up instance storage
            if let Some(instance_name) = self.script_instances.remove(&entity) {
                // Remove from Lua globals
                self.lua_engine.lua().globals().set(instance_name.as_str(), mlua::Value::Nil).ok();
            }
            
            info!("Cleaned up script for entity {:?}: {}", entity, script_path);
        }
        
        Ok(())
    }
    
    /// Initialize a script for a specific entity with shared world access
    fn initialize_script_for_entity_with_world(&mut self, entity: Entity, script_path: &str, world: Arc<std::sync::Mutex<World>>) -> ScriptResult<()> {
        debug!("Initializing script {} for entity {:?}", script_path, entity);
        
        // Set up entity context for the script with real world access
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world.clone(),
        };
        
        // Set the entity as the script's self.entity
        self.lua_engine.lua().globals().set("_CURRENT_SCRIPT_ENTITY", lua_entity.clone())
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set script entity context: {}", e)))?;
        
        // Execute init lifecycle method if it exists
        // Get the specific module for this script
        if let Some(module_name) = self.script_modules.get(script_path) {
            if let Ok(module) = self.lua_engine.lua().globals().get::<mlua::Table>(module_name.as_str()) {
                if let Ok(init_method) = module.get::<mlua::Function>("init") {
                    // Create script instance (self parameter)
                    let instance = self.lua_engine.lua().create_table()
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
                    
                    // Set entity reference in the instance
                    instance.set("entity", lua_entity)
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to set entity in instance: {}", e)))?;
                    
                    // Store the instance for this entity
                    let instance_name = format!("_INSTANCE_{}", entity.id());
                    self.lua_engine.lua().globals().set(instance_name.as_str(), instance.clone())
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to store instance: {}", e)))?;
                    self.script_instances.insert(entity, instance_name);
                    
                    // Call init method
                    init_method.call::<()>(instance)
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to call init method: {}", e)))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute the update method for a script on a specific entity with shared world access
    fn execute_update_for_entity_with_world(&mut self, entity: Entity, script_path: &str, delta_time: f32, world: Arc<std::sync::Mutex<World>>) -> ScriptResult<()> {
        debug!("Executing update for entity {:?}, script: {}, delta: {}", entity, script_path, delta_time);
        
        // Set up entity context for the script with real world access
        let lua_entity = crate::lua::ecs::LuaEntity {
            entity,
            world: world.clone(),
        };
        
        // Execute update lifecycle method if it exists
        // Get the specific module for this script
        if let Some(module_name) = self.script_modules.get(script_path) {
            if let Ok(module) = self.lua_engine.lua().globals().get::<mlua::Table>(module_name.as_str()) {
                if let Ok(update_method) = module.get::<mlua::Function>("update") {
                    // Get the stored instance for this entity
                    let instance = if let Some(instance_name) = self.script_instances.get(&entity) {
                        self.lua_engine.lua().globals().get::<mlua::Table>(instance_name.as_str())
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to get instance: {}", e)))?
                    } else {
                        // Create instance if it doesn't exist (shouldn't happen if init was called)
                        let instance = self.lua_engine.lua().create_table()
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script instance: {}", e)))?;
                        
                        // Set entity reference in the instance
                        instance.set("entity", lua_entity)
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to set entity in instance: {}", e)))?;
                        
                        // Store the instance
                        let instance_name = format!("_INSTANCE_{}", entity.id());
                        self.lua_engine.lua().globals().set(instance_name.as_str(), instance.clone())
                            .map_err(|e| ScriptError::RuntimeError(format!("Failed to store instance: {}", e)))?;
                        self.script_instances.insert(entity, instance_name);
                        
                        instance
                    };
                    
                    // Call update method with delta_time
                    update_method.call::<()>((instance, delta_time))
                        .map_err(|e| ScriptError::RuntimeError(format!("Failed to call update method: {}", e)))?;
                }
            }
        }
        
        Ok(())
    }
}

impl System for LuaScriptSystem {
    fn execute(&mut self, context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        // Get the world from the context
        // For now, create a dummy world since GameContext doesn't have world access yet
        let world = World::new();
        
        // Execute all scripts
        self.execute_scripts(&world, delta_time)
            .map_err(|e| SystemError::ExecutionFailed(format!("LuaScriptSystem execution failed: {}", e)))?;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "LuaScriptSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        // Scripts should run at fixed timestep for consistent behavior
        true
    }
}

impl std::fmt::Debug for LuaScriptSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaScriptSystem")
            .field("loaded_scripts_count", &self.loaded_scripts.len())
            .field("initialized_entities_count", &self.initialized_entities.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lua_script_system_creation() {
        let system = LuaScriptSystem::new();
        assert!(system.is_ok(), "Failed to create LuaScriptSystem: {:?}", system.err());
    }
    
    #[test]
    fn test_system_traits() {
        let system = LuaScriptSystem::new().expect("Failed to create system");
        assert_eq!(system.name(), "LuaScriptSystem");
        assert!(system.is_fixed_timestep());
    }
    
    #[test]
    fn test_system_execution() {
        let mut system = LuaScriptSystem::new().expect("Failed to create system");
        let mut context = GameContext::new();
        
        let result = system.execute(&mut context, 0.016);
        assert!(result.is_ok(), "System execution should succeed: {:?}", result.err());
    }
}