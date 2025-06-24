//! Archetype-based storage for ECS V2
//!
//! Archetypes group entities with the same component combination for cache-efficient access.
//! Each unique combination of components creates a new archetype.

use crate::ecs_v2::{
    component::{create_component_array, ErasedComponentArray},
    entity::Entity,
};
use crate::error::{EcsError, EcsResult};
use engine_component_traits::{Component, ComponentClone, ComponentTicks};
use std::any::TypeId;
use std::collections::{BTreeSet, HashMap};

/// Archetype ID - uniquely identifies a combination of component types
///
/// Uses a BTreeSet to ensure consistent ordering regardless of insertion order.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArchetypeId(BTreeSet<TypeId>);

impl ArchetypeId {
    /// Create a new empty archetype ID
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// Add a component type to this archetype ID
    pub fn with_component<T: Component>(mut self) -> Self {
        self.0.insert(TypeId::of::<T>());
        self
    }

    /// Add a component type by TypeId
    pub fn with_type_id(mut self, type_id: TypeId) -> Self {
        self.0.insert(type_id);
        self
    }

    /// Check if this archetype contains a specific component type
    pub fn has_component<T: Component>(&self) -> bool {
        self.0.contains(&TypeId::of::<T>())
    }

    /// Check if this archetype contains a component type by TypeId
    pub fn has_type_id(&self, type_id: &TypeId) -> bool {
        self.0.contains(type_id)
    }

    /// Create an archetype ID from an iterator of TypeIds
    pub fn from_types(types: impl IntoIterator<Item = TypeId>) -> Self {
        Self(types.into_iter().collect())
    }

    /// Get an iterator over the component TypeIds
    pub fn type_ids(&self) -> impl Iterator<Item = &TypeId> {
        self.0.iter()
    }

    /// Get the number of component types in this archetype
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if this archetype has no components
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for ArchetypeId {
    fn default() -> Self {
        Self::new()
    }
}

/// An archetype stores entities that share the same component combination
///
/// Components are stored in parallel arrays for optimal cache performance.
/// All arrays have the same length and entities at the same index across
/// arrays belong together.
pub struct Archetype {
    id: ArchetypeId,
    entities: Vec<Entity>,
    components: HashMap<TypeId, ErasedComponentArray>,
}

impl Archetype {
    /// Create a new archetype with the given ID
    pub fn new(id: ArchetypeId) -> Self {
        Self {
            id,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }

    /// Get the archetype's ID
    pub fn id(&self) -> &ArchetypeId {
        &self.id
    }

    /// Get the number of entities in this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if this archetype is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Add an entity to this archetype and return its index
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let index = self.entities.len();
        self.entities.push(entity);
        index
    }

    /// Add a component for the most recently added entity
    pub fn add_component<T: Component>(
        &mut self,
        component: T,
        ticks: ComponentTicks,
    ) -> EcsResult<()> {
        let type_id = TypeId::of::<T>();

        // Ensure this component type belongs to this archetype
        if !self.id.has_type_id(&type_id) {
            return Err(EcsError::ComponentNotInArchetype);
        }

        if let Some(array) = self.components.get_mut(&type_id) {
            array
                .inner_mut()
                .push_cloned(Box::new(component) as Box<dyn ComponentClone>, ticks)
        } else {
            // Create new component array
            if let Some(mut new_array) = create_component_array(type_id) {
                new_array.push_cloned(Box::new(component) as Box<dyn ComponentClone>, ticks)?;
                self.components
                    .insert(type_id, ErasedComponentArray::new(new_array));
                Ok(())
            } else {
                Err(EcsError::ComponentNotRegistered(type_id))
            }
        }
    }

    /// Get a component at the given index
    pub fn get_component<T: Component>(&self, index: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)?
            .downcast_ref::<T>()?
            .get(index)
    }

    /// Get a mutable component at the given index
    pub fn get_component_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .downcast_mut::<T>()?
            .get_mut(index)
    }

    /// Get the entire component array for a type
    pub fn get_component_array<T: Component>(&self) -> Option<&ComponentArray<T>> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.downcast_ref::<T>()
    }

    /// Get the entire component array for a type mutably
    pub fn get_component_array_mut<T: Component>(&mut self) -> Option<&mut ComponentArray<T>> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)?.downcast_mut::<T>()
    }

    /// Check if this archetype has a specific component type
    pub fn has_component<T: Component>(&self) -> bool {
        self.id.has_component::<T>()
    }

    /// Remove an entity at the given index
    ///
    /// This swaps the entity with the last one and removes it,
    /// maintaining array density.
    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index >= self.entities.len() {
            return None;
        }

        let entity = self.entities.swap_remove(index);

        // Remove components at the same index
        for array in self.components.values_mut() {
            array.inner_mut().swap_remove(index);
        }

        Some(entity)
    }

    /// Get a slice of all entities in this archetype
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Clone a component at a specific index
    pub fn clone_component_at(
        &self,
        type_id: TypeId,
        index: usize,
    ) -> EcsResult<Option<Box<dyn ComponentClone>>> {
        let array = self
            .components
            .get(&type_id)
            .ok_or(EcsError::ComponentNotRegistered(type_id))?;
        Ok(array.inner().clone_component_at(index))
    }

    /// Get component ticks at a specific index
    pub fn get_component_ticks_at(&self, type_id: TypeId, index: usize) -> Option<ComponentTicks> {
        self.components.get(&type_id)?.inner().get_ticks_at(index)
    }

    /// Add a component from a cloned box
    pub fn add_component_cloned(
        &mut self,
        type_id: TypeId,
        component: Box<dyn ComponentClone>,
        ticks: ComponentTicks,
    ) -> EcsResult<()> {
        if let Some(array) = self.components.get_mut(&type_id) {
            array.inner_mut().push_cloned(component, ticks)
        } else {
            // Try to create new array from registry
            if let Some(mut new_array) = create_component_array(type_id) {
                new_array.push_cloned(component, ticks)?;
                self.components
                    .insert(type_id, ErasedComponentArray::new(new_array));
                Ok(())
            } else {
                Err(EcsError::ComponentNotRegistered(type_id))
            }
        }
    }

    /// Initialize component arrays for all types in the archetype
    pub fn initialize_components(&mut self) -> EcsResult<()> {
        for &type_id in self.id.type_ids() {
            if let std::collections::hash_map::Entry::Vacant(e) = self.components.entry(type_id) {
                if let Some(array) = create_component_array(type_id) {
                    e.insert(ErasedComponentArray::new(array));
                } else {
                    return Err(EcsError::ComponentNotRegistered(type_id));
                }
            }
        }
        Ok(())
    }
}

use crate::ecs_v2::component::ComponentArray;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs_v2::{
        component::{register_component, ComponentArrayTrait},
        entity::Entity,
    };
    use engine_component_traits::Tick;

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Clone, Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Clone, Debug, PartialEq)]
    struct Health {
        value: u32,
    }
    impl Component for Health {}

    fn setup_test_components() {
        register_component::<Position>();
        register_component::<Velocity>();
        register_component::<Health>();
    }

    #[test]
    fn test_archetype_id_creation() {
        let empty_id = ArchetypeId::new();
        assert!(empty_id.is_empty());
        assert_eq!(empty_id.len(), 0);

        let id_with_components = ArchetypeId::new()
            .with_component::<Position>()
            .with_component::<Velocity>();
        assert!(!id_with_components.is_empty());
        assert_eq!(id_with_components.len(), 2);
    }

    #[test]
    fn test_archetype_id_equality() {
        let id1 = ArchetypeId::new()
            .with_component::<Position>()
            .with_component::<Velocity>();

        let id2 = ArchetypeId::new()
            .with_component::<Velocity>()
            .with_component::<Position>();

        // Order shouldn't matter due to BTreeSet
        assert_eq!(id1, id2);

        let id3 = ArchetypeId::new().with_component::<Position>();

        assert_ne!(id1, id3);
    }

    #[test]
    fn test_archetype_id_has_component() {
        let id = ArchetypeId::new()
            .with_component::<Position>()
            .with_component::<Velocity>();

        assert!(id.has_component::<Position>());
        assert!(id.has_component::<Velocity>());
        assert!(!id.has_component::<Health>());

        assert!(id.has_type_id(&TypeId::of::<Position>()));
        assert!(!id.has_type_id(&TypeId::of::<Health>()));
    }

    #[test]
    fn test_archetype_id_from_types() {
        let types = vec![TypeId::of::<Position>(), TypeId::of::<Velocity>()];
        let id = ArchetypeId::from_types(types);

        assert_eq!(id.len(), 2);
        assert!(id.has_component::<Position>());
        assert!(id.has_component::<Velocity>());
    }

    #[test]
    fn test_archetype_creation() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let archetype = Archetype::new(id.clone());

        assert_eq!(archetype.id(), &id);
        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_archetype_entity_management() {
        setup_test_components();

        let id = ArchetypeId::new();
        let mut archetype = Archetype::new(id);

        let e1 = Entity::new(1, 1);
        let e2 = Entity::new(2, 1);

        let idx1 = archetype.add_entity(e1);
        let idx2 = archetype.add_entity(e2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(archetype.len(), 2);
        assert!(!archetype.is_empty());
        assert_eq!(archetype.entities(), &[e1, e2]);
    }

    #[test]
    fn test_archetype_component_storage() {
        setup_test_components();

        let id = ArchetypeId::new()
            .with_component::<Position>()
            .with_component::<Velocity>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let entity = Entity::new(1, 1);
        let tick = ComponentTicks::new(Tick::new(1));

        archetype.add_entity(entity);
        archetype
            .add_component(Position { x: 10.0, y: 20.0 }, tick)
            .unwrap();
        archetype
            .add_component(Velocity { x: 1.0, y: 2.0 }, tick)
            .unwrap();

        let pos = archetype.get_component::<Position>(0);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().x, 10.0);
        assert_eq!(pos.unwrap().y, 20.0);

        let vel = archetype.get_component::<Velocity>(0);
        assert!(vel.is_some());
        assert_eq!(vel.unwrap().x, 1.0);
        assert_eq!(vel.unwrap().y, 2.0);
    }

    #[test]
    fn test_archetype_component_mutation() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let entity = Entity::new(1, 1);
        let tick = ComponentTicks::new(Tick::new(1));

        archetype.add_entity(entity);
        archetype
            .add_component(Position { x: 10.0, y: 20.0 }, tick)
            .unwrap();

        // Mutate component
        if let Some(pos) = archetype.get_component_mut::<Position>(0) {
            pos.x = 30.0;
            pos.y = 40.0;
        }

        let pos = archetype.get_component::<Position>(0).unwrap();
        assert_eq!(pos.x, 30.0);
        assert_eq!(pos.y, 40.0);
    }

    #[test]
    fn test_archetype_entity_removal() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let e1 = Entity::new(1, 1);
        let e2 = Entity::new(2, 1);
        let e3 = Entity::new(3, 1);
        let tick = ComponentTicks::new(Tick::new(1));

        archetype.add_entity(e1);
        archetype
            .add_component(Position { x: 1.0, y: 1.0 }, tick)
            .unwrap();

        archetype.add_entity(e2);
        archetype
            .add_component(Position { x: 2.0, y: 2.0 }, tick)
            .unwrap();

        archetype.add_entity(e3);
        archetype
            .add_component(Position { x: 3.0, y: 3.0 }, tick)
            .unwrap();

        // Remove middle entity
        let removed = archetype.remove_entity(1);
        assert_eq!(removed, Some(e2));
        assert_eq!(archetype.len(), 2);

        // Check that last entity was swapped into position 1
        assert_eq!(archetype.entities()[0], e1);
        assert_eq!(archetype.entities()[1], e3);

        // Check component was also swapped
        let pos = archetype.get_component::<Position>(1).unwrap();
        assert_eq!(pos.x, 3.0);
        assert_eq!(pos.y, 3.0);
    }

    #[test]
    fn test_archetype_component_arrays() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let tick = ComponentTicks::new(Tick::new(1));

        for i in 0..5 {
            archetype.add_entity(Entity::new(i, 1));
            archetype
                .add_component(
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                    },
                    tick,
                )
                .unwrap();
        }

        let array = archetype.get_component_array::<Position>();
        assert!(array.is_some());

        let array = array.unwrap();
        assert_eq!(array.len(), 5);

        for i in 0..5 {
            assert_eq!(array.get(i).unwrap().x, i as f32);
            assert_eq!(array.get(i).unwrap().y, i as f32 * 2.0);
        }
    }

    #[test]
    fn test_archetype_component_cloning() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let entity = Entity::new(1, 1);
        let tick = ComponentTicks::new(Tick::new(1));

        archetype.add_entity(entity);
        archetype
            .add_component(Position { x: 10.0, y: 20.0 }, tick)
            .unwrap();

        let cloned = archetype.clone_component_at(TypeId::of::<Position>(), 0);
        assert!(cloned.is_ok());

        let cloned_comp = cloned.unwrap();
        assert!(cloned_comp.is_some());

        let cloned_comp = cloned_comp.unwrap();
        let cloned_pos = cloned_comp.as_any().downcast_ref::<Position>().unwrap();
        assert_eq!(cloned_pos.x, 10.0);
        assert_eq!(cloned_pos.y, 20.0);
    }

    #[test]
    fn test_archetype_ticks() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let entity = Entity::new(1, 1);
        let tick = ComponentTicks::new(Tick::new(42));

        archetype.add_entity(entity);
        archetype
            .add_component(Position { x: 10.0, y: 20.0 }, tick)
            .unwrap();

        let ticks = archetype.get_component_ticks_at(TypeId::of::<Position>(), 0);
        assert!(ticks.is_some());
        assert_eq!(ticks.unwrap().added.get(), 42);
    }

    #[test]
    fn test_archetype_wrong_component() {
        setup_test_components();

        let id = ArchetypeId::new().with_component::<Position>();
        let mut archetype = Archetype::new(id);
        archetype.initialize_components().unwrap();

        let entity = Entity::new(1, 1);
        let tick = ComponentTicks::new(Tick::new(1));

        archetype.add_entity(entity);

        // Try to add component not in archetype
        let result = archetype.add_component(Velocity { x: 1.0, y: 2.0 }, tick);
        assert!(result.is_err());

        match result {
            Err(EcsError::ComponentNotInArchetype) => (),
            _ => panic!("Expected ComponentNotInArchetype error"),
        }
    }

    #[test]
    fn test_archetype_component_not_registered() {
        // Use a unique component type that is definitely not registered
        #[derive(Clone, Debug, PartialEq)]
        struct UnregisteredComponent {
            data: String,
        }
        impl Component for UnregisteredComponent {}

        let id = ArchetypeId::new().with_component::<UnregisteredComponent>();
        let mut archetype = Archetype::new(id);

        let result = archetype.initialize_components();
        assert!(result.is_err());

        match result {
            Err(EcsError::ComponentNotRegistered(_)) => (),
            _ => panic!("Expected ComponentNotRegistered error"),
        }
    }
}
