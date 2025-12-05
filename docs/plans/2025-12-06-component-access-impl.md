# Component Access Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable scripts to read/modify Transform and Sprite components through `self.transform` and `self.sprite` properties.

**Architecture:** Rust injects component data as JSON into the `self` parameter before calling lifecycle methods. After the method returns, Rust reads back the (possibly modified) data and writes to World.

**Tech Stack:** Rust, deno_core, serde_json, hecs ECS

---

### Task 1: Add JS-friendly component structs

**Files:**
- Modify: `crates/longhorn-scripting/src/ops.rs:26-47`

**Step 1: Update JsTransform and JsSprite to match JS naming conventions**

The existing structs use snake_case. Update to camelCase for JS:

```rust
/// Transform data for JS interop
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct JsSprite {
    pub texture: u64,
    pub size: JsVec2,
    pub color: [f64; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}
```

**Step 2: Add conversion From impls**

Add below the struct definitions:

```rust
impl From<&longhorn_core::Transform> for JsTransform {
    fn from(t: &longhorn_core::Transform) -> Self {
        Self {
            position: JsVec2 { x: t.position.x as f64, y: t.position.y as f64 },
            rotation: t.rotation as f64,
            scale: JsVec2 { x: t.scale.x as f64, y: t.scale.y as f64 },
        }
    }
}

impl From<JsTransform> for longhorn_core::Transform {
    fn from(t: JsTransform) -> Self {
        Self {
            position: glam::Vec2::new(t.position.x as f32, t.position.y as f32),
            rotation: t.rotation as f32,
            scale: glam::Vec2::new(t.scale.x as f32, t.scale.y as f32),
        }
    }
}

impl From<&longhorn_core::Sprite> for JsSprite {
    fn from(s: &longhorn_core::Sprite) -> Self {
        Self {
            texture: s.texture.as_u64(),
            size: JsVec2 { x: s.size.x as f64, y: s.size.y as f64 },
            color: [s.color[0] as f64, s.color[1] as f64, s.color[2] as f64, s.color[3] as f64],
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        }
    }
}

impl From<JsSprite> for longhorn_core::Sprite {
    fn from(s: JsSprite) -> Self {
        Self {
            texture: longhorn_core::AssetId::new(s.texture),
            size: glam::Vec2::new(s.size.x as f32, s.size.y as f32),
            color: [s.color[0] as f32, s.color[1] as f32, s.color[2] as f32, s.color[3] as f32],
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        }
    }
}
```

**Step 3: Add SelfData struct for the full self object**

```rust
/// The 'self' object passed to script lifecycle methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsSelf {
    pub id: u64,
    pub transform: Option<JsTransform>,
    pub sprite: Option<JsSprite>,
}
```

**Step 4: Run tests**

Run: `cargo test -p longhorn-scripting`

Expected: PASS (existing tests should still work)

**Step 5: Commit**

```bash
git add crates/longhorn-scripting/src/ops.rs
git commit -m "feat(scripting): add JS-friendly component structs with conversions"
```

---

### Task 2: Update run_lifecycle to pass component data

**Files:**
- Modify: `crates/longhorn-scripting/src/runtime.rs:241-321`

**Step 1: Update run_lifecycle signature to take &mut World**

Change line 242 from:
```rust
fn run_lifecycle(&mut self, method: &str, _world: &mut World, dt: f32) -> Result<()> {
```
To:
```rust
fn run_lifecycle(&mut self, method: &str, world: &mut World, dt: f32) -> Result<()> {
```

**Step 2: Add use statements at top of file**

Add after line 6:
```rust
use crate::ops::{JsSelf, JsSprite, JsTransform};
use longhorn_core::{Sprite, Transform};
```

**Step 3: Replace the call_code generation (lines 288-299)**

Replace the existing call_code block with:

```rust
// Build self object with component data
let transform: Option<JsTransform> = world
    .get::<Transform>(longhorn_core::EntityHandle::from_bits(entity_id))
    .ok()
    .map(|t| JsTransform::from(&*t));

let sprite: Option<JsSprite> = world
    .get::<Sprite>(longhorn_core::EntityHandle::from_bits(entity_id))
    .ok()
    .map(|s| JsSprite::from(&*s));

let self_data = JsSelf {
    id: entity_id,
    transform,
    sprite,
};

let self_json = serde_json::to_string(&self_data)
    .map_err(|e| LonghornError::Scripting(format!("Failed to serialize self: {}", e)))?;

// Build the call code
let call_code = format!(
    r#"(() => {{
        const inst = __instances["{}"];
        if (inst && typeof inst.{} === "function") {{
            const self = {};
            inst.{}(self, {});
            return JSON.stringify({{ transform: self.transform, sprite: self.sprite }});
        }} else {{
            return "no method";
        }}
    }})()"#,
    instance_key, method, self_json, method, dt
);
```

**Step 4: Add write-back logic after the JS call**

Replace the match block (lines 302-317) with:

```rust
match js_runtime.execute_script("longhorn:call_lifecycle", &call_code) {
    Ok(result) => {
        if result != "no method" {
            // Write back component changes
            if let Ok(changes) = serde_json::from_str::<JsSelf>(&result) {
                let entity_handle = longhorn_core::EntityHandle::from_bits(entity_id);

                if let Some(t) = changes.transform {
                    let _ = world.set(entity_handle, Transform::from(t));
                }
                if let Some(s) = changes.sprite {
                    let _ = world.set(entity_handle, Sprite::from(s));
                }
            }
        }

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
```

**Step 5: Run tests**

Run: `cargo test -p longhorn-scripting`

Expected: PASS

**Step 6: Commit**

```bash
git add crates/longhorn-scripting/src/runtime.rs
git commit -m "feat(scripting): pass component data to scripts and write back changes"
```

---

### Task 3: Add integration test for component access

**Files:**
- Modify: `crates/longhorn-scripting/tests/integration.rs`

**Step 1: Add test for reading transform**

Add at end of file:

```rust
#[test]
fn test_script_reads_transform() {
    let test_dir = std::env::temp_dir().join("test_script_reads_transform");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that logs transform position
    let script = r#"
export default class ReadTransform {
    onStart(self) {
        console.log("pos:", self.transform.position.x, self.transform.position.y);
    }
}
"#;
    std::fs::write(scripts_dir.join("ReadTransform.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    let entity = world
        .spawn()
        .with(Script::new("ReadTransform.ts"))
        .with(Transform::from_position(glam::Vec2::new(100.0, 200.0)))
        .build();

    runtime.initialize(&mut world).unwrap();

    // Verify transform is still there (wasn't corrupted)
    let t = world.get::<Transform>(entity).unwrap();
    assert_eq!(t.position.x, 100.0);
    assert_eq!(t.position.y, 200.0);

    std::fs::remove_dir_all(&test_dir).ok();
}
```

**Step 2: Add test for modifying transform**

```rust
#[test]
fn test_script_modifies_transform() {
    let test_dir = std::env::temp_dir().join("test_script_modifies_transform");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that modifies position
    let script = r#"
export default class MoveRight {
    onUpdate(self, dt) {
        self.transform.position.x += 10;
    }
}
"#;
    std::fs::write(scripts_dir.join("MoveRight.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    let entity = world
        .spawn()
        .with(Script::new("MoveRight.ts"))
        .with(Transform::from_position(glam::Vec2::new(0.0, 0.0)))
        .build();

    runtime.initialize(&mut world).unwrap();

    // Run update
    runtime.update(&mut world, 0.016).unwrap();

    // Verify position changed
    let t = world.get::<Transform>(entity).unwrap();
    assert_eq!(t.position.x, 10.0);

    // Run another update
    runtime.update(&mut world, 0.016).unwrap();

    let t = world.get::<Transform>(entity).unwrap();
    assert_eq!(t.position.x, 20.0);

    std::fs::remove_dir_all(&test_dir).ok();
}
```

**Step 3: Add test for null sprite handling**

```rust
#[test]
fn test_script_handles_null_sprite() {
    let test_dir = std::env::temp_dir().join("test_script_null_sprite");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that checks for null sprite
    let script = r#"
export default class CheckSprite {
    onStart(self) {
        if (self.sprite === null) {
            console.log("no sprite");
        } else {
            console.log("has sprite");
        }
    }
}
"#;
    std::fs::write(scripts_dir.join("CheckSprite.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    // Entity with transform but NO sprite
    world
        .spawn()
        .with(Script::new("CheckSprite.ts"))
        .with(Transform::new())
        .build();

    // Should not crash
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok());

    std::fs::remove_dir_all(&test_dir).ok();
}
```

**Step 4: Run integration tests**

Run: `cargo test -p longhorn-scripting --test integration`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/longhorn-scripting/tests/integration.rs
git commit -m "test(scripting): add integration tests for component access"
```

---

### Task 4: Update example script to use component access

**Files:**
- Modify: `crates/longhorn-scripting/examples/run_script.rs` (if exists)
- Or the test project's script

**Step 1: Find and update the example script**

Check `test-project/scripts/` for an example and update it to demonstrate component access:

```typescript
export default class TestScript {
    speed = 100;

    onStart(self) {
        console.log("Starting at position:", self.transform.position.x, self.transform.position.y);
    }

    onUpdate(self, dt) {
        // Move right
        self.transform.position.x += this.speed * dt;

        // Log every so often
        if (Math.floor(self.transform.position.x) % 50 === 0) {
            console.log("Position:", self.transform.position.x);
        }
    }
}
```

**Step 2: Run the example**

Run: `cargo run -p longhorn-scripting --example run_script`

Expected: Should see position updates in logs

**Step 3: Commit**

```bash
git add .
git commit -m "chore: update example script to demonstrate component access"
```

---

### Task 5: Run full test suite

**Step 1: Run all tests**

Run: `cargo test --workspace`

Expected: All tests PASS

**Step 2: Final commit if any cleanup needed**

```bash
git status
# If clean, done. If changes:
git add .
git commit -m "chore: final cleanup for component access"
```

---

## Summary

After completing all tasks:
- `self.transform` provides read/write access to Transform component
- `self.sprite` provides read/write access to Sprite component
- Missing components are `null`
- Changes are automatically written back to World after each lifecycle call
