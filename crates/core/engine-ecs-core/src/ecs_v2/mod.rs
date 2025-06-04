//! # ECS V2 - Data-Oriented Entity Component System
//! 
//! This module implements a high-performance, cache-efficient ECS using archetype-based storage.
//! 
//! ## Architecture
//! 
//! The ECS is built around several key concepts:
//! - **Entity**: A unique identifier for game objects
//! - **Component**: Pure data attached to entities
//! - **Archetype**: A unique combination of component types
//! - **World**: The container for all entities and their components
//! - **Query**: Type-safe component queries with filtering
//! 
//! ## Example
//! 
//! ```rust
//! use engine_ecs_core::ecs_v2::*;
//! 
//! // Register components
//! register_component::<Position>();
//! register_component::<Velocity>();
//! 
//! // Create world and spawn entities
//! let mut world = World::new();
//! let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
//! 
//! // Query components
//! for (entity, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
//!     pos.x += vel.x;
//!     pos.y += vel.y;
//! }
//! ```

pub mod entity;
pub mod component;
pub mod archetype;
pub mod world;
pub mod query;
pub mod bundle;

#[cfg(test)]
pub mod test_utils;

// Re-export main types
pub use entity::{Entity, EntityLocation};
pub use component::{register_component, ComponentArrayTrait};
pub use archetype::{Archetype, ArchetypeId};
pub use world::World;
pub use query::{Query, QueryMut, QueryData, Read, Write, Changed};
pub use bundle::{Bundle, WorldBundleExt};

// Re-export from engine_component_traits
pub use engine_component_traits::{Component, ComponentClone, ComponentTicks, Tick};