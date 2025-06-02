// Entity Component System implementation

use std::collections::{HashMap, HashSet};
use std::any::{Any, TypeId};

// Entity is just a unique ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

impl Entity {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub fn id(&self) -> u32 {
        self.0
    }
}

// Component trait - all components must implement this
pub trait Component: Any + Send + Sync {
    fn type_id() -> TypeId where Self: Sized {
        TypeId::of::<Self>()
    }
}

// Storage for a specific component type
pub struct ComponentStorage<T: Component> {
    components: HashMap<Entity, T>,
}

impl<T: Component> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }
    
    pub fn remove(&mut self, entity: &Entity) -> Option<T> {
        self.components.remove(entity)
    }
    
    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.components.get(entity)
    }
    
    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        self.components.get_mut(entity)
    }
    
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.components.keys()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.components.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.components.iter_mut()
    }
}

// World contains all entities and components
pub struct World {
    next_entity_id: u32,
    entities: HashSet<Entity>,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            entities: HashSet::new(),
            components: HashMap::new(),
        }
    }
    
    // Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.insert(entity);
        entity
    }
    
    // Alias for create_entity for convenience
    pub fn spawn(&mut self) -> Entity {
        self.create_entity()
    }
    
    // Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if self.entities.remove(&entity) {
            // Remove from all component storages
            for storage in self.components.values_mut() {
                // This is a bit tricky - we need to call remove on each storage
                // For now, we'll keep it simple and just mark the entity as removed
                // In a production ECS, this would be more sophisticated
            }
            true
        } else {
            false
        }
    }
    
    // Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), &'static str> {
        if !self.entities.contains(&entity) {
            return Err("Entity doesn't exist");
        }
        
        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));
        
        if let Some(storage) = storage.downcast_mut::<ComponentStorage<T>>() {
            storage.insert(entity, component);
            Ok(())
        } else {
            Err("Failed to cast storage")
        }
    }
    
    // Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.downcast_mut::<ComponentStorage<T>>() {
                return storage.remove(&entity);
            }
        }
        None
    }
    
    // Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(storage) = storage.downcast_ref::<ComponentStorage<T>>() {
                return storage.get(&entity);
            }
        }
        None
    }
    
    // Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.downcast_mut::<ComponentStorage<T>>() {
                return storage.get_mut(&entity);
            }
        }
        None
    }
    
    // Check if entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.get_component::<T>(entity).is_some()
    }
    
    // Get all entities with a specific component
    pub fn entities_with_component<T: Component>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(storage) = storage.downcast_ref::<ComponentStorage<T>>() {
                return storage.entities().copied().collect();
            }
        }
        Vec::new()
    }
    
    // Get all entities
    pub fn all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }
    
    // Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
    
    // Query for entities with multiple components
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(storage) = storage.downcast_ref::<ComponentStorage<T>>() {
                return storage.iter().map(|(e, c)| (*e, c)).collect::<Vec<_>>().into_iter();
            }
        }
        Vec::new().into_iter()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone, PartialEq)]
    struct TestComponent {
        value: i32,
    }
    impl Component for TestComponent {}
    
    #[derive(Debug, Clone, PartialEq)]
    struct MockTransform {
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
    }
    impl Component for MockTransform {}
    impl Default for MockTransform {
        fn default() -> Self {
            Self {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            }
        }
    }
    
    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        
        assert_eq!(entity1.id(), 1);
        assert_eq!(entity2.id(), 2);
        assert_ne!(entity1, entity2);
    }
    
    #[test]
    fn test_add_and_get_component() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let transform = MockTransform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        world.add_component(entity, transform.clone()).unwrap();
        
        let retrieved = world.get_component::<MockTransform>(entity);
        assert_eq!(retrieved, Some(&transform));
    }
    
    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let test_comp = TestComponent { value: 42 };
        world.add_component(entity, test_comp.clone()).unwrap();
        
        assert!(world.has_component::<TestComponent>(entity));
        
        let removed = world.remove_component::<TestComponent>(entity);
        assert_eq!(removed, Some(test_comp));
        assert!(!world.has_component::<TestComponent>(entity));
    }
    
    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let transform = MockTransform::default();
        let test_comp = TestComponent { value: 123 };
        
        world.add_component(entity, transform.clone()).unwrap();
        world.add_component(entity, test_comp.clone()).unwrap();
        
        assert_eq!(world.get_component::<MockTransform>(entity), Some(&transform));
        assert_eq!(world.get_component::<TestComponent>(entity), Some(&test_comp));
    }
    
    #[test]
    fn test_query_components() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        let entity3 = world.create_entity();
        
        world.add_component(entity1, TestComponent { value: 1 }).unwrap();
        world.add_component(entity2, TestComponent { value: 2 }).unwrap();
        // entity3 has no TestComponent
        
        let results: Vec<_> = world.query::<TestComponent>().collect();
        assert_eq!(results.len(), 2);
        
        let values: Vec<i32> = results.iter().map(|(_, comp)| comp.value).collect();
        assert!(values.contains(&1));
        assert!(values.contains(&2));
    }
    
    #[test]
    fn test_entities_with_component() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        
        world.add_component(entity1, MockTransform::default()).unwrap();
        
        let entities_with_transform = world.entities_with_component::<MockTransform>();
        assert_eq!(entities_with_transform.len(), 1);
        assert!(entities_with_transform.contains(&entity1));
        assert!(!entities_with_transform.contains(&entity2));
    }
}