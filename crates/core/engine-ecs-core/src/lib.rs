// Engine Core - Core data structures and systems for the mobile game engine

pub mod ecs_v2; // Data-oriented ECS
pub mod error;
pub mod memory;
pub mod time;

// Re-export ECS types
pub use ecs_v2::{
    register_component, ArchetypeId, Changed, Entity, Query, QueryMut, Read, World, WorldBundleExt,
    Write,
};
// Re-export component traits
pub use engine_component_traits::{Bundle, Component, ComponentClone, ComponentTicks, Tick};
// Re-export error types
pub use error::{EcsError, EcsResult};

#[cfg(test)]
mod tests {
    // Tests are in separate test files and ecs_v2.rs
}
