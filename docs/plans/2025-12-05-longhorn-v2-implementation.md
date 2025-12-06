# Longhorn v2 Implementation Plan

> **Note:** This implementation plan references deno_core for scripting. The implementation uses **rquickjs (QuickJS)** instead for better compile times and simpler embedding.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a working MVP of Longhorn v2 — a Rust-based 2D mobile game engine with TypeScript scripting, wgpu rendering, and egui editor.

**Architecture:** Cargo workspace with 8 crates. Core foundation types → rendering/input/assets/scripting → engine integration → platform (mobile/editor). Each crate has a single responsibility with clear dependency boundaries.

**Tech Stack:** Rust, wgpu, hecs, rquickjs, egui, winit, glam

---

## Phase 1: Workspace & Core Foundation

### Task 1.1: Create Workspace Structure

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/longhorn-core/Cargo.toml`
- Create: `crates/longhorn-core/src/lib.rs`
- Create: `.gitignore`

**Step 1: Create workspace root Cargo.toml**

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

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/user/longhorn-game-engine-v2"

[workspace.dependencies]
# ECS
hecs = "0.10"

# Math
glam = { version = "0.27", features = ["serde"] }

# Rendering
wgpu = "0.19"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }

# Windowing
winit = "0.29"
raw-window-handle = "0.6"

# Editor
egui = "0.27"
eframe = { version = "0.27", default-features = false, features = ["default_fonts", "glow", "persistence"] }

# Scripting
deno_core = "0.272"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.11"

# Internal crates
longhorn-core = { path = "crates/longhorn-core" }
longhorn-renderer = { path = "crates/longhorn-renderer" }
longhorn-input = { path = "crates/longhorn-input" }
longhorn-assets = { path = "crates/longhorn-assets" }
longhorn-scripting = { path = "crates/longhorn-scripting" }
longhorn-engine = { path = "crates/longhorn-engine" }
longhorn-mobile = { path = "crates/longhorn-mobile" }
longhorn-editor = { path = "crates/longhorn-editor" }
```

**Step 2: Create .gitignore**

```
/target
Cargo.lock
*.log
.DS_Store
*.swp
*.swo
.idea/
.vscode/
```

**Step 3: Create longhorn-core crate**

Create `crates/longhorn-core/Cargo.toml`:
```toml
[package]
name = "longhorn-core"
version.workspace = true
edition.workspace = true

[dependencies]
hecs = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
```

Create `crates/longhorn-core/src/lib.rs`:
```rust
pub mod ecs;
pub mod math;
pub mod time;
pub mod types;

pub use ecs::*;
pub use math::*;
pub use time::*;
pub use types::*;
```

**Step 4: Verify workspace compiles**

Run: `cargo check -p longhorn-core`
Expected: Should fail with missing modules (ecs, math, time, types)

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: initialize workspace structure with longhorn-core crate"
```

---

### Task 1.2: Implement Core Types Module

**Files:**
- Create: `crates/longhorn-core/src/types.rs`

**Step 1: Create types.rs**

```rust
use serde::{Deserialize, Serialize};

/// Unique identifier for an entity in the ECS world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl From<hecs::Entity> for EntityId {
    fn from(entity: hecs::Entity) -> Self {
        EntityId(entity.to_bits().get())
    }
}

/// Unique identifier for a loaded asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        AssetId(id)
    }
}

/// Result type alias for Longhorn operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for Longhorn.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

    #[error("Component not found on entity")]
    ComponentNotFound,

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Asset loading failed: {0}")]
    AssetLoadError(String),

    #[error("Script error: {0}")]
    ScriptError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

**Step 2: Verify compiles**

Run: `cargo check -p longhorn-core`
Expected: Still fails (missing other modules)

**Step 3: Commit**

```bash
git add crates/longhorn-core/src/types.rs
git commit -m "feat(core): add EntityId, AssetId, and Error types"
```

---

### Task 1.3: Implement Math Module

**Files:**
- Create: `crates/longhorn-core/src/math/mod.rs`
- Create: `crates/longhorn-core/src/math/vec2.rs`
- Create: `crates/longhorn-core/src/math/transform.rs`
- Create: `crates/longhorn-core/src/math/rect.rs`

**Step 1: Create math/mod.rs**

```rust
mod vec2;
mod transform;
mod rect;

pub use vec2::*;
pub use transform::*;
pub use rect::*;

// Re-export glam types we use
pub use glam::{Vec2, Vec3, Vec4, Mat4};
```

**Step 2: Create math/vec2.rs**

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Extension trait for Vec2 with game-specific helpers.
pub trait Vec2Ext {
    fn from_angle(angle: f32) -> Vec2;
    fn angle(&self) -> f32;
    fn rotate(&self, angle: f32) -> Vec2;
}

impl Vec2Ext for Vec2 {
    fn from_angle(angle: f32) -> Vec2 {
        Vec2::new(angle.cos(), angle.sin())
    }

    fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    fn rotate(&self, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_from_angle() {
        let v = Vec2::from_angle(0.0);
        assert!((v.x - 1.0).abs() < 0.001);
        assert!(v.y.abs() < 0.001);
    }

    #[test]
    fn test_angle() {
        let v = Vec2::new(1.0, 0.0);
        assert!(v.angle().abs() < 0.001);

        let v = Vec2::new(0.0, 1.0);
        assert!((v.angle() - PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_rotate() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = v.rotate(PI / 2.0);
        assert!(rotated.x.abs() < 0.001);
        assert!((rotated.y - 1.0).abs() < 0.001);
    }
}
```

**Step 3: Create math/transform.rs**

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// 2D transform component with position, rotation, and scale.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            ..Default::default()
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Vec2::new(x, y);
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32) -> Self {
        self.scale = Vec2::new(x, y);
        self
    }

    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec2::splat(scale);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let t = Transform::default();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_builder() {
        let t = Transform::new(100.0, 200.0)
            .with_rotation(1.5)
            .with_scale(2.0, 3.0);

        assert_eq!(t.position, Vec2::new(100.0, 200.0));
        assert_eq!(t.rotation, 1.5);
        assert_eq!(t.scale, Vec2::new(2.0, 3.0));
    }
}
```

**Step 4: Create math/rect.rs**

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding rectangle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + width, y + height),
        }
    }

    pub fn from_center(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.min, Vec2::new(10.0, 20.0));
        assert_eq!(r.max, Vec2::new(110.0, 70.0));
    }

    #[test]
    fn test_dimensions() {
        let r = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(r.width(), 100.0);
        assert_eq!(r.height(), 50.0);
        assert_eq!(r.size(), Vec2::new(100.0, 50.0));
        assert_eq!(r.center(), Vec2::new(50.0, 25.0));
    }

    #[test]
    fn test_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Vec2::new(50.0, 50.0)));
        assert!(r.contains(Vec2::new(0.0, 0.0)));
        assert!(!r.contains(Vec2::new(-1.0, 50.0)));
        assert!(!r.contains(Vec2::new(101.0, 50.0)));
    }

    #[test]
    fn test_intersects() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let c = Rect::new(200.0, 200.0, 50.0, 50.0);

        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p longhorn-core`
Expected: All math tests pass

**Step 6: Commit**

```bash
git add crates/longhorn-core/src/math/
git commit -m "feat(core): add math module with Vec2, Transform, Rect"
```

---

### Task 1.4: Implement Time Module

**Files:**
- Create: `crates/longhorn-core/src/time.rs`

**Step 1: Create time.rs**

```rust
use std::time::{Duration, Instant};

/// Tracks frame timing and provides delta time.
#[derive(Debug)]
pub struct Time {
    start: Instant,
    last_frame: Instant,
    delta: Duration,
    frame_count: u64,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_frame: now,
            delta: Duration::ZERO,
            frame_count: 0,
        }
    }

    /// Call at the start of each frame to update timing.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now - self.last_frame;
        self.last_frame = now;
        self.frame_count += 1;
    }

    /// Time elapsed since last frame in seconds.
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Time elapsed since last frame.
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Total time elapsed since Time was created.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Total time elapsed in seconds.
    pub fn elapsed_seconds(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Total number of frames since start.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Fixed timestep accumulator for physics/game logic.
#[derive(Debug)]
pub struct FixedTimestep {
    timestep: Duration,
    accumulator: Duration,
}

impl FixedTimestep {
    pub fn new(timestep_hz: f32) -> Self {
        Self {
            timestep: Duration::from_secs_f32(1.0 / timestep_hz),
            accumulator: Duration::ZERO,
        }
    }

    /// Accumulate delta time. Returns number of fixed steps to run.
    pub fn accumulate(&mut self, delta: Duration) -> u32 {
        self.accumulator += delta;
        let steps = (self.accumulator.as_nanos() / self.timestep.as_nanos()) as u32;
        self.accumulator -= self.timestep * steps;
        steps
    }

    /// Get the fixed timestep duration.
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    /// Get the fixed timestep in seconds.
    pub fn timestep_seconds(&self) -> f32 {
        self.timestep.as_secs_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_update() {
        let mut time = Time::new();
        std::thread::sleep(Duration::from_millis(10));
        time.update();

        assert!(time.delta_seconds() > 0.0);
        assert_eq!(time.frame_count(), 1);
    }

    #[test]
    fn test_fixed_timestep() {
        let mut fixed = FixedTimestep::new(60.0);

        // Accumulate 1/30th of a second (should trigger 2 steps at 60hz)
        let steps = fixed.accumulate(Duration::from_secs_f32(1.0 / 30.0));
        assert_eq!(steps, 2);

        // Accumulate small amount (should trigger 0 steps)
        let steps = fixed.accumulate(Duration::from_secs_f32(0.001));
        assert_eq!(steps, 0);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p longhorn-core`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/longhorn-core/src/time.rs
git commit -m "feat(core): add Time and FixedTimestep utilities"
```

---

### Task 1.5: Implement ECS Module

**Files:**
- Create: `crates/longhorn-core/src/ecs/mod.rs`
- Create: `crates/longhorn-core/src/ecs/world.rs`
- Create: `crates/longhorn-core/src/ecs/entity.rs`
- Create: `crates/longhorn-core/src/ecs/component.rs`

**Step 1: Create ecs/mod.rs**

```rust
mod world;
mod entity;
mod component;

pub use world::*;
pub use entity::*;
pub use component::*;

// Re-export hecs types we use
pub use hecs::Entity;
```

**Step 2: Create ecs/component.rs**

```rust
use serde::{Deserialize, Serialize};
use crate::types::AssetId;

/// Name component for identifying entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Name(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name(s.to_string())
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s)
    }
}

/// Enabled/disabled state for entities.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Enabled(pub bool);

impl Enabled {
    pub fn new(enabled: bool) -> Self {
        Enabled(enabled)
    }

    pub fn is_enabled(&self) -> bool {
        self.0
    }
}

/// Sprite component for 2D rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: String,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
    pub z_index: i32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture: String::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            z_index: 0,
        }
    }
}

impl Sprite {
    pub fn new(texture: impl Into<String>) -> Self {
        Self {
            texture: texture.into(),
            ..Default::default()
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    pub fn with_z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    pub fn with_flip(mut self, x: bool, y: bool) -> Self {
        self.flip_x = x;
        self.flip_y = y;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let name = Name::new("Player");
        assert_eq!(name.as_str(), "Player");
    }

    #[test]
    fn test_sprite_builder() {
        let sprite = Sprite::new("player.png")
            .with_color(1.0, 0.0, 0.0, 1.0)
            .with_z_index(5)
            .with_flip(true, false);

        assert_eq!(sprite.texture, "player.png");
        assert_eq!(sprite.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(sprite.z_index, 5);
        assert!(sprite.flip_x);
        assert!(!sprite.flip_y);
    }
}
```

**Step 3: Create ecs/entity.rs**

```rust
use hecs::Entity;
use crate::types::EntityId;

/// Handle to an entity with convenience methods.
#[derive(Debug, Clone, Copy)]
pub struct EntityHandle {
    pub entity: Entity,
}

impl EntityHandle {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn id(&self) -> EntityId {
        EntityId::from(self.entity)
    }
}

impl From<Entity> for EntityHandle {
    fn from(entity: Entity) -> Self {
        EntityHandle::new(entity)
    }
}
```

**Step 4: Create ecs/world.rs**

```rust
use hecs::{Entity, World as HecsWorld};
use crate::math::Transform;
use crate::types::{EntityId, Error, Result};
use super::component::{Name, Enabled, Sprite};
use super::entity::EntityHandle;

/// Game world containing all entities and components.
pub struct World {
    inner: HecsWorld,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            inner: HecsWorld::new(),
        }
    }

    /// Spawn a new entity with a name.
    pub fn spawn(&mut self, name: impl Into<String>) -> EntityBuilder<'_> {
        EntityBuilder::new(self, name.into())
    }

    /// Spawn an entity with components directly.
    pub fn spawn_with<C: hecs::DynamicBundle>(&mut self, components: C) -> EntityHandle {
        let entity = self.inner.spawn(components);
        EntityHandle::new(entity)
    }

    /// Despawn an entity.
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        self.inner
            .despawn(entity)
            .map_err(|_| Error::EntityNotFound(EntityId::from(entity)))
    }

    /// Find an entity by name.
    pub fn find(&self, name: &str) -> Option<EntityHandle> {
        for (entity, n) in self.inner.query::<&Name>().iter() {
            if n.as_str() == name {
                return Some(EntityHandle::new(entity));
            }
        }
        None
    }

    /// Get a component from an entity.
    pub fn get<C: hecs::Component + Clone>(&self, entity: Entity) -> Result<C> {
        self.inner
            .get::<&C>(entity)
            .map(|c| c.clone())
            .map_err(|_| Error::ComponentNotFound)
    }

    /// Get a mutable reference to a component.
    pub fn get_mut<C: hecs::Component>(&mut self, entity: Entity) -> Result<hecs::RefMut<'_, C>> {
        self.inner
            .get::<&mut C>(entity)
            .map_err(|_| Error::ComponentNotFound)
    }

    /// Set/replace a component on an entity.
    pub fn set<C: hecs::Component>(&mut self, entity: Entity, component: C) -> Result<()> {
        self.inner
            .insert_one(entity, component)
            .map_err(|_| Error::EntityNotFound(EntityId::from(entity)))
    }

    /// Check if entity has a component.
    pub fn has<C: hecs::Component>(&self, entity: Entity) -> bool {
        self.inner.get::<&C>(entity).is_ok()
    }

    /// Iterate over all entities with specific components.
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Get access to the underlying hecs world.
    pub fn inner(&self) -> &HecsWorld {
        &self.inner
    }

    /// Get mutable access to the underlying hecs world.
    pub fn inner_mut(&mut self) -> &mut HecsWorld {
        &mut self.inner
    }

    /// Count total entities.
    pub fn entity_count(&self) -> u32 {
        self.inner.len()
    }
}

/// Builder for spawning entities with components.
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    name: String,
    transform: Option<Transform>,
    sprite: Option<Sprite>,
    enabled: bool,
}

impl<'w> EntityBuilder<'w> {
    fn new(world: &'w mut World, name: String) -> Self {
        Self {
            world,
            name,
            transform: None,
            sprite: None,
            enabled: true,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.transform = Some(Transform::new(x, y));
        self
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
        self
    }

    pub fn with_texture(mut self, texture: impl Into<String>) -> Self {
        self.sprite = Some(Sprite::new(texture));
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn build(self) -> EntityHandle {
        let mut builder = hecs::EntityBuilder::new();

        builder.add(Name::new(self.name));
        builder.add(Enabled::new(self.enabled));

        if let Some(transform) = self.transform {
            builder.add(transform);
        } else {
            builder.add(Transform::default());
        }

        if let Some(sprite) = self.sprite {
            builder.add(sprite);
        }

        let entity = self.world.inner.spawn(builder.build());
        EntityHandle::new(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        let entity = world.spawn("Player")
            .with_position(100.0, 200.0)
            .with_texture("player.png")
            .build();

        assert_eq!(world.entity_count(), 1);

        let name: Name = world.get(entity.entity).unwrap();
        assert_eq!(name.as_str(), "Player");

        let transform: Transform = world.get(entity.entity).unwrap();
        assert_eq!(transform.position.x, 100.0);
        assert_eq!(transform.position.y, 200.0);
    }

    #[test]
    fn test_find_entity() {
        let mut world = World::new();
        world.spawn("Player").build();
        world.spawn("Enemy").build();

        let player = world.find("Player");
        assert!(player.is_some());

        let missing = world.find("NotFound");
        assert!(missing.is_none());
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let entity = world.spawn("Player").build();
        assert_eq!(world.entity_count(), 1);

        world.despawn(entity.entity).unwrap();
        assert_eq!(world.entity_count(), 0);
    }
}
```

**Step 5: Run all tests**

Run: `cargo test -p longhorn-core`
Expected: All tests pass

**Step 6: Verify full crate compiles**

Run: `cargo check -p longhorn-core`
Expected: Success, no errors

**Step 7: Commit**

```bash
git add crates/longhorn-core/src/ecs/
git commit -m "feat(core): add ECS module with World, EntityBuilder, components"
```

---

## Phase 2: Parallel Crate Implementation

The following crates can be implemented in parallel as they only depend on longhorn-core:

- Task 2.1: longhorn-input
- Task 2.2: longhorn-assets
- Task 2.3: longhorn-renderer
- Task 2.4: longhorn-scripting

---

### Task 2.1: Implement Input Crate

**Files:**
- Create: `crates/longhorn-input/Cargo.toml`
- Create: `crates/longhorn-input/src/lib.rs`
- Create: `crates/longhorn-input/src/touch.rs`
- Create: `crates/longhorn-input/src/events.rs`
- Create: `crates/longhorn-input/src/input_state.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "longhorn-input"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
```

**Step 2: Create src/events.rs**

```rust
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Touch event types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TouchEvent {
    Start { x: f32, y: f32 },
    Move { x: f32, y: f32 },
    End { x: f32, y: f32 },
}

impl TouchEvent {
    pub fn position(&self) -> Vec2 {
        match self {
            TouchEvent::Start { x, y } => Vec2::new(*x, *y),
            TouchEvent::Move { x, y } => Vec2::new(*x, *y),
            TouchEvent::End { x, y } => Vec2::new(*x, *y),
        }
    }

    pub fn is_start(&self) -> bool {
        matches!(self, TouchEvent::Start { .. })
    }

    pub fn is_move(&self) -> bool {
        matches!(self, TouchEvent::Move { .. })
    }

    pub fn is_end(&self) -> bool {
        matches!(self, TouchEvent::End { .. })
    }
}
```

**Step 3: Create src/touch.rs**

```rust
use glam::Vec2;
use super::events::TouchEvent;

/// Tracks the state of a single touch point.
#[derive(Debug, Clone, Default)]
pub struct Touch {
    pub position: Option<Vec2>,
    pub start_position: Option<Vec2>,
    pub is_down: bool,
}

impl Touch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_event(&mut self, event: TouchEvent) {
        match event {
            TouchEvent::Start { x, y } => {
                let pos = Vec2::new(x, y);
                self.position = Some(pos);
                self.start_position = Some(pos);
                self.is_down = true;
            }
            TouchEvent::Move { x, y } => {
                self.position = Some(Vec2::new(x, y));
            }
            TouchEvent::End { x, y } => {
                self.position = Some(Vec2::new(x, y));
                self.is_down = false;
            }
        }
    }

    pub fn drag_delta(&self) -> Option<Vec2> {
        match (self.position, self.start_position) {
            (Some(current), Some(start)) => Some(current - start),
            _ => None,
        }
    }
}
```

**Step 4: Create src/input_state.rs**

```rust
use glam::Vec2;
use super::events::TouchEvent;
use super::touch::Touch;

/// Current frame input snapshot.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    touch: Touch,
    just_pressed: bool,
    just_released: bool,
    events_this_frame: Vec<TouchEvent>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call at start of frame to reset per-frame state.
    pub fn begin_frame(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
        self.events_this_frame.clear();
    }

    /// Process a touch event.
    pub fn handle_event(&mut self, event: TouchEvent) {
        match &event {
            TouchEvent::Start { .. } => {
                self.just_pressed = true;
            }
            TouchEvent::End { .. } => {
                self.just_released = true;
            }
            _ => {}
        }
        self.touch.handle_event(event);
        self.events_this_frame.push(event);
    }

    /// Is touch currently down?
    pub fn is_touching(&self) -> bool {
        self.touch.is_down
    }

    /// Was touch just pressed this frame?
    pub fn just_pressed(&self) -> bool {
        self.just_pressed
    }

    /// Was touch just released this frame?
    pub fn just_released(&self) -> bool {
        self.just_released
    }

    /// Current touch position (if touching).
    pub fn position(&self) -> Option<Vec2> {
        self.touch.position
    }

    /// Get drag delta from start position.
    pub fn drag_delta(&self) -> Option<Vec2> {
        self.touch.drag_delta()
    }

    /// Get all events this frame.
    pub fn events(&self) -> &[TouchEvent] {
        &self.events_this_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_lifecycle() {
        let mut state = InputState::new();

        // Start touch
        state.begin_frame();
        state.handle_event(TouchEvent::Start { x: 100.0, y: 200.0 });

        assert!(state.is_touching());
        assert!(state.just_pressed());
        assert!(!state.just_released());
        assert_eq!(state.position(), Some(Vec2::new(100.0, 200.0)));

        // Move touch
        state.begin_frame();
        state.handle_event(TouchEvent::Move { x: 150.0, y: 250.0 });

        assert!(state.is_touching());
        assert!(!state.just_pressed());
        assert_eq!(state.position(), Some(Vec2::new(150.0, 250.0)));

        // End touch
        state.begin_frame();
        state.handle_event(TouchEvent::End { x: 150.0, y: 250.0 });

        assert!(!state.is_touching());
        assert!(state.just_released());
    }

    #[test]
    fn test_drag_delta() {
        let mut state = InputState::new();

        state.handle_event(TouchEvent::Start { x: 100.0, y: 100.0 });
        state.handle_event(TouchEvent::Move { x: 150.0, y: 120.0 });

        let delta = state.drag_delta().unwrap();
        assert_eq!(delta.x, 50.0);
        assert_eq!(delta.y, 20.0);
    }
}
```

**Step 5: Create src/lib.rs**

```rust
mod events;
mod touch;
mod input_state;

pub use events::*;
pub use touch::*;
pub use input_state::*;
```

**Step 6: Run tests**

Run: `cargo test -p longhorn-input`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/longhorn-input/
git commit -m "feat(input): add touch input handling with InputState"
```

---

### Task 2.2: Implement Assets Crate

**Files:**
- Create: `crates/longhorn-assets/Cargo.toml`
- Create: `crates/longhorn-assets/src/lib.rs`
- Create: `crates/longhorn-assets/src/handle.rs`
- Create: `crates/longhorn-assets/src/source.rs`
- Create: `crates/longhorn-assets/src/loader/mod.rs`
- Create: `crates/longhorn-assets/src/loader/texture.rs`
- Create: `crates/longhorn-assets/src/loader/json.rs`
- Create: `crates/longhorn-assets/src/asset_manager.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "longhorn-assets"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
image = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
```

**Step 2: Create src/handle.rs**

```rust
use std::marker::PhantomData;
use longhorn_core::AssetId;

/// Type-safe handle to a loaded asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle<T> {
    pub id: AssetId,
    _marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}
```

**Step 3: Create src/source.rs**

```rust
use std::path::{Path, PathBuf};
use longhorn_core::Result;

/// Trait for loading raw asset bytes.
pub trait AssetSource: Send + Sync {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>>;
    fn exists(&self, path: &str) -> bool;
}

/// Loads assets from the filesystem.
pub struct FilesystemSource {
    root: PathBuf,
}

impl FilesystemSource {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
}

impl AssetSource for FilesystemSource {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);
        std::fs::read(&full_path).map_err(|e| {
            longhorn_core::Error::AssetLoadError(format!(
                "Failed to load {}: {}",
                full_path.display(),
                e
            ))
        })
    }

    fn exists(&self, path: &str) -> bool {
        self.resolve_path(path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_filesystem_source() {
        let dir = std::env::temp_dir().join("longhorn_test_assets");
        std::fs::create_dir_all(&dir).unwrap();

        let test_file = dir.join("test.txt");
        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"hello").unwrap();

        let source = FilesystemSource::new(&dir);

        assert!(source.exists("test.txt"));
        assert!(!source.exists("missing.txt"));

        let bytes = source.load_bytes("test.txt").unwrap();
        assert_eq!(bytes, b"hello");

        std::fs::remove_file(test_file).unwrap();
    }
}
```

**Step 4: Create src/loader/mod.rs**

```rust
pub mod texture;
pub mod json;

pub use texture::*;
pub use json::*;
```

**Step 5: Create src/loader/texture.rs**

```rust
use image::{DynamicImage, GenericImageView};
use longhorn_core::Result;

/// Raw texture data loaded from an image file.
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl TextureData {
    /// Load texture from raw bytes (PNG/JPEG).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = image::load_from_memory(bytes).map_err(|e| {
            longhorn_core::Error::AssetLoadError(format!("Failed to decode image: {}", e))
        })?;

        Ok(Self::from_image(image))
    }

    /// Convert from a dynamic image.
    pub fn from_image(image: DynamicImage) -> Self {
        let (width, height) = image.dimensions();
        let rgba = image.to_rgba8();

        Self {
            width,
            height,
            pixels: rgba.into_raw(),
        }
    }

    /// Bytes per row (width * 4 for RGBA).
    pub fn bytes_per_row(&self) -> u32 {
        self.width * 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_from_bytes() {
        // Create a minimal 1x1 PNG
        let png_bytes = include_bytes!("../../test_assets/1x1.png");

        // This test will fail until we add test assets
        // For now, just verify the structure compiles
    }
}
```

**Step 6: Create src/loader/json.rs**

```rust
use serde::de::DeserializeOwned;
use longhorn_core::Result;

/// Load and parse JSON data.
pub fn load_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| {
        longhorn_core::Error::AssetLoadError(format!("Failed to parse JSON: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_load_json() {
        let bytes = br#"{"name": "test", "value": 42}"#;
        let data: TestData = load_json(bytes).unwrap();

        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }
}
```

**Step 7: Create src/asset_manager.rs**

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use longhorn_core::{AssetId, Result};
use crate::handle::AssetHandle;
use crate::source::AssetSource;
use crate::loader::TextureData;

static NEXT_ASSET_ID: AtomicU64 = AtomicU64::new(1);

fn next_asset_id() -> AssetId {
    AssetId::new(NEXT_ASSET_ID.fetch_add(1, Ordering::Relaxed))
}

/// Central registry for loaded assets.
pub struct AssetManager {
    source: Box<dyn AssetSource>,
    textures: HashMap<String, (AssetId, TextureData)>,
    json_cache: HashMap<String, (AssetId, Vec<u8>)>,
}

impl AssetManager {
    pub fn new(source: impl AssetSource + 'static) -> Self {
        Self {
            source: Box::new(source),
            textures: HashMap::new(),
            json_cache: HashMap::new(),
        }
    }

    /// Load a texture, caching by path.
    pub fn load_texture(&mut self, path: &str) -> Result<AssetHandle<TextureData>> {
        if let Some((id, _)) = self.textures.get(path) {
            return Ok(AssetHandle::new(*id));
        }

        let bytes = self.source.load_bytes(path)?;
        let texture = TextureData::from_bytes(&bytes)?;
        let id = next_asset_id();

        self.textures.insert(path.to_string(), (id, texture));
        log::debug!("Loaded texture: {} (id={:?})", path, id);

        Ok(AssetHandle::new(id))
    }

    /// Get a loaded texture by handle.
    pub fn get_texture(&self, handle: AssetHandle<TextureData>) -> Option<&TextureData> {
        self.textures
            .values()
            .find(|(id, _)| *id == handle.id)
            .map(|(_, data)| data)
    }

    /// Get a texture by path.
    pub fn get_texture_by_path(&self, path: &str) -> Option<&TextureData> {
        self.textures.get(path).map(|(_, data)| data)
    }

    /// Load JSON data.
    pub fn load_json<T: serde::de::DeserializeOwned>(&mut self, path: &str) -> Result<T> {
        let bytes = self.source.load_bytes(path)?;
        crate::loader::load_json(&bytes)
    }

    /// Check if an asset exists.
    pub fn exists(&self, path: &str) -> bool {
        self.source.exists(path)
    }

    /// Preload multiple assets.
    pub fn preload(&mut self, paths: &[&str]) -> Result<()> {
        for path in paths {
            if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") {
                self.load_texture(path)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FilesystemSource;

    #[test]
    fn test_asset_manager_caching() {
        // Test that loading same path returns same handle
        // This will need actual test assets to run properly
    }
}
```

**Step 8: Create src/lib.rs**

```rust
mod handle;
mod source;
mod loader;
mod asset_manager;

pub use handle::*;
pub use source::*;
pub use loader::*;
pub use asset_manager::*;
```

**Step 9: Run tests**

Run: `cargo test -p longhorn-assets`
Expected: Tests pass

**Step 10: Commit**

```bash
git add crates/longhorn-assets/
git commit -m "feat(assets): add AssetManager with texture and JSON loading"
```

---

### Task 2.3: Implement Renderer Crate

**Files:**
- Create: `crates/longhorn-renderer/Cargo.toml`
- Create: `crates/longhorn-renderer/src/lib.rs`
- Create: `crates/longhorn-renderer/src/color.rs`
- Create: `crates/longhorn-renderer/src/camera.rs`
- Create: `crates/longhorn-renderer/src/texture.rs`
- Create: `crates/longhorn-renderer/src/sprite_batch.rs`
- Create: `crates/longhorn-renderer/src/pipeline/mod.rs`
- Create: `crates/longhorn-renderer/src/pipeline/sprite.wgsl`
- Create: `crates/longhorn-renderer/src/renderer.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "longhorn-renderer"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
longhorn-assets = { workspace = true }
wgpu = { workspace = true }
glam = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
bytemuck = { version = "1.14", features = ["derive"] }
```

**Step 2: Create src/color.rs**

```rust
use serde::{Deserialize, Serialize};

/// RGBA color with f32 components (0.0 - 1.0).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_rgba8() {
        let color = Color::from_rgba8(255, 128, 0, 255);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }
}
```

**Step 3: Create src/camera.rs**

```rust
use glam::{Mat4, Vec2};
use longhorn_core::Rect;

/// 2D orthographic camera.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport_size: Vec2,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(width, height),
        }
    }

    /// Get the view-projection matrix.
    pub fn view_projection(&self) -> Mat4 {
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        let left = self.position.x - half_width;
        let right = self.position.x + half_width;
        let bottom = self.position.y - half_height;
        let top = self.position.y + half_height;

        Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0)
    }

    /// Get the visible bounds in world coordinates.
    pub fn visible_bounds(&self) -> Rect {
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        Rect {
            min: Vec2::new(self.position.x - half_width, self.position.y - half_height),
            max: Vec2::new(self.position.x + half_width, self.position.y + half_height),
        }
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let half_size = self.viewport_size / 2.0;
        let normalized = (screen_pos - half_size) / self.zoom;
        self.position + normalized
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_size = self.viewport_size / 2.0;
        let relative = (world_pos - self.position) * self.zoom;
        relative + half_size
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_center() {
        let camera = Camera::new(800.0, 600.0);

        // Center of screen should map to camera position
        let world = camera.screen_to_world(Vec2::new(400.0, 300.0));
        assert!(world.x.abs() < 0.001);
        assert!(world.y.abs() < 0.001);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 2.0;

        let bounds = camera.visible_bounds();
        assert_eq!(bounds.width(), 400.0);
        assert_eq!(bounds.height(), 300.0);
    }
}
```

**Step 4: Create src/texture.rs**

```rust
use longhorn_assets::TextureData;
use std::collections::HashMap;

/// GPU texture wrapper.
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

impl GpuTexture {
    pub fn from_texture_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        data: &TextureData,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(data.bytes_per_row()),
                rows_per_image: Some(data.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
            width: data.width,
            height: data.height,
        }
    }
}

/// Cache of GPU textures by path.
pub struct TextureCache {
    textures: HashMap<String, GpuTexture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn get(&self, path: &str) -> Option<&GpuTexture> {
        self.textures.get(path)
    }

    pub fn insert(&mut self, path: String, texture: GpuTexture) {
        self.textures.insert(path, texture);
    }

    pub fn contains(&self, path: &str) -> bool {
        self.textures.contains_key(path)
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 5: Create src/sprite_batch.rs**

```rust
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use crate::color::Color;

/// Vertex for sprite rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// A sprite to be drawn.
#[derive(Debug, Clone)]
pub struct SpriteInstance {
    pub texture_path: String,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub z_index: i32,
    pub texture_size: Vec2,
}

/// Batches sprites by texture for efficient rendering.
pub struct SpriteBatch {
    sprites: Vec<SpriteInstance>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            sprites: Vec::with_capacity(1024),
        }
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    pub fn add(&mut self, sprite: SpriteInstance) {
        self.sprites.push(sprite);
    }

    /// Sort sprites by z-index then by texture for batching.
    pub fn sort(&mut self) {
        self.sprites.sort_by(|a, b| {
            a.z_index
                .cmp(&b.z_index)
                .then_with(|| a.texture_path.cmp(&b.texture_path))
        });
    }

    /// Get all sprites.
    pub fn sprites(&self) -> &[SpriteInstance] {
        &self.sprites
    }

    /// Generate vertices for a sprite.
    pub fn generate_vertices(sprite: &SpriteInstance) -> [SpriteVertex; 6] {
        let half_w = sprite.texture_size.x * sprite.scale.x / 2.0;
        let half_h = sprite.texture_size.y * sprite.scale.y / 2.0;

        let cos = sprite.rotation.cos();
        let sin = sprite.rotation.sin();

        let rotate = |x: f32, y: f32| -> [f32; 2] {
            [
                sprite.position.x + x * cos - y * sin,
                sprite.position.y + x * sin + y * cos,
            ]
        };

        let (u_min, u_max) = if sprite.flip_x { (1.0, 0.0) } else { (0.0, 1.0) };
        let (v_min, v_max) = if sprite.flip_y { (1.0, 0.0) } else { (0.0, 1.0) };

        let color = sprite.color.to_array();

        let tl = SpriteVertex { position: rotate(-half_w, half_h), tex_coords: [u_min, v_min], color };
        let tr = SpriteVertex { position: rotate(half_w, half_h), tex_coords: [u_max, v_min], color };
        let bl = SpriteVertex { position: rotate(-half_w, -half_h), tex_coords: [u_min, v_max], color };
        let br = SpriteVertex { position: rotate(half_w, -half_h), tex_coords: [u_max, v_max], color };

        [tl, tr, bl, tr, br, bl]
    }
}

impl Default for SpriteBatch {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 6: Create src/pipeline/mod.rs**

```rust
pub const SPRITE_SHADER: &str = include_str!("sprite.wgsl");
```

**Step 7: Create src/pipeline/sprite.wgsl**

```wgsl
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return tex_color * in.color;
}
```

**Step 8: Create src/renderer.rs**

```rust
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;
use longhorn_core::{World, Transform, Sprite as SpriteComponent, Enabled};
use longhorn_assets::AssetManager;
use crate::camera::Camera;
use crate::color::Color;
use crate::sprite_batch::{SpriteBatch, SpriteInstance, SpriteVertex};
use crate::texture::{GpuTexture, TextureCache};
use crate::pipeline::SPRITE_SHADER;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// Main renderer for 2D sprites.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_cache: TextureCache,
    sprite_batch: SpriteBatch,
    vertex_buffer: wgpu::Buffer,
    clear_color: Color,
}

impl Renderer {
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Self, wgpu::RequestDeviceError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Camera uniform
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        // Render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[SpriteVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Vertex buffer (pre-allocated for batching)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: (std::mem::size_of::<SpriteVertex>() * 6 * 10000) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            texture_cache: TextureCache::new(),
            sprite_batch: SpriteBatch::new(),
            vertex_buffer,
            clear_color: Color::BLACK,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Upload a texture to GPU.
    pub fn upload_texture(&mut self, path: &str, data: &longhorn_assets::TextureData) {
        if !self.texture_cache.contains(path) {
            let gpu_texture = GpuTexture::from_texture_data(
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
                data,
                Some(path),
            );
            self.texture_cache.insert(path.to_string(), gpu_texture);
            log::debug!("Uploaded texture to GPU: {}", path);
        }
    }

    /// Render the world.
    pub fn render(&mut self, world: &World, assets: &AssetManager, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        // Update camera uniform
        let uniform = CameraUniform {
            view_proj: camera.view_projection().to_cols_array_2d(),
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));

        // Collect sprites from world
        self.sprite_batch.clear();

        for (_, (transform, sprite, enabled)) in world.query::<(&Transform, &SpriteComponent, &Enabled)>().iter() {
            if !enabled.is_enabled() {
                continue;
            }

            // Ensure texture is loaded
            if let Some(texture_data) = assets.get_texture_by_path(&sprite.texture) {
                self.upload_texture(&sprite.texture, texture_data);

                if let Some(gpu_texture) = self.texture_cache.get(&sprite.texture) {
                    self.sprite_batch.add(SpriteInstance {
                        texture_path: sprite.texture.clone(),
                        position: transform.position,
                        rotation: transform.rotation,
                        scale: transform.scale,
                        color: sprite.color.into(),
                        flip_x: sprite.flip_x,
                        flip_y: sprite.flip_y,
                        z_index: sprite.z_index,
                        texture_size: glam::Vec2::new(gpu_texture.width as f32, gpu_texture.height as f32),
                    });
                }
            }
        }

        self.sprite_batch.sort();

        // Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.to_wgpu()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Batch by texture
            let sprites = self.sprite_batch.sprites();
            let mut i = 0;
            while i < sprites.len() {
                let current_texture = &sprites[i].texture_path;
                let batch_start = i;

                // Find batch end (same texture)
                while i < sprites.len() && sprites[i].texture_path == *current_texture {
                    i += 1;
                }

                let batch = &sprites[batch_start..i];

                if let Some(gpu_texture) = self.texture_cache.get(current_texture) {
                    // Generate vertices for batch
                    let vertices: Vec<SpriteVertex> = batch
                        .iter()
                        .flat_map(|s| SpriteBatch::generate_vertices(s))
                        .collect();

                    self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

                    render_pass.set_bind_group(1, &gpu_texture.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass.draw(0..vertices.len() as u32, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
```

**Step 9: Create src/lib.rs**

```rust
mod color;
mod camera;
mod texture;
mod sprite_batch;
mod pipeline;
mod renderer;

pub use color::*;
pub use camera::*;
pub use texture::*;
pub use sprite_batch::*;
pub use renderer::*;
```

**Step 10: Verify compiles**

Run: `cargo check -p longhorn-renderer`
Expected: Compiles (may have warnings)

**Step 11: Commit**

```bash
git add crates/longhorn-renderer/
git commit -m "feat(renderer): add wgpu-based 2D sprite renderer"
```

---

### Task 2.4: Implement Scripting Crate (Stub)

For MVP, we'll create a stub scripting crate that can be expanded later. Full Deno integration is complex and can be deferred.

**Files:**
- Create: `crates/longhorn-scripting/Cargo.toml`
- Create: `crates/longhorn-scripting/src/lib.rs`
- Create: `crates/longhorn-scripting/src/runtime.rs`
- Create: `crates/longhorn-scripting/api/longhorn.d.ts`

**Step 1: Create Cargo.toml**

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
```

**Step 2: Create src/runtime.rs**

```rust
use std::path::Path;
use longhorn_core::{World, Result, Error};
use longhorn_input::InputState;

/// Placeholder script runtime.
///
/// In the full implementation, this would use deno_core to run TypeScript.
/// For MVP, this provides the interface that the engine expects.
pub struct ScriptRuntime {
    game_path: Option<String>,
    initialized: bool,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            game_path: None,
            initialized: false,
        }
    }

    /// Load a game from a directory containing game.json.
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::AssetNotFound(path.display().to_string()));
        }

        let manifest_path = path.join("game.json");
        if !manifest_path.exists() {
            return Err(Error::AssetNotFound("game.json".to_string()));
        }

        self.game_path = Some(path.display().to_string());
        log::info!("Loaded game from: {}", path.display());

        Ok(())
    }

    /// Initialize the game (call onStart).
    pub fn initialize(&mut self, _world: &mut World) -> Result<()> {
        if self.game_path.is_none() {
            return Err(Error::ScriptError("No game loaded".to_string()));
        }

        // In full implementation: call TypeScript onStart()
        log::debug!("ScriptRuntime::initialize (stub)");
        self.initialized = true;

        Ok(())
    }

    /// Update the game (call onUpdate).
    pub fn update(&mut self, _world: &mut World, _delta: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        // In full implementation: call TypeScript onUpdate(world, dt)
        Ok(())
    }

    /// Handle touch start event.
    pub fn on_touch_start(&mut self, _world: &mut World, _x: f32, _y: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        // In full implementation: call TypeScript onTouchStart(world, x, y)
        Ok(())
    }

    /// Check if a game is loaded.
    pub fn is_loaded(&self) -> bool {
        self.game_path.is_some()
    }

    /// Check if the game is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 3: Create src/lib.rs**

```rust
mod runtime;

pub use runtime::*;
```

**Step 4: Create TypeScript type definitions**

Create directory: `mkdir -p crates/longhorn-scripting/api`

Create `crates/longhorn-scripting/api/longhorn.d.ts`:

```typescript
// Longhorn Game Engine TypeScript API

declare module "longhorn" {
    export interface Vec2 {
        x: number;
        y: number;
    }

    export interface Transform {
        position: Vec2;
        rotation: number;
        scale: Vec2;
    }

    export interface Sprite {
        texture: string;
        color: [number, number, number, number];
        flipX: boolean;
        flipY: boolean;
        zIndex: number;
    }

    export interface Entity {
        id: number;
        get<T>(component: ComponentType<T>): T;
        set<T>(component: ComponentType<T>, value: T): void;
        has<T>(component: ComponentType<T>): boolean;
    }

    export interface ComponentType<T> {
        readonly name: string;
    }

    export const Transform: ComponentType<Transform>;
    export const Sprite: ComponentType<Sprite>;

    export interface EntityBuilder {
        with<T>(component: ComponentType<T>, value: Partial<T>): EntityBuilder;
        build(): Entity;
    }

    export interface World {
        spawn(name: string): EntityBuilder;
        find(name: string): Entity | null;
        despawn(entity: Entity): void;
    }

    export interface Input {
        isTouching(): boolean;
        justPressed(): boolean;
        justReleased(): boolean;
        position(): Vec2 | null;
    }

    export const input: Input;
}
```

**Step 5: Verify compiles**

Run: `cargo check -p longhorn-scripting`
Expected: Success

**Step 6: Commit**

```bash
git add crates/longhorn-scripting/
git commit -m "feat(scripting): add stub ScriptRuntime with TypeScript API definitions"
```

---

## Phase 3: Engine Integration

### Task 3.1: Implement Engine Crate

**Files:**
- Create: `crates/longhorn-engine/Cargo.toml`
- Create: `crates/longhorn-engine/src/lib.rs`
- Create: `crates/longhorn-engine/src/config.rs`
- Create: `crates/longhorn-engine/src/game.rs`
- Create: `crates/longhorn-engine/src/engine.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "longhorn-engine"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
longhorn-renderer = { workspace = true }
longhorn-input = { workspace = true }
longhorn-assets = { workspace = true }
longhorn-scripting = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
log = { workspace = true }
```

**Step 2: Create src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use longhorn_renderer::Color;

/// Engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub target_fps: u32,
    pub clear_color: [f32; 4],
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            target_fps: 60,
            clear_color: [0.1, 0.1, 0.1, 1.0],
        }
    }
}

impl EngineConfig {
    pub fn clear_color(&self) -> Color {
        Color::new(
            self.clear_color[0],
            self.clear_color[1],
            self.clear_color[2],
            self.clear_color[3],
        )
    }
}
```

**Step 3: Create src/game.rs**

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;
use longhorn_core::Result;

/// Game manifest (game.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub viewport: ViewportConfig,
    #[serde(default)]
    pub assets: AssetsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetsConfig {
    #[serde(default)]
    pub preload: Vec<String>,
}

impl GameManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().join("game.json");
        let content = std::fs::read_to_string(&path)?;
        let manifest: GameManifest = serde_json::from_str(&content).map_err(|e| {
            longhorn_core::Error::AssetLoadError(format!("Failed to parse game.json: {}", e))
        })?;
        Ok(manifest)
    }
}
```

**Step 4: Create src/engine.rs**

```rust
use std::path::Path;
use std::time::Duration;
use longhorn_core::{World, Time, FixedTimestep, Result};
use longhorn_renderer::{Renderer, Camera};
use longhorn_input::{InputState, TouchEvent};
use longhorn_assets::{AssetManager, FilesystemSource};
use longhorn_scripting::ScriptRuntime;
use crate::config::EngineConfig;
use crate::game::GameManifest;

/// Main game engine.
pub struct Engine {
    pub world: World,
    pub renderer: Option<Renderer>,
    pub camera: Camera,
    pub input: InputState,
    pub assets: AssetManager,
    pub scripting: ScriptRuntime,
    pub time: Time,
    pub config: EngineConfig,
    game_manifest: Option<GameManifest>,
    game_path: Option<String>,
}

impl Engine {
    /// Create a new engine without a renderer (for testing or headless).
    pub fn new_headless() -> Self {
        Self {
            world: World::new(),
            renderer: None,
            camera: Camera::default(),
            input: InputState::new(),
            assets: AssetManager::new(FilesystemSource::new(".")),
            scripting: ScriptRuntime::new(),
            time: Time::new(),
            config: EngineConfig::default(),
            game_manifest: None,
            game_path: None,
        }
    }

    /// Create an engine with a renderer attached to a window.
    pub async fn new_with_renderer(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let renderer = Renderer::new(window, width, height)
            .await
            .map_err(|e| longhorn_core::Error::RenderError(e.to_string()))?;

        let config = EngineConfig {
            viewport_width: width,
            viewport_height: height,
            ..Default::default()
        };

        Ok(Self {
            world: World::new(),
            renderer: Some(renderer),
            camera: Camera::new(width as f32, height as f32),
            input: InputState::new(),
            assets: AssetManager::new(FilesystemSource::new(".")),
            scripting: ScriptRuntime::new(),
            time: Time::new(),
            config,
            game_manifest: None,
            game_path: None,
        })
    }

    /// Load a game from a directory.
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Load manifest
        let manifest = GameManifest::load(path)?;
        log::info!("Loading game: {} v{}", manifest.name, manifest.version);

        // Update config from manifest
        self.config.viewport_width = manifest.viewport.width;
        self.config.viewport_height = manifest.viewport.height;
        self.camera = Camera::new(manifest.viewport.width as f32, manifest.viewport.height as f32);

        // Set up asset loading from game directory
        let assets_path = path.join("assets");
        self.assets = AssetManager::new(FilesystemSource::new(&assets_path));

        // Preload assets
        let preload: Vec<&str> = manifest.assets.preload.iter().map(|s| s.as_str()).collect();
        self.assets.preload(&preload)?;

        // Load script
        self.scripting.load_game(path)?;

        self.game_path = Some(path.display().to_string());
        self.game_manifest = Some(manifest);

        Ok(())
    }

    /// Initialize the game (call onStart in script).
    pub fn start(&mut self) -> Result<()> {
        self.scripting.initialize(&mut self.world)
    }

    /// Handle a touch event.
    pub fn handle_touch(&mut self, event: TouchEvent) {
        // Check for touch start to call script hook
        if let TouchEvent::Start { x, y } = event {
            let _ = self.scripting.on_touch_start(&mut self.world, x, y);
        }
        self.input.handle_event(event);
    }

    /// Run a single frame.
    pub fn update(&mut self) -> Result<()> {
        self.time.update();
        let dt = self.time.delta_seconds();

        // Begin frame for input
        self.input.begin_frame();

        // Update scripting
        self.scripting.update(&mut self.world, dt)?;

        // Render if we have a renderer
        if let Some(ref mut renderer) = self.renderer {
            renderer.set_clear_color(self.config.clear_color());
            renderer
                .render(&self.world, &self.assets, &self.camera)
                .map_err(|e| longhorn_core::Error::RenderError(e.to_string()))?;
        }

        Ok(())
    }

    /// Resize the renderer.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.resize(width, height);
        }
        self.camera.viewport_size = glam::Vec2::new(width as f32, height as f32);
    }

    /// Get the current game manifest.
    pub fn manifest(&self) -> Option<&GameManifest> {
        self.game_manifest.as_ref()
    }
}
```

**Step 5: Create src/lib.rs**

```rust
mod config;
mod game;
mod engine;

pub use config::*;
pub use game::*;
pub use engine::*;

// Re-export commonly used types
pub use longhorn_core::{World, Transform, Sprite, Name, Enabled, EntityHandle};
pub use longhorn_renderer::{Camera, Color};
pub use longhorn_input::{InputState, TouchEvent};
pub use longhorn_assets::AssetManager;
```

**Step 6: Verify compiles**

Run: `cargo check -p longhorn-engine`
Expected: Success

**Step 7: Commit**

```bash
git add crates/longhorn-engine/
git commit -m "feat(engine): add Engine struct integrating all subsystems"
```

---

## Phase 4: Editor & Mobile (Parallel)

### Task 4.1: Implement Editor Crate

**Files:**
- Create: `crates/longhorn-editor/Cargo.toml`
- Create: `crates/longhorn-editor/src/lib.rs`
- Create: `crates/longhorn-editor/src/state.rs`
- Create: `crates/longhorn-editor/src/panels/mod.rs`
- Create: `crates/longhorn-editor/src/panels/scene_tree.rs`
- Create: `crates/longhorn-editor/src/panels/inspector.rs`
- Create: `crates/longhorn-editor/src/panels/viewport.rs`
- Create: `crates/longhorn-editor/src/editor.rs`
- Create: `editor/Cargo.toml`
- Create: `editor/src/main.rs`

**Step 1: Create crates/longhorn-editor/Cargo.toml**

```toml
[package]
name = "longhorn-editor"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
longhorn-engine = { workspace = true }
egui = { workspace = true }
log = { workspace = true }
```

**Step 2: Create src/state.rs**

```rust
use longhorn_core::EntityId;

/// Editor state.
#[derive(Debug, Default)]
pub struct EditorState {
    pub selected_entity: Option<hecs::Entity>,
    pub game_path: Option<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, entity: Option<hecs::Entity>) {
        self.selected_entity = entity;
    }

    pub fn is_selected(&self, entity: hecs::Entity) -> bool {
        self.selected_entity == Some(entity)
    }
}
```

**Step 3: Create src/panels/mod.rs**

```rust
mod scene_tree;
mod inspector;
mod viewport;

pub use scene_tree::*;
pub use inspector::*;
pub use viewport::*;
```

**Step 4: Create src/panels/scene_tree.rs**

```rust
use egui::{Ui, Response};
use longhorn_core::{World, Name};
use crate::state::EditorState;

/// Scene tree panel showing entity hierarchy.
pub struct SceneTreePanel;

impl SceneTreePanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, world: &World, state: &mut EditorState) {
        ui.heading("Scene");
        ui.separator();

        for (entity, name) in world.query::<&Name>().iter() {
            let is_selected = state.is_selected(entity);

            let response = ui.selectable_label(is_selected, name.as_str());

            if response.clicked() {
                state.select(Some(entity));
            }
        }

        if world.entity_count() == 0 {
            ui.label("(No entities)");
        }
    }
}

impl Default for SceneTreePanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 5: Create src/panels/inspector.rs**

```rust
use egui::Ui;
use longhorn_core::{World, Name, Transform, Sprite, Enabled};
use crate::state::EditorState;

/// Inspector panel showing selected entity components.
pub struct InspectorPanel;

impl InspectorPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, world: &mut World, state: &EditorState) {
        ui.heading("Inspector");
        ui.separator();

        let Some(entity) = state.selected_entity else {
            ui.label("Select an entity");
            return;
        };

        // Name
        if let Ok(name) = world.get::<Name>(entity) {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.label(name.as_str());
            });
        }

        ui.separator();

        // Transform
        if let Ok(mut transform) = world.get_mut::<Transform>(entity) {
            ui.collapsing("Transform", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(egui::DragValue::new(&mut transform.position.x).prefix("x: ").speed(1.0));
                    ui.add(egui::DragValue::new(&mut transform.position.y).prefix("y: ").speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add(egui::DragValue::new(&mut transform.rotation).speed(0.01));
                });

                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.add(egui::DragValue::new(&mut transform.scale.x).prefix("x: ").speed(0.1));
                    ui.add(egui::DragValue::new(&mut transform.scale.y).prefix("y: ").speed(0.1));
                });
            });
        }

        // Sprite
        if let Ok(sprite) = world.get::<Sprite>(entity) {
            ui.collapsing("Sprite", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Texture:");
                    ui.label(&sprite.texture);
                });

                ui.horizontal(|ui| {
                    ui.label("Z-Index:");
                    ui.label(sprite.z_index.to_string());
                });
            });
        }

        // Enabled
        if let Ok(mut enabled) = world.get_mut::<Enabled>(entity) {
            ui.horizontal(|ui| {
                ui.label("Enabled:");
                ui.checkbox(&mut enabled.0, "");
            });
        }
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 6: Create src/panels/viewport.rs**

```rust
use egui::Ui;

/// Viewport panel (placeholder for game view).
pub struct ViewportPanel;

impl ViewportPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let available = ui.available_size();

        // Draw a placeholder rectangle
        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());

        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(30, 30, 40),
        );

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Game Viewport",
            egui::FontId::proportional(24.0),
            egui::Color32::from_rgb(100, 100, 100),
        );
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 7: Create src/editor.rs**

```rust
use longhorn_engine::Engine;
use crate::state::EditorState;
use crate::panels::{SceneTreePanel, InspectorPanel, ViewportPanel};

/// Main editor application.
pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, engine: &mut Engine) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Game...").clicked() {
                        // TODO: File dialog
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        egui::SidePanel::left("scene_tree")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.scene_tree.show(ui, &engine.world, &mut self.state);
            });

        egui::SidePanel::right("inspector")
            .default_width(300.0)
            .show(ctx, |ui| {
                self.inspector.show(ui, &mut engine.world, &self.state);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewport.show(ui);
        });
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 8: Create src/lib.rs**

```rust
mod state;
mod panels;
mod editor;

pub use state::*;
pub use panels::*;
pub use editor::*;
```

**Step 9: Create editor/Cargo.toml**

```toml
[package]
name = "editor"
version.workspace = true
edition.workspace = true

[[bin]]
name = "longhorn-editor"
path = "src/main.rs"

[dependencies]
longhorn-engine = { workspace = true }
longhorn-editor = { workspace = true }
eframe = { workspace = true }
env_logger = { workspace = true }
log = { workspace = true }
```

**Step 10: Create editor/src/main.rs**

```rust
use eframe::egui;
use longhorn_engine::Engine;
use longhorn_editor::Editor;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Longhorn Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Longhorn Editor",
        options,
        Box::new(|_cc| Ok(Box::new(EditorApp::new()))),
    )
}

struct EditorApp {
    engine: Engine,
    editor: Editor,
}

impl EditorApp {
    fn new() -> Self {
        let mut engine = Engine::new_headless();

        // Spawn some test entities
        engine.world.spawn("Player")
            .with_position(100.0, 200.0)
            .with_texture("player.png")
            .build();

        engine.world.spawn("Enemy")
            .with_position(300.0, 200.0)
            .with_texture("enemy.png")
            .build();

        Self {
            engine,
            editor: Editor::new(),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.editor.show(ctx, &mut self.engine);
    }
}
```

**Step 11: Verify compiles**

Run: `cargo check -p longhorn-editor -p editor`
Expected: Success

**Step 12: Test run editor**

Run: `cargo run -p editor`
Expected: Editor window opens with scene tree, inspector, and viewport

**Step 13: Commit**

```bash
git add crates/longhorn-editor/ editor/
git commit -m "feat(editor): add egui-based editor with scene tree and inspector"
```

---

### Task 4.2: Implement Mobile Crate (Stub)

**Files:**
- Create: `crates/longhorn-mobile/Cargo.toml`
- Create: `crates/longhorn-mobile/src/lib.rs`
- Create: `crates/longhorn-mobile/src/platform.rs`
- Create: `crates/longhorn-mobile/src/app.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "longhorn-mobile"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-engine = { workspace = true }
longhorn-input = { workspace = true }
winit = { workspace = true }
raw-window-handle = { workspace = true }
log = { workspace = true }
```

**Step 2: Create src/platform.rs**

```rust
use longhorn_input::TouchEvent;

/// Platform events.
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    Touch(TouchEvent),
    Resize { width: u32, height: u32 },
    Suspend,
    Resume,
    Quit,
}

/// Platform trait for abstracting OS-specific behavior.
pub trait Platform {
    fn get_display_size(&self) -> (u32, u32);
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
}
```

**Step 3: Create src/app.rs**

```rust
use std::path::Path;
use longhorn_engine::Engine;
use crate::platform::PlatformEvent;

/// Mobile app runner.
pub struct MobileApp {
    engine: Engine,
    running: bool,
}

impl MobileApp {
    pub fn new_headless() -> Self {
        Self {
            engine: Engine::new_headless(),
            running: false,
        }
    }

    pub fn load_game(&mut self, path: impl AsRef<Path>) -> longhorn_core::Result<()> {
        self.engine.load_game(path)
    }

    pub fn start(&mut self) -> longhorn_core::Result<()> {
        self.engine.start()?;
        self.running = true;
        Ok(())
    }

    pub fn handle_event(&mut self, event: PlatformEvent) {
        match event {
            PlatformEvent::Touch(touch) => {
                self.engine.handle_touch(touch);
            }
            PlatformEvent::Resize { width, height } => {
                self.engine.resize(width, height);
            }
            PlatformEvent::Suspend => {
                self.running = false;
                log::info!("App suspended");
            }
            PlatformEvent::Resume => {
                self.running = true;
                log::info!("App resumed");
            }
            PlatformEvent::Quit => {
                self.running = false;
            }
        }
    }

    pub fn update(&mut self) -> longhorn_core::Result<()> {
        if self.running {
            self.engine.update()
        } else {
            Ok(())
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }
}
```

**Step 4: Create src/lib.rs**

```rust
mod platform;
mod app;

pub use platform::*;
pub use app::*;
```

**Step 5: Verify compiles**

Run: `cargo check -p longhorn-mobile`
Expected: Success

**Step 6: Commit**

```bash
git add crates/longhorn-mobile/
git commit -m "feat(mobile): add stub MobileApp runner and platform abstraction"
```

---

## Phase 5: Example Game & Final Verification

### Task 5.1: Create Example Game

**Files:**
- Create: `examples/hello-world/game.json`
- Create: `examples/hello-world/src/main.ts`
- Create: `examples/hello-world/assets/.gitkeep`

**Step 1: Create example game structure**

```bash
mkdir -p examples/hello-world/src
mkdir -p examples/hello-world/assets
```

**Step 2: Create game.json**

```json
{
  "name": "Hello World",
  "version": "1.0.0",
  "entry": "src/main.ts",
  "viewport": {
    "width": 1280,
    "height": 720
  },
  "assets": {
    "preload": []
  }
}
```

**Step 3: Create src/main.ts**

```typescript
import { World, Transform, Sprite, input } from "longhorn";

export function onStart(world: World) {
  console.log("Hello from Longhorn v2!");

  // Create a simple entity
  world.spawn("Player")
    .with(Transform, { x: 640, y: 360 })
    .build();
}

export function onUpdate(world: World, dt: number) {
  const player = world.find("Player");
  if (!player) return;

  // Move player with touch
  if (input.justPressed()) {
    const pos = input.position();
    if (pos) {
      const transform = player.get(Transform);
      transform.x = pos.x;
      transform.y = pos.y;
    }
  }
}

export function onTouchStart(world: World, x: number, y: number) {
  console.log(`Touch at (${x}, ${y})`);
}
```

**Step 4: Create .gitkeep for assets**

```bash
touch examples/hello-world/assets/.gitkeep
```

**Step 5: Commit**

```bash
git add examples/
git commit -m "feat: add hello-world example game"
```

---

### Task 5.2: Final Build Verification

**Step 1: Build entire workspace**

Run: `cargo build --workspace`
Expected: All crates compile successfully

**Step 2: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 3: Run editor**

Run: `cargo run -p editor`
Expected: Editor opens, shows test entities, inspector works

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "chore: final build verification and fixes"
```

---

## Summary

This implementation plan covers:

1. **Phase 1:** Workspace setup and core foundation (types, math, time, ECS)
2. **Phase 2:** Parallel crate implementation (input, assets, renderer, scripting stub)
3. **Phase 3:** Engine integration
4. **Phase 4:** Editor and mobile platform (parallel)
5. **Phase 5:** Example game and verification

**Total crates:** 8 + 1 editor binary
**Estimated tasks for sub-agents:** Can parallelize Phase 2 (4 crates) and Phase 4 (2 crates)
