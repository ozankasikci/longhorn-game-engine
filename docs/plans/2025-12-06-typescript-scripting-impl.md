# TypeScript Scripting System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement TypeScript scripting as attachable components on entities, using Deno Core for JS execution.

**Architecture:** Scripts are ECS components with a path and properties. Deno Core runs compiled TypeScript. Rust ops expose engine functionality (components, world, input) to JS. Scripts have class-based lifecycle methods (onStart, onUpdate, onDestroy).

**Tech Stack:** Rust, deno_core, hecs ECS, TypeScript, serde_json for component serialization

---

## Phase 1: Core Infrastructure

### Task 1: Add Script Component

**Files:**
- Create: `crates/longhorn-core/src/ecs/script.rs`
- Modify: `crates/longhorn-core/src/ecs/mod.rs`
- Modify: `crates/longhorn-core/src/lib.rs`

**Step 1: Create script.rs with Script component and ScriptValue enum**

```rust
// crates/longhorn-core/src/ecs/script.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value types for script properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScriptValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Vec2 { x: f64, y: f64 },
}

impl ScriptValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            ScriptValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ScriptValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Script component - attached to entities to run TypeScript code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// Path to the script file (relative to scripts/ folder)
    pub path: String,
    /// Instance properties (overrides class defaults)
    pub properties: HashMap<String, ScriptValue>,
    /// Whether this script is enabled
    pub enabled: bool,
}

impl Script {
    /// Create a new script component
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            properties: HashMap::new(),
            enabled: true,
        }
    }

    /// Create a script with properties
    pub fn with_properties(path: impl Into<String>, properties: HashMap<String, ScriptValue>) -> Self {
        Self {
            path: path.into(),
            properties,
            enabled: true,
        }
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&ScriptValue> {
        self.properties.get(name)
    }

    /// Set a property value
    pub fn set_property(&mut self, name: impl Into<String>, value: ScriptValue) {
        self.properties.insert(name.into(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_new() {
        let script = Script::new("PlayerController.ts");
        assert_eq!(script.path, "PlayerController.ts");
        assert!(script.properties.is_empty());
        assert!(script.enabled);
    }

    #[test]
    fn test_script_with_properties() {
        let mut props = HashMap::new();
        props.insert("speed".to_string(), ScriptValue::Number(5.0));

        let script = Script::with_properties("PlayerController.ts", props);
        assert_eq!(script.get_property("speed"), Some(&ScriptValue::Number(5.0)));
    }

    #[test]
    fn test_script_value_accessors() {
        assert_eq!(ScriptValue::Number(5.0).as_number(), Some(5.0));
        assert_eq!(ScriptValue::String("test".into()).as_string(), Some("test"));
        assert_eq!(ScriptValue::Boolean(true).as_bool(), Some(true));
    }

    #[test]
    fn test_script_value_serialization() {
        let value = ScriptValue::Number(42.0);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "42.0");

        let parsed: ScriptValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, value);
    }
}
```

**Step 2: Update ecs/mod.rs to export script module**

In `crates/longhorn-core/src/ecs/mod.rs`, add:
```rust
pub mod script;
pub use script::*;
```

**Step 3: Run tests to verify**

Run: `cargo test -p longhorn-core script`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/longhorn-core/src/ecs/script.rs crates/longhorn-core/src/ecs/mod.rs
git commit -m "feat(core): add Script component and ScriptValue enum"
```

---

### Task 2: Add deno_core Dependency

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `crates/longhorn-scripting/Cargo.toml`

**Step 1: Add deno_core to workspace dependencies**

In `Cargo.toml` (workspace root), add under `[workspace.dependencies]`:
```toml
deno_core = "0.311"
```

**Step 2: Update longhorn-scripting Cargo.toml**

Replace contents of `crates/longhorn-scripting/Cargo.toml`:
```toml
[package]
name = "longhorn-scripting"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
longhorn-input = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
deno_core = { workspace = true }
```

**Step 3: Verify it compiles**

Run: `cargo check -p longhorn-scripting`
Expected: Compiles successfully (warnings okay)

**Step 4: Commit**

```bash
git add Cargo.toml crates/longhorn-scripting/Cargo.toml
git commit -m "chore(scripting): add deno_core dependency"
```

---

## Phase 2: Script Runtime Foundation

### Task 3: Create Basic JsRuntime Wrapper

**Files:**
- Create: `crates/longhorn-scripting/src/js_runtime.rs`
- Modify: `crates/longhorn-scripting/src/lib.rs`

**Step 1: Create js_runtime.rs with basic JsRuntime setup**

```rust
// crates/longhorn-scripting/src/js_runtime.rs
use deno_core::{JsRuntime, RuntimeOptions};
use std::rc::Rc;

/// Wrapper around deno_core::JsRuntime
pub struct LonghornJsRuntime {
    runtime: JsRuntime,
}

impl LonghornJsRuntime {
    /// Create a new JavaScript runtime
    pub fn new() -> Self {
        let runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        Self { runtime }
    }

    /// Execute JavaScript code and return the result as a string
    pub fn execute_script(&mut self, name: &str, code: &str) -> Result<String, JsRuntimeError> {
        let result = self.runtime.execute_script(name, code.to_string().into());

        match result {
            Ok(global) => {
                let scope = &mut self.runtime.handle_scope();
                let local = deno_core::v8::Local::new(scope, global);
                let result_str = local.to_rust_string_lossy(scope);
                Ok(result_str)
            }
            Err(e) => Err(JsRuntimeError::Execution(e.to_string())),
        }
    }

    /// Get mutable reference to inner runtime (for advanced ops)
    pub fn inner_mut(&mut self) -> &mut JsRuntime {
        &mut self.runtime
    }
}

impl Default for LonghornJsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsRuntimeError {
    #[error("JavaScript execution error: {0}")]
    Execution(String),

    #[error("Script compilation error: {0}")]
    Compilation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let _runtime = LonghornJsRuntime::new();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_execute_simple_script() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_execute_string_script() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "'hello' + ' world'").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_execute_error() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "throw new Error('test error')");
        assert!(result.is_err());
    }
}
```

**Step 2: Update lib.rs to export js_runtime**

Replace `crates/longhorn-scripting/src/lib.rs`:
```rust
mod js_runtime;
mod runtime;

pub use js_runtime::*;
pub use runtime::*;
```

**Step 3: Run tests**

Run: `cargo test -p longhorn-scripting`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/src/js_runtime.rs crates/longhorn-scripting/src/lib.rs
git commit -m "feat(scripting): add basic JsRuntime wrapper"
```

---

### Task 4: Add TypeScript Compilation Support

**Files:**
- Create: `crates/longhorn-scripting/src/compiler.rs`
- Modify: `crates/longhorn-scripting/src/lib.rs`

**Step 1: Create compiler.rs with TypeScript transpilation**

```rust
// crates/longhorn-scripting/src/compiler.rs
use crate::js_runtime::{JsRuntimeError, LonghornJsRuntime};
use std::collections::HashMap;
use std::path::Path;

/// Compiled script with metadata
#[derive(Debug, Clone)]
pub struct CompiledScript {
    /// Original TypeScript source path
    pub source_path: String,
    /// Compiled JavaScript code
    pub js_code: String,
    /// Execution order (parsed from static executionOrder)
    pub execution_order: i32,
    /// Property definitions (name -> default value as JSON)
    pub properties: HashMap<String, String>,
}

/// TypeScript compiler using deno_core
pub struct TypeScriptCompiler {
    runtime: LonghornJsRuntime,
}

impl TypeScriptCompiler {
    pub fn new() -> Self {
        Self {
            runtime: LonghornJsRuntime::new(),
        }
    }

    /// Compile TypeScript source to JavaScript
    pub fn compile(&mut self, source: &str, filename: &str) -> Result<String, JsRuntimeError> {
        // For now, just pass through (deno_core handles TS natively in modules)
        // In a full implementation, we'd use swc or deno's TS compiler
        // For MVP, we'll require pre-compiled JS or use simple TS that's valid JS

        // Strip type annotations (very basic - production would use swc)
        let js = self.strip_types(source);
        Ok(js)
    }

    /// Very basic type stripping (MVP only - use swc in production)
    fn strip_types(&self, source: &str) -> String {
        let mut result = String::new();
        let mut chars = source.chars().peekable();

        while let Some(c) = chars.next() {
            // Skip type annotations after :
            if c == ':' {
                // Check if this looks like a type annotation (not object key)
                let mut type_annotation = String::new();
                let mut depth = 0;

                while let Some(&next) = chars.peek() {
                    if next == '{' || next == '<' || next == '(' {
                        depth += 1;
                        type_annotation.push(chars.next().unwrap());
                    } else if next == '}' || next == '>' || next == ')' {
                        if depth > 0 {
                            depth -= 1;
                            type_annotation.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    } else if (next == '=' || next == ',' || next == ';' || next == '\n') && depth == 0 {
                        break;
                    } else {
                        type_annotation.push(chars.next().unwrap());
                    }
                }

                // Keep the colon only if it's an object literal
                if type_annotation.trim().starts_with('{') ||
                   type_annotation.contains('\n') ||
                   type_annotation.trim().is_empty() {
                    result.push(':');
                    result.push_str(&type_annotation);
                }
                // Otherwise strip the type annotation
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Load and compile a TypeScript file
    pub fn compile_file(&mut self, path: &Path) -> Result<CompiledScript, CompilerError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompilerError::Io(e.to_string()))?;

        let js_code = self.compile(&source, path.to_str().unwrap_or("unknown"))
            .map_err(|e| CompilerError::Compilation(e.to_string()))?;

        // Parse execution order from source (look for static executionOrder = N)
        let execution_order = self.parse_execution_order(&source);

        // Parse property definitions
        let properties = self.parse_properties(&source);

        Ok(CompiledScript {
            source_path: path.display().to_string(),
            js_code,
            execution_order,
            properties,
        })
    }

    fn parse_execution_order(&self, source: &str) -> i32 {
        // Look for: static executionOrder = N
        for line in source.lines() {
            if line.contains("static") && line.contains("executionOrder") {
                if let Some(eq_pos) = line.find('=') {
                    let after_eq = &line[eq_pos + 1..];
                    let num_str: String = after_eq
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '-')
                        .collect();
                    if let Ok(n) = num_str.parse() {
                        return n;
                    }
                }
            }
        }
        0 // default
    }

    fn parse_properties(&self, source: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        // Look for class properties with defaults: name = value;
        // This is a simplified parser - production would use proper AST
        for line in source.lines() {
            let trimmed = line.trim();

            // Skip if it's a method or static
            if trimmed.starts_with("static") ||
               trimmed.contains("(") ||
               trimmed.starts_with("//") ||
               trimmed.starts_with("on") {
                continue;
            }

            // Look for: propName = value
            if let Some(eq_pos) = trimmed.find('=') {
                let name = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim().trim_end_matches(';');

                // Only include simple names (no types)
                let name = name.split(':').next().unwrap_or(name).trim();

                if !name.is_empty() && !name.contains(' ') {
                    props.insert(name.to_string(), value.to_string());
                }
            }
        }

        props
    }
}

impl Default for TypeScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Compilation error: {0}")]
    Compilation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_execution_order() {
        let compiler = TypeScriptCompiler::new();

        let source = r#"
export default class Test {
    static executionOrder = -10;
}
"#;
        assert_eq!(compiler.parse_execution_order(source), -10);
    }

    #[test]
    fn test_parse_execution_order_default() {
        let compiler = TypeScriptCompiler::new();
        let source = "export default class Test {}";
        assert_eq!(compiler.parse_execution_order(source), 0);
    }

    #[test]
    fn test_parse_properties() {
        let compiler = TypeScriptCompiler::new();

        let source = r#"
export default class Test {
    speed = 5.0;
    name = "player";
    active = true;

    onUpdate() {}
}
"#;
        let props = compiler.parse_properties(source);
        assert_eq!(props.get("speed"), Some(&"5.0".to_string()));
        assert_eq!(props.get("name"), Some(&"\"player\"".to_string()));
        assert_eq!(props.get("active"), Some(&"true".to_string()));
    }
}
```

**Step 2: Update lib.rs to export compiler**

In `crates/longhorn-scripting/src/lib.rs`, add:
```rust
mod compiler;
pub use compiler::*;
```

**Step 3: Run tests**

Run: `cargo test -p longhorn-scripting compiler`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/src/compiler.rs crates/longhorn-scripting/src/lib.rs
git commit -m "feat(scripting): add TypeScript compiler with property parsing"
```

---

## Phase 3: Rust-JS Bindings (Ops)

### Task 5: Create Deno Ops for Component Access

**Files:**
- Create: `crates/longhorn-scripting/src/ops.rs`
- Modify: `crates/longhorn-scripting/src/lib.rs`

**Step 1: Create ops.rs with component access ops**

```rust
// crates/longhorn-scripting/src/ops.rs
use deno_core::op2;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// Shared state accessible from ops
pub struct OpsState {
    // These will be set before script execution
    pub current_entity_id: Option<u64>,
}

impl OpsState {
    pub fn new() -> Self {
        Self {
            current_entity_id: None,
        }
    }
}

impl Default for OpsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform data for JS interop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTransform {
    pub position: JsVec2,
    pub rotation: f64,
    pub scale: JsVec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsVec2 {
    pub x: f64,
    pub y: f64,
}

/// Sprite data for JS interop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsSprite {
    pub texture: u64,
    pub size: JsVec2,
    pub color: [f64; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

// Note: In a full implementation, these ops would access the actual World.
// For now, we define the interface. The ScriptRuntime will inject the World
// via op state before calling scripts.

#[op2]
#[string]
pub fn op_log(#[string] level: String, #[string] message: String) {
    match level.as_str() {
        "error" => log::error!("[Script] {}", message),
        "warn" => log::warn!("[Script] {}", message),
        "info" => log::info!("[Script] {}", message),
        "debug" => log::debug!("[Script] {}", message),
        _ => log::info!("[Script] {}", message),
    }
}

#[op2]
#[serde]
pub fn op_get_current_entity() -> Option<u64> {
    // This will be replaced with actual state lookup
    // For now, return None to indicate no current entity
    None
}

// Extension definition for all longhorn ops
deno_core::extension!(
    longhorn_ops,
    ops = [
        op_log,
        op_get_current_entity,
    ],
    esm_entry_point = "ext:longhorn_ops/runtime.js",
    esm = ["ext:longhorn_ops/runtime.js" = "src/runtime.js"],
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops_state_default() {
        let state = OpsState::default();
        assert!(state.current_entity_id.is_none());
    }

    #[test]
    fn test_js_transform_serialization() {
        let transform = JsTransform {
            position: JsVec2 { x: 10.0, y: 20.0 },
            rotation: 0.5,
            scale: JsVec2 { x: 1.0, y: 1.0 },
        };

        let json = serde_json::to_string(&transform).unwrap();
        assert!(json.contains("position"));
        assert!(json.contains("10"));
    }
}
```

**Step 2: Create runtime.js for JS-side API**

Create `crates/longhorn-scripting/src/runtime.js`:
```javascript
// Longhorn Runtime API
((globalThis) => {
    const core = Deno.core;

    // Console implementation
    globalThis.console = {
        log: (...args) => core.ops.op_log("info", args.map(String).join(" ")),
        info: (...args) => core.ops.op_log("info", args.map(String).join(" ")),
        warn: (...args) => core.ops.op_log("warn", args.map(String).join(" ")),
        error: (...args) => core.ops.op_log("error", args.map(String).join(" ")),
        debug: (...args) => core.ops.op_log("debug", args.map(String).join(" ")),
    };

    // Entity class
    class Entity {
        constructor(id) {
            this.id = id;
        }

        get(componentType) {
            // Will be implemented with op_get_component
            return null;
        }

        set(componentType, value) {
            // Will be implemented with op_set_component
        }

        has(componentType) {
            // Will be implemented with op_has_component
            return false;
        }
    }

    // Component type markers
    const Transform = { name: "Transform" };
    const Sprite = { name: "Sprite" };
    const Name = { name: "Name" };

    // World API
    const world = {
        spawn(name) {
            // Will be implemented with op_spawn_entity
            return null;
        },
        find(name) {
            // Will be implemented with op_find_entity
            return null;
        },
        despawn(entity) {
            // Will be implemented with op_despawn_entity
        },
    };

    // Input API
    const input = {
        isTouching() {
            // Will be implemented with op_input_is_touching
            return false;
        },
        position() {
            // Will be implemented with op_input_position
            return null;
        },
        justPressed() {
            return false;
        },
        justReleased() {
            return false;
        },
    };

    // Expose to global scope
    globalThis.Entity = Entity;
    globalThis.Transform = Transform;
    globalThis.Sprite = Sprite;
    globalThis.Name = Name;
    globalThis.world = world;
    globalThis.input = input;

})(globalThis);
```

**Step 3: Update lib.rs**

Add to `crates/longhorn-scripting/src/lib.rs`:
```rust
mod ops;
pub use ops::*;
```

**Step 4: Run tests**

Run: `cargo test -p longhorn-scripting ops`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/longhorn-scripting/src/ops.rs crates/longhorn-scripting/src/runtime.js crates/longhorn-scripting/src/lib.rs
git commit -m "feat(scripting): add deno ops and JS runtime API"
```

---

## Phase 4: Script Execution

### Task 6: Implement Script Instance Management

**Files:**
- Modify: `crates/longhorn-scripting/src/runtime.rs`

**Step 1: Replace runtime.rs with full implementation**

```rust
// crates/longhorn-scripting/src/runtime.rs
use crate::compiler::{CompiledScript, CompilerError, TypeScriptCompiler};
use crate::js_runtime::{JsRuntimeError, LonghornJsRuntime};
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

    /// Initialize the game (create JS runtime, call onStart for existing scripts)
    pub fn initialize(&mut self, world: &mut World) -> Result<()> {
        if self.game_path.is_none() {
            return Err(LonghornError::Scripting("No game loaded".to_string()));
        }

        // Create JS runtime
        self.js_runtime = Some(LonghornJsRuntime::new());

        // Find all entities with Script components and create instances
        self.sync_instances(world);

        // Call onStart on all instances
        self.run_lifecycle("onStart", world, 0.0)?;

        self.initialized = true;
        log::info!("Script runtime initialized");

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
    fn run_lifecycle(&mut self, method: &str, world: &mut World, dt: f32) -> Result<()> {
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
}
```

**Step 2: Run tests**

Run: `cargo test -p longhorn-scripting runtime`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/longhorn-scripting/src/runtime.rs
git commit -m "feat(scripting): implement script instance management and lifecycle"
```

---

## Phase 5: Integration

### Task 7: Update Engine to Use New Script Runtime

**Files:**
- Modify: `crates/longhorn-engine/src/engine.rs`
- Modify: `crates/longhorn-engine/Cargo.toml`

**Step 1: Add longhorn-core dependency for Script component**

Ensure `crates/longhorn-engine/Cargo.toml` has:
```toml
[dependencies]
longhorn-core = { workspace = true }
```

**Step 2: Verify engine still compiles and tests pass**

Run: `cargo test -p longhorn-engine`
Expected: All tests pass (existing tests should still work)

**Step 3: Commit**

```bash
git add crates/longhorn-engine/Cargo.toml
git commit -m "feat(engine): ensure Script component integration"
```

---

### Task 8: Add Script Component to Editor Inspector

**Files:**
- Modify: `crates/longhorn-editor/src/panels/inspector.rs` (create if doesn't exist)

This task depends on the editor's inspector panel implementation. The goal is to:
1. Display Script components when an entity is selected
2. Show editable properties from the script
3. Add "Add Script" button
4. Add remove button per script

**Step 1: Check if inspector panel exists**

Run: `ls crates/longhorn-editor/src/panels/`

If inspector.rs doesn't exist, create a basic one. If it does, modify it to handle Script components.

**Step 2: Add script display to inspector**

The implementation depends on existing panel structure. Key additions:
- Query entity for `Script` components
- For each Script, display path and properties
- Add remove button
- Add "Add Script" dropdown at bottom

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/panels/
git commit -m "feat(editor): add Script component display in inspector"
```

---

## Phase 6: End-to-End Testing

### Task 9: Create Integration Test

**Files:**
- Create: `crates/longhorn-scripting/tests/integration.rs`
- Create: `crates/longhorn-scripting/tests/fixtures/test_game/scripts/TestScript.ts`
- Create: `crates/longhorn-scripting/tests/fixtures/test_game/game.json`

**Step 1: Create test fixtures**

Create `crates/longhorn-scripting/tests/fixtures/test_game/game.json`:
```json
{
    "name": "Test Game",
    "version": "1.0.0",
    "entry": "main.ts",
    "viewport": {
        "width": 800,
        "height": 600
    },
    "assets": {
        "preload": []
    }
}
```

Create `crates/longhorn-scripting/tests/fixtures/test_game/scripts/TestScript.ts`:
```typescript
export default class TestScript {
    speed = 5.0;
    name = "test";

    static executionOrder = 0;

    onStart(self) {
        console.log("TestScript started");
    }

    onUpdate(self, dt) {
        // Move logic would go here
    }

    onDestroy(self) {
        console.log("TestScript destroyed");
    }
}
```

**Step 2: Create integration test**

Create `crates/longhorn-scripting/tests/integration.rs`:
```rust
use longhorn_core::{Script, World};
use longhorn_scripting::ScriptRuntime;
use std::path::PathBuf;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test_game")
}

#[test]
fn test_load_game_with_scripts() {
    let mut runtime = ScriptRuntime::new();
    let result = runtime.load_game(fixtures_path());

    assert!(result.is_ok());
    assert!(runtime.is_loaded());

    let scripts = runtime.available_scripts();
    assert!(!scripts.is_empty());
    assert!(scripts.contains(&"TestScript.ts"));
}

#[test]
fn test_script_properties_parsed() {
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(fixtures_path()).unwrap();

    let props = runtime.script_properties("TestScript.ts");
    assert!(props.is_some());

    let props = props.unwrap();
    assert!(props.contains_key("speed"));
    assert!(props.contains_key("name"));
}

#[test]
fn test_script_lifecycle() {
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(fixtures_path()).unwrap();

    let mut world = World::new();

    // Create an entity with a script
    world.spawn()
        .with(Script::new("TestScript.ts"))
        .build();

    // Initialize should succeed
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok());
    assert!(runtime.is_initialized());

    // Update should succeed
    let result = runtime.update(&mut world, 0.016);
    assert!(result.is_ok());
}
```

**Step 3: Run integration tests**

Run: `cargo test -p longhorn-scripting --test integration`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/longhorn-scripting/tests/
git commit -m "test(scripting): add integration tests for script runtime"
```

---

### Task 10: Full Build and Test

**Step 1: Build entire workspace**

Run: `cargo build --workspace`
Expected: All crates compile

**Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 3: Run editor to verify**

Run: `cargo run -p editor`
Expected: Editor opens without errors

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete TypeScript scripting system MVP"
```

---

## Summary

This plan implements the TypeScript scripting system in 10 tasks:

1. **Script Component** - Add Script and ScriptValue to longhorn-core
2. **Deno Dependency** - Add deno_core to workspace
3. **JsRuntime Wrapper** - Basic JavaScript execution
4. **TypeScript Compiler** - TS to JS compilation with property parsing
5. **Deno Ops** - Rust-JS bridge functions
6. **Script Instance Management** - Lifecycle and execution order
7. **Engine Integration** - Wire up Script component
8. **Editor Inspector** - Display and edit scripts in UI
9. **Integration Tests** - End-to-end testing
10. **Full Build** - Verify everything works together

Each task is designed to be completed independently with its own tests and commit.
