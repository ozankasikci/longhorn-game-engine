# Script Execution Design (Phase 1)

## Goal

Wire up the TypeScript scripting system so scripts actually execute. Currently `run_lifecycle` logs "Would call..." without executing JavaScript. This phase makes lifecycle methods (onStart, onUpdate, onDestroy) actually run.

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Script instances | Class instance per entity | Familiar model, supports runtime state |
| Entity self parameter | Pre-built Entity class | Clean API, aligns with TS types |
| Error handling | Let deno_core propagate | Simple, good stack traces |

## Architecture

### Runtime Bootstrap

Injected once when JS runtime initializes. Defines global API.

```javascript
// Component type markers
const Transform = { name: "Transform" };
const Sprite = { name: "Sprite" };

// Entity class for self parameter
class Entity {
  constructor(id) { this.id = id; }
  get(componentType) {
    return Deno.core.ops.op_get_component(this.id, componentType.name);
  }
  set(componentType, value) {
    Deno.core.ops.op_set_component(this.id, componentType.name, value);
  }
  has(componentType) {
    return Deno.core.ops.op_has_component(this.id, componentType.name);
  }
}

// Script and instance registries
const __scripts = {};
const __instances = {};

// Console override to route through op_log
console.log = (...args) => Deno.core.ops.op_log("info", args.join(" "));
console.error = (...args) => Deno.core.ops.op_log("error", args.join(" "));
console.warn = (...args) => Deno.core.ops.op_log("warn", args.join(" "));
```

### Script Loading

When compiled script is loaded into JS:

```javascript
// Execute compiled JS and register the class
__scripts["PlayerController.ts"] = (() => {
  // ... compiled JS code ...
  return PlayerController;
})();
```

### Instance Creation

When entity gets Script component:

```javascript
// Create instance
__instances["12345_PlayerController.ts"] = new __scripts["PlayerController.ts"]();

// Apply inspector properties
__instances["12345_PlayerController.ts"].speed = 5.0;
__instances["12345_PlayerController.ts"].jumpHeight = 2.0;
```

### Lifecycle Calls

Each frame for onUpdate:

```javascript
const instance = __instances["12345_PlayerController.ts"];
const self = new Entity(12345n);  // BigInt for u64
if (instance.onUpdate) {
  instance.onUpdate(self, 0.016);
}
```

## Rust Changes

### ScriptRuntime.initialize()

```rust
pub fn initialize(&mut self, world: &mut World) -> Result<()> {
    // 1. Create JS runtime
    self.js_runtime = Some(LonghornJsRuntime::new());
    let rt = self.js_runtime.as_mut().unwrap();

    // 2. Execute bootstrap
    rt.execute_script("longhorn:bootstrap", BOOTSTRAP_JS)?;

    // 3. Load all compiled scripts
    for (path, compiled) in &self.compiled_scripts {
        let register_code = format!(
            "__scripts[\"{}\"] = (() => {{ {}; return {}; }})();",
            path, compiled.js_code, compiled.class_name
        );
        rt.execute_script("longhorn:load_script", &register_code)?;
    }

    // 4. Sync instances and call onStart
    self.sync_instances(world);
    self.run_lifecycle("onStart", world, 0.0)?;

    self.initialized = true;
    Ok(())
}
```

### ScriptRuntime.call_method()

```rust
fn call_method(&mut self, instance_key: &str, method: &str, entity_id: u64, dt: f32) -> Result<()> {
    let code = format!(
        r#"(() => {{
            const inst = __instances["{}"];
            if (inst && inst.{}) {{
                inst.{}(new Entity({}n), {});
            }}
        }})()"#,
        instance_key, method, method, entity_id, dt
    );

    self.js_runtime.as_mut().unwrap()
        .execute_script("longhorn:call", &code)?;
    Ok(())
}
```

### TypeScriptCompiler changes

Extract class name during compilation:

```rust
pub struct CompiledScript {
    pub source_path: String,
    pub js_code: String,
    pub class_name: String,  // NEW: extracted from "export default class Foo"
    pub execution_order: i32,
    pub properties: HashMap<String, String>,
}
```

## Scope

### This phase delivers:
- Scripts actually execute (console.log works)
- Class instances created per entity
- Inspector properties applied to instances
- Execution order respected
- Errors caught, game paused

### Deferred to next phase:
- `self.get(Transform)` / `self.set()` - component access ops
- `world.find()` / `world.spawn()` - world API ops
- `input.isTouching()` - input state ops

## Test Case

```typescript
// scripts/TestScript.ts
export default class TestScript {
  speed = 5.0;
  static executionOrder = 0;

  onStart(self) {
    console.log("Started with speed:", this.speed);
  }

  onUpdate(self, dt) {
    console.log("Update dt:", dt, "speed:", this.speed);
  }
}
```

Expected output:
```
[Script] Started with speed: 5
[Script] Update dt: 0.016 speed: 5
[Script] Update dt: 0.016 speed: 5
```

## Files to Modify

1. `crates/longhorn-scripting/src/runtime.rs` - Add bootstrap, call_method, update run_lifecycle
2. `crates/longhorn-scripting/src/compiler.rs` - Extract class_name
3. `crates/longhorn-scripting/src/bootstrap.js` - New file with JS bootstrap code
