# Component Access Design (Phase 2)

## Goal

Enable scripts to read and modify entity components (Transform, Sprite) through the `self` parameter.

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Data flow | Inject as JS, read after | No ops needed, simple JSON boundary |
| Which components | Hardcode Transform + Sprite | Simple start, expand later |
| API shape | Direct properties (`self.transform`) | Simpler than `get()`/`set()` methods |
| Write-back | Always write back | Simple, minimal overhead |
| Missing components | Pass `null` | Explicit, easy to check |

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│ BEFORE onUpdate                                         │
│                                                         │
│   Rust:                                                 │
│     1. Read Transform from World (or null if missing)   │
│     2. Read Sprite from World (or null if missing)      │
│     3. Build self object as JSON                        │
│     4. Call: instance.onUpdate(self, dt)                │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│ JS executes                                             │
│                                                         │
│   onUpdate(self, dt) {                                  │
│     self.transform.position.x += 5;                     │
│   }                                                     │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│ AFTER onUpdate                                          │
│                                                         │
│   Rust:                                                 │
│     1. Read self.transform from JS                      │
│     2. Read self.sprite from JS                         │
│     3. Write both back to World (if not null)           │
└─────────────────────────────────────────────────────────┘
```

### The `self` Object

```typescript
self = {
  id: 42n,                    // BigInt entity ID

  transform: {                // or null if entity has no Transform
    position: { x: 10, y: 20 },
    rotation: 0,
    scale: { x: 1, y: 1 }
  },

  sprite: {                   // or null if entity has no Sprite
    texture: 5,
    size: { x: 32, y: 32 },
    color: [1, 1, 1, 1],
    flipX: false,
    flipY: false
  }
}
```

### Rust Implementation

```rust
// In runtime.rs, when calling onUpdate for an entity

fn call_lifecycle_method(&mut self, entity: EntityHandle, method: &str, world: &World, dt: f32) {
    // 1. Build self object
    let transform = world.get::<Transform>(entity).ok();
    let sprite = world.get::<Sprite>(entity).ok();

    let self_json = build_self_json(entity.id(), transform, sprite);

    // 2. Call JS
    let code = format!(r#"
        (() => {{
            const self = {self_json};
            const inst = __instances["{key}"];
            inst.{method}(self, {dt});
            return JSON.stringify({{ transform: self.transform, sprite: self.sprite }});
        }})()
    "#);

    let result = self.js_runtime.execute_script("call", &code)?;

    // 3. Write back
    let changes: SelfChanges = serde_json::from_str(&result)?;

    if let Some(t) = changes.transform {
        world.set(entity, Transform::from(t));
    }
    if let Some(s) = changes.sprite {
        world.set(entity, Sprite::from(s));
    }
}
```

## Scope

### This phase delivers:
- `self.transform` - read and mutate position, rotation, scale
- `self.sprite` - read and mutate texture, size, color, flip
- Changes automatically written back to World each frame

### Not included (future phases):
- `world.find("Player")` - accessing other entities
- `world.spawn()` - creating entities
- `input.isTouching()` - input state
- Other components beyond Transform/Sprite

## Example Script

```typescript
export default class MoveRight {
  speed = 100;

  onUpdate(self, dt) {
    self.transform.position.x += this.speed * dt;

    // Flip sprite based on direction
    if (this.speed < 0) {
      self.sprite.flipX = true;
    }
  }
}
```

## Files to Modify

1. `crates/longhorn-scripting/src/runtime.rs` - Update `call_method` to pass/read component data
2. `crates/longhorn-scripting/src/ops.rs` - Add JSON conversion helpers for Transform/Sprite
