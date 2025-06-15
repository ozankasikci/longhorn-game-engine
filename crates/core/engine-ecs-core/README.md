# Engine ECS Core

Modern Entity Component System (ECS) implementation for the Longhorn Game Engine.

## Overview

This crate provides the core ECS architecture using an archetype-based storage system optimized for cache performance and batch operations.

## Key Features

- **Archetype-based storage**: Entities with the same component combination are grouped together
- **Cache-efficient iteration**: Components are stored contiguously in memory
- **Type-safe queries**: Compile-time guarantees for component access
- **Bundle system**: Efficiently spawn entities with multiple components
- **Change tracking**: Track component modifications for efficient system scheduling

## Main Types

- `World`: The ECS world containing all entities and components
- `Entity`: Unique identifier for game objects
- `Component`: Trait that types must implement to be used as components
- `Query`: Type-safe component queries
- `Bundle`: Groups of components that can be spawned together
- `Archetype`: Storage for entities with the same component combination

## Usage Example

```rust
use engine_ecs_core::{World, Bundle};

// Define components
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { x: f32, y: f32 }

// Create world and spawn entities
let mut world = World::new();
let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

// Query components
for (entity, (pos, vel)) in world.query::<(&Position, &Velocity)>() {
    println!("Entity {:?} at ({}, {})", entity, pos.x, pos.y);
}
```

## Dependencies

This is a core crate with minimal dependencies:
- `serde`: For serialization support
- Standard library only