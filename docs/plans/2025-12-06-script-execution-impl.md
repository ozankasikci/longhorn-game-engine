# Script Execution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire up TypeScript scripts to actually execute lifecycle methods (onStart, onUpdate, onDestroy).

**Architecture:** JS runtime initialized with bootstrap code that defines Entity class and global registries. Compiled scripts are loaded and registered, instances created per entity, lifecycle methods called via dynamically generated JS.

**Tech Stack:** Rust, deno_core, JavaScript

---

### Task 1: Add class_name to CompiledScript

**Files:**
- Modify: `crates/longhorn-scripting/src/compiler.rs`

**Step 1: Add class_name field to CompiledScript**

In `compiler.rs`, update the struct:

```rust
/// Compiled script with metadata
#[derive(Debug, Clone)]
pub struct CompiledScript {
    /// Original TypeScript source path
    pub source_path: String,
    /// Compiled JavaScript code
    pub js_code: String,
    /// Class name extracted from "export default class Foo"
    pub class_name: String,
    /// Execution order (parsed from static executionOrder)
    pub execution_order: i32,
    /// Property definitions (name -> default value as JSON)
    pub properties: HashMap<String, String>,
}
```

**Step 2: Add parse_class_name method**

Add after `parse_properties`:

```rust
fn parse_class_name(&self, source: &str) -> String {
    // Look for: export default class ClassName
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("export") && trimmed.contains("default") && trimmed.contains("class") {
            // Extract class name after "class"
            if let Some(class_pos) = trimmed.find("class") {
                let after_class = &trimmed[class_pos + 5..].trim_start();
                let class_name: String = after_class
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !class_name.is_empty() {
                    return class_name;
                }
            }
        }
    }
    "UnnamedScript".to_string()
}
```

**Step 3: Update compile_file to use parse_class_name**

Update the `compile_file` method:

```rust
pub fn compile_file(&mut self, path: &Path) -> Result<CompiledScript, CompilerError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompilerError::Io(e.to_string()))?;

    let js_code = self.compile(&source, path.to_str().unwrap_or("unknown"))
        .map_err(|e| CompilerError::Compilation(e.to_string()))?;

    let execution_order = self.parse_execution_order(&source);
    let properties = self.parse_properties(&source);
    let class_name = self.parse_class_name(&source);

    Ok(CompiledScript {
        source_path: path.display().to_string(),
        js_code,
        class_name,
        execution_order,
        properties,
    })
}
```

**Step 4: Add test for parse_class_name**

```rust
#[test]
fn test_parse_class_name() {
    let compiler = TypeScriptCompiler::new();

    let source = r#"
export default class PlayerController {
    speed = 5.0;
}
"#;
    assert_eq!(compiler.parse_class_name(source), "PlayerController");
}

#[test]
fn test_parse_class_name_default() {
    let compiler = TypeScriptCompiler::new();
    let source = "const x = 1;";
    assert_eq!(compiler.parse_class_name(source), "UnnamedScript");
}
```

**Step 5: Run tests**

Run: `cargo test -p longhorn-scripting parse_class_name`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/longhorn-scripting/src/compiler.rs
git commit -m "feat(scripting): extract class name from compiled scripts"
```

---

### Task 2: Create Bootstrap JavaScript

**Files:**
- Create: `crates/longhorn-scripting/src/bootstrap.js`
- Modify: `crates/longhorn-scripting/src/lib.rs`

**Step 1: Create bootstrap.js**

Create `crates/longhorn-scripting/src/bootstrap.js`:

```javascript
// Longhorn Runtime Bootstrap
// Injected before any user scripts

// Component type markers (used for self.get(Transform))
const Transform = { name: "Transform" };
const Sprite = { name: "Sprite" };

// Entity class - passed as 'self' to lifecycle methods
class Entity {
  constructor(id) {
    this.id = id;
  }

  get(componentType) {
    // TODO: Wire to op_get_component when implemented
    return null;
  }

  set(componentType, value) {
    // TODO: Wire to op_set_component when implemented
  }

  has(componentType) {
    // TODO: Wire to op_has_component when implemented
    return false;
  }
}

// Script class registry (populated when scripts are loaded)
const __scripts = {};

// Script instance registry (populated when entities get scripts)
const __instances = {};

// Console override to route through Rust logging
const __console_log = (...args) => {
  try {
    Deno.core.ops.op_log("info", args.map(a => String(a)).join(" "));
  } catch (e) {
    // Fallback if op not available
  }
};

const __console_error = (...args) => {
  try {
    Deno.core.ops.op_log("error", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

const __console_warn = (...args) => {
  try {
    Deno.core.ops.op_log("warn", args.map(a => String(a)).join(" "));
  } catch (e) {}
};

// Override global console
globalThis.console = {
  log: __console_log,
  info: __console_log,
  error: __console_error,
  warn: __console_warn,
  debug: __console_log,
};

// Make classes globally available
globalThis.Entity = Entity;
globalThis.Transform = Transform;
globalThis.Sprite = Sprite;
globalThis.__scripts = __scripts;
globalThis.__instances = __instances;

"bootstrap loaded";
```

**Step 2: Embed bootstrap in Rust**

Add to `crates/longhorn-scripting/src/lib.rs` near the top:

```rust
/// Embedded bootstrap JavaScript code
pub const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");
```

**Step 3: Verify it compiles**

Run: `cargo build -p longhorn-scripting`
Expected: Success

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/src/bootstrap.js crates/longhorn-scripting/src/lib.rs
git commit -m "feat(scripting): add JavaScript bootstrap with Entity class"
```

---

### Task 3: Wire Up Ops Extension to Runtime

**Files:**
- Modify: `crates/longhorn-scripting/src/js_runtime.rs`

**Step 1: Update LonghornJsRuntime to include ops**

Update `js_runtime.rs`:

```rust
use deno_core::{FastString, JsRuntime, RuntimeOptions};
use crate::ops::longhorn_ops;

/// Wrapper around deno_core::JsRuntime
pub struct LonghornJsRuntime {
    runtime: JsRuntime,
}

impl LonghornJsRuntime {
    /// Create a new JavaScript runtime with Longhorn ops
    pub fn new() -> Self {
        let runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![longhorn_ops::init_ops()],
            ..Default::default()
        });

        Self { runtime }
    }

    // ... rest of impl unchanged
}
```

**Step 2: Add import for ops module**

Make sure `js_runtime.rs` has `use crate::ops::longhorn_ops;` at the top.

**Step 3: Run tests**

Run: `cargo test -p longhorn-scripting js_runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/src/js_runtime.rs
git commit -m "feat(scripting): wire longhorn_ops extension to JS runtime"
```

---

### Task 4: Update ScriptRuntime Initialize

**Files:**
- Modify: `crates/longhorn-scripting/src/runtime.rs`

**Step 1: Add BOOTSTRAP_JS import**

At the top of `runtime.rs`, add:

```rust
use crate::BOOTSTRAP_JS;
```

**Step 2: Update initialize method**

Replace the `initialize` method:

```rust
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
```

**Step 3: Verify it compiles**

Run: `cargo build -p longhorn-scripting`
Expected: Success

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/src/runtime.rs
git commit -m "feat(scripting): initialize JS runtime with bootstrap and scripts"
```

---

### Task 5: Create JS Instances for Entities

**Files:**
- Modify: `crates/longhorn-scripting/src/runtime.rs`

**Step 1: Update sync_instances to create JS instances**

Replace `sync_instances`:

```rust
/// Sync script instances with world state
fn sync_instances(&mut self, world: &World) {
    let js_runtime = match &mut self.js_runtime {
        Some(rt) => rt,
        None => return,
    };

    // Query all entities with Script components
    for (entity_id, script) in world.query::<&Script>().iter() {
        let entity_bits = entity_id.to_bits().get();
        let instance_key = format!("{}_{}", entity_bits, script.path);

        if self.instances.contains_key(&(entity_bits, script.path.clone())) {
            continue; // Already exists
        }

        // Create instance in JS
        let create_code = format!(
            r#"(() => {{
                const ScriptClass = __scripts["{}"];
                if (ScriptClass) {{
                    __instances["{}"] = new ScriptClass();
                    "created"
                }} else {{
                    "script not found"
                }}
            }})()"#,
            script.path, instance_key
        );

        match js_runtime.execute_script("longhorn:create_instance", &create_code) {
            Ok(result) => {
                if result == "created" {
                    // Apply inspector properties
                    for (prop_name, prop_value) in &script.properties {
                        let value_js = match prop_value {
                            longhorn_core::ScriptValue::Number(n) => n.to_string(),
                            longhorn_core::ScriptValue::String(s) => format!("\"{}\"", s),
                            longhorn_core::ScriptValue::Boolean(b) => b.to_string(),
                            longhorn_core::ScriptValue::Vec2 { x, y } => {
                                format!("{{x: {}, y: {}}}", x, y)
                            }
                        };
                        let set_prop_code = format!(
                            r#"__instances["{}"].{} = {};"#,
                            instance_key, prop_name, value_js
                        );
                        let _ = js_runtime.execute_script("longhorn:set_prop", &set_prop_code);
                    }

                    log::debug!("Created JS instance: {}", instance_key);
                } else {
                    log::warn!("Script not found: {}", script.path);
                }
            }
            Err(e) => {
                log::error!("Failed to create instance {}: {}", instance_key, e);
            }
        }

        // Track in Rust
        let properties: HashMap<String, String> = script
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::to_string(v).unwrap_or_default()))
            .collect();

        self.instances.insert(
            (entity_bits, script.path.clone()),
            ScriptInstance {
                script_path: script.path.clone(),
                properties,
                started: false,
                enabled: script.enabled,
            },
        );
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p longhorn-scripting`
Expected: Success

**Step 3: Commit**

```bash
git add crates/longhorn-scripting/src/runtime.rs
git commit -m "feat(scripting): create JS instances when entities get scripts"
```

---

### Task 6: Actually Call Lifecycle Methods

**Files:**
- Modify: `crates/longhorn-scripting/src/runtime.rs`

**Step 1: Replace run_lifecycle with actual JS calls**

Replace the `run_lifecycle` method:

```rust
/// Run a lifecycle method on all script instances
fn run_lifecycle(&mut self, method: &str, _world: &mut World, dt: f32) -> Result<()> {
    // Check if we have an error (game should be paused)
    if self.error.is_some() {
        return Err(LonghornError::Scripting(self.error.clone().unwrap()));
    }

    let js_runtime = match &mut self.js_runtime {
        Some(rt) => rt,
        None => return Ok(()),
    };

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
    for (entity_id, script_path) in sorted_instances {
        let instance = match self.instances.get_mut(&(entity_id, script_path.clone())) {
            Some(inst) => inst,
            None => continue,
        };

        if !instance.enabled {
            continue;
        }

        // Skip onStart if already called
        if method == "onStart" && instance.started {
            continue;
        }

        let instance_key = format!("{}_{}", entity_id, script_path);

        // Build the call code
        let call_code = format!(
            r#"(() => {{
                const inst = __instances["{}"];
                if (inst && typeof inst.{} === "function") {{
                    const self = new Entity({}n);
                    inst.{}(self, {});
                    "called"
                }} else {{
                    "no method"
                }}
            }})()"#,
            instance_key, method, entity_id, method, dt
        );

        match js_runtime.execute_script("longhorn:call_lifecycle", &call_code) {
            Ok(_) => {
                if method == "onStart" {
                    instance.started = true;
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "Script error in {}.{}(): {}",
                    script_path, method, e
                );
                log::error!("{}", error_msg);
                self.error = Some(error_msg.clone());
                return Err(LonghornError::Scripting(error_msg));
            }
        }
    }

    Ok(())
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p longhorn-scripting`
Expected: Success

**Step 3: Commit**

```bash
git add crates/longhorn-scripting/src/runtime.rs
git commit -m "feat(scripting): actually call JS lifecycle methods"
```

---

### Task 7: Integration Test

**Files:**
- Modify: `crates/longhorn-scripting/tests/integration.rs`

**Step 1: Add test for lifecycle execution**

Add to `integration.rs`:

```rust
#[test]
fn test_script_execution_logs() {
    // Create a temporary test game directory
    let test_dir = std::env::temp_dir().join("test_game_execution");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Create a test script that logs
    let script_content = r#"
export default class LoggingScript {
    speed = 42.0;
    static executionOrder = 0;

    onStart(self) {
        console.log("onStart called with speed:", this.speed);
    }

    onUpdate(self, dt) {
        console.log("onUpdate dt:", dt);
    }
}
"#;
    std::fs::write(scripts_dir.join("LoggingScript.ts"), script_content).unwrap();

    // Load and initialize
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    world.spawn()
        .with(Script::new("LoggingScript.ts"))
        .build();

    // Initialize (calls onStart)
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok(), "Initialize failed: {:?}", result);

    // Update (calls onUpdate)
    let result = runtime.update(&mut world, 0.016);
    assert!(result.is_ok(), "Update failed: {:?}", result);

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}
```

**Step 2: Run the test**

Run: `cargo test -p longhorn-scripting test_script_execution_logs -- --nocapture`
Expected: PASS with log output showing "onStart called" and "onUpdate dt"

**Step 3: Commit**

```bash
git add crates/longhorn-scripting/tests/integration.rs
git commit -m "test(scripting): add integration test for script execution"
```

---

### Task 8: Run Example and Verify

**Files:**
- None (verification only)

**Step 1: Run the example**

Run: `cargo run -p longhorn-scripting --example run_script`

Expected output:
```
Loading scripts from: "/Users/.../test_project"
Scripts loaded successfully!
Available scripts: ["PlayerController.ts"]

PlayerController.ts properties:
  speed : 5.0
  jumpHeight : 2.0
  playerName : "Hero"

Created entity EntityHandle { id: 0v1 } with PlayerController script

--- Initializing (calling onStart) ---
[INFO] [Script] Started with speed: 5

--- Running 3 update frames ---

Frame 1: dt=0.016s
[INFO] [Script] Update dt: 0.016 speed: 5

Frame 2: dt=0.016s
[INFO] [Script] Update dt: 0.016 speed: 5

Frame 3: dt=0.016s
[INFO] [Script] Update dt: 0.016 speed: 5

Done!
```

**Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat(scripting): complete script execution wiring"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Add class_name extraction to compiler |
| 2 | Create bootstrap.js with Entity class |
| 3 | Wire ops extension to JS runtime |
| 4 | Update initialize to run bootstrap and load scripts |
| 5 | Create JS instances for entities |
| 6 | Actually call lifecycle methods |
| 7 | Integration test |
| 8 | Verify with example |
