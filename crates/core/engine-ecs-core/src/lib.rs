// Engine Core - Core data structures and systems for the mobile game engine

pub mod ecs;
pub mod ecs_v2; // New data-oriented ECS
pub mod time;
pub mod memory;

// Re-export ECS types (legacy)
pub use ecs::{Entity, Component, World};

// Re-export new ECS types
pub use ecs_v2::{Entity as EntityV2, Component as ComponentV2, World as WorldV2, ArchetypeId, Query, QueryMut, Read, Write, Changed, Tick};

// Re-export all components from the dedicated components crate
pub use engine_components_core::*;

// Implement Component traits for all standard components
impl Component for Transform {}
impl ComponentV2 for Transform {}

impl Component for Mesh {}
impl ComponentV2 for Mesh {}

impl Component for Material {}
impl ComponentV2 for Material {}

impl Component for Name {}
impl ComponentV2 for Name {}

impl Component for Visibility {}
impl ComponentV2 for Visibility {}

impl Component for Light {}
impl ComponentV2 for Light {}

impl Component for Sprite {}
impl ComponentV2 for Sprite {}

impl Component for SpriteRenderer {}
impl ComponentV2 for SpriteRenderer {}

impl Component for Canvas {}
impl ComponentV2 for Canvas {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }
}