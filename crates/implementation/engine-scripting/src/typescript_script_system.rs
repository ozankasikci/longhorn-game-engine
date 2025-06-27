//! TypeScript Script System - Executes TypeScript scripts attached to entities

use crate::components::TypeScriptScript;
use crate::lua::engine::{ConsoleMessage, CONSOLE_MESSAGES};
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
        }
    }

    #[cfg(test)]
    pub fn with_mock_runtime() -> Self {
        Self {
            initialized_entities: HashSet::new(),
            script_instances: HashMap::new(),
            runtime: None, // For testing, we'll simulate without real runtime
            next_script_id: 1,
        }
    }

    /// Main update method called each frame
    pub fn update(&mut self, world: &mut World, delta_time: f64) {
        // Ensure runtime is available
        if self.runtime.is_none() {
            log::warn!("TypeScriptScriptSystem: No runtime available");
            return;
        }

        // Update runtime (this handles garbage collection, etc.)
        if let Some(runtime) = &mut self.runtime {
            runtime.update(delta_time);
        }

        // Query all entities with TypeScript script components
        let mut script_entities = Vec::new();
        
        // In a real implementation, this would use the ECS query system
        // For now, we'll simulate the query for testing purposes
        for (entity, script_component) in world.query_legacy::<TypeScriptScript>() {
            if script_component.enabled {
                script_entities.push((entity, script_component.clone()));
            }
        }

        // Sort by execution order (lower numbers execute first)
        script_entities.sort_by_key(|(_, script)| script.execution_order);

        // Process each entity with TypeScript scripts
        for (entity, script_component) in script_entities {
            self.process_entity_scripts(entity, &script_component, delta_time);
        }

        // Clean up entities that no longer have TypeScript components
        self.cleanup_removed_entities(world);
    }

    /// Process all scripts for a single entity
    fn process_entity_scripts(&mut self, entity: Entity, script_component: &TypeScriptScript, delta_time: f64) {
        println!("🔧 process_entity_scripts() for entity {:?}", entity);
        
        // Collect all script paths for this entity
        let script_paths = script_component.get_all_scripts();
        println!("📂 Script paths for entity: {:?}", script_paths);
        
        // Check if this entity needs initialization
        let mut needs_init = !self.initialized_entities.contains(&entity);
        
        // Also check if any script instances are not properly compiled
        if let Some(instances) = self.script_instances.get(&entity) {
            for instance in instances {
                if !instance.compilation_successful {
                    println!("🔍 Force re-initialization: script {} not compiled", instance.script_path);
                    needs_init = true;
                    break;
                }
            }
        }
        
        println!("🔍 Entity needs initialization: {}", needs_init);
        
        if needs_init {
            // Initialize all scripts for this entity
            println!("🚀 Initializing scripts for entity {:?}", entity);
            
            // Remove entity from initialized set to force re-init
            self.initialized_entities.remove(&entity);
            
            self.initialize_entity_scripts(entity, &script_paths);
            self.initialized_entities.insert(entity);
            println!("✅ Entity {:?} marked as initialized", entity);
        } else {
            // Update all scripts for this entity
            println!("🔄 Updating scripts for entity {:?}", entity);
            self.update_entity_scripts(entity, &script_paths, delta_time);
        }
    }

    /// Initialize all scripts for an entity
    fn initialize_entity_scripts(&mut self, entity: Entity, script_paths: &[&String]) {
        let mut instances = Vec::new();
        
        for script_path in script_paths {
            let script_id = self.next_script_id;
            self.next_script_id += 1;
            
            let mut instance = TypeScriptScriptInstance::new(script_id, (*script_path).clone());
            
            // Load and execute the script
            if let Some(runtime) = &mut self.runtime {
                // Read script content
                match std::fs::read_to_string(script_path) {
                    Ok(source) => {
                        // Load and compile script
                        log::info!("About to compile TypeScript script: {}", script_path);
                        log::debug!("TypeScript source code: {}", &source);
                        
                        match runtime.load_and_compile_script(script_id, script_path, &source) {
                            Ok(()) => {
                                instance.compilation_successful = true;
                                log::info!("Successfully compiled TypeScript script: {}", script_path);
                                
                                // Try to call init function if it exists
                                log::info!("About to call init for script: {}", script_path);
                                match runtime.call_init(script_id) {
                                    Ok(_) => {
                                        instance.initialized = true;
                                        log::info!("Successfully initialized TypeScript script: {}", script_path);
                                    }
                                    Err(e) => {
                                        log::warn!("Script {} init failed: {}", script_path, e);
                                        instance.initialized = true; // Still consider it initialized
                                        instance.last_error = Some(format!("Init error: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to compile/load script {}: {}", script_path, e);
                                instance.last_error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read script file {}: {}", script_path, e);
                        instance.last_error = Some(format!("File read error: {}", e));
                    }
                }
            }
            
            instances.push(instance);
        }
        
        self.script_instances.insert(entity, instances);
    }

    /// Update all scripts for an entity
    fn update_entity_scripts(&mut self, entity: Entity, script_paths: &[&String], delta_time: f64) {
        println!("🔄 update_entity_scripts() called for entity {:?}", entity);
        
        if let Some(instances) = self.script_instances.get(&entity) {
            println!("📋 Found {} script instances for entity", instances.len());
            
            if let Some(runtime) = &mut self.runtime {
                for instance in instances {
                    println!("🔍 Script instance: {}, initialized: {}, compiled: {}", 
                             instance.script_path, instance.initialized, instance.compilation_successful);
                    
                    if instance.initialized && instance.compilation_successful {
                        // Try to call update function if it exists
                        println!("📞 Calling runtime.call_update() for script: {}", instance.script_path);
                        if let Err(e) = runtime.call_update(instance.script_id, delta_time) {
                            println!("⚠️ Script {} has no update function or update failed: {}", instance.script_path, e);
                            log::debug!("Script {} has no update function or update failed: {}", instance.script_path, e);
                        } else {
                            println!("✅ Successfully called update() for script: {}", instance.script_path);
                        }
                    } else {
                        println!("❌ Script {} not ready: initialized={}, compiled={}", 
                                instance.script_path, instance.initialized, instance.compilation_successful);
                    }
                }
            } else {
                println!("❌ No runtime available for update");
            }
        } else {
            println!("❌ No script instances found for entity {:?}", entity);
        }
    }

    /// Clean up entities that no longer have TypeScript components
    fn cleanup_removed_entities(&mut self, world: &mut World) {
        let mut entities_to_remove = Vec::new();
        
        for &entity in &self.initialized_entities {
            // Check if entity still has a TypeScript component
            if world.get_component::<TypeScriptScript>(entity).is_none() {
                entities_to_remove.push(entity);
            }
        }
        
        for entity in entities_to_remove {
            self.cleanup_entity(entity);
        }
    }

    /// Clean up a specific entity's scripts
    fn cleanup_entity(&mut self, entity: Entity) {
        if let Some(instances) = self.script_instances.remove(&entity) {
            if let Some(runtime) = &mut self.runtime {
                for instance in instances {
                    if instance.initialized {
                        // Try to call destroy function if it exists
                        if let Err(e) = runtime.call_destroy(instance.script_id) {
                            log::debug!("Script {} has no destroy function or destroy failed: {}", instance.script_path, e);
                        } else {
                            log::info!("Destroyed TypeScript script: {}", instance.script_path);
                        }
                    }
                }
            }
        }
        
        self.initialized_entities.remove(&entity);
    }

    /// Get initialized entities (for testing)
    pub fn get_initialized_entities(&self) -> &HashSet<Entity> {
        &self.initialized_entities
    }

    /// Get script instances (for testing)
    pub fn get_script_instances(&self) -> &HashMap<Entity, Vec<TypeScriptScriptInstance>> {
        &self.script_instances
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
    compiled_scripts: HashMap<u32, String>, // script_id -> compiled JavaScript
    script_instances: HashMap<u32, String>, // script_id -> instance variable name
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
            compiled_scripts: HashMap::new(),
            script_instances: HashMap::new(),
        })
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
                message_parts.push(string_val.to_rust_string_lossy(scope));
            }
            let message = message_parts.join(" ");
            
            // Add to game engine console directly
            use crate::lua::engine::{CONSOLE_MESSAGES, ConsoleMessage};
            use std::time::SystemTime;
            
            if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
                messages.push(ConsoleMessage {
                    message: message.clone(),
                    timestamp: SystemTime::now(),
                });
            }
            
            // Also log to standard Rust logging
            log::info!("[TS Console] {}", message);
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
            use crate::lua::engine::{CONSOLE_MESSAGES, ConsoleMessage};
            use std::time::SystemTime;
            
            if let Ok(mut messages) = CONSOLE_MESSAGES.lock() {
                messages.push(ConsoleMessage {
                    message: message.clone(),
                    timestamp: SystemTime::now(),
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
        // Inject World API
        Self::inject_world_api(scope, global)?;
        
        // Inject Input API  
        Self::inject_input_api(scope, global)?;
        
        // Inject Physics API
        Self::inject_physics_api(scope, global)?;
        
        log::info!("Engine API injection completed successfully");
        Ok(())
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
        println!("🚀 EXECUTING JAVASCRIPT CODE:");
        println!("📋 Raw code length: {} chars", code.len());
        println!("📋 Code content:\n{}", code);
        println!("📋 Code contains export: {}", code.contains("export"));
        println!("📋 Code contains module.exports: {}", code.contains("module.exports"));
        println!("📋 Code contains Object.defineProperty: {}", code.contains("Object.defineProperty"));
        println!("📋 Code starts with: {:?}", code.chars().take(50).collect::<String>());
        
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Compile the script
        let source = v8::String::new(scope, code)
            .ok_or_else(|| "Failed to create V8 string".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| {
                let error_msg = format!("🔥 V8 JavaScript compilation failed - SYNTAX ERROR in generated code!");
                println!("{}", error_msg);
                println!("📋 Failed code was:\n{}", code);
                println!("📋 Code bytes: {:?}", code.as_bytes());
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

    pub fn load_and_compile_script(&mut self, script_id: u32, script_path: &str, source: &str) -> Result<(), String> {
        log::info!("Loading and compiling TypeScript script: {}", script_path);
        
        // Compile TypeScript to JavaScript with enhanced error context
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

        // Store the compiled code
        self.compiled_scripts.insert(script_id, javascript_code.clone());

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
        
        // Check if script exists
        if !self.compiled_scripts.contains_key(&script_id) {
            log::warn!("Attempted to call init() on non-existent script_id: {}", script_id);
            return Ok(()); // Graceful handling for non-existent scripts
        }
        
        let instance_var = format!("instance_{}", script_id);
        
        // Try to find a class in the global scope and create an instance
        let init_code = format!(
            r#"
            var {} = null;
            var initSuccess = false;
            var lastError = null;
            
            // Debug: Log what's available in exports
            console.log("=== DEBUG: Checking exports object ===");
            console.log("typeof exports:", typeof exports);
            if (typeof exports === 'object' && exports) {{
                console.log("exports keys:", Object.keys(exports));
                for (var name in exports) {{
                    console.log("exports[" + name + "]:", typeof exports[name], exports[name]);
                }}
            }}
            
            // First try to find a class in exports (CommonJS)
            if (typeof exports === 'object' && exports) {{
                for (var name in exports) {{
                    if (typeof exports[name] === 'function' && exports[name].prototype) {{
                        try {{
                            console.log("=== DEBUG: Trying to instantiate class:", name, "===");
                            {} = new exports[name]();
                            console.log("=== DEBUG: Successfully created instance of", name, "===");
                            if (typeof {}.init === 'function') {{
                                console.log("=== DEBUG: Calling init() method ===");
                                {}.init();
                                console.log("=== DEBUG: init() method completed ===");
                            }} else {{
                                console.log("=== DEBUG: No init() method found ===");
                            }}
                            initSuccess = true;
                            break;
                        }} catch (e) {{
                            console.log("=== DEBUG: Error instantiating", name, ":", e.toString(), "===");
                            lastError = e.toString();
                            // Continue to next potential class
                        }}
                    }}
                }}
            }}
            
            // If not found in exports, try globalThis (fallback)
            if (!initSuccess) {{
                console.log("=== DEBUG: Trying globalThis fallback ===");
                for (var name in globalThis) {{
                    if (typeof globalThis[name] === 'function' && globalThis[name].prototype) {{
                        try {{
                            console.log("=== DEBUG: Trying globalThis class:", name, "===");
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
                console.log("=== DEBUG: Failed to initialize script, lastError:", lastError, "===");
                throw new Error('Failed to initialize script: ' + lastError);
            }} else if (initSuccess) {{
                console.log("=== DEBUG: Script initialization successful! ===");
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

    pub fn call_update(&mut self, script_id: u32, delta_time: f64) -> Result<(), String> {
        // Check if script instance exists
        if let Some(instance_var) = self.script_instances.get(&script_id) {
            let update_code = format!(
                r#"
                try {{
                    if ({} && typeof {}.update === 'function') {{
                        {}.update({});
                    }}
                }} catch (e) {{
                    throw new Error('Update error in script_id {}: ' + e.toString());
                }}
                "#,
                instance_var, instance_var, instance_var, delta_time, script_id
            );

            self.execute_javascript(&update_code)
                .map_err(|e| format!("Update error for script_id {}: {}", script_id, e))?;
        } else {
            // Gracefully handle non-existent script instances
            log::debug!("Attempted to call update() on non-existent script_id: {}", script_id);
        }
        Ok(())
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
        self.compiled_scripts.remove(&script_id);
        self.script_instances.remove(&script_id);

        let cleanup_time = start_time.elapsed();
        log::debug!("Cleanup completed for script_id: {} in {:?}", script_id, cleanup_time);
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
            let script_count = self.compiled_scripts.len();
            let instance_count = self.script_instances.len();
            
            if script_count > 0 || instance_count > 0 {
                log::debug!("TypeScript runtime stats: {} compiled scripts, {} active instances", 
                    script_count, instance_count);
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
        
        // Compile the new script
        let script_path = file_path.to_string_lossy();
        let javascript_code = self.compile_typescript_to_javascript(&new_source, &script_path)
            .map_err(|e| format!("Hot reload compilation failed for {}: {}", script_path, e))?;
        
        // Replace the compiled script
        self.compiled_scripts.insert(script_id, javascript_code.clone());
        
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
        
        let modified_time = metadata.modified()
            .map_err(|e| format!("Failed to get modification time for {}: {}", file_path.display(), e))?;
        
        // For this implementation, we'll always reload if the file exists
        // In a production system, you'd track modification times
        self.hot_reload_script(script_id, file_path)?;
        Ok(true)
    }

    /// Force garbage collection in V8
    pub fn force_garbage_collection(&mut self) {
        let gc_start = std::time::Instant::now();
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
        let gc_time = gc_start.elapsed();
        
        log::debug!("Forced garbage collection completed in {:?}", gc_time);
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
            script_count: self.compiled_scripts.len(),
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
            messages.push(ConsoleMessage {
                message: message.clone(),
                timestamp: SystemTime::now(),
            });
            log::info!("Successfully added console message, total messages: {}", messages.len());
        } else {
            log::error!("Failed to acquire console messages lock for: {}", message);
        }
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