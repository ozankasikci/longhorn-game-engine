use crate::ecs::{EntityHandle, Name};
use crate::types::{LonghornError, Result};
use hecs::{Entity, World as HecsWorld};

/// ECS World wrapper
pub struct World {
    world: HecsWorld,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            world: HecsWorld::new(),
        }
    }

    /// Spawn a new entity and return a builder
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(self)
    }

    /// Spawn an entity with components
    pub fn spawn_with(&mut self, components: impl hecs::DynamicBundle) -> EntityHandle {
        let id = self.world.spawn(components);
        EntityHandle::new(id)
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: EntityHandle) -> Result<()> {
        self.world
            .despawn(entity.id)
            .map_err(|_| LonghornError::EntityNotFound(entity.id))
    }

    /// Check if an entity exists
    pub fn exists(&self, entity: EntityHandle) -> bool {
        self.world.contains(entity.id)
    }

    /// Find an entity by name
    pub fn find(&self, name: &str) -> Option<EntityHandle> {
        for (id, entity_name) in self.world.query::<&Name>().iter() {
            if entity_name.as_str() == name {
                return Some(EntityHandle::new(id));
            }
        }
        None
    }

    /// Get a component from an entity
    pub fn get<T: hecs::Component>(&self, entity: EntityHandle) -> Result<hecs::Ref<'_, T>> {
        self.world
            .get::<&T>(entity.id)
            .map_err(|_| LonghornError::ComponentNotFound(entity.id))
    }

    /// Get a mutable component from an entity
    pub fn get_mut<T: hecs::Component>(&self, entity: EntityHandle) -> Result<hecs::RefMut<'_, T>> {
        self.world
            .get::<&mut T>(entity.id)
            .map_err(|_| LonghornError::ComponentNotFound(entity.id))
    }

    /// Set a component on an entity (insert or replace)
    pub fn set<T: hecs::Component>(&mut self, entity: EntityHandle, component: T) -> Result<()> {
        if !self.exists(entity) {
            return Err(LonghornError::EntityNotFound(entity.id));
        }
        self.world
            .insert_one(entity.id, component)
            .map_err(|_| LonghornError::EntityNotFound(entity.id))
    }

    /// Remove a component from an entity
    pub fn remove<T: hecs::Component>(&mut self, entity: EntityHandle) -> Result<T> {
        self.world
            .remove_one::<T>(entity.id)
            .map_err(|_| LonghornError::ComponentNotFound(entity.id))
    }

    /// Check if an entity has a component
    pub fn has<T: hecs::Component>(&self, entity: EntityHandle) -> bool {
        self.world.get::<&T>(entity.id).is_ok()
    }

    /// Query entities
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    /// Query entities mutably
    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    /// Get the number of entities
    pub fn len(&self) -> usize {
        self.world.len() as usize
    }

    /// Check if the world is empty
    pub fn is_empty(&self) -> bool {
        self.world.is_empty()
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.world.clear();
    }

    /// Get access to the underlying hecs world
    pub fn inner(&self) -> &HecsWorld {
        &self.world
    }

    /// Get mutable access to the underlying hecs world
    pub fn inner_mut(&mut self) -> &mut HecsWorld {
        &mut self.world
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating entities with a fluent API
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity: Option<Entity>,
}

impl<'w> EntityBuilder<'w> {
    fn new(world: &'w mut World) -> Self {
        Self {
            world,
            entity: None,
        }
    }

    /// Add a component to the entity being built
    pub fn with<T: hecs::Component>(mut self, component: T) -> Self {
        if let Some(entity) = self.entity {
            // Entity already exists, insert component
            let _ = self.world.world.insert_one(entity, component);
        } else {
            // First component, spawn the entity
            self.entity = Some(self.world.world.spawn((component,)));
        }
        self
    }

    /// Add multiple components to the entity
    pub fn with_bundle(mut self, bundle: impl hecs::DynamicBundle) -> Self {
        if let Some(entity) = self.entity {
            // Entity already exists, insert bundle
            let _ = self.world.world.insert(entity, bundle);
        } else {
            // First components, spawn the entity
            self.entity = Some(self.world.world.spawn(bundle));
        }
        self
    }

    /// Finish building and return the entity handle
    pub fn build(self) -> EntityHandle {
        let entity = self.entity.unwrap_or_else(|| {
            // No components were added, spawn empty entity
            self.world.world.spawn(())
        });
        EntityHandle::new(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Enabled, Name, Sprite};
    use crate::math::Transform;
    use crate::types::AssetId;
    use glam::Vec2;

    #[test]
    fn test_world_new() {
        let world = World::new();
        assert_eq!(world.len(), 0);
        assert!(world.is_empty());
    }

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        let entity = world.spawn().build();

        assert_eq!(world.len(), 1);
        assert!(world.exists(entity));
    }

    #[test]
    fn test_spawn_with_components() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::new())
            .build();

        assert!(world.exists(entity));
        assert!(world.has::<Name>(entity));
        assert!(world.has::<Transform>(entity));
    }

    #[test]
    fn test_spawn_with_bundle() {
        let mut world = World::new();
        let entity = world.spawn_with((Name::new("Enemy"), Transform::new()));

        assert!(world.exists(entity));
        assert!(world.has::<Name>(entity));
        assert!(world.has::<Transform>(entity));
    }

    #[test]
    fn test_despawn_entity() {
        let mut world = World::new();
        let entity = world.spawn().build();

        assert!(world.exists(entity));

        world.despawn(entity).unwrap();
        assert!(!world.exists(entity));
    }

    #[test]
    fn test_get_set_component() {
        let mut world = World::new();
        let entity = world.spawn().with(Name::new("Test")).build();

        {
            let name = world.get::<Name>(entity).unwrap();
            assert_eq!(name.as_str(), "Test");
        }

        world.set(entity, Name::new("Updated")).unwrap();

        {
            let name = world.get::<Name>(entity).unwrap();
            assert_eq!(name.as_str(), "Updated");
        }
    }

    #[test]
    fn test_get_mut_component() {
        let mut world = World::new();
        let entity = world.spawn().with(Transform::new()).build();

        {
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.position = Vec2::new(10.0, 20.0);
        }

        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let entity = world.spawn().with(Name::new("Test")).build();

        assert!(world.has::<Name>(entity));

        let removed = world.remove::<Name>(entity).unwrap();
        assert_eq!(removed.as_str(), "Test");
        assert!(!world.has::<Name>(entity));
    }

    #[test]
    fn test_find_entity() {
        let mut world = World::new();
        let entity1 = world.spawn().with(Name::new("Player")).build();
        let _entity2 = world.spawn().with(Name::new("Enemy")).build();

        let found = world.find("Player").unwrap();
        assert_eq!(found, entity1);

        let not_found = world.find("NonExistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_query() {
        let mut world = World::new();

        world.spawn().with(Name::new("Entity1")).build();
        world.spawn().with(Name::new("Entity2")).build();
        world.spawn().with(Transform::new()).build();

        let mut count = 0;
        for (_id, _name) in world.query::<&Name>().iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_mut() {
        let mut world = World::new();

        world
            .spawn()
            .with(Transform::from_position(Vec2::new(1.0, 1.0)))
            .build();
        world
            .spawn()
            .with(Transform::from_position(Vec2::new(2.0, 2.0)))
            .build();

        for (_id, transform) in world.query_mut::<&mut Transform>().iter() {
            transform.position *= 2.0;
        }

        for (_id, transform) in world.query::<&Transform>().iter() {
            assert!(transform.position.x >= 2.0);
        }
    }

    #[test]
    fn test_clear() {
        let mut world = World::new();
        world.spawn().build();
        world.spawn().build();

        assert_eq!(world.len(), 2);

        world.clear();
        assert_eq!(world.len(), 0);
        assert!(world.is_empty());
    }

    #[test]
    fn test_entity_builder_fluent() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::new())
            .with(Enabled::default())
            .with(Sprite::new(AssetId::new(1), Vec2::new(32.0, 32.0)))
            .build();

        assert!(world.has::<Name>(entity));
        assert!(world.has::<Transform>(entity));
        assert!(world.has::<Enabled>(entity));
        assert!(world.has::<Sprite>(entity));
    }
}
