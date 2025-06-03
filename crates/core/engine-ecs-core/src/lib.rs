// Engine Core - Core data structures and systems for the mobile game engine

pub mod ecs;    // Legacy ECS (for comparison)
pub mod ecs_v2; // Data-oriented ECS
pub mod time;
pub mod memory;

// Re-export ECS types
pub use ecs_v2::{Entity, Component, World, ArchetypeId, Query, QueryMut, Read, Write, Changed, Tick, Bundle, ComponentTicks, register_component};


#[cfg(test)]
mod tests {
    // Tests are in separate test files and ecs_v2.rs
}