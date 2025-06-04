// Engine Core - Core data structures and systems for the mobile game engine

pub mod ecs_v2; // Data-oriented ECS
pub mod time;
pub mod memory;
pub mod error;

// Re-export ECS types
pub use ecs_v2::{Entity, World, ArchetypeId, Query, QueryMut, Read, Write, Changed, register_component, WorldBundleExt};
// Re-export component traits
pub use engine_component_traits::{Component, ComponentClone, ComponentTicks, Tick, Bundle};
// Re-export error types
pub use error::{EcsError, EcsResult};


#[cfg(test)]
mod tests {
    // Tests are in separate test files and ecs_v2.rs
}