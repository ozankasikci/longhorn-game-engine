// crates/longhorn-scripting/src/runtime.rs
use crate::compiler::{CompiledScript, TypeScriptCompiler};
use crate::js_runtime::LonghornJsRuntime;
use crate::BOOTSTRAP_JS;
use longhorn_core::{Script, World, LonghornError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Unique identifier for a script instance (entity_id, script_path)
type ScriptInstanceId = (u64, String);

/// Runtime state for a single script instance
struct ScriptInstance {
    /// Compiled script reference
    script_path: String,
    /// Instance properties (copied from Script component)
    #[allow(dead_code)] // TODO: Will be used when calling JS methods
    properties: HashMap<String, String>,
    /// Whether onStart has been called
    started: bool,
    /// Whether the script is enabled
    enabled: bool,
}

/// Script runtime - manages TypeScript execution via Deno Core
pub struct ScriptRuntime {
    /// Compiled scripts cache (path -> compiled)
    compiled_scripts: HashMap<String, CompiledScript>,
    /// Active script instances
    instances: HashMap<ScriptInstanceId, ScriptInstance>,
    /// TypeScript compiler
    compiler: TypeScriptCompiler,
    /// JavaScript runtime
    js_runtime: Option<LonghornJsRuntime>,
    /// Game path (scripts directory)
    game_path: Option<String>,
    /// Whether the runtime has been initialized
    initialized: bool,
    /// Script execution error (pauses game)
    error: Option<String>,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            compiled_scripts: HashMap::new(),
            instances: HashMap::new(),
            compiler: TypeScriptCompiler::new(),
            js_runtime: None,
            game_path: None,
            initialized: false,
            error: None,
        }
    }

    /// Load a game from a directory containing scripts/
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(LonghornError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Game directory not found: {}", path.display()),
            )));
        }

        self.game_path = Some(path.display().to_string());
        self.compiled_scripts.clear();
        self.instances.clear();

        // Compile all scripts in scripts/ directory
        let scripts_dir = path.join("scripts");
        if scripts_dir.exists() {
            self.compile_directory(&scripts_dir)?;
        }

        log::info!(
            "Loaded {} scripts from: {}",
            self.compiled_scripts.len(),
            path.display()
        );

        Ok(())
    }

    /// Compile all TypeScript files in a directory
    fn compile_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(LonghornError::Io)? {
            let entry = entry.map_err(LonghornError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "ts" || ext == "js" {
                        match self.compiler.compile_file(&path) {
                            Ok(compiled) => {
                                let rel_path = path
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string();
                                log::debug!("Compiled script: {}", rel_path);
                                self.compiled_scripts.insert(rel_path, compiled);
                            }
                            Err(e) => {
                                log::error!("Failed to compile {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize the game (create JS runtime, load scripts, call onStart)
    pub fn initialize(&mut self, world: &mut World) -> Result<()> {
        if self.game_path.is_none() {
            return Err(LonghornError::Scripting("No game loaded".to_string()));
        }

        // Create JS runtime
        let mut js_runtime = LonghornJsRuntime::new();

        // Execute bootstrap (defines Entity, __scripts, __instances, console)
        js_runtime
            .execute_script("longhorn:bootstrap", BOOTSTRAP_JS)
            .map_err(|e| LonghornError::Scripting(format!("Bootstrap failed: {}", e)))?;

        // Load all compiled scripts into JS
        for (path, compiled) in &self.compiled_scripts {
            let register_code = format!(
                r#"__scripts["{}"] = (() => {{
                {};
                return {};
            }})();"#,
                path, compiled.js_code, compiled.class_name
            );
            js_runtime
                .execute_script("longhorn:load_script", &register_code)
                .map_err(|e| LonghornError::Scripting(format!("Failed to load {}: {}", path, e)))?;

            log::debug!("Loaded script: {} (class: {})", path, compiled.class_name);
        }

        self.js_runtime = Some(js_runtime);

        // Sync instances and call onStart
        self.sync_instances(world);
        self.run_lifecycle("onStart", world, 0.0)?;

        self.initialized = true;
        log::info!("Script runtime initialized with {} scripts", self.compiled_scripts.len());

        Ok(())
    }

    /// Sync script instances with world state
    fn sync_instances(&mut self, world: &World) {
        // Query all entities with Script components
        for (entity_id, script) in world.query::<&Script>().iter() {
            let instance_id = (entity_id.to_bits().get(), script.path.clone());

            if !self.instances.contains_key(&instance_id) {
                // Create new instance
                let properties: HashMap<String, String> = script
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::to_string(v).unwrap_or_default()))
                    .collect();

                self.instances.insert(
                    instance_id,
                    ScriptInstance {
                        script_path: script.path.clone(),
                        properties,
                        started: false,
                        enabled: script.enabled,
                    },
                );
            }
        }

        // TODO: Remove instances for despawned entities
    }

    /// Run a lifecycle method on all script instances
    fn run_lifecycle(&mut self, method: &str, _world: &mut World, _dt: f32) -> Result<()> {
        // Check if we have an error (game should be paused)
        if self.error.is_some() {
            return Err(LonghornError::Scripting(self.error.clone().unwrap()));
        }

        // Get sorted instance IDs by execution order
        let mut sorted_instances: Vec<_> = self.instances.keys().cloned().collect();
        sorted_instances.sort_by(|a, b| {
            let order_a = self
                .compiled_scripts
                .get(&a.1)
                .map(|s| s.execution_order)
                .unwrap_or(0);
            let order_b = self
                .compiled_scripts
                .get(&b.1)
                .map(|s| s.execution_order)
                .unwrap_or(0);
            order_a.cmp(&order_b).then_with(|| a.0.cmp(&b.0))
        });

        // Call method on each instance
        for instance_id in sorted_instances {
            if let Some(instance) = self.instances.get_mut(&instance_id) {
                if !instance.enabled {
                    continue;
                }

                // Skip onStart if already called
                if method == "onStart" && instance.started {
                    continue;
                }

                // TODO: Actually call the JS method
                // For now, just log
                log::debug!(
                    "Would call {}.{}(entity={})",
                    instance.script_path,
                    method,
                    instance_id.0
                );

                if method == "onStart" {
                    instance.started = true;
                }
            }
        }

        Ok(())
    }

    /// Update the game (call onUpdate)
    pub fn update(&mut self, world: &mut World, delta: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        // Sync instances first
        self.sync_instances(world);

        // Run onUpdate
        self.run_lifecycle("onUpdate", world, delta)
    }

    /// Handle touch start event
    pub fn on_touch_start(&mut self, _world: &mut World, _x: f32, _y: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        // Touch events are handled via input API, not lifecycle methods
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.game_path.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Get list of available script paths
    pub fn available_scripts(&self) -> Vec<&str> {
        self.compiled_scripts.keys().map(|s| s.as_str()).collect()
    }

    /// Get property definitions for a script
    pub fn script_properties(&self, path: &str) -> Option<&HashMap<String, String>> {
        self.compiled_scripts.get(path).map(|s| &s.properties)
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_runtime_new() {
        let runtime = ScriptRuntime::new();
        assert!(!runtime.is_loaded());
        assert!(!runtime.is_initialized());
    }

    #[test]
    fn test_script_runtime_load_nonexistent() {
        let mut runtime = ScriptRuntime::new();
        let result = runtime.load_game("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_script_runtime_initialize_without_load() {
        let mut runtime = ScriptRuntime::new();
        let mut world = World::new();
        let result = runtime.initialize(&mut world);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_game() {
        // Create a temporary test game directory
        let test_dir = std::env::temp_dir().join("test_game_runtime_load");
        let scripts_dir = test_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir).unwrap();

        // Create a test script
        let script_content = r#"
export default class TestScript {
    speed = 5.0;
    name = "test";
    static executionOrder = 0;
    onStart(self) {}
    onUpdate(self, dt) {}
}
"#;
        std::fs::write(scripts_dir.join("TestScript.ts"), script_content).unwrap();

        // Test loading
        let mut runtime = ScriptRuntime::new();
        let result = runtime.load_game(&test_dir);
        assert!(result.is_ok());
        assert!(runtime.is_loaded());
        assert!(!runtime.is_initialized());

        // Verify script was compiled
        let scripts = runtime.available_scripts();
        assert_eq!(scripts.len(), 1);
        assert!(scripts.contains(&"TestScript.ts"));

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_sync_instances() {
        // Create a temporary test game directory
        let test_dir = std::env::temp_dir().join("test_game_runtime_sync");
        let scripts_dir = test_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir).unwrap();

        // Create a test script
        let script_content = r#"
export default class TestScript {
    speed = 5.0;
    name = "test";
    static executionOrder = 0;
}
"#;
        std::fs::write(scripts_dir.join("TestScript.ts"), script_content).unwrap();

        // Load game
        let mut runtime = ScriptRuntime::new();
        runtime.load_game(&test_dir).unwrap();

        // Create world with script entities
        let mut world = World::new();
        let entity = world.spawn().with(Script::new("TestScript.ts")).build();

        // Sync instances (without initializing, which creates V8 runtime)
        runtime.sync_instances(&world);

        // Verify instance was created
        assert_eq!(runtime.instances.len(), 1);
        let instance_id = (entity.id().to_bits().get(), "TestScript.ts".to_string());
        assert!(runtime.instances.contains_key(&instance_id));

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_execution_order_sorting() {
        // Create temporary test game
        let test_dir = std::env::temp_dir().join("test_game_exec_order");
        let scripts_dir = test_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir).unwrap();

        // Create scripts with different execution orders
        std::fs::write(
            scripts_dir.join("Early.ts"),
            "export default class Early { static executionOrder = -10; }",
        )
        .unwrap();
        std::fs::write(
            scripts_dir.join("Late.ts"),
            "export default class Late { static executionOrder = 10; }",
        )
        .unwrap();
        std::fs::write(
            scripts_dir.join("Middle.ts"),
            "export default class Middle { static executionOrder = 0; }",
        )
        .unwrap();

        let mut runtime = ScriptRuntime::new();
        runtime.load_game(&test_dir).unwrap();

        // Verify all scripts loaded
        let scripts = runtime.available_scripts();
        assert_eq!(scripts.len(), 3);

        // Verify execution orders were parsed
        assert_eq!(
            runtime
                .compiled_scripts
                .get("Early.ts")
                .unwrap()
                .execution_order,
            -10
        );
        assert_eq!(
            runtime
                .compiled_scripts
                .get("Late.ts")
                .unwrap()
                .execution_order,
            10
        );
        assert_eq!(
            runtime
                .compiled_scripts
                .get("Middle.ts")
                .unwrap()
                .execution_order,
            0
        );

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();
    }
}
