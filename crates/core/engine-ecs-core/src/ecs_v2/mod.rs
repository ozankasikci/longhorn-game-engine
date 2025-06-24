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
//! use engine_component_traits::Component;
//!
//! // Define component types
//! #[derive(Clone, Debug)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! #[derive(Clone, Debug)]
//! struct Velocity { x: f32, y: f32 }
//! impl Component for Velocity {}
//!
//! // Register components
//! register_component::<Position>();
//! register_component::<Velocity>();
//!
//! // Create world and spawn entities
//! let mut world = World::new();
//! let entity = world.spawn_bundle((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
//!
//! // Query components individually  
//! for (entity, pos) in world.query_legacy::<Position>() {
//!     println!("Entity {:?} position: {:?}", entity, pos);
//! }
//! ```

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod entity;
pub mod query;
pub mod world;

#[cfg(test)]
pub mod test_utils;

// Re-export main types
pub use archetype::{Archetype, ArchetypeId};
pub use bundle::{Bundle, WorldBundleExt};
pub use component::{register_component, ComponentArrayTrait};
pub use entity::{Entity, EntityLocation};
pub use query::{Changed, Query, QueryData, QueryMut, Read, Write};
pub use world::World;

// Re-export from engine_component_traits
pub use engine_component_traits::{Component, ComponentClone, ComponentTicks, Tick};
