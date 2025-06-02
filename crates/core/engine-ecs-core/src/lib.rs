// Engine Core - Core data structures and systems for the mobile game engine

pub mod ecs;
pub mod ecs_v2; // New data-oriented ECS
pub mod time;
pub mod memory;

// Re-export ECS types (legacy)
pub use ecs::{Entity, Component, World};

// Re-export new ECS types
pub use ecs_v2::{Entity as EntityV2, Component as ComponentV2, World as WorldV2, ArchetypeId, Query, QueryMut, Read, Write, Changed, Tick};


#[cfg(test)]
mod tests {
    use super::*;
    use engine_components_core::Transform;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }
}