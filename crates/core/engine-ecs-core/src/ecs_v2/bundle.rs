//! Bundle system for ECS V2
//!
//! Bundles allow spawning entities with multiple components at once.

use crate::ecs_v2::{ArchetypeId, Entity, World};
use crate::error::EcsResult;
use engine_component_traits::Component;

/// Extension trait for spawning bundles on World
pub trait WorldBundleExt {
    /// Spawn an entity with a bundle of components
    fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity;
}

impl WorldBundleExt for World {
    fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = self.spawn();
        bundle.insert_into_world(self, entity).unwrap_or_else(|e| {
            // If insertion fails, despawn the entity
            self.despawn(entity);
            panic!("Failed to insert bundle: {:?}", e);
        });
        entity
    }
}

/// Trait for types that can be spawned as a bundle of components
pub trait Bundle: Send + Sync + 'static {
    /// Get the archetype ID for this bundle
    fn archetype_id() -> ArchetypeId;

    /// Insert this bundle's components into the world for the given entity
    fn insert_into_world(self, world: &mut World, entity: Entity) -> EcsResult<()>;
}

// For now, we won't implement Bundle for single components to avoid conflicts
// Users should use world.add_component() for single components

// Implement Bundle for tuples
impl<T1: Component> Bundle for (T1,) {
    fn archetype_id() -> ArchetypeId {
        ArchetypeId::new().with_component::<T1>()
    }

    fn insert_into_world(self, world: &mut World, entity: Entity) -> EcsResult<()> {
        world.add_component(entity, self.0)?;
        Ok(())
    }
}

impl<T1: Component, T2: Component> Bundle for (T1, T2) {
    fn archetype_id() -> ArchetypeId {
        ArchetypeId::new()
            .with_component::<T1>()
            .with_component::<T2>()
    }

    fn insert_into_world(self, world: &mut World, entity: Entity) -> EcsResult<()> {
        world.add_component(entity, self.0)?;
        world.add_component(entity, self.1)?;
        Ok(())
    }
}

impl<T1: Component, T2: Component, T3: Component> Bundle for (T1, T2, T3) {
    fn archetype_id() -> ArchetypeId {
        ArchetypeId::new()
            .with_component::<T1>()
            .with_component::<T2>()
            .with_component::<T3>()
    }

    fn insert_into_world(self, world: &mut World, entity: Entity) -> EcsResult<()> {
        world.add_component(entity, self.0)?;
        world.add_component(entity, self.1)?;
        world.add_component(entity, self.2)?;
        Ok(())
    }
}

impl<T1: Component, T2: Component, T3: Component, T4: Component> Bundle for (T1, T2, T3, T4) {
    fn archetype_id() -> ArchetypeId {
        ArchetypeId::new()
            .with_component::<T1>()
            .with_component::<T2>()
            .with_component::<T3>()
            .with_component::<T4>()
    }

    fn insert_into_world(self, world: &mut World, entity: Entity) -> EcsResult<()> {
        world.add_component(entity, self.0)?;
        world.add_component(entity, self.1)?;
        world.add_component(entity, self.2)?;
        world.add_component(entity, self.3)?;
        Ok(())
    }
}

// Re-export the trait from engine_component_traits
pub use engine_component_traits::Bundle as BundleComponentTrait;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs_v2::test_utils::*;

    #[test]
    fn test_single_component_bundle() {
        register_test_components();

        let mut world = World::new();
        let entity = world.spawn_bundle((Position::new(1.0, 2.0, 3.0),));

        assert!(world.contains(entity));
        let pos = world.get_component::<Position>(entity);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().x, 1.0);
    }

    #[test]
    fn test_tuple_bundle() {
        register_test_components();

        let mut world = World::new();
        let bundle = (Position::new(1.0, 2.0, 3.0), Velocity::new(4.0, 5.0, 6.0));
        let entity = world.spawn_bundle(bundle);

        assert!(world.contains(entity));
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));

        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);

        let vel = world.get_component::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 4.0);
    }

    #[test]
    fn test_triple_bundle() {
        register_test_components();

        let mut world = World::new();
        let bundle = (
            Position::new(1.0, 2.0, 3.0),
            Velocity::new(4.0, 5.0, 6.0),
            Health::new(100.0),
        );
        let entity = world.spawn_bundle(bundle);

        assert!(world.contains(entity));
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));
        assert!(world.has_component::<Health>(entity));
    }

    #[test]
    fn test_bundle_archetype_id() {
        let single_id = <(Position,)>::archetype_id();
        assert!(single_id.has_component::<Position>());
        assert_eq!(single_id.len(), 1);

        let tuple_id = <(Position, Velocity)>::archetype_id();
        assert!(tuple_id.has_component::<Position>());
        assert!(tuple_id.has_component::<Velocity>());
        assert_eq!(tuple_id.len(), 2);

        let triple_id = <(Position, Velocity, Health)>::archetype_id();
        assert!(triple_id.has_component::<Position>());
        assert!(triple_id.has_component::<Velocity>());
        assert!(triple_id.has_component::<Health>());
        assert_eq!(triple_id.len(), 3);
    }

    #[test]
    fn test_zero_sized_component_bundle() {
        register_test_components();

        let mut world = World::new();
        let bundle = (Player, Position::new(1.0, 2.0, 3.0));
        let entity = world.spawn_bundle(bundle);

        assert!(world.has_component::<Player>(entity));
        assert!(world.has_component::<Position>(entity));
    }

    #[test]
    fn test_multiple_bundles() {
        register_test_components();

        let mut world = World::new();

        // Spawn multiple entities with different bundles
        let e1 = world.spawn_bundle((Position::new(1.0, 0.0, 0.0),));
        let e2 = world.spawn_bundle((Position::new(2.0, 0.0, 0.0), Velocity::new(1.0, 1.0, 1.0)));
        let e3 = world.spawn_bundle((Position::new(3.0, 0.0, 0.0), Health::new(50.0)));

        assert!(world.has_component::<Position>(e1));
        assert!(!world.has_component::<Velocity>(e1));
        assert!(!world.has_component::<Health>(e1));

        assert!(world.has_component::<Position>(e2));
        assert!(world.has_component::<Velocity>(e2));
        assert!(!world.has_component::<Health>(e2));

        assert!(world.has_component::<Position>(e3));
        assert!(!world.has_component::<Velocity>(e3));
        assert!(world.has_component::<Health>(e3));
    }
}
