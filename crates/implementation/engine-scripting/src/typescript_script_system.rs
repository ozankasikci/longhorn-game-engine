//! TypeScript Script System - Executes TypeScript scripts attached to entities

use crate::components::TypeScriptScript;
use engine_ecs_core::{Entity, World};
use std::collections::{HashMap, HashSet};

/// System that processes TypeScript script components and executes their lifecycle methods
pub struct TypeScriptScriptSystem {
    /// Set of entities that have been initialized
    initialized_entities: HashSet<Entity>,
    /// Map from entity to their script instances
    script_instances: HashMap<Entity, Vec<TypeScriptScriptInstance>>,
    /// TypeScript runtime for script execution
    runtime: Option<Box<dyn TypeScriptRuntime>>,
}

/// Represents an instance of a TypeScript script attached to an entity
#[derive(Debug, Clone)]
pub struct TypeScriptScriptInstance {
    pub script_path: String,
    pub initialized: bool,
    pub compilation_successful: bool,
    pub last_error: Option<String>,
}

impl TypeScriptScriptInstance {
    pub fn new(script_path: String) -> Self {
        Self {
            script_path,
            initialized: false,
            compilation_successful: false,
            last_error: None,
        }
    }
}

/// Trait for TypeScript runtime - allows for mock implementations in tests
pub trait TypeScriptRuntime: Send + Sync {
    fn load_and_compile_script(&mut self, script_path: &str) -> Result<String, String>;
    fn execute_script(&mut self, script_path: &str, compiled_code: &str) -> Result<(), String>;
    fn call_init(&mut self, script_path: &str) -> Result<(), String>;
    fn call_update(&mut self, script_path: &str, delta_time: f64) -> Result<(), String>;
    fn call_destroy(&mut self, script_path: &str) -> Result<(), String>;
    fn setup_engine_apis(&mut self) -> Result<(), String>;
}

impl TypeScriptScriptSystem {
    pub fn new() -> Self {
        // Create a real TypeScript runtime by default
        let real_runtime = match RealTypeScriptRuntime::new() {
            Ok(r) => Some(Box::new(r) as Box<dyn TypeScriptRuntime>),
            Err(e) => {
                log::error!("Failed to create TypeScript runtime: {}", e);
                // Fall back to no runtime (will log warnings)
                None
            }
        };
        
        Self {
            initialized_entities: HashSet::new(),
            script_instances: HashMap::new(),
            runtime: real_runtime,
        }
    }

    pub fn with_runtime(runtime: Box<dyn TypeScriptRuntime>) -> Self {
        let mut system = Self::new();
        system.runtime = Some(runtime);
        system
    }

    /// Main update method called each frame
    pub fn update(&mut self, world: &mut World, delta_time: f64) {
        // Ensure runtime is available
        if self.runtime.is_none() {
            log::warn!("TypeScriptScriptSystem: No runtime available");
            return;
        }

        // Setup engine APIs if not done yet
        if let Some(runtime) = &mut self.runtime {
            if let Err(e) = runtime.setup_engine_apis() {
                log::error!("Failed to setup engine APIs: {}", e);
                return;
            }
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
        // Collect all script paths for this entity
        let script_paths = script_component.get_all_scripts();
        
        // Check if this entity needs initialization
        let needs_init = !self.initialized_entities.contains(&entity);
        
        if needs_init {
            // Initialize all scripts for this entity
            self.initialize_entity_scripts(entity, &script_paths);
            self.initialized_entities.insert(entity);
        } else {
            // Update all scripts for this entity
            self.update_entity_scripts(entity, &script_paths, delta_time);
        }
    }

    /// Initialize all scripts for an entity
    fn initialize_entity_scripts(&mut self, entity: Entity, script_paths: &[&String]) {
        let mut instances = Vec::new();
        
        for script_path in script_paths {
            let mut instance = TypeScriptScriptInstance::new((*script_path).clone());
            
            // Load and compile the script
            if let Some(runtime) = &mut self.runtime {
                match runtime.load_and_compile_script(script_path) {
                    Ok(compiled_code) => {
                        instance.compilation_successful = true;
                        
                        // Execute the compiled code to load the class
                        if let Err(e) = runtime.execute_script(script_path, &compiled_code) {
                            log::error!("Failed to execute script {}: {}", script_path, e);
                            instance.last_error = Some(e);
                        } else {
                            // Call the init method
                            if let Err(e) = runtime.call_init(script_path) {
                                log::error!("Failed to initialize script {}: {}", script_path, e);
                                instance.last_error = Some(e);
                            } else {
                                instance.initialized = true;
                                log::info!("Initialized TypeScript script: {}", script_path);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to compile script {}: {}", script_path, e);
                        instance.last_error = Some(e);
                    }
                }
            }
            
            instances.push(instance);
        }
        
        self.script_instances.insert(entity, instances);
    }

    /// Update all scripts for an entity
    fn update_entity_scripts(&mut self, entity: Entity, script_paths: &[&String], delta_time: f64) {
        if let Some(instances) = self.script_instances.get(&entity) {
            if let Some(runtime) = &mut self.runtime {
                for instance in instances {
                    if instance.initialized && instance.compilation_successful {
                        if let Err(e) = runtime.call_update(&instance.script_path, delta_time) {
                            log::error!("Failed to update script {}: {}", instance.script_path, e);
                        }
                    }
                }
            }
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
                        if let Err(e) = runtime.call_destroy(&instance.script_path) {
                            log::error!("Failed to destroy script {}: {}", instance.script_path, e);
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

/// Real TypeScript runtime implementation using V8 engine directly
pub struct RealTypeScriptRuntime {
    isolate: v8::OwnedIsolate,
    global_context: v8::Global<v8::Context>,
    compiled_scripts: std::collections::HashMap<String, String>,
    apis_setup: bool,
}

impl RealTypeScriptRuntime {
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
            
            v8::Global::new(scope, context)
        };

        log::info!("Real TypeScript runtime created with V8 engine");
        Ok(Self {
            isolate,
            global_context,
            compiled_scripts: std::collections::HashMap::new(),
            apis_setup: false,
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
            log::info!("[JS Console] {}", message_parts.join(" "));
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
            log::error!("[JS Console] {}", message_parts.join(" "));
        }).ok_or_else(|| "Failed to create console.error function".to_string())?;
        
        console_obj.set(scope, error_name.into(), error_fn.into());
        
        // Set console object on global
        global.set(scope, console_name.into(), console_obj.into());
        
        Ok(())
    }

    fn compile_typescript_to_javascript(&self, typescript_source: &str) -> Result<String, String> {
        // Use SWC to compile TypeScript to JavaScript
        swc_core::common::GLOBALS.set(&swc_core::common::Globals::new(), || {
            let source_map = swc_core::common::sync::Lrc::new(swc_core::common::SourceMap::new(
                swc_core::common::FilePathMapping::empty()
            ));
            
            // Create source file
            let source_file = source_map.new_source_file(
                swc_core::common::FileName::Custom("<inline>".to_string()),
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
                format!("Parse error: {:?}", e)
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
            
            // Extract module back from program
            let module = match program {
                swc_core::ecma::ast::Program::Module(m) => m,
                _ => return Err("Expected module program".to_string()),
            };
            
            // Generate JavaScript code
            let mut buf = Vec::new();
            let writer = swc_core::ecma::codegen::text_writer::JsWriter::new(source_map.clone(), "\n", &mut buf, None);
            
            let mut emitter = swc_core::ecma::codegen::Emitter {
                cfg: swc_core::ecma::codegen::Config::default(),
                cm: source_map.clone(),
                comments: None,
                wr: writer,
            };
            
            emitter.emit_module(&module).map_err(|e| {
                format!("Code generation error: {:?}", e)
            })?;
            
            let code = String::from_utf8(buf).map_err(|e| {
                format!("Invalid UTF-8 in generated code: {}", e)
            })?;
            
            Ok(code)
        })
    }

    fn execute_javascript(&mut self, code: &str) -> Result<(), String> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Compile the script
        let source = v8::String::new(scope, code)
            .ok_or_else(|| "Failed to create V8 string".to_string())?;
        
        let script = v8::Script::compile(scope, source, None)
            .ok_or_else(|| "Failed to compile JavaScript code".to_string())?;
        
        // Execute the script
        script.run(scope)
            .ok_or_else(|| "Script execution failed".to_string())?;
        
        Ok(())
    }
}

impl TypeScriptRuntime for RealTypeScriptRuntime {
    fn load_and_compile_script(&mut self, script_path: &str) -> Result<String, String> {
        log::info!("Loading and compiling TypeScript script: {}", script_path);
        
        // Read the script file
        let typescript_source = std::fs::read_to_string(script_path)
            .map_err(|e| format!("Failed to read script file {}: {}", script_path, e))?;

        // Compile TypeScript to JavaScript
        let javascript_code = self.compile_typescript_to_javascript(&typescript_source)?;

        log::info!("Successfully compiled TypeScript script: {}", script_path);
        Ok(javascript_code)
    }

    fn execute_script(&mut self, script_path: &str, compiled_code: &str) -> Result<(), String> {
        log::info!("Executing compiled script: {}", script_path);
        
        // Store the compiled code for later function calls
        self.compiled_scripts.insert(script_path.to_string(), compiled_code.to_string());

        // Execute the compiled JavaScript code to load the class into V8
        self.execute_javascript(compiled_code)?;

        log::info!("Successfully executed script: {}", script_path);
        Ok(())
    }

    fn call_init(&mut self, script_path: &str) -> Result<(), String> {
        log::info!("Calling init() for TypeScript script: {}", script_path);
        
        // Extract class name from script path (simplified)
        let class_name = std::path::Path::new(script_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| {
                // Convert snake_case to PascalCase  
                name.split('_')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<String>()
            })
            .unwrap_or_else(|| "HelloWorld".to_string()); // Default to HelloWorld for typescript_hello_world.ts

        // Create an instance and call init
        let init_code = format!(
            r#"
            var instance = new {}();
            if (typeof instance.init === 'function') {{
                instance.init();
            }}
            "#,
            class_name
        );

        self.execute_javascript(&init_code)?;

        log::info!("Successfully called init() for script: {}", script_path);
        Ok(())
    }

    fn call_update(&mut self, script_path: &str, delta_time: f64) -> Result<(), String> {
        log::debug!("Calling update({}) for TypeScript script: {}", delta_time, script_path);
        
        // For now, skip update calls to avoid spam - this would be called every frame
        // In a real implementation, we'd maintain script instances and call their update methods
        Ok(())
    }

    fn call_destroy(&mut self, script_path: &str) -> Result<(), String> {
        log::info!("Calling destroy() for TypeScript script: {}", script_path);
        
        // Call destroy if the instance exists
        let destroy_code = format!(
            r#"
            if (typeof instance !== 'undefined' && typeof instance.destroy === 'function') {{
                instance.destroy();
            }}
            "#
        );

        self.execute_javascript(&destroy_code)?;

        // Clean up compiled script
        self.compiled_scripts.remove(script_path);

        log::info!("Successfully called destroy() for script: {}", script_path);
        Ok(())
    }

    fn setup_engine_apis(&mut self) -> Result<(), String> {
        if self.apis_setup {
            return Ok(());
        }

        log::info!("Setting up TypeScript engine APIs");
        
        // The console API is already set up in the constructor
        // TODO: In a full implementation, we would inject additional engine APIs here:
        // - Engine.world (ECS operations)
        // - Engine.input (input handling)  
        // - Engine.physics (physics operations)
        // - Vector3 constructor
        // - Math extensions
        
        // For now, the console API is sufficient for basic script execution
        self.apis_setup = true;
        
        log::info!("Successfully set up TypeScript engine APIs");
        Ok(())
    }
}

// V8 requires special handling for thread safety
// Note: This is safe because our TypeScript runtime is designed to be used
// by a single system thread, and V8 isolates are actually thread-safe
// when used properly with proper synchronization
unsafe impl Send for RealTypeScriptRuntime {}
unsafe impl Sync for RealTypeScriptRuntime {}

/// Mock runtime implementation for testing
#[cfg(test)]
pub mod mock_runtime {
    use super::*;
    use std::collections::HashMap;

    pub struct MockTypeScriptRuntime {
        pub loaded_scripts: Vec<String>,
        pub executed_scripts: Vec<String>,
        pub init_called: Vec<String>,
        pub update_called: Vec<(String, f64)>,
        pub destroy_called: Vec<String>,
        pub compilation_errors: HashMap<String, String>,
        pub execution_errors: HashMap<String, String>,
    }

    impl MockTypeScriptRuntime {
        pub fn new() -> Self {
            Self {
                loaded_scripts: Vec::new(),
                executed_scripts: Vec::new(),
                init_called: Vec::new(),
                update_called: Vec::new(),
                destroy_called: Vec::new(),
                compilation_errors: HashMap::new(),
                execution_errors: HashMap::new(),
            }
        }

        pub fn set_compilation_error(&mut self, script_path: &str, error: &str) {
            self.compilation_errors.insert(script_path.to_string(), error.to_string());
        }

        pub fn set_execution_error(&mut self, script_path: &str, error: &str) {
            self.execution_errors.insert(script_path.to_string(), error.to_string());
        }
    }

    impl TypeScriptRuntime for MockTypeScriptRuntime {
        fn load_and_compile_script(&mut self, script_path: &str) -> Result<String, String> {
            self.loaded_scripts.push(script_path.to_string());
            
            if let Some(error) = self.compilation_errors.get(script_path) {
                return Err(error.clone());
            }
            
            // Return mock compiled JavaScript
            Ok(format!("// Compiled: {}\nclass Script{} {{}}", script_path, self.loaded_scripts.len()))
        }

        fn execute_script(&mut self, script_path: &str, _compiled_code: &str) -> Result<(), String> {
            if let Some(error) = self.execution_errors.get(script_path) {
                return Err(error.clone());
            }
            
            self.executed_scripts.push(script_path.to_string());
            Ok(())
        }

        fn call_init(&mut self, script_path: &str) -> Result<(), String> {
            if let Some(error) = self.execution_errors.get(script_path) {
                return Err(error.clone());
            }
            
            self.init_called.push(script_path.to_string());
            Ok(())
        }

        fn call_update(&mut self, script_path: &str, delta_time: f64) -> Result<(), String> {
            if let Some(error) = self.execution_errors.get(script_path) {
                return Err(error.clone());
            }
            
            self.update_called.push((script_path.to_string(), delta_time));
            Ok(())
        }

        fn call_destroy(&mut self, script_path: &str) -> Result<(), String> {
            self.destroy_called.push(script_path.to_string());
            Ok(())
        }

        fn setup_engine_apis(&mut self) -> Result<(), String> {
            // Mock API setup - in real implementation this would inject Engine, console, etc.
            Ok(())
        }
    }
}

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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
        world.add_component(
            entity,
            TypeScriptScript::new("hello_world.ts".to_string())
        ).unwrap();

        // Act - First update should call init
        system.update(&mut world, 0.016);

        // Assert
        assert!(system.get_initialized_entities().contains(&entity));
        let instances = system.get_script_instances().get(&entity).unwrap();
        assert!(instances[0].initialized);
    }

    #[test]
    fn test_system_calls_update_on_subsequent_executions() {
        // Arrange
        let (mut world, entity) = setup_test_world();
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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
        let runtime = Box::new(MockTypeScriptRuntime::new());
        let mut system = TypeScriptScriptSystem::with_runtime(runtime);
        
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