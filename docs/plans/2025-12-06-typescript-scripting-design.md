# TypeScript Scripting System Design

## Goal

Add TypeScript as the scripting language for Longhorn, with scripts as components that can be attached to entities. Scripts affect gameplay like Unity MonoBehaviours or Godot scripts.

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| JS Runtime | Deno Core (V8) | Mature, excellent TS support, battle-tested |
| Script model | Class-based, entity-focused | Familiar to Unity/Godot devs, clean property syntax |
| Scripts per entity | Multiple allowed | Flexibility, separation of concerns |
| Properties | Class fields with defaults | Simple, inspector-editable, per-instance |
| Component access | Copy on access (get/set) | Simple, explicit, minimal overhead for 2D |
| Execution order | Priority-based | Deterministic, explicit control |
| TS compilation | Editor startup + file watch | Fast iteration during development |
| Error handling | Strict - pause game | Catches bugs immediately |

## Script Structure

```typescript
// scripts/PlayerController.ts
export default class PlayerController {
  // Properties - editable in inspector
  speed = 5.0;
  jumpHeight = 2.0;

  // Execution order (optional) - lower runs first, default is 0
  static executionOrder = 0;

  onStart(self: Entity) {
    console.log("Player spawned");
  }

  onUpdate(self: Entity, dt: number) {
    const transform = self.get(Transform);
    transform.position.x += this.speed * dt;
    self.set(Transform, transform);

    // World access via global
    const enemy = world.find("Enemy");
    if (enemy) {
      // ...
    }
  }

  onDestroy(self: Entity) {
    console.log("Player destroyed");
  }
}
```

### Lifecycle Methods

- `onStart(self: Entity)` - Called once when entity spawns or game starts
- `onUpdate(self: Entity, dt: number)` - Called every frame
- `onDestroy(self: Entity)` - Called when entity is despawned

### Property Types

Supported types for inspector-editable properties:
- `number` - Numeric input
- `string` - Text input
- `boolean` - Checkbox
- `{ x: number, y: number }` - Vec2 input (future)

## Rust Architecture

### Script Component

```rust
// In longhorn-core
pub struct Script {
    pub path: String,                          // "scripts/PlayerController.ts"
    pub properties: HashMap<String, ScriptValue>,
    pub enabled: bool,
}

pub enum ScriptValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Vec2 { x: f64, y: f64 },
}
```

### Script Runtime

```rust
// In longhorn-scripting
pub struct ScriptRuntime {
    js_runtime: deno_core::JsRuntime,
    compiled_scripts: HashMap<String, CompiledScript>,
    script_instances: HashMap<(EntityId, String), ScriptInstance>,
}

impl ScriptRuntime {
    pub fn compile_all(&mut self, scripts_dir: &Path) -> Result<()>;
    pub fn instantiate(&mut self, entity: EntityId, script: &Script) -> Result<()>;
    pub fn run_start(&mut self, world: &mut World) -> Result<()>;
    pub fn run_update(&mut self, world: &mut World, dt: f32) -> Result<()>;
    pub fn destroy_instance(&mut self, entity: EntityId, path: &str);
}
```

### Frame Flow

```
1. Collect entities with Script components
2. Sort by executionOrder, then entity ID (deterministic)
3. For each script instance:
   a. Prepare entity context (self)
   b. Call onUpdate(self, dt)
   c. If error: pause game, show error in console
```

## Rust-JS Bindings

### Deno Ops

```rust
// Component access
#[op]
fn op_get_component(entity_id: u64, component_type: String) -> Result<serde_json::Value>;

#[op]
fn op_set_component(entity_id: u64, component_type: String, value: serde_json::Value) -> Result<()>;

#[op]
fn op_has_component(entity_id: u64, component_type: String) -> bool;

// World access
#[op]
fn op_spawn_entity(name: String) -> u64;

#[op]
fn op_despawn_entity(entity_id: u64) -> Result<()>;

#[op]
fn op_find_entity(name: String) -> Option<u64>;

// Input
#[op]
fn op_input_is_touching() -> bool;

#[op]
fn op_input_position() -> Option<(f32, f32)>;

// Logging
#[op]
fn op_log(level: String, message: String);
```

### TypeScript API

```typescript
declare global {
  const world: {
    spawn(name: string): Entity;
    find(name: string): Entity | null;
    despawn(entity: Entity): void;
  };

  const input: {
    isTouching(): boolean;
    position(): { x: number, y: number } | null;
  };

  const Transform: ComponentType<Transform>;
  const Sprite: ComponentType<Sprite>;
}

interface Entity {
  readonly id: number;
  get<T>(component: ComponentType<T>): T;
  set<T>(component: ComponentType<T>, value: T): void;
  has<T>(component: ComponentType<T>): boolean;
}
```

## TypeScript Compilation

### Editor Startup

```
1. Scan project/scripts/ for .ts files
2. Compile each to JS using deno_core's built-in TS compiler
3. Cache compiled JS in project/.longhorn/compiled/
4. Parse class to extract:
   - Property names, types, defaults
   - executionOrder if present
   - Lifecycle method presence
```

### File Watching

```
1. Developer saves PlayerController.ts
2. File watcher detects change
3. Recompile that script
4. If game is running:
   - Replace compiled code
   - Keep existing script instances (preserve property values)
   - Next frame uses new code
5. If compile error:
   - Show error in console
   - Keep old code running
```

### Exported Game

- Bundle pre-compiled JS files
- No TypeScript compiler needed at runtime

## Editor Integration

### Inspector Panel

```
┌─────────────────────────────┐
│ Entity: Player              │
├─────────────────────────────┤
│ ▼ Transform                 │
│   Position: (100, 200)      │
│   Rotation: 0               │
│   Scale: (1, 1)             │
├─────────────────────────────┤
│ ▼ Sprite                    │
│   Texture: player.png       │
├─────────────────────────────┤
│ ▼ PlayerController.ts    [x]│
│   Speed: 5.0                │
│   Jump Height: 2.0          │
├─────────────────────────────┤
│ [+ Add Component]           │
│ [+ Add Script]              │
└─────────────────────────────┘
```

### Script Attachment Flow

1. Select entity in Hierarchy panel
2. Click "Add Script" button
3. Dropdown shows available scripts (from scripts/ folder)
4. Select script
5. Script component added with default property values
6. Properties appear in inspector for editing

## Error Handling

### Strict Mode

When a script throws an error:
1. Pause the game immediately
2. Display error in Console panel:
   - File path and line number
   - Error message
   - Stack trace
3. Game stays paused until:
   - Developer fixes and saves the script (triggers recompile)
   - Developer clicks "Retry" in console

### Console Output

```
┌─ Console ─────────────────────────────────────┐
│ [INFO] Game started                           │
│ [INFO] PlayerController: Player spawned       │
│ [ERROR] EnemyAI.ts:23                         │
│   TypeError: Cannot read property 'x' of null │
│   at EnemyAI.onUpdate (EnemyAI.ts:23:15)      │
│   Game paused - fix error to continue         │
└───────────────────────────────────────────────┘
```

## Execution Order

Scripts execute in order:
1. Sort all script instances by `static executionOrder` (default: 0)
2. Within same priority, sort by entity ID (deterministic)
3. Lower numbers run first

```typescript
// GameManager runs first
export default class GameManager {
  static executionOrder = -100;
}

// Default priority
export default class PlayerController {
  // executionOrder = 0 (default)
}

// Runs after most scripts
export default class UIController {
  static executionOrder = 100;
}
```

## File Structure

```
project/
├── game.json                    # Project manifest
├── scripts/
│   ├── PlayerController.ts
│   ├── EnemyAI.ts
│   └── GameManager.ts
├── assets/
│   └── ...
└── .longhorn/
    └── compiled/                # Cached compiled JS
        ├── PlayerController.js
        ├── EnemyAI.js
        └── GameManager.js
```

## Dependencies

Add to `longhorn-scripting/Cargo.toml`:
```toml
[dependencies]
deno_core = "0.272"  # Or latest compatible version
serde_json = { workspace = true }
notify = "6.0"       # File watching
```
