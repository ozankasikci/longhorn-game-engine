//! Command execution logic for editor control

use crate::types::*;
use engine_ecs_core::{Entity, World};
use engine_scripting::components::TypeScriptScript;
use engine_scripting::typescript_script_system::{CONSOLE_MESSAGES, ConsoleMessage};
use engine_components_3d::Transform;
use engine_components_ui::Name;
use engine_editor_framework::PlayStateManager;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

/// Handler for executing editor commands
#[derive(Clone)]
pub struct EditorCommandHandler {
    world: Arc<Mutex<World>>,
    game_state: Arc<Mutex<GameStateInfo>>,
    logs: Arc<Mutex<Vec<String>>>,
    script_errors: Arc<Mutex<Vec<ScriptError>>>,
    compilation_events: Arc<Mutex<Vec<CompilationEvent>>>,
    action_sender: Option<mpsc::Sender<EditorAction>>,
    action_processor: Option<Arc<Mutex<dyn Fn(EditorAction) -> bool + Send + Sync>>>,
    play_state_manager: Option<Arc<Mutex<PlayStateManager>>>,
}

impl EditorCommandHandler {
    pub fn new(
        world: Arc<Mutex<World>>,
        game_state: Arc<Mutex<GameStateInfo>>,
        logs: Arc<Mutex<Vec<String>>>,
        script_errors: Arc<Mutex<Vec<ScriptError>>>,
        compilation_events: Arc<Mutex<Vec<CompilationEvent>>>,
        action_sender: Option<mpsc::Sender<EditorAction>>,
    ) -> Self {
        Self {
            world,
            game_state,
            logs,
            script_errors,
            compilation_events,
            action_sender,
            action_processor: None,
            play_state_manager: None,
        }
    }
    
    /// Set a callback for immediate action processing
    pub fn set_action_processor<F>(&mut self, processor: F)
    where
        F: Fn(EditorAction) -> bool + Send + Sync + 'static,
    {
        self.action_processor = Some(Arc::new(Mutex::new(processor)));
    }
    
    /// Set the play state manager for direct action processing
    pub fn set_play_state_manager(&mut self, play_state_manager: Arc<Mutex<PlayStateManager>>) {
        self.play_state_manager = Some(play_state_manager);
    }

    /// Find an entity by ID, returning the entity with the correct generation
    fn find_entity_by_id(&self, world: &World, entity_id: u32) -> Option<Entity> {
        // Try to find the entity by checking for Transform components
        // This is a reasonable heuristic since most scene entities have transforms
        for (entity, _) in world.query_legacy::<Transform>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        // If not found via Transform, try TypeScriptScript components
        for (entity, _) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        // If not found via TypeScriptScript, try Name components
        for (entity, _) in world.query_legacy::<engine_components_ui::Name>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        None
    }

    pub fn execute_command(&self, command: EditorCommand) -> EditorResponse {
        match command {
            EditorCommand::Ping => EditorResponse::Pong,
            
            EditorCommand::AddScript { entity_id, script_path } => {
                self.add_script_to_entity(entity_id, script_path)
            }
            
            EditorCommand::RemoveScript { entity_id, script_path } => {
                self.remove_script_from_entity(entity_id, script_path)
            }
            
            EditorCommand::ReplaceScript { entity_id, old_path, new_path } => {
                self.replace_script_on_entity(entity_id, old_path, new_path)
            }
            
            EditorCommand::GetEntityScripts { entity_id } => {
                self.get_entity_scripts(entity_id)
            }
            
            EditorCommand::GetSceneObjects => {
                self.get_scene_objects()
            }
            
            EditorCommand::GetEntityInfo { entity_id } => {
                self.get_entity_info(entity_id)
            }
            
            EditorCommand::GetLogs { lines } => {
                self.get_logs(lines)
            }
            
            EditorCommand::GetGameState => {
                let state = self.game_state.lock().unwrap().clone();
                EditorResponse::GameState(state)
            }
            
            EditorCommand::GetScriptErrors => {
                let errors = self.script_errors.lock().unwrap().clone();
                EditorResponse::ScriptErrors(errors)
            }
            
            EditorCommand::GetCompilationEvents => {
                let events = self.compilation_events.lock().unwrap().clone();
                EditorResponse::CompilationEvents(events)
            }
            
            EditorCommand::TriggerHotReload { script_path } => {
                self.trigger_hot_reload(script_path)
            }
            
            EditorCommand::ForceScriptReinitialization => {
                self.force_script_reinitialization()
            }
            
            EditorCommand::StartGame => {
                self.send_editor_action(EditorAction::StartPlay)
            }
            
            EditorCommand::StopGame => {
                self.send_editor_action(EditorAction::StopPlay)
            }
            
            EditorCommand::PauseGame => {
                self.send_editor_action(EditorAction::PausePlay)
            }
            
            EditorCommand::ResumeGame => {
                self.send_editor_action(EditorAction::ResumePlay)
            }
            
            // TypeScript Debugging Commands
            EditorCommand::GetTypeScriptSystemStatus => {
                self.get_typescript_system_status()
            }
            
            EditorCommand::GetScriptInstances => {
                self.get_script_instances()
            }
            
            EditorCommand::GetInitializedEntities => {
                self.get_initialized_entities()
            }
            
            EditorCommand::GetDeadScripts => {
                self.get_dead_scripts()
            }
            
            EditorCommand::GetScriptExecutionLogs => {
                self.get_script_execution_logs()
            }
            
            EditorCommand::TestScriptExecution { entity_id } => {
                self.test_script_execution(entity_id)
            }
            
            EditorCommand::ValidateScriptFiles => {
                self.validate_script_files()
            }
            
            EditorCommand::GetV8RuntimeStats => {
                self.get_v8_runtime_stats()
            }
            
            EditorCommand::TriggerScriptRecompilation { script_path } => {
                self.trigger_script_recompilation(script_path)
            }
            
            EditorCommand::SimulateFileChange { script_path } => {
                self.simulate_file_change(script_path)
            }
            
            _ => EditorResponse::Error {
                message: "Command not implemented yet".to_string(),
            },
        }
    }

    fn add_script_to_entity(&self, entity_id: u32, script_path: String) -> EditorResponse {
        let mut world = self.world.lock().unwrap();
        
        // Find the correct entity by ID (don't hardcode generation to 0)
        let entity = self.find_entity_by_id(&world, entity_id)
            .unwrap_or_else(|| Entity::new(entity_id, 0)); // Fallback to generation 0
        
        // Only support TypeScript/JavaScript scripts
        if script_path.ends_with(".ts") || script_path.ends_with(".js") {
            if let Some(mut ts_script) = world.get_component_mut::<TypeScriptScript>(entity) {
                // Entity already has TypeScript component, add to additional scripts
                ts_script.add_script(script_path.clone());
                self.log(format!("Added TypeScript script '{}' to entity {}", script_path, entity_id));
                
            } else {
                // Create new TypeScript component
                let ts_component = TypeScriptScript::new(script_path.clone());
                if world.add_component(entity, ts_component).is_ok() {
                    self.log(format!("Created TypeScript component with script '{}' for entity {}", script_path, entity_id));
                } else {
                    return EditorResponse::EntityNotFound { entity_id };
                }
            }
        } else {
            return EditorResponse::Error {
                message: format!("Only TypeScript (.ts) and JavaScript (.js) scripts are supported: {}", script_path),
            };
        }
        
        EditorResponse::Success
    }

    fn remove_script_from_entity(&self, entity_id: u32, script_path: String) -> EditorResponse {
        self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Starting removal of script '{}' from entity {}", script_path, entity_id));
        
        let mut world = self.world.lock().unwrap();
        
        // Find the correct entity by ID (don't hardcode generation to 0)
        let entity = self.find_entity_by_id(&world, entity_id)
            .unwrap_or_else(|| Entity::new(entity_id, 0)); // Fallback to generation 0
        
        self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Found entity {:?} for ID {}", entity, entity_id));
        
        // Try to remove from TypeScript component
        if let Some(mut ts_script) = world.get_component_mut::<TypeScriptScript>(entity) {
            self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Entity has TypeScriptScript component with scripts: {:?}", 
                ts_script.get_all_scripts()));
            
            // Check if the script exists in this component first
            let script_exists = ts_script.get_all_scripts().iter().any(|s| *s == &script_path);
            self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Script '{}' exists in component: {}", script_path, script_exists));
            
            if script_exists {
                let remove_result = ts_script.remove_script(&script_path);
                self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: remove_script() returned: {}", remove_result));
                
                if remove_result {
                    self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Script removed successfully. Remaining scripts: {:?}", 
                        ts_script.get_all_scripts()));
                    
                    // If no scripts left, remove the component entirely
                    if ts_script.get_all_scripts().is_empty() {
                        self.log("🗑️ SCRIPT REMOVAL DEBUG: No scripts left, removing entire TypeScriptScript component".to_string());
                        let remove_component_result = world.remove_component::<TypeScriptScript>(entity);
                        self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: remove_component result: {:?}", remove_component_result));
                    }
                } else {
                    // remove_script returned false, meaning the component should be removed entirely
                    self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: remove_script returned false, removing entire component"));
                    let remove_component_result = world.remove_component::<TypeScriptScript>(entity);
                    self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: remove_component result: {:?}", remove_component_result));
                }
                
                // CRITICAL: Trigger hot reload to force script cleanup in V8 runtime
                self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Triggering hot reload to clean up V8 instances for script: {}", script_path));
                self.trigger_hot_reload(script_path.clone());
                
                // CRITICAL: Also send action to sync removal to coordinator world if in play mode
                self.log("🗑️ SCRIPT REMOVAL DEBUG: Sending SyncScriptRemoval action to coordinator".to_string());
                self.send_editor_action(EditorAction::SyncScriptRemoval { 
                    entity_id, 
                    script_path: script_path.clone() 
                });
                
                // VERIFICATION: Check if component was actually removed
                if world.get_component::<TypeScriptScript>(entity).is_some() {
                    self.log("🗑️ SCRIPT REMOVAL DEBUG: ⚠️ WARNING: TypeScriptScript component still exists after removal!".to_string());
                    if let Some(remaining_script) = world.get_component::<TypeScriptScript>(entity) {
                        self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Remaining scripts in component: {:?}", 
                            remaining_script.get_all_scripts()));
                    }
                } else {
                    self.log("🗑️ SCRIPT REMOVAL DEBUG: ✅ TypeScriptScript component successfully removed from entity".to_string());
                }
                
                return EditorResponse::Success;
            } else {
                self.log(format!("🗑️ SCRIPT REMOVAL DEBUG: Script '{}' not found in component scripts", script_path));
            }
        } else {
            self.log("🗑️ SCRIPT REMOVAL DEBUG: Entity has no TypeScriptScript component".to_string());
        }
        
        self.log("🗑️ SCRIPT REMOVAL DEBUG: Script removal failed - script not found".to_string());
        EditorResponse::ScriptNotFound { script_path }
    }
    

    fn replace_script_on_entity(&self, entity_id: u32, old_path: String, new_path: String) -> EditorResponse {
        // First remove the old script
        match self.remove_script_from_entity(entity_id, old_path) {
            EditorResponse::Success => {
                // Then add the new script
                self.add_script_to_entity(entity_id, new_path)
            }
            error => error,
        }
    }

    fn get_entity_scripts(&self, entity_id: u32) -> EditorResponse {
        let world = self.world.lock().unwrap();
        
        // Find the correct entity by ID (don't hardcode generation to 0)
        let entity = self.find_entity_by_id(&world, entity_id)
            .unwrap_or_else(|| Entity::new(entity_id, 0)); // Fallback to generation 0
        
        let mut scripts = Vec::new();
        
        // Get TypeScript scripts
        if let Some(ts_script) = world.get_component::<TypeScriptScript>(entity) {
            for script_path in ts_script.get_all_scripts() {
                scripts.push(script_path.clone());
            }
        }
        
        EditorResponse::EntityScripts(scripts)
    }

    fn get_scene_objects(&self) -> EditorResponse {
        let world = self.world.lock().unwrap();
        let mut objects = Vec::new();
        
        // Query all entities with Transform components (assuming these are scene objects)
        for (entity, transform) in world.query_legacy::<Transform>() {
            let entity_id = entity.id();
            
            // Get name
            let name = world.get_component::<Name>(entity)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("Entity {}", entity_id));
            
            // Get transform info
            let transform_info = Some(TransformInfo {
                position: transform.position,
                rotation: transform.rotation,
                scale: transform.scale,
            });
            
            // Get scripts
            let mut scripts = Vec::new();
            if let Some(ts_script) = world.get_component::<TypeScriptScript>(entity) {
                for script_path in ts_script.get_all_scripts() {
                    scripts.push(script_path.clone());
                }
            }
            
            // Get component types (simplified)
            let mut components = Vec::new();
            if world.get_component::<Transform>(entity).is_some() {
                components.push("Transform".to_string());
            }
            if world.get_component::<Name>(entity).is_some() {
                components.push("Name".to_string());
            }
            if world.get_component::<TypeScriptScript>(entity).is_some() {
                components.push("TypeScriptScript".to_string());
            }
            
            objects.push(SceneObject {
                entity_id,
                name,
                transform: transform_info,
                scripts,
                components,
            });
        }
        
        EditorResponse::SceneObjects(objects)
    }

    fn get_entity_info(&self, entity_id: u32) -> EditorResponse {
        let world = self.world.lock().unwrap();
        
        // Find the correct entity by ID (don't hardcode generation to 0)
        let entity = self.find_entity_by_id(&world, entity_id)
            .unwrap_or_else(|| Entity::new(entity_id, 0)); // Fallback to generation 0
        
        // Check if entity exists by trying to get any component
        if world.get_component::<Transform>(entity).is_none() && 
           world.get_component::<Name>(entity).is_none() &&
           world.get_component::<TypeScriptScript>(entity).is_none() {
            return EditorResponse::EntityNotFound { entity_id };
        }
        
        // Get name
        let name = world.get_component::<Name>(entity).map(|n| n.name.clone());
        
        // Get transform
        let transform = world.get_component::<Transform>(entity).map(|t| TransformInfo {
            position: t.position,
            rotation: t.rotation,
            scale: t.scale,
        });
        
        // Get scripts
        let mut typescript_scripts = Vec::new();
        
        if let Some(ts_script) = world.get_component::<TypeScriptScript>(entity) {
            for script_path in ts_script.get_all_scripts() {
                typescript_scripts.push(script_path.clone());
            }
        }
        
        let scripts = ScriptInfo {
            typescript_scripts,
        };
        
        // Get components (simplified for now)
        let mut components = Vec::new();
        if world.get_component::<Transform>(entity).is_some() {
            components.push(ComponentInfo {
                component_type: "Transform".to_string(),
                data: serde_json::json!(transform),
            });
        }
        if let Some(name_comp) = world.get_component::<Name>(entity) {
            components.push(ComponentInfo {
                component_type: "Name".to_string(),
                data: serde_json::json!({ "name": name_comp.name }),
            });
        }
        
        let entity_info = EntityInfo {
            entity_id,
            name,
            transform,
            scripts,
            components,
        };
        
        EditorResponse::EntityInfo(entity_info)
    }

    fn get_logs(&self, lines: Option<usize>) -> EditorResponse {
        let mut all_logs = Vec::new();
        
        // Get control system logs
        let control_logs = self.logs.lock().unwrap();
        for log in control_logs.iter() {
            all_logs.push(format!("[Control] {}", log));
        }
        
        // Get console messages from scripts
        if let Ok(console_messages) = CONSOLE_MESSAGES.lock() {
            for msg in console_messages.iter() {
                all_logs.push(format!("[Script] {}", msg.message));
            }
        }
        
        // Sort by timestamp would be ideal, but for now just show them in order
        // Apply line limit if specified
        let result = if let Some(limit) = lines {
            if all_logs.len() > limit {
                all_logs.split_off(all_logs.len() - limit)
            } else {
                all_logs
            }
        } else {
            all_logs
        };
        
        EditorResponse::Logs(result)
    }
    
    fn trigger_hot_reload(&self, script_path: String) -> EditorResponse {
        // Manually trigger a compilation event for hot reload
        self.log(format!("Manually triggering hot reload for: {}", script_path));
        
        // Add compilation event to trigger hot reload
        let mut events = self.compilation_events.lock().unwrap();
        events.push(CompilationEvent {
            script_path: script_path.clone(),
            event_type: "started".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            success: None,
        });
        drop(events); // Release lock before next operation
        
        // Also trigger via the global compilation event system
        engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
            script_path: script_path.clone(),
        });
        
        // Force update the file modification time to ensure hot reload detection
        if std::path::Path::new(&script_path).exists() {
            // Touch the file to update its modification time
            if let Ok(content) = std::fs::read_to_string(&script_path) {
                if std::fs::write(&script_path, content).is_ok() {
                    self.log(format!("Updated modification time for: {}", script_path));
                }
            }
        }
        
        EditorResponse::Success
    }
    
    fn force_script_reinitialization(&self) -> EditorResponse {
        self.log("🔄 EDITOR CONTROL: Forcing complete script reinitialization".to_string());
        
        // Send action to coordinator to force script reinitialization
        self.send_editor_action(EditorAction::ForceScriptReinitialization)
    }

    fn log(&self, message: String) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(message); // Don't double-prefix, get_logs will add [Control] prefix
        
        // Keep only last 1000 log entries
        if logs.len() > 1000 {
            let excess = logs.len() - 1000;
            logs.drain(0..excess);
        }
    }
    
    fn send_editor_action(&self, action: EditorAction) -> EditorResponse {
        // Try direct play state processing first if available
        if let Some(play_state_manager) = &self.play_state_manager {
            if let Ok(mut manager) = play_state_manager.lock() {
                match &action {
                    EditorAction::StartPlay | EditorAction::StopPlay => {
                        // StartPlay/StopPlay actions require world access for snapshots
                        // These must be handled by the main editor, not the control system
                        // Fall through to channel processing to ensure proper snapshot handling
                        self.log(format!("StartPlay/StopPlay action requires main editor processing for snapshots: {:?}", action));
                    }
                    EditorAction::PausePlay => {
                        manager.pause();
                        // Update game state for control system
                        if let Ok(mut state) = self.game_state.lock() {
                            state.is_paused = true;
                        }
                        self.log(format!("Processed PausePlay action directly: {:?}", action));
                        return EditorResponse::Success;
                    }
                    EditorAction::ResumePlay => {
                        manager.resume();
                        // Update game state for control system
                        if let Ok(mut state) = self.game_state.lock() {
                            state.is_paused = false;
                        }
                        self.log(format!("Processed ResumePlay action directly: {:?}", action));
                        return EditorResponse::Success;
                    }
                    _ => {
                        // Other actions fall through to channel processing
                    }
                }
            }
        }
        
        // Try callback processor for other actions
        if let Some(processor) = &self.action_processor {
            if let Ok(processor_fn) = processor.lock() {
                if processor_fn(action.clone()) {
                    self.log(format!("Processed editor action via callback: {:?}", action));
                    return EditorResponse::Success;
                }
            }
        }
        
        // Fall back to sending via channel for GUI processing
        if let Some(sender) = &self.action_sender {
            match sender.send(action.clone()) {
                Ok(_) => {
                    self.log(format!("Sent editor action: {:?}", action));
                    EditorResponse::Success
                }
                Err(e) => {
                    self.log(format!("Failed to send editor action: {}", e));
                    EditorResponse::Error {
                        message: format!("Failed to send action to editor: {}", e),
                    }
                }
            }
        } else {
            EditorResponse::Error {
                message: "Editor action channel not available".to_string(),
            }
        }
    }

    /// Get comprehensive TypeScript system status
    fn get_typescript_system_status(&self) -> EditorResponse {
        self.log("Getting TypeScript system status".to_string());
        
        let world = self.world.lock().unwrap();
        
        // Count entities with TypeScript components
        let mut total_entities = 0;
        let mut enabled_entities = 0;
        
        for (_, script_component) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            total_entities += 1;
            if script_component.enabled {
                enabled_entities += 1;
            }
        }
        
        // Get compilation events count
        let compilation_events = engine_scripting::get_and_clear_compilation_events();
        let events_count = compilation_events.len();
        
        // Put events back (we just wanted to count them)
        for event in compilation_events {
            engine_scripting::add_compilation_event(event);
        }
        
        let status = crate::types::TypeScriptSystemStatus {
            runtime_available: true, // TODO: Check actual runtime status
            total_entities,
            initialized_entities: enabled_entities, // Approximation
            script_instances: 0, // TODO: Get from script system
            dead_scripts: 0, // TODO: Get from script system
            last_update_time: Some(chrono::Utc::now().to_rfc3339()),
            compilation_events_pending: events_count,
        };
        
        EditorResponse::TypeScriptSystemStatus(status)
    }
    
    /// Get all script instances information
    fn get_script_instances(&self) -> EditorResponse {
        self.log("Getting script instances".to_string());
        
        let world = self.world.lock().unwrap();
        let mut instances = Vec::new();
        
        for (entity, script_component) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            for script_path in script_component.get_all_scripts() {
                instances.push(crate::types::ScriptInstanceInfo {
                    script_id: 0, // TODO: Get actual script ID
                    entity_id: entity.id(),
                    script_path: script_path.clone(),
                    initialized: script_component.enabled,
                    compilation_successful: true, // TODO: Get actual status
                    last_error: None,
                });
            }
        }
        
        EditorResponse::ScriptInstances(instances)
    }
    
    /// Get list of initialized entities
    fn get_initialized_entities(&self) -> EditorResponse {
        self.log("Getting initialized entities".to_string());
        
        let world = self.world.lock().unwrap();
        let mut entity_ids = Vec::new();
        
        for (entity, script_component) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            if script_component.enabled {
                entity_ids.push(entity.id());
            }
        }
        
        EditorResponse::InitializedEntities(entity_ids)
    }
    
    /// Get list of dead scripts
    fn get_dead_scripts(&self) -> EditorResponse {
        self.log("Getting dead scripts".to_string());
        
        // TODO: Access the actual dead scripts set from TypeScriptScriptSystem
        let dead_scripts = Vec::new();
        
        EditorResponse::DeadScripts(dead_scripts)
    }
    
    /// Get script execution logs
    fn get_script_execution_logs(&self) -> EditorResponse {
        self.log("Getting script execution logs".to_string());
        
        // Get recent logs that contain script execution information
        let logs = self.logs.lock().unwrap();
        let script_logs: Vec<String> = logs.iter()
            .filter(|log| log.contains("SCRIPT") || log.contains("TypeScript") || log.contains("🔍"))
            .cloned()
            .collect();
        
        EditorResponse::ScriptExecutionLogs(script_logs)
    }
    
    /// Test script execution for a specific entity
    fn test_script_execution(&self, entity_id: u32) -> EditorResponse {
        self.log(format!("Testing script execution for entity {}", entity_id));
        
        let world = self.world.lock().unwrap();
        
        // Find the entity
        if let Some(entity) = self.find_entity_by_id(&world, entity_id) {
            if let Some(script_component) = world.get_component::<engine_scripting::components::TypeScriptScript>(entity) {
                let scripts = script_component.get_all_scripts();
                self.log(format!("Entity {} has {} scripts: {:?}", entity_id, scripts.len(), scripts));
                
                // TODO: Trigger actual script execution test
                EditorResponse::Success
            } else {
                EditorResponse::Error {
                    message: format!("Entity {} has no TypeScript component", entity_id),
                }
            }
        } else {
            EditorResponse::EntityNotFound { entity_id }
        }
    }
    
    /// Validate all script files
    fn validate_script_files(&self) -> EditorResponse {
        self.log("Validating script files".to_string());
        
        let world = self.world.lock().unwrap();
        let mut validation_results = Vec::new();
        
        for (entity, script_component) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            for script_path in script_component.get_all_scripts() {
                let result = self.validate_single_file(script_path);
                validation_results.push(result);
            }
        }
        
        EditorResponse::FileValidationResults(validation_results)
    }
    
    /// Validate a single script file
    fn validate_single_file(&self, script_path: &str) -> crate::types::FileValidationResult {
        let path = std::path::Path::new(script_path);
        
        let exists = path.exists();
        let mut readable = false;
        let mut size_bytes = None;
        let mut last_modified = None;
        let mut content_hash = None;
        let mut syntax_valid = None;
        let mut error = None;
        
        if exists {
            match std::fs::read_to_string(script_path) {
                Ok(content) => {
                    readable = true;
                    size_bytes = Some(content.len() as u64);
                    
                    // Calculate content hash
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    content_hash = Some(format!("{:x}", hasher.finish()));
                    
                    // Get file metadata
                    if let Ok(metadata) = std::fs::metadata(script_path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                last_modified = Some(duration.as_secs().to_string());
                            }
                        }
                    }
                    
                    // Basic TypeScript syntax validation
                    syntax_valid = Some(content.contains("class") || content.contains("function"));
                }
                Err(e) => {
                    error = Some(format!("Failed to read file: {}", e));
                }
            }
        } else {
            error = Some("File does not exist".to_string());
        }
        
        crate::types::FileValidationResult {
            file_path: script_path.to_string(),
            exists,
            readable,
            size_bytes,
            last_modified,
            content_hash,
            syntax_valid,
            error,
        }
    }
    
    /// Get V8 runtime statistics
    fn get_v8_runtime_stats(&self) -> EditorResponse {
        self.log("Getting V8 runtime stats".to_string());
        
        // TODO: Get actual V8 runtime statistics
        let stats = crate::types::V8RuntimeStats {
            heap_used_bytes: 0,
            heap_total_bytes: 0,
            external_memory_bytes: 0,
            script_instances_count: 0,
            global_context_available: true,
        };
        
        EditorResponse::V8RuntimeStats(stats)
    }
    
    /// Trigger script recompilation
    fn trigger_script_recompilation(&self, script_path: String) -> EditorResponse {
        self.log(format!("Triggering recompilation for script: {}", script_path));
        
        // Add a compilation event to trigger recompilation
        engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
            script_path: script_path.clone(),
        });
        
        self.log(format!("Added compilation event for: {}", script_path));
        EditorResponse::Success
    }
    
    /// Simulate file change for testing
    fn simulate_file_change(&self, script_path: String) -> EditorResponse {
        self.log(format!("Simulating file change for: {}", script_path));
        
        // Touch the file to update its modification time
        if let Ok(content) = std::fs::read_to_string(&script_path) {
            if let Err(e) = std::fs::write(&script_path, content) {
                return EditorResponse::Error {
                    message: format!("Failed to touch file {}: {}", script_path, e),
                };
            }
        }
        
        // Trigger compilation event
        engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
            script_path: script_path.clone(),
        });
        
        self.log(format!("Simulated file change and triggered compilation for: {}", script_path));
        EditorResponse::Success
    }
}