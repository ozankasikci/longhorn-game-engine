//! Command execution logic for editor control

use crate::types::*;
use engine_ecs_core::{Entity, World};
use engine_scripting::components::TypeScriptScript;
use engine_scripting::lua::engine::{CONSOLE_MESSAGES, ConsoleMessage};
use engine_components_3d::Transform;
use engine_components_ui::Name;
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
        }
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
}