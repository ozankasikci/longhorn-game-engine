// Data-Oriented ECS Implementation - Version 2
// Based on archetype storage for maximum cache efficiency

use std::any::{Any, TypeId};
use std::collections::{HashMap, BTreeSet};
// Removed unused imports
use rayon::prelude::*;

/// Entity is just an index into component arrays
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    id: u32,
    generation: u32, // For entity recycling safety
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
    
    pub fn id(&self) -> u32 {
        self.id
    }
    
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

/// Component trait - marker for types that can be stored as components
pub trait Component: 'static + Send + Sync {
    fn type_id() -> TypeId where Self: Sized {
        TypeId::of::<Self>()
    }
}

/// Archetype ID - uniquely identifies a combination of component types
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArchetypeId(BTreeSet<TypeId>);

impl ArchetypeId {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    
    pub fn with_component<T: Component>(mut self) -> Self {
        self.0.insert(TypeId::of::<T>());
        self
    }
    
    pub fn has_component<T: Component>(&self) -> bool {
        self.0.contains(&TypeId::of::<T>())
    }
    
    pub fn from_types(types: impl IntoIterator<Item = TypeId>) -> Self {
        Self(types.into_iter().collect())
    }
}

/// Storage for a single component type within an archetype
/// Components are stored in contiguous arrays for cache efficiency
pub struct ComponentArray {
    data: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
    element_size: usize,
    len: usize,
    capacity: usize,
}

impl ComponentArray {
    pub fn new<T: Component>() -> Self {
        Self {
            data: Box::new(Vec::<T>::new()),
            type_id: TypeId::of::<T>(),
            element_size: std::mem::size_of::<T>(),
            len: 0,
            capacity: 0,
        }
    }
    
    pub fn push<T: Component>(&mut self, component: T) {
        if let Some(vec) = self.data.downcast_mut::<Vec<T>>() {
            vec.push(component);
            self.len = vec.len();
            self.capacity = vec.capacity();
        }
    }
    
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        if let Some(vec) = self.data.downcast_ref::<Vec<T>>() {
            vec.get(index)
        } else {
            None
        }
    }
    
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        if let Some(vec) = self.data.downcast_mut::<Vec<T>>() {
            vec.get_mut(index)
        } else {
            None
        }
    }
    
    pub fn swap_remove(&mut self, index: usize) {
        // Implement type-erased swap_remove
        // This is complex but necessary for efficient entity removal
        match self.element_size {
            size if size == std::mem::size_of::<u8>() => {
                if let Some(vec) = self.data.downcast_mut::<Vec<u8>>() {
                    if index < vec.len() { vec.swap_remove(index); }
                }
            }
            size if size == std::mem::size_of::<u32>() => {
                if let Some(vec) = self.data.downcast_mut::<Vec<u32>>() {
                    if index < vec.len() { vec.swap_remove(index); }
                }
            }
            // Add more sizes as needed, or use unsafe code for general case
            _ => {
                // For now, just mark as removed (more complex swap_remove later)
                self.len = self.len.saturating_sub(1);
            }
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn as_slice<T: Component>(&self) -> Option<&[T]> {
        if let Some(vec) = self.data.downcast_ref::<Vec<T>>() {
            Some(vec.as_slice())
        } else {
            None
        }
    }
    
    pub fn as_mut_slice<T: Component>(&mut self) -> Option<&mut [T]> {
        if let Some(vec) = self.data.downcast_mut::<Vec<T>>() {
            Some(vec.as_mut_slice())
        } else {
            None
        }
    }
}

/// Archetype - stores entities with the same component signature
/// All components of the same type are stored contiguously
pub struct Archetype {
    id: ArchetypeId,
    entities: Vec<Entity>,
    components: HashMap<TypeId, ComponentArray>,
}

impl Archetype {
    pub fn new(id: ArchetypeId) -> Self {
        Self {
            id,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let index = self.entities.len();
        self.entities.push(entity);
        index
    }
    
    pub fn add_component<T: Component>(&mut self, component: T) {
        let type_id = TypeId::of::<T>();
        if let Some(array) = self.components.get_mut(&type_id) {
            array.push(component);
        } else {
            let mut array = ComponentArray::new::<T>();
            array.push(component);
            self.components.insert(type_id, array);
        }
    }
    
    pub fn get_component<T: Component>(&self, index: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.get::<T>(index)
    }
    
    pub fn get_component_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)?.get_mut::<T>(index)
    }
    
    pub fn get_component_array<T: Component>(&self) -> Option<&[T]> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.as_slice::<T>()
    }
    
    pub fn get_component_array_mut<T: Component>(&mut self) -> Option<&mut [T]> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)?.as_mut_slice::<T>()
    }
    
    pub fn has_component<T: Component>(&self) -> bool {
        self.id.has_component::<T>()
    }
    
    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index >= self.entities.len() {
            return None;
        }
        
        let entity = self.entities.swap_remove(index);
        
        // Remove components at the same index
        for array in self.components.values_mut() {
            array.swap_remove(index);
        }
        
        Some(entity)
    }
    
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

/// Entity location within an archetype
#[derive(Debug, Clone)]
pub struct EntityLocation {
    archetype_id: ArchetypeId,
    index: usize,
}

/// Data-Oriented World - stores entities in archetypes for cache efficiency
pub struct World {
    next_entity_id: u32,
    next_generation: u32,
    entity_locations: HashMap<Entity, EntityLocation>,
    archetypes: HashMap<ArchetypeId, Archetype>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            next_generation: 1,
            entity_locations: HashMap::new(),
            archetypes: HashMap::new(),
        }
    }
    
    /// Create a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id, self.next_generation);
        self.next_entity_id += 1;
        entity
    }
    
    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), &'static str> {
        // Determine target archetype
        let target_archetype_id = if let Some(location) = self.entity_locations.get(&entity) {
            // Entity exists, move to new archetype with additional component
            location.archetype_id.clone().with_component::<T>()
        } else {
            // New entity, create archetype with just this component
            ArchetypeId::new().with_component::<T>()
        };
        
        // If entity already exists, we need to move it to the new archetype
        if let Some(old_location) = self.entity_locations.get(&entity) {
            if old_location.archetype_id == target_archetype_id {
                // Same archetype, just add component
                if let Some(archetype) = self.archetypes.get_mut(&target_archetype_id) {
                    archetype.add_component(component);
                    return Ok(());
                }
            } else {
                // Need to move entity to new archetype
                // This is complex - for now, we'll implement simple case
                return Err("Moving entities between archetypes not yet implemented");
            }
        }
        
        // Create new archetype if it doesn't exist
        if !self.archetypes.contains_key(&target_archetype_id) {
            self.archetypes.insert(target_archetype_id.clone(), Archetype::new(target_archetype_id.clone()));
        }
        
        // Add entity and component to archetype
        let archetype = self.archetypes.get_mut(&target_archetype_id).unwrap();
        let index = archetype.add_entity(entity);
        archetype.add_component(component);
        
        // Update entity location
        self.entity_locations.insert(entity, EntityLocation {
            archetype_id: target_archetype_id,
            index,
        });
        
        Ok(())
    }
    
    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let location = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get(&location.archetype_id)?;
        archetype.get_component::<T>(location.index)
    }
    
    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let location = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get_mut(&location.archetype_id)?;
        archetype.get_component_mut::<T>(location.index)
    }
    
    /// Query for entities with a specific component type
    /// Returns iterator over (Entity, &Component)
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.archetypes.values()
            .filter(|archetype| archetype.has_component::<T>())
            .flat_map(|archetype| {
                archetype.entities().iter().zip(
                    archetype.get_component_array::<T>().unwrap_or(&[])
                ).map(|(entity, component)| (*entity, component))
            })
    }
    
    /// Parallel query for high-performance iteration
    pub fn par_query<T: Component + Sync>(&self) -> impl ParallelIterator<Item = (Entity, &T)> {
        self.archetypes.par_iter()
            .filter(|(_, archetype)| archetype.has_component::<T>())
            .flat_map(|(_, archetype)| {
                archetype.entities().par_iter().zip(
                    archetype.get_component_array::<T>().unwrap_or(&[]).par_iter()
                ).map(|(entity, component)| (*entity, component))
            })
    }
    
    /// Get all archetypes that contain a specific component
    pub fn archetypes_with<T: Component>(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.values()
            .filter(|archetype| archetype.has_component::<T>())
    }
    
    /// Get all archetypes that contain a specific component (mutable)
    pub fn archetypes_with_mut<T: Component>(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.values_mut()
            .filter(|archetype| archetype.has_component::<T>())
    }
    
    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entity_locations.remove(&entity) {
            if let Some(archetype) = self.archetypes.get_mut(&location.archetype_id) {
                archetype.remove_entity(location.index);
                return true;
            }
        }
        false
    }
    
    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entity_locations.len()
    }
    
    /// Get archetype count (for debugging)
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
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
    use crate::Transform;
    
    #[derive(Debug, Clone, PartialEq)]
    struct TestComponent {
        value: i32,
    }
    impl Component for TestComponent {}
    
    #[test]
    fn test_archetype_creation() {
        let archetype_id = ArchetypeId::new()
            .with_component::<Transform>()
            .with_component::<TestComponent>();
        
        assert!(archetype_id.has_component::<Transform>());
        assert!(archetype_id.has_component::<TestComponent>());
        // Test with a component type that wasn't added
        #[derive(Debug)]
        struct UnusedComponent;
        impl Component for UnusedComponent {}
        
        assert!(!archetype_id.has_component::<UnusedComponent>());
    }
    
    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = world.spawn();
        
        assert_eq!(entity.id(), 1);
        assert_eq!(entity.generation(), 1);
    }
    
    #[test]
    fn test_component_storage() {
        let mut world = World::new();
        let entity = world.spawn();
        
        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        world.add_component(entity, transform.clone()).unwrap();
        
        let retrieved = world.get_component::<Transform>(entity);
        assert_eq!(retrieved, Some(&transform));
    }
    
    #[test]
    fn test_query_iteration() {
        let mut world = World::new();
        
        // Create entities with components
        for i in 0..5 {
            let entity = world.spawn();
            world.add_component(entity, TestComponent { value: i }).unwrap();
        }
        
        // Query all TestComponents
        let components: Vec<_> = world.query::<TestComponent>()
            .map(|(_, component)| component.value)
            .collect();
        
        assert_eq!(components.len(), 5);
        assert!(components.contains(&0));
        assert!(components.contains(&4));
    }
    
    #[test]
    fn test_archetype_efficiency() {
        let mut world = World::new();
        
        // Create 500 entities with Transform only
        for _i in 0..500 {
            let entity = world.spawn();
            world.add_component(entity, Transform::default()).unwrap();
        }
        
        // Create 500 entities with TestComponent only  
        for i in 0..500 {
            let entity = world.spawn();
            world.add_component(entity, TestComponent { value: i }).unwrap();
        }
        
        // Should have created 2 archetypes (Transform, TestComponent)
        assert_eq!(world.archetype_count(), 2);
        assert_eq!(world.entity_count(), 1000);
    }
}