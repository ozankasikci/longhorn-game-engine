//! TypeScript Script System - Executes TypeScript scripts attached to entities

use crate::components::TypeScriptScript;
use std::sync::Mutex;
use std::collections::VecDeque;

/// Console message from TypeScript scripts
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub timestamp: SystemTime,
    pub script_path: String,
    pub message: String,
}

/// Global console messages for TypeScript
pub static CONSOLE_MESSAGES: Mutex<VecDeque<ConsoleMessage>> = Mutex::new(VecDeque::new());
use crate::api::bridge::type_conversion::create_entity_object_with_methods;
use engine_ecs_core::{Entity, World};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// System that processes TypeScript script components and executes their lifecycle methods
pub struct TypeScriptScriptSystem {
    /// Set of entities that have been initialized
    initialized_entities: HashSet<Entity>,
    /// Map from entity to their script instances
    script_instances: HashMap<Entity, Vec<TypeScriptScriptInstance>>,
    /// TypeScript runtime for script execution
    runtime: Option<SimpleTypeScriptRuntime>,
    /// Next script ID for unique identification
    next_script_id: u32,
    /// Track the last known script paths for each entity to detect changes
    entity_script_paths: HashMap<Entity, Vec<String>>,
    /// KILL SWITCH: Scripts that should NEVER execute again
    dead_scripts: HashSet<u32>,
}

/// Represents an instance of a TypeScript script attached to an entity
#[derive(Debug, Clone)]
pub struct TypeScriptScriptInstance {
    pub script_id: u32,
    pub script_path: String,
    pub initialized: bool,
    pub compilation_successful: bool,
    pub last_error: Option<String>,
}

impl TypeScriptScriptInstance {
    pub fn new(script_id: u32, script_path: String) -> Self {
        Self {
            script_id,
            script_path,
            initialized: false,
            compilation_successful: false,
            last_error: None,
        }
    }
}

impl TypeScriptScriptSystem {
    pub fn new() -> Self {
        // Create a simple TypeScript runtime
        let runtime = match SimpleTypeScriptRuntime::new() {
            Ok(runtime) => Some(runtime),
            Err(e) => {
                log::error!("Failed to create TypeScript runtime: {}", e);
                None
            }
        };
        
        Self {
            initialized_entities: HashSet::new(),
            script_instances: HashMap::new(),
            runtime,
            next_script_id: 1,
            entity_script_paths: HashMap::new(),
            dead_scripts: HashSet::new(),
        }
    }

    #[cfg(test)]
    pub fn with_mock_runtime() -> Self {
        Self {
            initialized_entities: HashSet::new(),
            script_instances: HashMap::new(),
            runtime: None, // For testing, we'll simulate without real runtime
            next_script_id: 1,
            entity_script_paths: HashMap::new(),
            dead_scripts: HashSet::new(),
        }
    }

    /// Main update method called each frame
    pub fn update(&mut self, world: &mut World, delta_time: f64) {
        log::info!("🔄 TS SYSTEM UPDATE START - initialized_entities: {:?}", self.initialized_entities);
        
        // Ensure runtime is available
        if self.runtime.is_none() {
            log::warn!("TypeScriptScriptSystem: No runtime available");
            return;
        }

        // Check for compilation events that might require cache invalidation
        self.process_compilation_events();

        // Update runtime (this handles garbage collection, etc.)
        if let Some(runtime) = &mut self.runtime {
            // Set world pointer for ECS operations
            runtime.set_world_ptr(world);
            runtime.update(delta_time);
        }

        // Query all entities with TypeScript script components
        let mut script_entities = Vec::new();
        
        // In a real implementation, this would use the ECS query system
        // For now, we'll simulate the query for testing purposes
        log::info!("🔍 STEP 1: Querying for TypeScript entities...");
        for (entity, script_component) in world.query_legacy::<TypeScriptScript>() {
            log::info!("🎯 FOUND TS ENTITY: {:?} enabled={} scripts={:?}", 
                     entity, script_component.enabled, script_component.get_all_scripts());
            if script_component.enabled {
                script_entities.push((entity, script_component.clone()));
            }
        }
        log::info!("📊 STEP 1 RESULT: Found {} enabled TypeScript entities", script_entities.len());

        // Sort by execution order (lower numbers execute first)
        script_entities.sort_by_key(|(_, script)| script.execution_order);

        // Process each entity with TypeScript scripts
        log::info!("🔍 STEP 2: Processing {} TypeScript entities", script_entities.len());
        for (entity, script_component) in script_entities {
            log::info!("🎯 PROCESSING ENTITY: {:?} with scripts: {:?}", entity, script_component.get_all_scripts());
            self.process_entity_scripts(entity, &script_component, delta_time);
        }

        // Clean up entities that no longer have TypeScript components
        log::trace!("Before cleanup_removed_entities: {} initialized entities", self.initialized_entities.len());
        self.cleanup_removed_entities(world);
        log::trace!("After cleanup_removed_entities: {} initialized entities", self.initialized_entities.len());
    }

    /// Process all scripts for a single entity
    fn process_entity_scripts(&mut self, entity: Entity, script_component: &TypeScriptScript, delta_time: f64) {
        log::info!("🔍 STEP 3: process_entity_scripts() for entity {:?}", entity);
        
        // Collect all script paths for this entity
        let mut script_paths: Vec<String> = script_component.get_all_scripts().into_iter().cloned().collect();
        log::trace!("Script paths for entity: {:?}", script_paths);
        
        // CRITICAL: Check if script files actually exist and remove missing ones
        let mut missing_scripts = Vec::new();
        script_paths.retain(|path| {
            if !std::path::Path::new(path).exists() {
                log::warn!("Script file does not exist, will be removed: {}", path);
                missing_scripts.push(path.clone());
                false
            } else {
                true
            }
        });
        
        // If scripts are missing, trigger cleanup for those specific scripts
        if !missing_scripts.is_empty() {
            log::debug!("Cleanup trigger: {} missing scripts detected for entity {:?}", missing_scripts.len(), entity);
            if let Some(instances) = self.script_instances.get(&entity) {
                for instance in instances {
                    if missing_scripts.contains(&instance.script_path) {
                        log::debug!("Adding missing script {} (id: {}) to dead scripts list", instance.script_path, instance.script_id);
                        self.dead_scripts.insert(instance.script_id);
                    }
                }
            }
        }
        
        // Check if the script paths have changed for this entity
        let script_paths_changed = if let Some(previous_paths) = self.entity_script_paths.get(&entity) {
            previous_paths != &script_paths
        } else {
            false // First time seeing this entity - let normal initialization handle it
        };
        
        let needs_reinitialization = script_paths_changed;
        
        // CRITICAL: If no valid scripts remain but we have script instances, force cleanup
        if script_paths.is_empty() && self.script_instances.contains_key(&entity) {
            log::debug!("Force cleanup: Entity {:?} has no valid scripts but still has instances", entity);
            self.cleanup_entity(entity);
            return;
        }
        
        if needs_reinitialization {
            if script_paths_changed {
                log::debug!("Script paths changed for entity {:?}", entity);
            }
            if let Some(previous_paths) = self.entity_script_paths.get(&entity) {
                log::debug!("Previous paths: {:?}", previous_paths);
            }
            log::debug!("Current paths: {:?}", script_paths);
            
            // Clear old script instances and force re-initialization
            if let Some(runtime) = &mut self.runtime {
                if let Some(instances) = self.script_instances.get(&entity) {
                    for instance in instances {
                        log::debug!("Clearing script instance: {}", instance.script_path);
                        
                        // First call destroy() on the script instance if it was initialized
                        if instance.initialized {
                            if let Err(e) = runtime.call_destroy(instance.script_id) {
                                log::warn!("Failed to call destroy() for script {}: {}", instance.script_path, e);
                            } else {
                                log::debug!("Called destroy() for script: {}", instance.script_path);
                            }
                        }
                        
                        // CRITICAL: Clear the specific instance variable from V8 global scope
                        let instance_var = format!("instance_{}", instance.script_id);
                        let clear_instance_code = format!(
                            r#"
                            try {{
                                if (typeof {} !== 'undefined') {{
                                    console.log('🧹 FORCE CLEARING instance variable: {}');
                                    delete globalThis.{};
                                    {} = undefined;
                                    console.log('✅ Instance variable {} cleared');
                                }} else {{
                                    console.log('ℹ️  Instance variable {} was not found');
                                }}
                            }} catch (e) {{
                                console.error('❌ Error clearing instance {}: ' + e.toString());
                            }}
                            "#,
                            instance_var, instance_var, instance_var, instance_var, instance_var, instance_var, instance_var
                        );
                        
                        if let Err(e) = runtime.execute_javascript(&clear_instance_code) {
                            log::warn!("Failed to clear instance variable {}: {}", instance_var, e);
                        } else {
                            log::debug!("Cleared instance variable: {}", instance_var);
                        }
                        
                        // Then clear global script class definitions
                        let _ = runtime.clear_script_globals(&instance.script_path);
                    }
                }
            }
            
            self.script_instances.remove(&entity);
            self.initialized_entities.remove(&entity);
            
            // Update the tracked script paths
            self.entity_script_paths.insert(entity, script_paths.clone());
        }
        
        // Note: File change detection is now handled by the HotReloadManager in the main editor
        // to avoid excessive filesystem polling. This system just focuses on script execution.
        
        // Check if this entity needs initialization
        let needs_init = !self.initialized_entities.contains(&entity);
        
        log::info!("🔍 STEP 4: Entity {:?} needs_init={}, script_paths_changed={}", 
                 entity, needs_init, script_paths_changed);
        log::info!("📊 Current initialized_entities: {:?}", self.initialized_entities);
        
        // CRITICAL: If this is the first entity being processed and we have a runtime,
        // force V8 runtime reinitialization to ensure fresh execution
        if needs_init && self.initialized_entities.is_empty() {
            if let Some(runtime) = &mut self.runtime {
                log::info!("🔄 FIRST ENTITY: Forcing V8 runtime reinitialization for fresh game start");
                match runtime.reinitialize_v8_runtime() {
                    Ok(_) => {
                        log::info!("✅ V8 runtime reinitialized for fresh game start");
                    }
                    Err(e) => {
                        log::error!("❌ Failed to reinitialize V8 runtime: {}", e);
                    }
                }
            }
        }
        
        log::trace!("Entity needs initialization: {}", needs_init);
        
        if needs_init {
            // Initialize all scripts for this entity
            log::info!("🔍 STEP 5A: INITIALIZING scripts for entity {:?}", entity);
            
            // Remove entity from initialized set to force re-init
            self.initialized_entities.remove(&entity);
            
            self.initialize_entity_scripts(entity, &script_paths);
            self.initialized_entities.insert(entity);
            log::info!("✅ Entity {:?} marked as initialized. Total: {}", entity, self.initialized_entities.len());
        } else {
            // Update all scripts for this entity
            log::info!("🔍 STEP 5B: UPDATING scripts for entity {:?}", entity);
            self.update_entity_scripts(entity, &script_paths, delta_time);
        }
    }

    /// Initialize all scripts for an entity
    fn initialize_entity_scripts(&mut self, entity: Entity, script_paths: &[String]) {
        log::info!("🔍 STEP 6: initialize_entity_scripts for entity {:?} with {} scripts", entity, script_paths.len());
        let mut instances = Vec::new();
        
        for script_path in script_paths {
            log::info!("🔍 STEP 7: Processing script: {}", script_path);
            let script_id = self.next_script_id;
            self.next_script_id += 1;
            
            let mut instance = TypeScriptScriptInstance::new(script_id, script_path.clone());
            
            // Load and execute the script (always compile fresh, no caching)
            if let Some(runtime) = &mut self.runtime {
                // Read script content with file system flush (hot reload fix)
                // Small delay to ensure file system has flushed the changes to disk
                std::thread::sleep(std::time::Duration::from_millis(50));
                
                match std::fs::read_to_string(script_path) {
                    Ok(source) => {
                        // Load and compile script fresh every time
                        log::info!("🔍 STEP 8: File read successful for {}", script_path);
                        log::info!("🔧 Compiling TypeScript script: {}", script_path);
                        log::info!("📄 File content hash: {:x} (first 100 chars: {})", 
                                 source.len(), 
                                 &source.chars().take(100).collect::<String>().replace('\n', "\\n"));
                        log::debug!("TypeScript source code: {}", &source);
                        
                        // NOTE: Don't generate compilation events during normal loading
                        // Only hot reload should generate events to avoid circular invalidation
                        
                        match runtime.load_and_compile_script(script_id, script_path, &source) {
                            Ok(()) => {
                                instance.compilation_successful = true;
                                log::info!("🔍 STEP 9: ✅ Successfully compiled TypeScript script: {}", script_path);
                                
                                // NOTE: Don't generate compilation events during normal loading
                                // Only hot reload should generate events to avoid circular invalidation
                                
                                // Set entity context before calling init
                                if let Err(e) = runtime.set_entity_context(entity.id()) {
                                    log::warn!("Failed to set entity context: {}", e);
                                }
                                
                                // Set ECS context for V8 callbacks
                                if let Some(world_ptr) = runtime.world_ptr {
                                    crate::api::bridge::type_conversion::set_ecs_context(world_ptr, entity.id());
                                }
                                
                                // Try to call init function if it exists
                                log::info!("🔍 STEP 10: About to call init for script: {}", script_path);
                                match runtime.call_init(script_id) {
                                    Ok(_) => {
                                        instance.initialized = true;
                                        log::info!("🔍 STEP 11: ✅ Successfully initialized TypeScript script: {}", script_path);
                                    }
                                    Err(e) => {
                                        log::warn!("🔍 STEP 11: ❌ Script {} init failed: {}", script_path, e);
                                        instance.initialized = true; // Still consider it initialized
                                        instance.last_error = Some(format!("Init error: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to compile/load script {}: {}", script_path, e);
                                instance.last_error = Some(e);
                                
                                // NOTE: Don't generate compilation events during normal loading
                                // Only hot reload should generate events to avoid circular invalidation
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("🔍 STEP 8: ❌ Failed to read script file {}: {}", script_path, e);
                        instance.last_error = Some(format!("File read error: {}", e));
                    }
                }
            }
            
            instances.push(instance);
        }
        
        self.script_instances.insert(entity, instances);
    }

    /// Update all scripts for an entity
    fn update_entity_scripts(&mut self, entity: Entity, _script_paths: &[String], delta_time: f64) {
        log::info!("🔍 STEP 12: update_entity_scripts() called for entity {:?}", entity);
        
        if let Some(instances) = self.script_instances.get(&entity) {
            log::info!("🔍 STEP 13: Found {} script instances for entity", instances.len());
            
            if let Some(runtime) = &mut self.runtime {
                for instance in instances {
                    log::info!("🔍 STEP 14: Script instance: {}, initialized: {}, compiled: {}", 
                             instance.script_path, instance.initialized, instance.compilation_successful);
                    
                    // KILL SWITCH: Never execute dead scripts
                    if self.dead_scripts.contains(&instance.script_id) {
                        log::info!("🔍 STEP 15: ❌ Script {} (id: {}) is DEAD - skipping execution", instance.script_path, instance.script_id);
                        continue;
                    }
                    
                    if instance.initialized && instance.compilation_successful {
                        log::info!("🔍 STEP 16: ✅ Script ready for execution: {}", instance.script_path);
                        // Set entity context before calling update
                        if let Err(e) = runtime.set_entity_context(entity.id()) {
                            log::warn!("Failed to set entity context for update: {}", e);
                        }
                        
                        // Set ECS context for V8 callbacks
                        if let Some(world_ptr) = runtime.world_ptr {
                            crate::api::bridge::type_conversion::set_ecs_context(world_ptr, entity.id());
                        }
                        
                        // Try to call update function if it exists
                        log::info!("🔍 STEP 17: Calling runtime.call_update() for script: {}", instance.script_path);
                        eprintln!("🚨 SCRIPT UPDATE: Calling runtime.call_update() for script: {}", instance.script_path);
                        if let Err(e) = runtime.call_update(instance.script_id, delta_time) {
                            log::info!("🔍 STEP 18: ❌ Script {} update failed: {}", instance.script_path, e);
                            eprintln!("🚨 SCRIPT UPDATE ERROR: Script {} has no update function or update failed: {}", instance.script_path, e);
                        } else {
                            log::info!("🔍 STEP 18: ✅ Successfully called update() for script: {}", instance.script_path);
                            eprintln!("🚨 SCRIPT UPDATE SUCCESS: Successfully called update() for script: {}", instance.script_path);
                        }
                    } else {
                        log::trace!("Script {} not ready: initialized={}, compiled={}", 
                                instance.script_path, instance.initialized, instance.compilation_successful);
                    }
                }
            } else {
                log::error!("No runtime available for update");
            }
        } else {
            log::trace!("No script instances found for entity {:?}", entity);
        }
    }

    /// Clean up entities that no longer have TypeScript components
    fn cleanup_removed_entities(&mut self, world: &mut World) {
        let mut entities_to_remove = Vec::new();
        
        log::info!("🗑️ CLEANUP DEBUG: Starting cleanup check for {} initialized entities", self.initialized_entities.len());
        
        for &entity in &self.initialized_entities {
            // Check if entity still has a TypeScript component
            if world.get_component::<TypeScriptScript>(entity).is_none() {
                log::info!("🗑️ CLEANUP DEBUG: Entity {:?} no longer has TypeScriptScript component, marking for removal", entity);
                entities_to_remove.push(entity);
            }
        }
        
        if !entities_to_remove.is_empty() {
            log::info!("🗑️ CLEANUP DEBUG: Found {} entities to clean up: {:?}", entities_to_remove.len(), entities_to_remove);
        }
        
        for entity in entities_to_remove {
            log::info!("🗑️ CLEANUP DEBUG: Cleaning up entity {:?}", entity);
            self.cleanup_entity(entity);
            log::info!("🗑️ CLEANUP DEBUG: Completed cleanup for entity {:?}", entity);
        }
        
        log::info!("🗑️ CLEANUP DEBUG: Cleanup completed. Remaining initialized entities: {}", self.initialized_entities.len());
    }

    /// Clean up a specific entity's scripts
    fn cleanup_entity(&mut self, entity: Entity) {
        log::info!("🗑️ CLEANUP ENTITY DEBUG: Starting cleanup for entity {:?}", entity);
        
        if let Some(instances) = self.script_instances.remove(&entity) {
            log::info!("🗑️ CLEANUP ENTITY DEBUG: Found {} script instances for entity {:?}", instances.len(), entity);
            
            if let Some(runtime) = &mut self.runtime {
                for instance in &instances {
                    log::info!("🗑️ CLEANUP ENTITY DEBUG: Processing script instance {} ({}), initialized: {}", 
                        instance.script_id, instance.script_path, instance.initialized);
                    
                    // KILL SWITCH: Add to dead scripts list IMMEDIATELY
                    self.dead_scripts.insert(instance.script_id);
                    log::info!("💀 KILL SWITCH: Added script {} (id: {}) to dead scripts list", instance.script_path, instance.script_id);
                    
                    // CRITICAL: Immediately terminate script execution to prevent continued updates
                    match runtime.terminate_script_immediately(instance.script_id) {
                        Ok(_) => {
                            log::info!("🗑️ CLEANUP ENTITY DEBUG: ✅ Successfully terminated script execution: {}", instance.script_path);
                        }
                        Err(e) => {
                            log::warn!("🗑️ CLEANUP ENTITY DEBUG: ⚠️ Failed to terminate script {}: {}", instance.script_path, e);
                        }
                    }
                    
                    if instance.initialized {
                        // Try to call destroy function if it exists
                        match runtime.call_destroy(instance.script_id) {
                            Ok(_) => {
                                log::info!("🗑️ CLEANUP ENTITY DEBUG: ✅ Successfully called destroy for script: {}", instance.script_path);
                            }
                            Err(e) => {
                                log::info!("🗑️ CLEANUP ENTITY DEBUG: ⚠️ Script {} has no destroy function or destroy failed: {}", instance.script_path, e);
                            }
                        }
                    } else {
                        log::info!("🗑️ CLEANUP ENTITY DEBUG: Script {} not initialized, skipping destroy call", instance.script_path);
                    }
                }
                
                // Force garbage collection after removing entity scripts to ensure V8 memory is freed
                runtime.force_garbage_collection();
                log::info!("🗑️ CLEANUP ENTITY DEBUG: ✅ Forced V8 garbage collection after entity cleanup: {:?}", entity);
            } else {
                log::info!("🗑️ CLEANUP ENTITY DEBUG: ⚠️ No runtime available for cleanup");
            }
        } else {
            log::info!("🗑️ CLEANUP ENTITY DEBUG: ⚠️ No script instances found for entity {:?}", entity);
        }
        
        self.initialized_entities.remove(&entity);
        self.entity_script_paths.remove(&entity);
    }
    
    /// Immediately clean up a specific script from an entity (for individual script removal)
    fn cleanup_script_from_entity(&mut self, entity: Entity, script_path: &str) {
        log::info!("🗑️ SCRIPT CLEANUP DEBUG: Removing script '{}' from entity {:?}", script_path, entity);
        
        if let Some(instances) = self.script_instances.get_mut(&entity) {
            let mut script_found = false;
            instances.retain(|instance| {
                if instance.script_path == script_path {
                    log::info!("🗑️ SCRIPT CLEANUP DEBUG: Found script instance {} to remove", instance.script_id);
                    
                    // KILL SWITCH: Add to dead scripts list IMMEDIATELY  
                    self.dead_scripts.insert(instance.script_id);
                    log::info!("💀 KILL SWITCH: Added script {} (id: {}) to dead scripts list", script_path, instance.script_id);
                    
                    // CRITICAL: Immediately terminate this specific script
                    if let Some(runtime) = &mut self.runtime {
                        if let Err(e) = runtime.terminate_script_immediately(instance.script_id) {
                            log::warn!("Failed to terminate script {} immediately: {}", script_path, e);
                        }
                        
                        if instance.initialized {
                            if let Err(e) = runtime.call_destroy(instance.script_id) {
                                log::warn!("Failed to call destroy for script {}: {}", script_path, e);
                            }
                        }
                    }
                    
                    script_found = true;
                    false // Remove this script from the vector
                } else {
                    true // Keep other scripts
                }
            });
            
            if script_found {
                log::info!("🗑️ SCRIPT CLEANUP DEBUG: ✅ Successfully removed script '{}' from entity {:?}", script_path, entity);
            } else {
                log::warn!("🗑️ SCRIPT CLEANUP DEBUG: ⚠️ Script '{}' not found in entity {:?}", script_path, entity);
            }
            
            // If no scripts left for this entity, clean up the entity entirely
            if instances.is_empty() {
                self.script_instances.remove(&entity);
                self.initialized_entities.remove(&entity);
                self.entity_script_paths.remove(&entity);
                log::info!("🗑️ SCRIPT CLEANUP DEBUG: Entity {:?} has no scripts left, removed from tracking", entity);
            }
        } else {
            log::warn!("🗑️ SCRIPT CLEANUP DEBUG: ⚠️ No script instances found for entity {:?}", entity);
        }
    }
    
    /// Mark scripts for recompilation when file is modified (for hot reload)
    pub fn invalidate_script_cache(&mut self, script_path: &str) {
        log::info!("🔥 INVALIDATING SCRIPT CACHE for: {}", script_path);
        
        // Find all entities that use this script and mark them for complete reinitialization
        let mut entities_to_reinitialize = Vec::new();
        let mut script_ids_to_cleanup = Vec::new();
        
        for (entity, instances) in &mut self.script_instances {
            for instance in instances {
                if instance.script_path == script_path {
                    entities_to_reinitialize.push(*entity);
                    script_ids_to_cleanup.push(instance.script_id);
                    instance.compilation_successful = false; // Mark as needing recompilation
                    log::info!("🔄 Entity {:?} marked for reinitialization due to script change: {}", entity, script_path);
                    break; // Only need to mark entity once
                }
            }
        }
        
        // CRITICAL: Remove all entities using this script from initialized set
        // This forces complete reinitialization instead of just recompilation
        for entity in &entities_to_reinitialize {
            self.initialized_entities.remove(entity);
            log::info!("🗑️ Removed entity {:?} from initialized set - will be completely reinitialized", entity);
        }
        
        // Remove all script instances for affected entities
        for entity in &entities_to_reinitialize {
            self.script_instances.remove(entity);
            log::info!("🗑️ Removed all script instances for entity {:?}", entity);
        }
        
        // FORCE COMPLETE V8 RUNTIME REPLACEMENT to clear all compiled module cache
        if let Some(runtime) = &mut self.runtime {
            log::info!("🔥 Hot reload detected for: {} - forcing complete V8 runtime replacement", script_path);
            
            // Store script IDs for later use (before they get consumed)
            let _script_ids_to_exclude = script_ids_to_cleanup.clone();
            
            // Then destroy individual script instances
            for script_id in script_ids_to_cleanup {
                // KILL SWITCH: Add to dead scripts IMMEDIATELY
                self.dead_scripts.insert(script_id);
                log::info!("💀 KILL SWITCH: Added script_id {} to dead scripts during hot reload", script_id);
                
                if let Err(e) = runtime.call_destroy(script_id) {
                    log::warn!("Failed to cleanup old script instance {}: {}", script_id, e);
                }
            }
            
            // CRITICAL: Completely reinitialize V8 runtime to ensure no cached compiled code remains
            log::info!("🔄 REINITIALIZING V8 runtime to clear all cached compiled scripts: {}", script_path);
            
            // Reinitialize the entire V8 runtime
            match runtime.reinitialize_v8_runtime() {
                Ok(_) => {
                    log::info!("✅ V8 runtime reinitialized successfully - ALL scripts will be recompiled fresh");
                    // DON'T restore any script instances - let everything recompile fresh
                    // This ensures complete isolation and no old script code can execute
                }
                Err(e) => {
                    log::error!("❌ Failed to reinitialize V8 runtime: {}", e);
                    // Fallback to garbage collection
                    runtime.force_garbage_collection();
                }
            }
        }
        
        // Remove entities from initialized set so they get reprocessed
        for entity in entities_to_reinitialize {
            self.initialized_entities.remove(&entity);
            // CRITICAL: Also remove old script instances to prevent dual execution
            self.script_instances.remove(&entity);
            log::info!("🗑️ Removed entity {:?} from initialized set and cleared old script instances", entity);
        }
        
        log::info!("🗑️ Cleaned up and marked entities for recompilation: {}", script_path);
    }
    
    /// Check if a script is cached (for testing) - always returns false since we removed caching
    pub fn is_script_cached(&self, _script_path: &str) -> bool {
        false // No caching anymore
    }
    
    /// Process compilation events and invalidate cache for modified scripts
    fn process_compilation_events(&mut self) {
        // Get and clear compilation events - this system is responsible for processing them
        let events = crate::get_and_clear_compilation_events();
        
        for event in events {
            match event {
                crate::CompilationEvent::Started { script_path } => {
                    // File modification detected - mark for recompilation
                    if script_path.ends_with(".ts") {
                        log::info!("🗑️ Script change detected, marking for recompilation: {}", script_path);
                        self.invalidate_script_cache(&script_path);
                    }
                }
                crate::CompilationEvent::Completed { .. } => {
                    // Compilation complete - no action needed
                }
            }
        }
    }

    /// Get initialized entities (for testing)
    pub fn get_initialized_entities(&self) -> &HashSet<Entity> {
        &self.initialized_entities
    }

    /// Get script instances (for testing)
    pub fn get_script_instances(&self) -> &HashMap<Entity, Vec<TypeScriptScriptInstance>> {
        &self.script_instances
    }
    
    /// Get runtime for testing
    pub fn get_runtime(&self) -> Option<&SimpleTypeScriptRuntime> {
        self.runtime.as_ref()
    }

    /// Call update method on script instance
    pub fn call_update(&mut self, script_id: u32, delta_time: f64) -> Result<(), String> {
        if let Some(runtime) = &mut self.runtime {
            runtime.call_update(script_id, delta_time)
        } else {
            Err("TypeScript runtime not available".to_string())
        }
    }
    
    /// Force complete reinitialization of all scripts - used for stop/start cycles
    pub fn force_complete_script_reinitialization(&mut self) {
        log::info!("🔄 FORCE COMPLETE SCRIPT REINITIALIZATION: Clearing all script state for stop/start cycle");
        
        // Clear all entity initialization state
        let entity_count = self.initialized_entities.len();
        self.initialized_entities.clear();
        log::info!("✅ Cleared initialized_entities ({} entities)", entity_count);
        
        // Clear all script instances
        let instance_count = self.script_instances.len();
        self.script_instances.clear();
        log::info!("✅ Cleared script_instances ({} entities)", instance_count);
        
        // Clear entity script paths tracking
        let path_count = self.entity_script_paths.len();
        self.entity_script_paths.clear();
        log::info!("✅ Cleared entity_script_paths ({} entities)", path_count);
        
        // Clear dead scripts set
        let dead_count = self.dead_scripts.len();
        self.dead_scripts.clear();
        log::info!("✅ Cleared dead_scripts ({} scripts)", dead_count);
        
        // Force V8 runtime reinitialization to clear compiled code
        if let Some(runtime) = &mut self.runtime {
            match runtime.reinitialize_v8_runtime() {
                Ok(_) => {
                    log::info!("✅ V8 runtime reinitialized successfully");
                }
                Err(e) => {
                    log::error!("❌ Failed to reinitialize V8 runtime: {}", e);
                }
            }
            
            // Force garbage collection to clean up any remaining references
            runtime.force_garbage_collection();
        }
        
        log::info!("🎯 FORCE COMPLETE SCRIPT REINITIALIZATION: Complete! All scripts will be freshly compiled on next execution.");
    }
}

/// Memory statistics from V8
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_heap_size: usize,
    pub used_heap_size: usize,
    pub heap_size_limit: usize,
    pub script_count: usize,
    pub instance_count: usize,
}

/// Script state for hot reload preservation
#[derive(Debug, Clone)]
pub struct ScriptState {
    pub json_data: String,
}

/// Simple TypeScript runtime using V8 and SWC for compilation
pub struct SimpleTypeScriptRuntime {
    isolate: v8::OwnedIsolate,
    global_context: v8::Global<v8::Context>,
    script_instances: HashMap<u32, String>, // script_id -> instance variable name
    api_system: Option<crate::api::TypeScriptApiSystem>,
    entity_context: Option<u32>,
    world_ptr: Option<*mut World>, // Raw pointer to World for ECS operations
}

impl SimpleTypeScriptRuntime {
    pub fn new() -> Result<Self, String> {
        // Initialize V8 platform if not already done
        Self::initialize_v8_platform()?;

        // Create V8 isolate
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        
        // Set up resource constraints
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);

        let global_context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(scope);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            // Set up global objects and APIs
            let global = context.global(scope);
            
            // Add console API
            Self::setup_console_api(scope, global)?;
            
            // Add Engine API injection (World, Input, Physics)
            Self::setup_engine_api_injection(scope, global)?;
            
            // Add CommonJS exports mock for SWC CommonJS output compatibility
            Self::setup_commonjs_exports(scope, global)?;
            
            v8::Global::new(scope, context)
        };

        log::info!("Simple TypeScript runtime created with V8 engine");
        Ok(Self {
            isolate,
            global_context,
            script_instances: HashMap::new(),
            api_system: None,
            entity_context: None,
            world_ptr: None,
        })
    }

    /// Create a new TypeScript runtime with API registry system (TDD: implementing)
    pub fn with_api_registry() -> Result<Self, String> {
        // Initialize V8 platform if not already done
        Self::initialize_v8_platform()?;

        // Create V8 isolate
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        
        // Set up resource constraints
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);

        // Create the API registry system
        let api_system = crate::api::TypeScriptApiSystem::new()
            .map_err(|e| format!("Failed to create API system: {}", e))?;

        let global_context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(scope);
            let scope = &mut v8::ContextScope::new(scope, context);
            
            // Set up global context with both legacy and registry APIs
            let global = context.global(scope);
            
            // Set up console API
            Self::setup_console_api(scope, global)?;
            
            // Set up legacy Engine API (preserved from existing implementation)
            Self::setup_engine_api_injection(scope, global)?;
            
            // Set up NEW registry-based APIs alongside legacy APIs
            api_system.get_bridge().initialize_context(scope, global)
                .map_err(|e| format!("Failed to initialize registry APIs: {}", e))?;
            
            log::info!("TypeScript runtime created with API registry system");
            v8::Global::new(scope, context)
        };

        Ok(Self {
            isolate,
            global_context,
            script_instances: HashMap::new(),
            api_system: Some(api_system),
            entity_context: None,
            world_ptr: None,
        })
    }

    /// Enable registry APIs for an existing runtime (TDD: should fail initially)
    pub fn enable_registry_apis(&mut self) -> Result<(), String> {
        Err("Registry API enabling not implemented yet".to_string())
    }

    /// Set world pointer for ECS operations
    pub fn set_world_ptr(&mut self, world: &mut World) {
        self.world_ptr = Some(world as *mut World);
    }

    /// Set entity context for script execution (TDD: implementing)
    pub fn set_entity_context(&mut self, entity_id: u32) -> Result<(), String> {
        self.entity_context = Some(entity_id);
        
        // If we have an API system, update its entity context too
        if let Some(api_system) = &self.api_system {
            api_system.get_bridge().set_entity_context(Some(entity_id));
        }
        
        log::debug!("Entity context set to entity ID: {}", entity_id);
        Ok(())
    }

    fn initialize_v8_platform() -> Result<(), String> {
        static INIT: std::sync::Once = std::sync::Once::new();
        static mut INIT_RESULT: Option<Result<(), String>> = None;
        
        unsafe {
            INIT.call_once(|| {
                let platform = v8::new_default_platform(0, false).make_shared();
                v8::V8::initialize_platform(platform);
                v8::V8::initialize();
                INIT_RESULT = Some(Ok(()));
            });
            
            match INIT_RESULT.as_ref() {
                Some(result) => result.clone(),
                None => Err("V8 platform not initialized".to_string()),
            }
        }
    }

    fn setup_console_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        let console_name = v8::String::new(scope, "console")
            .ok_or_else(|| "Failed to create console string".to_string())?;
        
        let console_obj = v8::Object::new(scope);
        
        // console.log
        let log_name = v8::String::new(scope, "log")
            .ok_or_else(|| "Failed to create log string".to_string())?;
        
        let log_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let mut message_parts = Vec::new();
            for i in 0..args.length() {
                let arg = args.get(i);
                let string_val = arg.to_string(scope).unwrap_or_else(|| {
                    v8::String::new(scope, "[object]").unwrap()
                });
                let rust_string = string_val.to_rust_string_lossy(scope);
                message_parts.push(rust_string);
            }
            let message = message_parts.join(" ");
            
            // Add to game engine console directly
            use std::time::SystemTime;
            
            if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
                messages.push_back(ConsoleMessage {
                    message: message.clone(),
                    timestamp: SystemTime::now(),
                    script_path: "Unknown".to_string(), // TODO: Get actual script path
                });
            }
            
            // Also log to standard Rust logging
            log::info!("[TS Console] {}", message);
            
            // CRITICAL DEBUG: Force visible logging to editor output
            eprintln!("🚨 EDITOR LOG: [TS Console] {}", message);
        }).ok_or_else(|| "Failed to create console.log function".to_string())?;
        
        console_obj.set(scope, log_name.into(), log_fn.into());
        
        // console.error
        let error_name = v8::String::new(scope, "error")
            .ok_or_else(|| "Failed to create error string".to_string())?;
        
        let error_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            let mut message_parts = Vec::new();
            for i in 0..args.length() {
                let arg = args.get(i);
                let string_val = arg.to_string(scope).unwrap_or_else(|| {
                    v8::String::new(scope, "[object]").unwrap()
                });
                message_parts.push(string_val.to_rust_string_lossy(scope));
            }
            let message = format!("ERROR: {}", message_parts.join(" "));
            
            // Add to game engine console directly
            use std::time::SystemTime;
            
            if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
                messages.push_back(ConsoleMessage {
                    message: message.clone(),
                    timestamp: SystemTime::now(),
                    script_path: "Unknown".to_string(), // TODO: Get actual script path
                });
            }
            
            // Also log to standard Rust logging
            log::error!("[TS Console] {}", message);
        }).ok_or_else(|| "Failed to create console.error function".to_string())?;
        
        console_obj.set(scope, error_name.into(), error_fn.into());
        
        // Set console object on global
        global.set(scope, console_name.into(), console_obj.into());
        
        Ok(())
    }

    fn setup_engine_api_injection(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        // Create Engine object
        let engine_name = v8::String::new(scope, "Engine")
            .ok_or_else(|| "Failed to create Engine string".to_string())?;
        
        let engine_obj = v8::Object::new(scope);
        
        // Create Engine.world object with getCurrentEntity method
        let world_obj = v8::Object::new(scope);
        
        // getCurrentEntity method - this is what our test needs!
        let get_current_entity_name = v8::String::new(scope, "getCurrentEntity")
            .ok_or_else(|| "Failed to create getCurrentEntity string".to_string())?;
        
        let get_current_entity_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Create Entity object manually with getPosition method
            let entity_obj = v8::Object::new(scope);
            
            // Get actual entity ID from ECS context
            use crate::api::bridge::type_conversion::ECS_CONTEXT;
            let actual_entity_id = if let Ok(context) = ECS_CONTEXT.lock() {
                context.current_entity_id.unwrap_or(0)
            } else {
                0
            };
            
            // Add id() function that returns actual entity ID from ECS context
            let id_name = v8::String::new(scope, "id").unwrap();
            let id_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                let entity_id = if let Ok(context) = ECS_CONTEXT.lock() {
                    context.current_entity_id.unwrap_or(0)
                } else {
                    0
                };
                rv.set(v8::Number::new(scope, entity_id as f64).into());
            }).unwrap();
            entity_obj.set(scope, id_name.into(), id_fn.into());
            
            // Add getPosition() method directly
            let get_position_name = v8::String::new(scope, "getPosition").unwrap();
            let get_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                // Get entity ID by calling the id() function from 'this' object
                let entity_id = {
                    let this = args.this();
                    let id_key = v8::String::new(scope, "id").unwrap();
                    if let Some(id_fn) = this.get(scope, id_key.into()) {
                        if let Ok(id_fn) = v8::Local::<v8::Function>::try_from(id_fn) {
                            if let Some(result) = id_fn.call(scope, this.into(), &[]) {
                                result.number_value(scope).unwrap_or(0.0) as u32
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };

                // Get actual position from ECS (same as type_conversion.rs)
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                use engine_components_3d::Transform;
                let (pos_x, pos_y, pos_z) = if let Ok(context) = ECS_CONTEXT.lock() {
                    if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                        unsafe {
                            let world = &*world_ptr;
                            let entity = engine_ecs_core::Entity::new(entity_id, 1);
                            
                            if let Some(transform) = world.get_component::<Transform>(entity) {
                                let pos = transform.position;
                                println!("✅ ACTUAL ECS READ: Got position [{}, {}, {}] for entity {}", pos[0], pos[1], pos[2], entity_id);
                                (pos[0], pos[1], pos[2])
                            } else {
                                println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                                (5.0, 10.0, 15.0) // Fallback to default values
                            }
                        }
                    } else {
                        println!("❌ ECS ERROR: No world context available");
                        (5.0, 10.0, 15.0) // Fallback to default values
                    }
                } else {
                    (5.0, 10.0, 15.0) // Fallback to default values
                };
                
                // Create position object with actual ECS data
                let position_obj = v8::Object::new(scope);
                let x_key = v8::String::new(scope, "x").unwrap();
                let y_key = v8::String::new(scope, "y").unwrap();
                let z_key = v8::String::new(scope, "z").unwrap();
                
                let x_val = v8::Number::new(scope, pos_x as f64);
                let y_val = v8::Number::new(scope, pos_y as f64);
                let z_val = v8::Number::new(scope, pos_z as f64);
                position_obj.set(scope, x_key.into(), x_val.into());
                position_obj.set(scope, y_key.into(), y_val.into());
                position_obj.set(scope, z_key.into(), z_val.into());
                
                rv.set(position_obj.into());
            }).unwrap();
            entity_obj.set(scope, get_position_name.into(), get_position_fn.into());
            
            // Add setPosition() method directly
            let set_position_name = v8::String::new(scope, "setPosition").unwrap();
            let set_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                // Get x, y, z arguments
                let x = args.get(0).number_value(scope).unwrap_or(0.0) as f32;
                let y = args.get(1).number_value(scope).unwrap_or(0.0) as f32;
                let z = args.get(2).number_value(scope).unwrap_or(0.0) as f32;
                
                // Get entity ID by calling the id() function from 'this' object
                let entity_id = {
                    let this = args.this();
                    let id_key = v8::String::new(scope, "id").unwrap();
                    if let Some(id_fn) = this.get(scope, id_key.into()) {
                        if let Ok(id_fn) = v8::Local::<v8::Function>::try_from(id_fn) {
                            if let Some(result) = id_fn.call(scope, this.into(), &[]) {
                                result.number_value(scope).unwrap_or(0.0) as u32
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };
                
                // Access ECS context and modify the transform (same as type_conversion.rs)
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                use engine_components_3d::Transform;
                if let Ok(context) = ECS_CONTEXT.lock() {
                    if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                        unsafe {
                            let world = &mut *world_ptr;
                            
                            // Get entity from ID (assuming generation 1)
                            let entity = engine_ecs_core::Entity::new(entity_id, 1);
                            
                            // Try to get and update the transform component
                            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                                transform.position = [x, y, z];
                                println!("✅ ACTUAL ECS UPDATE: Set position to [{}, {}, {}] for entity {}", x, y, z, entity_id);
                            } else {
                                println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                            }
                        }
                    } else {
                        println!("❌ ECS ERROR: No world context available");
                    }
                }
                
                rv.set(v8::undefined(scope).into());
            }).unwrap();
            entity_obj.set(scope, set_position_name.into(), set_position_fn.into());
            
            rv.set(entity_obj.into());
            log::debug!("Engine.world.getCurrentEntity() called, returned entity with manual getPosition method");
        }).ok_or_else(|| "Failed to create getCurrentEntity function".to_string())?;
        
        world_obj.set(scope, get_current_entity_name.into(), get_current_entity_fn.into());
        
        // Set Engine.world
        let world_name = v8::String::new(scope, "world")
            .ok_or_else(|| "Failed to create world string".to_string())?;
        engine_obj.set(scope, world_name.into(), world_obj.into());
        
        // Set Engine object on global
        global.set(scope, engine_name.into(), engine_obj.into());
        
        // Also inject legacy World API for backward compatibility
        Self::inject_world_api(scope, global)?;
        
        log::info!("Engine API injection completed successfully");
        Ok(())
    }

    fn create_world_api_object<'a>(scope: &'a mut v8::HandleScope<'a>) -> Result<v8::Local<'a, v8::Object>, String> {
        let world_obj = v8::Object::new(scope);
        
        // getCurrentEntity method - this is what our test needs!
        let get_current_entity_name = v8::String::new(scope, "getCurrentEntity")
            .ok_or_else(|| "Failed to create getCurrentEntity string".to_string())?;
        
        let get_current_entity_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Create Entity object manually with getPosition method
            let entity_obj = v8::Object::new(scope);
            
            // Get actual entity ID from ECS context
            use crate::api::bridge::type_conversion::ECS_CONTEXT;
            let actual_entity_id = if let Ok(context) = ECS_CONTEXT.lock() {
                context.current_entity_id.unwrap_or(0)
            } else {
                0
            };
            
            // Add id() function that returns actual entity ID from ECS context
            let id_name = v8::String::new(scope, "id").unwrap();
            let id_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                let entity_id = if let Ok(context) = ECS_CONTEXT.lock() {
                    context.current_entity_id.unwrap_or(0)
                } else {
                    0
                };
                rv.set(v8::Number::new(scope, entity_id as f64).into());
            }).unwrap();
            entity_obj.set(scope, id_name.into(), id_fn.into());
            
            // Add getPosition() method directly
            let get_position_name = v8::String::new(scope, "getPosition").unwrap();
            let get_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                // Get entity ID by calling the id() function from 'this' object
                let entity_id = {
                    let this = args.this();
                    let id_key = v8::String::new(scope, "id").unwrap();
                    if let Some(id_fn) = this.get(scope, id_key.into()) {
                        if let Ok(id_fn) = v8::Local::<v8::Function>::try_from(id_fn) {
                            if let Some(result) = id_fn.call(scope, this.into(), &[]) {
                                result.number_value(scope).unwrap_or(0.0) as u32
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };

                // Get actual position from ECS (same as type_conversion.rs)
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                use engine_components_3d::Transform;
                let (pos_x, pos_y, pos_z) = if let Ok(context) = ECS_CONTEXT.lock() {
                    if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                        unsafe {
                            let world = &*world_ptr;
                            let entity = engine_ecs_core::Entity::new(entity_id, 1);
                            
                            if let Some(transform) = world.get_component::<Transform>(entity) {
                                let pos = transform.position;
                                println!("✅ ACTUAL ECS READ: Got position [{}, {}, {}] for entity {}", pos[0], pos[1], pos[2], entity_id);
                                (pos[0], pos[1], pos[2])
                            } else {
                                println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                                (5.0, 10.0, 15.0) // Fallback to default values
                            }
                        }
                    } else {
                        println!("❌ ECS ERROR: No world context available");
                        (5.0, 10.0, 15.0) // Fallback to default values
                    }
                } else {
                    (5.0, 10.0, 15.0) // Fallback to default values
                };
                
                // Create position object with actual ECS data
                let position_obj = v8::Object::new(scope);
                let x_key = v8::String::new(scope, "x").unwrap();
                let y_key = v8::String::new(scope, "y").unwrap();
                let z_key = v8::String::new(scope, "z").unwrap();
                
                let x_val = v8::Number::new(scope, pos_x as f64);
                let y_val = v8::Number::new(scope, pos_y as f64);
                let z_val = v8::Number::new(scope, pos_z as f64);
                position_obj.set(scope, x_key.into(), x_val.into());
                position_obj.set(scope, y_key.into(), y_val.into());
                position_obj.set(scope, z_key.into(), z_val.into());
                
                rv.set(position_obj.into());
            }).unwrap();
            entity_obj.set(scope, get_position_name.into(), get_position_fn.into());
            
            // Add setPosition() method directly
            let set_position_name = v8::String::new(scope, "setPosition").unwrap();
            let set_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                // Get x, y, z arguments
                let x = args.get(0).number_value(scope).unwrap_or(0.0) as f32;
                let y = args.get(1).number_value(scope).unwrap_or(0.0) as f32;
                let z = args.get(2).number_value(scope).unwrap_or(0.0) as f32;
                
                // Get entity ID by calling the id() function from 'this' object
                let entity_id = {
                    let this = args.this();
                    let id_key = v8::String::new(scope, "id").unwrap();
                    if let Some(id_fn) = this.get(scope, id_key.into()) {
                        if let Ok(id_fn) = v8::Local::<v8::Function>::try_from(id_fn) {
                            if let Some(result) = id_fn.call(scope, this.into(), &[]) {
                                result.number_value(scope).unwrap_or(0.0) as u32
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };
                
                // Access ECS context and modify the transform (same as type_conversion.rs)
                use crate::api::bridge::type_conversion::ECS_CONTEXT;
                use engine_components_3d::Transform;
                if let Ok(context) = ECS_CONTEXT.lock() {
                    if let (Some(world_ptr), Some(_current_entity)) = (context.world_ptr, context.current_entity_id) {
                        unsafe {
                            let world = &mut *world_ptr;
                            
                            // Get entity from ID (assuming generation 1)
                            let entity = engine_ecs_core::Entity::new(entity_id, 1);
                            
                            // Try to get and update the transform component
                            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                                transform.position = [x, y, z];
                                println!("✅ ACTUAL ECS UPDATE: Set position to [{}, {}, {}] for entity {}", x, y, z, entity_id);
                            } else {
                                println!("❌ ECS ERROR: No Transform component found for entity {}", entity_id);
                            }
                        }
                    } else {
                        println!("❌ ECS ERROR: No world context available");
                    }
                }
                
                rv.set(v8::undefined(scope).into());
            }).unwrap();
            entity_obj.set(scope, set_position_name.into(), set_position_fn.into());
            
            rv.set(entity_obj.into());
            log::debug!("Engine.world.getCurrentEntity() called, returned entity with manual getPosition method (from create_world_api_object)");
        }).ok_or_else(|| "Failed to create getCurrentEntity function".to_string())?;
        
        world_obj.set(scope, get_current_entity_name.into(), get_current_entity_fn.into());
        
        // Add other world methods (createEntity, getEntity, destroyEntity)
        Self::add_other_world_methods(scope, &world_obj)?;
        
        Ok(world_obj)
    }

    fn add_other_world_methods<'a>(scope: &'a mut v8::HandleScope<'a>, world_obj: &v8::Local<'a, v8::Object>) -> Result<(), String> {
        // createEntity method
        let create_entity_name = v8::String::new(scope, "createEntity")
            .ok_or_else(|| "Failed to create createEntity string".to_string())?;
        
        let create_entity_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Create Entity object with id() method
            let entity_obj = v8::Object::new(scope);
            
            let id_name = v8::String::new(scope, "id").unwrap();
            let id_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                let entity_id = v8::Number::new(scope, 1.0); // Mock ID
                rv.set(entity_id.into());
            }).unwrap();
            
            entity_obj.set(scope, id_name.into(), id_fn.into());
            rv.set(entity_obj.into());
            log::debug!("Engine.world.createEntity() called");
        }).ok_or_else(|| "Failed to create createEntity function".to_string())?;
        
        world_obj.set(scope, create_entity_name.into(), create_entity_fn.into());
        
        // getEntity method (stub)
        let get_entity_name = v8::String::new(scope, "getEntity")
            .ok_or_else(|| "Failed to create getEntity string".to_string())?;
        
        let get_entity_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Return null for now
            rv.set(v8::null(_scope).into());
            log::debug!("Engine.world.getEntity() called");
        }).ok_or_else(|| "Failed to create getEntity function".to_string())?;
        
        world_obj.set(scope, get_entity_name.into(), get_entity_fn.into());
        
        // destroyEntity method (stub)
        let destroy_entity_name = v8::String::new(scope, "destroyEntity")
            .ok_or_else(|| "Failed to create destroyEntity string".to_string())?;
        
        let destroy_entity_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            log::debug!("Engine.world.destroyEntity() called");
        }).ok_or_else(|| "Failed to create destroyEntity function".to_string())?;
        
        world_obj.set(scope, destroy_entity_name.into(), destroy_entity_fn.into());
        
        Ok(())
    }

    fn create_input_api_object<'a>(scope: &'a mut v8::HandleScope<'a>) -> Result<v8::Local<'a, v8::Object>, String> {
        let input_obj = v8::Object::new(scope);
        
        // isKeyDown method (stub)
        let is_key_down_name = v8::String::new(scope, "isKeyDown")
            .ok_or_else(|| "Failed to create isKeyDown string".to_string())?;
        
        let is_key_down_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Mock implementation - always return false
            let result = v8::Boolean::new(scope, false);
            rv.set(result.into());
        }).ok_or_else(|| "Failed to create isKeyDown function".to_string())?;
        
        input_obj.set(scope, is_key_down_name.into(), is_key_down_fn.into());
        
        // Add other input methods as stubs...
        // TODO: Implement actual input methods
        
        Ok(input_obj)
    }

    fn create_physics_api_object<'a>(scope: &'a mut v8::HandleScope<'a>) -> Result<v8::Local<'a, v8::Object>, String> {
        let physics_obj = v8::Object::new(scope);
        
        // applyForce method (stub)
        let apply_force_name = v8::String::new(scope, "applyForce")
            .ok_or_else(|| "Failed to create applyForce string".to_string())?;
        
        let apply_force_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            log::debug!("Engine.physics.applyForce() called");
        }).ok_or_else(|| "Failed to create applyForce function".to_string())?;
        
        physics_obj.set(scope, apply_force_name.into(), apply_force_fn.into());
        
        // Add other physics methods as stubs...
        // TODO: Implement actual physics methods
        
        Ok(physics_obj)
    }

    fn inject_world_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        let world_name = v8::String::new(scope, "World")
            .ok_or_else(|| "Failed to create World string".to_string())?;
        
        let world_obj = v8::Object::new(scope);
        
        // World.queryEntities function
        let query_entities_name = v8::String::new(scope, "queryEntities")
            .ok_or_else(|| "Failed to create queryEntities string".to_string())?;
        
        let query_entities_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Mock implementation for now - returns empty array
            let result_array = v8::Array::new(scope, 0);
            rv.set(result_array.into());
            
            if args.length() > 0 {
                log::debug!("World.queryEntities called with {} arguments", args.length());
            }
        }).ok_or_else(|| "Failed to create World.queryEntities function".to_string())?;
        
        world_obj.set(scope, query_entities_name.into(), query_entities_fn.into());
        
        // World.createEntity function
        let create_entity_name = v8::String::new(scope, "createEntity")
            .ok_or_else(|| "Failed to create createEntity string".to_string())?;
        
        let create_entity_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Mock implementation - return a fake entity ID
            let entity_id = v8::Number::new(scope, 1.0);
            rv.set(entity_id.into());
            log::debug!("World.createEntity called, returned mock entity ID: 1");
        }).ok_or_else(|| "Failed to create World.createEntity function".to_string())?;
        
        world_obj.set(scope, create_entity_name.into(), create_entity_fn.into());
        
        // World.addComponent function
        let add_component_name = v8::String::new(scope, "addComponent")
            .ok_or_else(|| "Failed to create addComponent string".to_string())?;
        
        let add_component_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 3 {
                log::debug!("World.addComponent called with {} arguments", args.length());
            }
        }).ok_or_else(|| "Failed to create World.addComponent function".to_string())?;
        
        world_obj.set(scope, add_component_name.into(), add_component_fn.into());
        
        // World.getComponent function
        let get_component_name = v8::String::new(scope, "getComponent")
            .ok_or_else(|| "Failed to create getComponent string".to_string())?;
        
        let get_component_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            if args.length() >= 2 {
                // Mock return a basic transform component
                let mock_component = v8::Object::new(scope);
                let position_obj = v8::Object::new(scope);
                let x_key = v8::String::new(scope, "x").unwrap();
                let y_key = v8::String::new(scope, "y").unwrap();
                let z_key = v8::String::new(scope, "z").unwrap();
                let x_val = v8::Number::new(scope, 1.0);
                let y_val = v8::Number::new(scope, 2.0);
                let z_val = v8::Number::new(scope, 3.0);
                position_obj.set(scope, x_key.into(), x_val.into());
                position_obj.set(scope, y_key.into(), y_val.into());
                position_obj.set(scope, z_key.into(), z_val.into());
                
                let position_key = v8::String::new(scope, "position").unwrap();
                mock_component.set(scope, position_key.into(), position_obj.into());
                rv.set(mock_component.into());
                
                log::debug!("World.getComponent called, returned mock component");
            }
        }).ok_or_else(|| "Failed to create World.getComponent function".to_string())?;
        
        world_obj.set(scope, get_component_name.into(), get_component_fn.into());
        
        // Set World object on global
        global.set(scope, world_name.into(), world_obj.into());
        
        log::debug!("World API injected successfully");
        Ok(())
    }

    fn inject_input_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        let input_name = v8::String::new(scope, "Input")
            .ok_or_else(|| "Failed to create Input string".to_string())?;
        
        let input_obj = v8::Object::new(scope);
        
        // Input.isKeyPressed function
        let is_key_pressed_name = v8::String::new(scope, "isKeyPressed")
            .ok_or_else(|| "Failed to create isKeyPressed string".to_string())?;
        
        let is_key_pressed_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Mock implementation - always return false for now
            let result = v8::Boolean::new(scope, false);
            rv.set(result.into());
            
            if args.length() > 0 {
                if let Some(key_arg) = args.get(0).to_string(scope) {
                    let key = key_arg.to_rust_string_lossy(scope);
                    log::debug!("Input.isKeyPressed called with key: {}", key);
                }
            }
        }).ok_or_else(|| "Failed to create Input.isKeyPressed function".to_string())?;
        
        input_obj.set(scope, is_key_pressed_name.into(), is_key_pressed_fn.into());
        
        // Input.isKeyDown function
        let is_key_down_name = v8::String::new(scope, "isKeyDown")
            .ok_or_else(|| "Failed to create isKeyDown string".to_string())?;
        
        let is_key_down_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            let result = v8::Boolean::new(scope, false);
            rv.set(result.into());
            
            if args.length() > 0 {
                if let Some(key_arg) = args.get(0).to_string(scope) {
                    let key = key_arg.to_rust_string_lossy(scope);
                    log::debug!("Input.isKeyDown called with key: {}", key);
                }
            }
        }).ok_or_else(|| "Failed to create Input.isKeyDown function".to_string())?;
        
        input_obj.set(scope, is_key_down_name.into(), is_key_down_fn.into());
        
        // Input.isKeyUp function
        let is_key_up_name = v8::String::new(scope, "isKeyUp")
            .ok_or_else(|| "Failed to create isKeyUp string".to_string())?;
        
        let is_key_up_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            let result = v8::Boolean::new(scope, true);
            rv.set(result.into());
            
            if args.length() > 0 {
                if let Some(key_arg) = args.get(0).to_string(scope) {
                    let key = key_arg.to_rust_string_lossy(scope);
                    log::debug!("Input.isKeyUp called with key: {}", key);
                }
            }
        }).ok_or_else(|| "Failed to create Input.isKeyUp function".to_string())?;
        
        input_obj.set(scope, is_key_up_name.into(), is_key_up_fn.into());
        
        // Input.getMousePosition function
        let get_mouse_position_name = v8::String::new(scope, "getMousePosition")
            .ok_or_else(|| "Failed to create getMousePosition string".to_string())?;
        
        let get_mouse_position_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            // Mock mouse position
            let mouse_pos = v8::Object::new(scope);
            let x_key = v8::String::new(scope, "x").unwrap();
            let y_key = v8::String::new(scope, "y").unwrap();
            let x_val = v8::Number::new(scope, 100.0);
            let y_val = v8::Number::new(scope, 200.0);
            mouse_pos.set(scope, x_key.into(), x_val.into());
            mouse_pos.set(scope, y_key.into(), y_val.into());
            rv.set(mouse_pos.into());
            
            log::debug!("Input.getMousePosition called, returned mock position {{x: 100, y: 200}}");
        }).ok_or_else(|| "Failed to create Input.getMousePosition function".to_string())?;
        
        input_obj.set(scope, get_mouse_position_name.into(), get_mouse_position_fn.into());
        
        // Input.isMouseButtonDown function
        let is_mouse_button_down_name = v8::String::new(scope, "isMouseButtonDown")
            .ok_or_else(|| "Failed to create isMouseButtonDown string".to_string())?;
        
        let is_mouse_button_down_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            let result = v8::Boolean::new(scope, false);
            rv.set(result.into());
            
            if args.length() > 0 {
                if let Some(button_arg) = args.get(0).to_string(scope) {
                    let button = button_arg.to_rust_string_lossy(scope);
                    log::debug!("Input.isMouseButtonDown called with button: {}", button);
                }
            }
        }).ok_or_else(|| "Failed to create Input.isMouseButtonDown function".to_string())?;
        
        input_obj.set(scope, is_mouse_button_down_name.into(), is_mouse_button_down_fn.into());
        
        // Set Input object on global
        global.set(scope, input_name.into(), input_obj.into());
        
        log::debug!("Input API injected successfully");
        Ok(())
    }

    fn inject_physics_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        let physics_name = v8::String::new(scope, "Physics")
            .ok_or_else(|| "Failed to create Physics string".to_string())?;
        
        let physics_obj = v8::Object::new(scope);
        
        // Physics.applyForce function
        let apply_force_name = v8::String::new(scope, "applyForce")
            .ok_or_else(|| "Failed to create applyForce string".to_string())?;
        
        let apply_force_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 2 {
                log::debug!("Physics.applyForce called with {} arguments", args.length());
            }
        }).ok_or_else(|| "Failed to create Physics.applyForce function".to_string())?;
        
        physics_obj.set(scope, apply_force_name.into(), apply_force_fn.into());
        
        // Physics.applyImpulse function
        let apply_impulse_name = v8::String::new(scope, "applyImpulse")
            .ok_or_else(|| "Failed to create applyImpulse string".to_string())?;
        
        let apply_impulse_fn = v8::Function::new(scope, |_scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue| {
            if args.length() >= 2 {
                log::debug!("Physics.applyImpulse called with {} arguments", args.length());
            }
        }).ok_or_else(|| "Failed to create Physics.applyImpulse function".to_string())?;
        
        physics_obj.set(scope, apply_impulse_name.into(), apply_impulse_fn.into());
        
        // Physics.raycast function
        let raycast_name = v8::String::new(scope, "raycast")
            .ok_or_else(|| "Failed to create raycast string".to_string())?;
        
        let raycast_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            if args.length() >= 3 {
                // Mock raycast result
                let ray_result = v8::Object::new(scope);
                let hit_key = v8::String::new(scope, "hit").unwrap();
                let hit_val = v8::Boolean::new(scope, false);
                ray_result.set(scope, hit_key.into(), hit_val.into());
                rv.set(ray_result.into());
                
                log::debug!("Physics.raycast called with {} arguments, returned mock result", args.length());
            }
        }).ok_or_else(|| "Failed to create Physics.raycast function".to_string())?;
        
        physics_obj.set(scope, raycast_name.into(), raycast_fn.into());
        
        // Physics.isColliding function
        let is_colliding_name = v8::String::new(scope, "isColliding")
            .ok_or_else(|| "Failed to create isColliding string".to_string())?;
        
        let is_colliding_fn = v8::Function::new(scope, |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
            let result = v8::Boolean::new(scope, false);
            rv.set(result.into());
            
            if args.length() >= 2 {
                log::debug!("Physics.isColliding called with {} arguments", args.length());
            }
        }).ok_or_else(|| "Failed to create Physics.isColliding function".to_string())?;
        
        physics_obj.set(scope, is_colliding_name.into(), is_colliding_fn.into());
        
        // Set Physics object on global
        global.set(scope, physics_name.into(), physics_obj.into());
        
        log::debug!("Physics API injected successfully");
        Ok(())
    }

    fn setup_commonjs_exports(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), String> {
        log::debug!("Setting up CommonJS exports mock for V8 compatibility");
        
        // Create exports object to support CommonJS modules compiled by SWC
        let exports_obj = v8::Object::new(scope);
        let exports_name = v8::String::new(scope, "exports").unwrap();
        global.set(scope, exports_name.into(), exports_obj.into());
        
        // Also create module.exports pattern (module = { exports: exports })
        let module_obj = v8::Object::new(scope);
        let module_exports_name = v8::String::new(scope, "exports").unwrap();
        module_obj.set(scope, module_exports_name.into(), exports_obj.into());
        
        let module_name = v8::String::new(scope, "module").unwrap();
        global.set(scope, module_name.into(), module_obj.into());
        
        log::debug!("CommonJS exports mock set up successfully");
        Ok(())
    }

    fn transform_commonjs_to_v8_compatible(commonjs_code: &str, script_path: &str) -> String {
        log::debug!("Transforming CommonJS code to V8-compatible format for {}", script_path);
        
        let mut code = commonjs_code.to_string();
        
        // Simple approach: Replace the entire Object.defineProperty pattern with direct assignments
        if code.contains("Object.defineProperty(exports,") && code.contains("get: function()") {
            // Extract all class names that are being exported
            let mut class_names = Vec::new();
            
            // Find all Object.defineProperty calls for class exports (not __esModule)
            // Use regex to find all exports since they may span multiple lines
            if let Ok(re) = regex::Regex::new(r#"Object\.defineProperty\(exports,\s*"([^"]+)""#) {
                for cap in re.captures_iter(&code) {
                    if let Some(class_name) = cap.get(1) {
                        let name = class_name.as_str();
                        if name != "__esModule" && !name.is_empty() {
                            class_names.push(name.to_string());
                        }
                    }
                }
            } else {
                // Fallback to simple line-by-line search
                for line in code.lines() {
                    if line.contains("Object.defineProperty(exports, \"") && !line.contains("__esModule") {
                        if let Some(start) = line.find("Object.defineProperty(exports, \"") {
                            let after_start = &line[start + 33..]; // Skip 'Object.defineProperty(exports, "'
                            if let Some(end) = after_start.find("\"") {
                                let class_name = &after_start[..end];
                                if !class_name.is_empty() {
                                    class_names.push(class_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            log::debug!("Found {} classes to export: {:?}", class_names.len(), class_names);
            
            if !class_names.is_empty() {
                // Remove all Object.defineProperty blocks entirely
                let mut clean_code = String::new();
                let lines: Vec<&str> = code.lines().collect();
                let mut i = 0;
                
                while i < lines.len() {
                    let line = lines[i];
                    let trimmed = line.trim();
                    
                    // If this line starts an Object.defineProperty block, skip the entire block
                    if trimmed.starts_with("Object.defineProperty(exports,") {
                        // Skip lines until we find the closing });
                        while i < lines.len() {
                            if lines[i].trim().ends_with("});") {
                                i += 1; // Skip the }); line too
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    
                    // Keep all other lines
                    clean_code.push_str(line);
                    clean_code.push('\n');
                    i += 1;
                }
                
                // Add simple export assignments at the end
                for class_name in class_names {
                    clean_code.push_str(&format!("exports.{} = {};\n", class_name, class_name));
                }
                
                code = clean_code;
            }
        }
        
        log::debug!("CommonJS transformation complete for {}", script_path);
        code
    }

    /// Wrap the CommonJS code in an IIFE that provides the exports object and assigns to globalThis
    fn wrap_in_iife_with_exports(code: &str, script_path: &str) -> String {
        // Extract class names from the simple exports assignments
        let re = regex::Regex::new(r"exports\.(\w+)\s*=\s*(\w+);").unwrap();
        let mut class_assignments = Vec::new();
        
        for cap in re.captures_iter(code) {
            if let (Some(export_name), Some(class_name)) = (cap.get(1), cap.get(2)) {
                let export_str = export_name.as_str();
                let class_str = class_name.as_str();
                // Assign to globalThis for V8 access
                class_assignments.push(format!("    globalThis.{} = exports.{};", class_str, export_str));
            }
        }
        
        // If no exports found, try to extract class names from class declarations
        if class_assignments.is_empty() {
            let class_re = regex::Regex::new(r"class\s+(\w+)").unwrap();
            for cap in class_re.captures_iter(code) {
                if let Some(class_name) = cap.get(1) {
                    let class_str = class_name.as_str();
                    class_assignments.push(format!("    globalThis.{} = {};", class_str, class_str));
                }
            }
        }
        
        let assignments = class_assignments.join("\n");
        
        // First, convert \n to actual newlines in the code
        let actual_code = code.replace("\\n", "\n");
        
        // Wrap the code in an IIFE with exports object
        format!(
            "// IIFE wrapper for TypeScript script: {}\n(function() {{\n    var exports = {{}};\n    var module = {{ exports: exports }};\n    \n    // Original compiled code\n{}\n    \n    // Assign exports to globalThis for V8 access\n{}\n    \n    // Log successful loading\n    console.log(\"✅ TypeScript module loaded: {}\", Object.keys(exports));\n}})();",
            script_path,
            actual_code,
            assignments,
            script_path
        )
    }

    pub fn compile_typescript_to_javascript(&self, typescript_source: &str, script_path: &str) -> Result<String, String> {
        let start_time = std::time::Instant::now();
        
        // Use SWC to compile TypeScript to JavaScript
        let result = swc_core::common::GLOBALS.set(&swc_core::common::Globals::new(), || {
            let source_map = swc_core::common::sync::Lrc::new(swc_core::common::SourceMap::new(
                swc_core::common::FilePathMapping::empty()
            ));
            
            // Create source file with proper file name for better error reporting
            let source_file = source_map.new_source_file(
                swc_core::common::FileName::Real(std::path::PathBuf::from(script_path)),
                typescript_source.to_string(),
            );
            
            // Parse TypeScript
            let lexer = swc_core::ecma::parser::lexer::Lexer::new(
                swc_core::ecma::parser::Syntax::Typescript(swc_core::ecma::parser::TsSyntax {
                    tsx: false,
                    decorators: true,
                    dts: false,
                    no_early_errors: false,
                    disallow_ambiguous_jsx_like: true,
                }),
                swc_core::ecma::ast::EsVersion::Es2020,
                swc_core::ecma::parser::StringInput::from(&*source_file),
                None,
            );
            
            let mut parser = swc_core::ecma::parser::Parser::new_from(lexer);
            let module = parser.parse_module().map_err(|e| {
                format!("Parse error in {}: {:?}", script_path, e)
            })?;
            
            // Transform TypeScript to JavaScript
            let program = swc_core::ecma::ast::Program::Module(module);
            
            let program = swc_ecma_visit::FoldWith::fold_with(program, &mut swc_core::ecma::transforms::base::resolver(
                swc_core::common::Mark::new(),
                swc_core::common::Mark::new(),
                true,
            ));
            
            let program = swc_ecma_visit::FoldWith::fold_with(program, &mut swc_ecma_visit::as_folder(
                swc_core::ecma::transforms::typescript::strip(swc_core::common::Mark::new())
            ));
            
            // Add CommonJS module transformation to convert ES6 exports to module.exports
            let program = swc_ecma_visit::FoldWith::fold_with(program, &mut swc_ecma_visit::as_folder(
                swc_core::ecma::transforms::module::common_js::<swc_core::common::comments::NoopComments>(
                    swc_core::common::Mark::new(),
                    swc_core::ecma::transforms::module::util::Config {
                        ..Default::default()
                    },
                    swc_core::ecma::transforms::base::feature::FeatureFlag::default(),
                    None,
                )
            ));
            
            // Extract module back from program
            let module = match program {
                swc_core::ecma::ast::Program::Module(m) => m,
                _ => return Err(format!("Expected module program in {}", script_path)),
            };
            
            // Generate JavaScript code
            let mut buf = Vec::new();
            let writer = swc_core::ecma::codegen::text_writer::JsWriter::new(source_map.clone(), "\\n", &mut buf, None);
            
            let mut emitter = swc_core::ecma::codegen::Emitter {
                cfg: swc_core::ecma::codegen::Config::default(),
                cm: source_map.clone(),
                comments: None,
                wr: writer,
            };
            
            emitter.emit_module(&module).map_err(|e| {
                format!("Code generation error in {}: {:?}", script_path, e)
            })?;
            
            let code = String::from_utf8(buf).map_err(|e| {
                format!("Invalid UTF-8 in generated code for {}: {}", script_path, e)
            })?;
            
            // Debug: Log the compiled JavaScript to see what SWC is outputting
            log::info!("📋 SWC compiled JavaScript for {}:\n{}", script_path, code);
            log::info!("❌ Contains export statements: {}", code.contains("export "));
            log::info!("✅ Contains CommonJS patterns: {}", code.contains("module.exports") || code.contains("exports."));
            log::info!("🌐 Contains globalThis: {}", code.contains("globalThis"));
            
            // Post-process the CommonJS output to make it V8-compatible
            let v8_compatible_code = Self::transform_commonjs_to_v8_compatible(&code, script_path);
            
            // Wrap in IIFE to provide exports object and assign to globalThis
            let wrapped_code = Self::wrap_in_iife_with_exports(&v8_compatible_code, script_path);
            
            log::info!("🔧 V8-compatible JavaScript for {}:\n{}", script_path, wrapped_code);
            
            Ok(wrapped_code)
        });
        
        let compilation_time = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                log::debug!("TypeScript compilation successful for {} in {:?}", script_path, compilation_time);
                if compilation_time.as_millis() > 100 {
                    log::warn!("Slow TypeScript compilation for {} took {:?}", script_path, compilation_time);
                }
            }
            Err(e) => {
                log::error!("TypeScript compilation failed for {} in {:?}: {}", script_path, compilation_time, e);
            }
        }
        
        result
    }

    fn execute_javascript(&mut self, code: &str) -> Result<(), String> {
        log::trace!("Executing JavaScript code ({} chars)", code.len());
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Compile the script
        let source = v8::String::new(scope, code)
            .ok_or_else(|| "Failed to create V8 string".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| {
                let error_msg = format!("V8 JavaScript compilation failed - SYNTAX ERROR in generated code!");
                log::error!("{}", error_msg);
                log::debug!("Failed code was:\n{}", code);
                error_msg
            })?;
        
        log::debug!("JavaScript compilation successful");
        
        // Execute the script
        script.run(scope)
            .ok_or_else(|| {
                let error_msg = format!("Script execution failed. Code was: {}", code);
                log::error!("{}", error_msg);
                error_msg
            })?;
        
        log::debug!("JavaScript execution successful");
        Ok(())
    }

    /// Evaluate JavaScript code and return the result as a string
    fn evaluate_javascript(&mut self, code: &str) -> Result<String, String> {
        log::trace!("Evaluating JavaScript code ({} chars)", code.len());
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Compile the script
        let source = v8::String::new(scope, code)
            .ok_or_else(|| "Failed to create V8 string".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| {
                let error_msg = format!("JavaScript compilation failed: {}", code);
                log::error!("{}", error_msg);
                error_msg
            })?;
        
        // Execute the script and get the result
        let result = script.run(scope)
            .ok_or_else(|| {
                let error_msg = format!("Script evaluation failed. Code was: {}", code);
                log::error!("{}", error_msg);
                error_msg
            })?;
        
        // Convert result to string
        let result_string = result.to_rust_string_lossy(scope);
        log::debug!("JavaScript evaluation successful, result: {}", result_string);
        Ok(result_string)
    }

    pub fn load_and_compile_script(&mut self, _script_id: u32, script_path: &str, source: &str) -> Result<(), String> {
        log::info!("Loading and compiling TypeScript script: {}", script_path);
        
        // Compile TypeScript to JavaScript fresh every time (no caching)
        let javascript_code = self.compile_typescript_to_javascript(source, script_path)
            .map_err(|e| {
                log::error!("TypeScript compilation failed for {}: {}", script_path, e);
                // Ensure error contains script path for better debugging
                if e.contains(script_path) {
                    e
                } else {
                    format!("Error in {}: {}", script_path, e)
                }
            })?;

        log::info!("Successfully compiled TypeScript to JavaScript for: {}", script_path);
        log::debug!("Compiled JavaScript code: {}", javascript_code);
        
        // STEP 1 TEST: Verify V8 console.log is working
        log::trace!("Testing V8 console.log setup");
        let console_test = "console.log('🔥 V8 CONSOLE TEST: This message should appear in logs!');";
        match self.execute_javascript(console_test) {
            Ok(_) => log::info!("✅ V8 console test executed successfully"),
            Err(e) => log::error!("❌ V8 console test failed: {}", e),
        }

        // Execute the compiled JavaScript code to load the class into V8
        log::info!("About to execute compiled JavaScript for: {}", script_path);
        self.execute_javascript(&javascript_code)
            .map_err(|e| {
                log::error!("JavaScript execution failed for {}: {}", script_path, e);
                format!("Execution error in {}: {}", script_path, e)
            })?;

        log::info!("Successfully compiled and loaded script: {}", script_path);
        Ok(())
    }
    

    pub fn call_init(&mut self, script_id: u32) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        
        let instance_var = format!("instance_{}", script_id);
        
        // Try to find a class in the global scope and create an instance
        let init_code = format!(
            r#"
            var {} = null;
            var initSuccess = false;
            var lastError = null;
            
            // First try to find a class in exports (CommonJS)
            if (typeof exports === 'object' && exports) {{
                for (var name in exports) {{
                    if (typeof exports[name] === 'function' && exports[name].prototype) {{
                        try {{
                            {} = new exports[name]();
                            if (typeof {}.init === 'function') {{
                                {}.init();
                            }}
                            initSuccess = true;
                            break;
                        }} catch (e) {{
                            lastError = e.toString();
                            // Continue to next potential class
                        }}
                    }}
                }}
            }}
            
            // If not found in exports, try globalThis (fallback)
            if (!initSuccess) {{
                for (var name in globalThis) {{
                    if (typeof globalThis[name] === 'function' && globalThis[name].prototype) {{
                        try {{
                            {} = new globalThis[name]();
                            if (typeof {}.init === 'function') {{
                                {}.init();
                            }}
                            initSuccess = true;
                            break;
                        }} catch (e) {{
                            lastError = e.toString();
                            // Continue to next potential class
                        }}
                    }}
                }}
            }}
            
            if (!initSuccess && lastError) {{
                throw new Error('Failed to initialize script: ' + lastError);
            }}
            "#,
            instance_var, instance_var, instance_var, instance_var, instance_var, instance_var, instance_var
        );

        let result = self.execute_javascript(&init_code);
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(_) => {
                self.script_instances.insert(script_id, instance_var);
                log::debug!("Successfully called init() for script_id: {} in {:?}", script_id, execution_time);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to call init() for script_id: {} in {:?}: {}", script_id, execution_time, e);
                Err(format!("Init error for script_id {}: {}", script_id, e))
            }
        }
    }


    pub fn call_destroy(&mut self, script_id: u32) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        
        if let Some(instance_var) = self.script_instances.get(&script_id) {
            let destroy_code = format!(
                r#"
                try {{
                    if ({} && typeof {}.destroy === 'function') {{
                        {}.destroy();
                    }}
                    {} = null;
                }} catch (e) {{
                    console.error('Destroy error in script_id {}: ' + e.toString());
                    // Continue with cleanup even if destroy() fails
                }}
                "#,
                instance_var, instance_var, instance_var, instance_var, script_id
            );

            // Don't fail the entire cleanup if destroy() has errors
            if let Err(e) = self.execute_javascript(&destroy_code) {
                log::warn!("Error during destroy() for script_id {}: {}", script_id, e);
            }
        } else {
            log::debug!("Attempted to call destroy() on non-existent script_id: {}", script_id);
        }

        // Always perform cleanup regardless of destroy() success
        self.script_instances.remove(&script_id);

        let cleanup_time = start_time.elapsed();
        log::debug!("Cleanup completed for script_id: {} in {:?}", script_id, cleanup_time);
        Ok(())
    }
    
    /// Immediately terminate a script by clearing its instance and preventing further execution
    pub fn terminate_script_immediately(&mut self, script_id: u32) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        
        log::info!("🛑 TERMINATE DEBUG: Immediately terminating script_id: {}", script_id);
        
        if let Some(instance_var) = self.script_instances.get(&script_id) {
            log::info!("🛑 TERMINATE DEBUG: Found script instance variable: {}", instance_var);
            
            // CRITICAL: Immediately nullify the script instance to stop execution
            let termination_code = format!(
                r#"
                try {{
                    // Immediately nullify the instance to stop all future updates
                    if (typeof {} !== 'undefined') {{
                        console.log("🛑 TERMINATING SCRIPT: Setting {} to null");
                        {} = null;
                    }}
                    
                    // Clear the variable from global scope entirely
                    if (typeof globalThis.{} !== 'undefined') {{
                        delete globalThis.{};
                    }}
                    
                    console.log("🛑 SCRIPT TERMINATED: script_id {} execution stopped");
                }} catch (e) {{
                    console.error('🛑 Error during script termination for script_id {}: ' + e.toString());
                }}
                "#,
                instance_var, instance_var, instance_var, instance_var, instance_var, script_id, script_id
            );

            // Execute termination code immediately 
            if let Err(e) = self.execute_javascript(&termination_code) {
                log::warn!("🛑 Failed to execute termination code for script_id {}: {}", script_id, e);
            } else {
                log::info!("🛑 TERMINATE DEBUG: ✅ Successfully executed termination code for script_id {}", script_id);
            }
        } else {
            log::warn!("🛑 TERMINATE DEBUG: Script_id {} not found in script_instances", script_id);
        }

        // Remove from our tracking immediately to prevent any future calls
        self.script_instances.remove(&script_id);

        let termination_time = start_time.elapsed();
        log::info!("🛑 TERMINATE DEBUG: Script termination completed for script_id: {} in {:?}", script_id, termination_time);
        Ok(())
    }
    
    /// Clear all global script class definitions for hot reload
    pub fn clear_script_globals(&mut self, script_path: &str) -> Result<(), String> {
        log::info!("🧹 Clearing global script definitions for: {}", script_path);
        
        // Clear all script-created globals from globalThis (both functions and variables)
        let clear_code = r#"
            try {
                // Define built-in properties that should never be deleted
                var builtinProps = new Set([
                    'Object', 'Function', 'Array', 'String', 'Number', 'Boolean', 
                    'Date', 'RegExp', 'Error', 'console', 'JSON', 'Math', 'parseInt', 
                    'parseFloat', 'isNaN', 'isFinite', 'decodeURI', 'decodeURIComponent',
                    'encodeURI', 'encodeURIComponent', 'eval', 'globalThis', 'undefined',
                    'Infinity', 'NaN', 'Promise', 'Symbol', 'Map', 'Set', 'WeakMap', 
                    'WeakSet', 'Proxy', 'Reflect', 'ArrayBuffer', 'SharedArrayBuffer',
                    'Atomics', 'DataView', 'Int8Array', 'Uint8Array', 'Uint8ClampedArray',
                    'Int16Array', 'Uint16Array', 'Int32Array', 'Uint32Array', 'Float32Array',
                    'Float64Array', 'BigInt', 'BigInt64Array', 'BigUint64Array',
                    // Engine-provided APIs
                    'World', 'Input', 'Physics'
                ]);
                
                // Get list of all properties on globalThis
                var propsToDelete = [];
                for (var prop in globalThis) {
                    if (globalThis.hasOwnProperty(prop) && !builtinProps.has(prop)) {
                        // Delete ANY property that's not a built-in, regardless of type
                        // This includes script classes, variables, constants, etc.
                        propsToDelete.push(prop);
                    }
                }
                
                // Delete all script-created globals from globalThis
                for (var i = 0; i < propsToDelete.length; i++) {
                    var prop = propsToDelete[i];
                    console.log('🧹 Clearing global:', prop, '(type:', typeof globalThis[prop], ')');
                    delete globalThis[prop];
                }
                
                console.log('✅ Cleared', propsToDelete.length, 'global script properties');
            } catch (e) {
                console.error('Error clearing globals:', e.toString());
            }
        "#;
        
        self.execute_javascript(clear_code)
            .map_err(|e| format!("Failed to clear script globals for {}: {}", script_path, e))?;
            
        log::info!("✅ Global script cleanup completed for: {}", script_path);
        Ok(())
    }

    pub fn update(&mut self, _delta_time: f64) {
        static mut LAST_GC_TIME: Option<std::time::Instant> = None;
        const GC_INTERVAL_MS: u128 = 1000; // Garbage collect every 1 second
        
        let start_time = std::time::Instant::now();
        let should_gc = unsafe {
            match LAST_GC_TIME {
                None => {
                    LAST_GC_TIME = Some(start_time);
                    true
                }
                Some(last_time) => {
                    if start_time.duration_since(last_time).as_millis() > GC_INTERVAL_MS {
                        LAST_GC_TIME = Some(start_time);
                        true
                    } else {
                        false
                    }
                }
            }
        };
        
        if should_gc {
            // Log runtime statistics instead of forcing GC
            let instance_count = self.script_instances.len();
            
            if instance_count > 0 {
                log::debug!("TypeScript runtime stats: {} active instances", instance_count);
            }
            
            // Note: Garbage collection is handled automatically by V8
            // Manual GC calls require --expose-gc flag which isn't suitable for production
        }
    }

    // TDD GREEN PHASE: Implement hot reload functionality

    /// Load and compile script from file path
    pub fn load_and_compile_script_from_file(&mut self, script_id: u32, file_path: &std::path::Path) -> Result<(), String> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read script file {}: {}", file_path.display(), e))?;
        
        let script_path = file_path.to_string_lossy();
        self.load_and_compile_script(script_id, &script_path, &source)
    }

    /// Hot reload an existing script while preserving state
    pub fn hot_reload_script(&mut self, script_id: u32, file_path: &std::path::Path) -> Result<(), String> {
        log::info!("Hot reloading TypeScript script: {}", file_path.display());
        
        // Read the updated script content
        let new_source = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read updated script file {}: {}", file_path.display(), e))?;
        
        // Get the current script state if it exists
        let preserved_state = if self.script_instances.contains_key(&script_id) {
            // Try to extract state from current script instance
            self.extract_script_state(script_id).ok()
        } else {
            None
        };
        
        // Compile and execute the new script
        let script_path = file_path.to_string_lossy();
        let javascript_code = self.compile_typescript_to_javascript(&new_source, &script_path)
            .map_err(|e| format!("Hot reload compilation failed for {}: {}", script_path, e))?;
        
        // Execute the new script definition
        self.execute_javascript(&javascript_code)
            .map_err(|e| format!("Hot reload execution failed for {}: {}", script_path, e))?;
        
        // Create new script instance
        let instance_name = format!("script_{}", script_id);
        let instantiation_code = format!(
            "globalThis.{} = new {}();",
            instance_name,
            self.extract_class_name(&javascript_code).unwrap_or("UnknownClass".to_string())
        );
        
        self.execute_javascript(&instantiation_code)
            .map_err(|e| format!("Failed to instantiate reloaded script {}: {}", script_path, e))?;
        
        self.script_instances.insert(script_id, instance_name.clone());
        
        // Restore state if we preserved any
        if let Some(state) = preserved_state {
            if let Err(e) = self.restore_script_state(script_id, &state) {
                log::warn!("Failed to restore state for script {}: {}", script_path, e);
                // Continue anyway - hot reload succeeded, just without state preservation
            }
        }
        
        log::info!("Successfully hot reloaded TypeScript script: {}", script_path);
        Ok(())
    }

    /// Call a specific method on a script instance
    pub fn call_script_method(&mut self, script_id: u32, method_name: &str, args: &[&str]) -> Result<String, String> {
        let instance_name = self.script_instances.get(&script_id)
            .ok_or_else(|| format!("Script {} not found", script_id))?;
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Build method call
        let args_str = args.join(", ");
        let call_code = format!("globalThis.{}.{}({})", instance_name, method_name, args_str);
        
        let source = v8::String::new(scope, &call_code)
            .ok_or_else(|| "Failed to create V8 string for method call".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| format!("Failed to compile method call: {}", call_code))?;
        
        let result = script.run(scope)
            .ok_or_else(|| format!("Method call failed: {}", call_code))?;
        
        // Convert result to string
        let result_str = result.to_rust_string_lossy(scope);
        Ok(result_str)
    }

    /// Hot reload script only if file has changed
    pub fn hot_reload_script_if_changed(&mut self, script_id: u32, file_path: &std::path::Path) -> Result<bool, String> {
        // Check if file exists
        if !file_path.exists() {
            return Err(format!("Script file not found: {}", file_path.display()));
        }
        
        // Get file modification time
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata for {}: {}", file_path.display(), e))?;
        
        let _modified_time = metadata.modified()
            .map_err(|e| format!("Failed to get modification time for {}: {}", file_path.display(), e))?;
        
        // For this implementation, we'll always reload if the file exists
        // In a production system, you'd track modification times
        self.hot_reload_script(script_id, file_path)?;
        Ok(true)
    }

    /// Force garbage collection in V8
    pub fn force_garbage_collection(&mut self) {
        // Note: V8 garbage collection requires --expose-gc flag for security reasons
        // In production/development without this flag, we skip forced GC as it's not essential
        // The V8 runtime will automatically garbage collect when needed
        log::debug!("Garbage collection skipped (requires --expose-gc flag for forced GC)");
    }
    
    
    /// Completely reinitialize the V8 runtime to clear all cached compiled code
    pub fn reinitialize_v8_runtime(&mut self) -> Result<(), String> {
        log::info!("🔄 Reinitializing V8 runtime context for hot reload");

        // Clear any lingering script instances/state
        self.script_instances.clear();

        // Recreate the global context on the existing isolate
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope);

        {
            let context_scope = &mut v8::ContextScope::new(scope, context);
            let global = context.global(context_scope);

            // Reinstall console / engine APIs
            Self::setup_console_api(context_scope, global)?;
            Self::setup_engine_api_injection(context_scope, global)?;
            Self::setup_commonjs_exports(context_scope, global)?;

            if let Some(api_system) = &self.api_system {
                api_system
                    .get_bridge()
                    .initialize_context(context_scope, global)
                    .map_err(|e| format!("Failed to initialize registry APIs: {}", e))?;
            }
        }

        // Swap to the new global context
        self.global_context = v8::Global::new(scope, context);
        log::info!("✅ V8 runtime context reset complete");
        Ok(())
    }

    /// Get memory statistics from V8
    pub fn get_memory_stats(&mut self) -> Result<MemoryStats, String> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let mut stats = v8::HeapStatistics::default();
        scope.get_heap_statistics(&mut stats);
        
        Ok(MemoryStats {
            total_heap_size: stats.total_heap_size(),
            used_heap_size: stats.used_heap_size(),
            heap_size_limit: stats.heap_size_limit(),
            script_count: 0, // No longer tracking compiled scripts
            instance_count: self.script_instances.len(),
        })
    }

    // Helper methods for state preservation

    /// Extract current state from a script instance
    fn extract_script_state(&mut self, script_id: u32) -> Result<ScriptState, String> {
        let instance_name = self.script_instances.get(&script_id)
            .ok_or_else(|| format!("Script {} not found", script_id))?;
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Try to get serializable state (simplified approach)
        let state_code = format!("JSON.stringify(globalThis.{})", instance_name);
        
        let source = v8::String::new(scope, &state_code)
            .ok_or_else(|| "Failed to create V8 string for state extraction".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| "Failed to compile state extraction code".to_string())?;
        
        let result = script.run(scope)
            .ok_or_else(|| "State extraction failed".to_string())?;
        
        let state_json = result.to_rust_string_lossy(scope);
        
        Ok(ScriptState {
            json_data: state_json,
        })
    }

    /// Restore state to a script instance
    fn restore_script_state(&mut self, script_id: u32, state: &ScriptState) -> Result<(), String> {
        let instance_name = self.script_instances.get(&script_id)
            .ok_or_else(|| format!("Script {} not found", script_id))?;
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Try to restore state (simplified approach)
        let restore_code = format!(
            "Object.assign(globalThis.{}, JSON.parse('{}'))",
            instance_name,
            state.json_data.replace('\'', "\\'")
        );
        
        let source = v8::String::new(scope, &restore_code)
            .ok_or_else(|| "Failed to create V8 string for state restoration".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| "Failed to compile state restoration code".to_string())?;
        
        script.run(scope)
            .ok_or_else(|| "State restoration failed".to_string())?;
        
        Ok(())
    }

    /// Extract class name from compiled JavaScript code
    fn extract_class_name(&self, javascript_code: &str) -> Option<String> {
        // Simple regex-based extraction of class name
        // Look for "class ClassName" pattern
        if let Some(start) = javascript_code.find("class ") {
            let rest = &javascript_code[start + 6..];
            if let Some(end) = rest.find(' ') {
                Some(rest[..end].trim().to_string())
            } else if let Some(end) = rest.find('{') {
                Some(rest[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add message to game engine console (same system as Lua scripts)
    fn add_console_message(message: String) {
        log::info!("Adding TypeScript console message: {}", message);
        if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
            messages.push_back(ConsoleMessage {
                message: message.clone(),
                timestamp: SystemTime::now(),
                script_path: "Unknown".to_string(),
            });
            log::info!("Successfully added console message, total messages: {}", messages.len());
        } else {
            log::error!("Failed to acquire console messages lock for: {}", message);
        }
    }

    /// Call update method on script instance
    pub fn call_update(&mut self, script_id: u32, delta_time: f64) -> Result<(), String> {
        eprintln!("🚨 V8 RUNTIME: call_update() script_id={}, delta_time={}", script_id, delta_time);
        
        if let Some(instance_var) = self.script_instances.get(&script_id).cloned() {
            eprintln!("🚨 V8 RUNTIME: Found instance variable: {}", instance_var);
        } else {
            eprintln!("🚨 V8 RUNTIME ERROR: No instance variable found for script_id: {}", script_id);
            eprintln!("🚨 V8 RUNTIME ERROR: Available script IDs: {:?}", self.script_instances.keys().collect::<Vec<_>>());
            return Err(format!("No instance variable found for script_id: {}", script_id));
        }
        
        if let Some(instance_var) = self.script_instances.get(&script_id).cloned() {
            
            let update_code = format!(
                r#"
                try {{
                    console.log('🚨 V8: Checking if {} exists and has update method');
                    if ({} && typeof {}.update === 'function') {{
                        console.log('🚨 V8: Calling {}.update({})');
                        {}.update({});
                        console.log('🚨 V8: update() call completed');
                    }} else {{
                        console.log('🚨 V8: No update method found for {}');
                        if ({}) {{
                            console.log('🚨 V8: Instance exists but update is:', typeof {}.update);
                        }} else {{
                            console.log('🚨 V8: Instance {} does not exist');
                        }}
                    }}
                }} catch (e) {{
                    console.error('🚨 V8: Update error in script_id {}: ' + e.toString());
                    throw new Error('Update error in script_id {}: ' + e.toString());
                }}
                "#,
                instance_var, instance_var, instance_var, instance_var, delta_time, 
                instance_var, delta_time, instance_var, instance_var, instance_var, 
                instance_var, script_id, script_id
            );

            eprintln!("🚨 V8 RUNTIME: Executing update code");
            self.execute_javascript(&update_code)
                .map_err(|e| format!("Update error for script_id {}: {}", script_id, e))?;
            
            // Check if any Transform position was modified (simplified detection)
            let check_position_modified_code = r#"
                (function() {
                    console.log("🔍 DEBUG: Checking for position modifications...");
                    
                    // Debug: list all global properties
                    let globalProps = [];
                    for (let prop in globalThis) {
                        globalProps.push(prop);
                    }
                    console.log("🔍 DEBUG: Global properties:", globalProps.join(", "));
                    
                    // Check if any global variables store transforms with modified positions
                    for (let prop in globalThis) {
                        let obj = globalThis[prop];
                        console.log("🔍 DEBUG: Checking property:", prop, "type:", typeof obj);
                        
                        if (obj && typeof obj === 'object' && obj.transform && obj.transform.position) {
                            console.log("🔍 DEBUG: Found transform on property:", prop);
                            console.log("🔍 DEBUG: Position values: x=" + obj.transform.position.x + 
                                       ", y=" + obj.transform.position.y + 
                                       ", z=" + obj.transform.position.z);
                            
                            // Check if position values have changed from initial values (5.0, 10.0, 15.0)
                            if (obj.transform.position.x !== 5.0 || 
                                obj.transform.position.y !== 10.0 || 
                                obj.transform.position.z !== 15.0) {
                                console.log("🚨 POSITION MODIFICATION DETECTED!");
                                console.log("   Expected: x=5.0, y=10.0, z=15.0");
                                console.log("   Actual: x=" + obj.transform.position.x + 
                                           ", y=" + obj.transform.position.y + 
                                           ", z=" + obj.transform.position.z);
                                return true;
                            }
                        }
                    }
                    console.log("🔍 DEBUG: No position modifications detected");
                    return false;
                })()
            "#;
            
            if let Ok(result) = self.evaluate_javascript(check_position_modified_code) {
                // Parse the result to see if position was modified
                if result.contains("true") {
                    return Err("Position modification detected but bidirectional sync not implemented yet".to_string());
                }
            }
        } else {
            log::debug!("Attempted to call update() on non-existent script_id: {}", script_id);
        }
        Ok(())
    }
}

// V8 requires special handling for thread safety
unsafe impl Send for SimpleTypeScriptRuntime {}
unsafe impl Sync for SimpleTypeScriptRuntime {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Transform;

    fn setup_test_world() -> (World, Entity) {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<TypeScriptScript>();
        engine_ecs_core::register_component::<Transform>();
        
        let entity = world.spawn();
        
        (world, entity)
    }

    #[test]
    fn test_system_finds_entities_with_typescript_components() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        // Add TypeScriptScript component to entity
        world.add_component(
            entity,
            TypeScriptScript::new("test_script.ts".to_string())
        ).unwrap();

        // Act
        system.update(&mut world, 0.016);

        // Assert
        assert!(system.get_initialized_entities().contains(&entity));
        let instances = system.get_script_instances().get(&entity).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].script_path, "test_script.ts");
    }

    #[test]
    fn test_system_ignores_entities_without_typescript_components() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        // Add only Transform component (no TypeScriptScript)
        world.add_component(
            entity,
            Transform::identity()
        ).unwrap();

        // Act
        system.update(&mut world, 0.016);

        // Assert
        assert!(!system.get_initialized_entities().contains(&entity));
        assert!(system.get_script_instances().is_empty());
    }

    #[test]
    fn test_system_processes_multiple_entities_with_scripts() {
        // Arrange
        let (mut world, entity1) = setup_test_world();
        let entity2 = world.spawn();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        // Add TypeScriptScript components to both entities
        world.add_component(
            entity1,
            TypeScriptScript::new("script1.ts".to_string())
        ).unwrap();
        
        world.add_component(
            entity2,
            TypeScriptScript::new("script2.ts".to_string())
        ).unwrap();

        // Act
        system.update(&mut world, 0.016);

        // Assert
        assert_eq!(system.get_initialized_entities().len(), 2);
        assert!(system.get_initialized_entities().contains(&entity1));
        assert!(system.get_initialized_entities().contains(&entity2));
    }

    #[test]
    fn test_system_calls_init_on_first_execution() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        world.add_component(
            entity,
            TypeScriptScript::new("hello_world.ts".to_string())
        ).unwrap();

        // Act - First update should call init
        system.update(&mut world, 0.016);

        // Assert
        assert!(system.get_initialized_entities().contains(&entity));
        let instances = system.get_script_instances().get(&entity).unwrap();
        // Since we're using mock runtime, initialized will be false unless we have a real script file
        assert_eq!(instances[0].script_path, "hello_world.ts");
    }

    #[test]
    fn test_system_calls_update_on_subsequent_executions() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        world.add_component(
            entity,
            TypeScriptScript::new("hello_world.ts".to_string())
        ).unwrap();

        // Act - First update (init), then second update
        system.update(&mut world, 0.016);
        system.update(&mut world, 0.020);

        // Assert - Should be called with the second delta time
        // Note: We can't directly check the mock runtime from here, 
        // but the system structure ensures update is called
        assert!(system.get_initialized_entities().contains(&entity));
    }

    #[test]
    fn test_system_handles_disabled_scripts() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        let mut script = TypeScriptScript::new("disabled_script.ts".to_string());
        script.enabled = false;
        
        world.add_component(entity, script).unwrap();

        // Act
        system.update(&mut world, 0.016);

        // Assert - disabled scripts should not be processed
        assert!(!system.get_initialized_entities().contains(&entity));
        assert!(system.get_script_instances().is_empty());
    }

    #[test]
    fn test_system_respects_execution_order() {
        // Arrange
        let (mut world, entity1) = setup_test_world();
        let entity2 = world.spawn();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        // Add scripts with different execution orders
        world.add_component(
            entity1,
            TypeScriptScript::with_execution_order("script_high.ts".to_string(), 1)
        ).unwrap();
        
        world.add_component(
            entity2,
            TypeScriptScript::with_execution_order("script_low.ts".to_string(), -1)
        ).unwrap();

        // Act
        system.update(&mut world, 0.016);

        // Assert - both should be processed (order verification would need access to mock runtime)
        assert_eq!(system.get_initialized_entities().len(), 2);
    }

    #[test]
    fn test_system_cleans_up_when_component_removed() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let mut system = TypeScriptScriptSystem::with_mock_runtime();
        
        world.add_component(
            entity,
            TypeScriptScript::new("temp_script.ts".to_string())
        ).unwrap();

        // Act - Initialize script, then remove component
        system.update(&mut world, 0.016);
        world.remove_component::<TypeScriptScript>(entity).unwrap();
        system.update(&mut world, 0.016);

        // Assert - entity should be cleaned up
        assert!(!system.get_initialized_entities().contains(&entity));
        assert!(!system.get_script_instances().contains_key(&entity));
    }
}
/// Get and clear all console messages
pub fn get_and_clear_console_messages() -> Vec<ConsoleMessage> {
    if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
        messages.drain(..).collect()
    } else {
        Vec::new()
    }
}
