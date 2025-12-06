# Longhorn v2: Mobile 2D Game Engine Design

> **Note:** This design document references Deno Core for scripting. The implementation has since migrated to **rquickjs (QuickJS)** for better compile times and simpler embedding. See `crates/longhorn-scripting/` for current implementation.

## Overview

Longhorn v2 is a Rust-based 2D game engine specifically built for mobile games (iOS and Android). It features TypeScript scripting for game logic, wgpu-based rendering, and an egui desktop editor.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance, safety, cross-platform |
| Platforms | iOS + Android | Validate cross-platform architecture early |
| Rendering | wgpu | Pure Rust, abstracts Metal/Vulkan/OpenGL ES |
| Scripting | Deno core (v8) | First-class TypeScript, battle-tested |
| ECS | hecs with friendly API | Performance under the hood, simple API for game devs |
| Editor | egui | Pure Rust, immediate mode, easy to iterate |
| Asset loading | Runtime (MVP) | Simple now, designed for build-time later |
| Audio | Deferred | Out of MVP scope |
| Input | Single touch | Tap, drag, position tracking |

## Project Structure

```
longhorn-game-engine-v2/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── longhorn-core/         # ECS, math, time, core types
│   ├── longhorn-renderer/     # wgpu-based 2D rendering
│   ├── longhorn-scripting/    # Deno/v8 TypeScript runtime
│   ├── longhorn-input/        # Touch input handling
│   ├── longhorn-assets/       # Asset loading (images, data)
│   ├── longhorn-engine/       # Ties everything together, game loop
│   ├── longhorn-editor/       # egui-based editor (desktop only)
│   └── longhorn-mobile/       # iOS/Android platform layer
├── editor/                    # Editor application entry point
├── examples/                  # Example games (TypeScript bundles)
└── docs/                      # Documentation and design docs
```

## Crate Designs

### longhorn-core

Foundation types and ECS that everything else builds on.

```
longhorn-core/
├── src/
│   ├── lib.rs
│   ├── ecs/
│   │   ├── mod.rs
│   │   ├── world.rs          # Wraps hecs::World with friendly API
│   │   ├── entity.rs         # Entity handle, builder pattern
│   │   └── component.rs      # Component trait, common components
│   ├── math/
│   │   ├── mod.rs
│   │   ├── vec2.rs           # 2D vector
│   │   ├── transform.rs      # Position, rotation, scale
│   │   └── rect.rs           # Bounding rectangles
│   ├── time.rs               # Delta time, fixed timestep
│   └── types.rs              # EntityId, AssetId, common aliases
```

**Built-in components (MVP):**
- `Transform` — position (Vec2), rotation (f32), scale (Vec2)
- `Sprite` — texture reference, color tint, flip flags
- `Name` — debug name for entities
- `Enabled` — active/inactive state

**TypeScript API:**
```typescript
const player = world.spawn("Player")
  .with(Transform, { x: 100, y: 200 })
  .with(Sprite, { texture: "player.png" })
  .build();

player.get(Transform).x += 10;
```

**Dependencies:** `hecs`, `glam`

### longhorn-renderer

2D rendering via wgpu, optimized for sprites and mobile GPUs.

```
longhorn-renderer/
├── src/
│   ├── lib.rs
│   ├── renderer.rs           # Main Renderer struct, frame lifecycle
│   ├── sprite_batch.rs       # Batches sprites for efficient drawing
│   ├── texture.rs            # Texture loading, atlas support
│   ├── camera.rs             # 2D orthographic camera
│   ├── color.rs              # RGBA color type
│   └── pipeline/
│       ├── mod.rs
│       └── sprite.wgsl       # Sprite shader
```

**Rendering strategy:**
- Sprite batching — Group sprites by texture, minimize draw calls
- Texture atlases — Support packing multiple images
- Orthographic camera — 2D projection with position, zoom, viewport
- Draw order — Z-index sorting for layering sprites

**Dependencies:** `wgpu`, `image`, `longhorn-core`

### longhorn-scripting

Run TypeScript game code via Deno core (v8).

```
longhorn-scripting/
├── src/
│   ├── lib.rs
│   ├── runtime.rs            # Deno core setup, module loading
│   ├── bindings/
│   │   ├── mod.rs
│   │   ├── world.rs          # ECS bindings (spawn, query, etc.)
│   │   ├── transform.rs      # Transform component ops
│   │   ├── sprite.rs         # Sprite component ops
│   │   ├── input.rs          # Touch input bindings
│   │   └── time.rs           # Delta time, elapsed time
│   └── api/
│       └── longhorn.d.ts     # TypeScript type definitions
```

**Game lifecycle hooks:**
```typescript
export function onStart(world: World) {
  // Called once when game starts
}

export function onUpdate(world: World, dt: number) {
  // Called every frame
}

export function onTouchStart(world: World, x: number, y: number) {
  // Called on touch down
}
```

**Dependencies:** `deno_core`, `serde`, `longhorn-core`

### longhorn-input

Handle touch input, abstract platform differences.

```
longhorn-input/
├── src/
│   ├── lib.rs
│   ├── touch.rs              # Touch state, events
│   ├── input_state.rs        # Current frame input snapshot
│   └── events.rs             # TouchStart, TouchMove, TouchEnd
```

**Touch events:**
```rust
pub enum TouchEvent {
    Start { x: f32, y: f32 },
    Move { x: f32, y: f32 },
    End { x: f32, y: f32 },
}
```

**Input state:**
```rust
pub struct InputState {
    pub touch_down: bool,
    pub touch_position: Option<Vec2>,
    pub touch_just_pressed: bool,
    pub touch_just_released: bool,
}
```

**TypeScript API:**
```typescript
if (input.justPressed()) {
  const pos = input.position();
  // Handle tap at pos.x, pos.y
}
```

**Dependencies:** `longhorn-core`

### longhorn-assets

Load and manage game assets, designed for future build-time compilation.

```
longhorn-assets/
├── src/
│   ├── lib.rs
│   ├── asset_manager.rs      # Central asset registry, loading
│   ├── handle.rs             # AssetHandle<T> for safe references
│   ├── loader/
│   │   ├── mod.rs
│   │   ├── texture.rs        # PNG/JPEG loading
│   │   └── json.rs           # JSON data files
│   └── source.rs             # AssetSource trait (filesystem, bundled)
```

**AssetSource abstraction:**
```rust
pub trait AssetSource: Send + Sync {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>>;
    fn exists(&self, path: &str) -> bool;
}

// MVP: FilesystemSource
// Future: BundledSource (compiled asset pack)
```

**Dependencies:** `image`, `serde_json`, `longhorn-core`

### longhorn-engine

Ties all systems together, owns the game loop.

```
longhorn-engine/
├── src/
│   ├── lib.rs
│   ├── engine.rs             # Main Engine struct
│   ├── game_loop.rs          # Fixed timestep loop
│   ├── game.rs               # Game trait / loaded game bundle
│   └── config.rs             # Engine configuration
```

**Engine struct:**
```rust
pub struct Engine {
    world: World,
    renderer: Renderer,
    scripting: ScriptRuntime,
    input: InputState,
    assets: AssetManager,
    config: EngineConfig,
}
```

**Game loop:**
```rust
impl Engine {
    pub fn run_frame(&mut self, dt: Duration) {
        // 1. Update input state
        self.input.update(&self.pending_events);

        // 2. Call TypeScript onUpdate
        self.scripting.call_update(&mut self.world, dt);

        // 3. Run built-in systems
        self.run_systems();

        // 4. Render
        self.renderer.render(&self.world, &self.assets);
    }
}
```

**Dependencies:** All other longhorn crates

### longhorn-mobile

Platform-specific code for iOS and Android.

```
longhorn-mobile/
├── src/
│   ├── lib.rs
│   ├── platform.rs           # Platform trait abstraction
│   ├── app.rs                # Mobile app runner
│   └── android/
│       ├── mod.rs
│       └── activity.rs       # Android Activity integration
│   └── ios/
│       ├── mod.rs
│       └── app_delegate.rs   # iOS UIApplicationDelegate
```

**Platform abstraction:**
```rust
pub trait Platform {
    fn create_window(&self) -> RawWindowHandle;
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
    fn get_display_size(&self) -> (u32, u32);
}

pub enum PlatformEvent {
    Touch(TouchEvent),
    Resize { width: u32, height: u32 },
    Suspend,
    Resume,
    Quit,
}
```

**Dependencies:** `winit`, `raw-window-handle`, `longhorn-engine`

### longhorn-editor

Desktop-only egui editor for scene viewing and entity inspection.

```
longhorn-editor/
├── src/
│   ├── lib.rs
│   ├── editor.rs             # Main editor application
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── scene_tree.rs     # Entity hierarchy view
│   │   ├── inspector.rs      # Component property editor
│   │   └── viewport.rs       # Game preview (rendered scene)
│   ├── state.rs              # Editor state (selection, etc.)
│   └── project.rs            # Load/save game projects

editor/                       # Separate binary entry point
├── Cargo.toml
└── src/
    └── main.rs
```

**MVP panels:**
1. Scene Tree — Lists all entities by name, click to select
2. Inspector — Shows components of selected entity, edit properties
3. Viewport — Renders the game scene (view-only)

**Dependencies:** `egui`, `eframe`, `longhorn-engine`

## Game Bundle Structure

```
my-game/
├── game.json                 # Manifest file
├── src/
│   └── main.ts               # Entry point
├── assets/
│   ├── sprites/
│   │   └── player.png
│   └── data/
│       └── levels.json
└── tsconfig.json             # Optional
```

**game.json manifest:**
```json
{
  "name": "My Awesome Game",
  "version": "1.0.0",
  "entry": "src/main.ts",
  "viewport": {
    "width": 1280,
    "height": 720
  },
  "assets": {
    "preload": [
      "sprites/player.png",
      "sprites/enemy.png",
      "data/levels.json"
    ]
  }
}
```

**Example game (src/main.ts):**
```typescript
import { World, Transform, Sprite, input } from "longhorn";

export function onStart(world: World) {
  world.spawn("Player")
    .with(Transform, { x: 640, y: 360 })
    .with(Sprite, { texture: "sprites/player.png" })
    .build();
}

export function onUpdate(world: World, dt: number) {
  const player = world.find("Player");

  if (input.justPressed()) {
    const pos = input.position();
    player.get(Transform).x = pos.x;
    player.get(Transform).y = pos.y;
  }
}
```

## Dependency Graph

```
longhorn-core
    ↑
    ├── longhorn-renderer
    ├── longhorn-input
    ├── longhorn-assets
    ├── longhorn-scripting
    │
    └───────┬───────────────┘
            ↓
      longhorn-engine
            ↑
            ├── longhorn-mobile  (iOS/Android runtime)
            └── longhorn-editor  (desktop editor)
```

## Workspace Configuration

```toml
[workspace]
resolver = "2"
members = [
    "crates/longhorn-core",
    "crates/longhorn-renderer",
    "crates/longhorn-input",
    "crates/longhorn-assets",
    "crates/longhorn-scripting",
    "crates/longhorn-engine",
    "crates/longhorn-mobile",
    "crates/longhorn-editor",
    "editor",
]

[workspace.dependencies]
hecs = "0.10"
glam = "0.27"
wgpu = "0.19"
winit = "0.29"
egui = "0.27"
eframe = "0.27"
image = "0.25"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Build Targets

| Target | Command | Output |
|--------|---------|--------|
| Editor (desktop) | `cargo run -p editor` | macOS/Windows/Linux app |
| Android | `cargo ndk -t arm64-v8a -p longhorn-mobile` | .so library |
| iOS | `cargo build -p longhorn-mobile --target aarch64-apple-ios` | .a library |

## MVP Scope

### In Scope

| Area | Included |
|------|----------|
| Platforms | iOS + Android |
| Rendering | Sprite batching, texture atlases, 2D camera, z-ordering |
| ECS | hecs-based world, Transform, Sprite, Name, Enabled components |
| Scripting | TypeScript via Deno, onStart/onUpdate/onTouchStart hooks |
| Input | Single touch (tap, drag, position, justPressed/justReleased) |
| Assets | PNG/JPEG textures, JSON data, runtime loading |
| Editor | Scene tree, inspector, viewport (view-only) |

### Out of Scope (Future)

| Area | Deferred |
|------|----------|
| Audio | Sound effects, music |
| Physics | Collision detection, rigid bodies |
| UI | In-game UI widgets, text rendering |
| Animation | Sprite animation, tweening |
| Particles | Particle systems |
| Tilemaps | Tilemap rendering, editors |
| Networking | Multiplayer, leaderboards |
| Editor | Visual editing, gizmos, undo/redo, asset browser |
| Build pipeline | Asset compilation, bundling, app store packaging |

### MVP Success Criteria

1. Load a game bundle on iOS and Android
2. Render sprites with transforms
3. Handle touch input
4. Run TypeScript game logic
5. View/inspect scene in desktop editor
