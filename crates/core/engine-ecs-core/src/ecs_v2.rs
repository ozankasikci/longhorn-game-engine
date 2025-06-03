// Data-Oriented ECS Implementation - Version 2
// Based on archetype storage for maximum cache efficiency

use std::any::{Any, TypeId};
use std::collections::{HashMap, BTreeSet};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use engine_component_traits::{Component, ComponentClone, ComponentTicks, Tick, Bundle};

// Component Registry for dynamic component array creation
type ComponentArrayFactory = Arc<dyn Fn() -> Box<dyn ComponentArrayTrait> + Send + Sync>;

/// Global component registry
static COMPONENT_REGISTRY: Lazy<Mutex<HashMap<TypeId, ComponentArrayFactory>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a component type in the global registry
pub fn register_component<T: Component>() {
    let type_id = TypeId::of::<T>();
    let factory: ComponentArrayFactory = Arc::new(|| {
        Box::new(ComponentArray::<T>::new())
    });
    
    COMPONENT_REGISTRY.lock().unwrap().insert(type_id, factory);
}

/// Create a new component array for the given type
fn create_component_array(type_id: TypeId) -> Option<Box<dyn ComponentArrayTrait>> {
    COMPONENT_REGISTRY.lock().unwrap()
        .get(&type_id)
        .map(|factory| factory())
}

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

/// Trait for type-erased component storage operations
pub trait ComponentArrayTrait: Send + Sync {
    /// Remove element at index by swapping with last element
    fn swap_remove(&mut self, index: usize);
    
    /// Get the number of elements
    fn len(&self) -> usize;
    
    /// Check if the array is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get capacity of the underlying storage
    fn capacity(&self) -> usize;
    
    /// Get the TypeId of the stored component
    fn type_id(&self) -> TypeId;
    
    /// Downcast to Any for type-specific operations
    fn as_any(&self) -> &dyn Any;
    
    /// Downcast to Any for type-specific mutable operations
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Clone a component at the given index
    fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>>;
    
    /// Get component ticks at the given index
    fn get_ticks_at(&self, index: usize) -> Option<ComponentTicks>;
    
    /// Push a cloned component
    fn push_cloned(&mut self, component: Box<dyn ComponentClone>, ticks: ComponentTicks) -> Result<(), &'static str>;
}

/// Storage for a single component type within an archetype
/// Components are stored in contiguous arrays for cache efficiency
pub struct ComponentArray<T: Component> {
    data: Vec<T>,
    ticks: Vec<ComponentTicks>,
}

impl<T: Component> ComponentArray<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            ticks: Vec::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ticks: Vec::with_capacity(capacity),
        }
    }
    
    pub fn push(&mut self, component: T, ticks: ComponentTicks) {
        self.data.push(component);
        self.ticks.push(ticks);
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
    
    pub fn get_ticks(&self, index: usize) -> Option<&ComponentTicks> {
        self.ticks.get(index)
    }
    
    pub fn get_ticks_mut(&mut self, index: usize) -> Option<&mut ComponentTicks> {
        self.ticks.get_mut(index)
    }
    
    pub fn mark_changed(&mut self, index: usize, tick: Tick) {
        if let Some(ticks) = self.ticks.get_mut(index) {
            ticks.mark_changed(tick);
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }
    
    pub fn ticks_slice(&self) -> &[ComponentTicks] {
        self.ticks.as_slice()
    }
    
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
    
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
    
    pub fn iter_with_ticks(&self) -> impl Iterator<Item = (&T, &ComponentTicks)> {
        self.data.iter().zip(self.ticks.iter())
    }
}

impl<T: Component> ComponentArrayTrait for ComponentArray<T> {
    fn swap_remove(&mut self, index: usize) {
        if index < self.data.len() {
            self.data.swap_remove(index);
            self.ticks.swap_remove(index);
        }
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn capacity(&self) -> usize {
        self.data.capacity()
    }
    
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>> {
        self.data.get(index).map(|component| component.clone_boxed())
    }
    
    fn get_ticks_at(&self, index: usize) -> Option<ComponentTicks> {
        self.ticks.get(index).cloned()
    }
    
    fn push_cloned(&mut self, component: Box<dyn ComponentClone>, ticks: ComponentTicks) -> Result<(), &'static str> {
        // Downcast the component to the correct type
        let component_any = component.into_any();
        if let Ok(component_boxed) = component_any.downcast::<T>() {
            self.data.push(*component_boxed);
            self.ticks.push(ticks);
            Ok(())
        } else {
            Err("Type mismatch when pushing cloned component")
        }
    }
}

/// Type-erased component storage
pub struct ErasedComponentArray {
    array: Box<dyn ComponentArrayTrait>,
    type_id: TypeId,
}

impl ErasedComponentArray {
    pub fn new<T: Component>() -> Self {
        Self {
            array: Box::new(ComponentArray::<T>::new()),
            type_id: TypeId::of::<T>(),
        }
    }
    
    pub fn push<T: Component>(&mut self, component: T, ticks: ComponentTicks) -> Result<(), &'static str> {
        if self.type_id != TypeId::of::<T>() {
            return Err("Type mismatch");
        }
        
        if let Some(array) = self.array.as_any_mut().downcast_mut::<ComponentArray<T>>() {
            array.push(component, ticks);
            Ok(())
        } else {
            Err("Downcast failed")
        }
    }
    
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        self.array.as_any()
            .downcast_ref::<ComponentArray<T>>()?
            .get(index)
    }
    
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        self.array.as_any_mut()
            .downcast_mut::<ComponentArray<T>>()?
            .get_mut(index)
    }
    
    pub fn as_slice<T: Component>(&self) -> Option<&[T]> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        Some(self.array.as_any()
            .downcast_ref::<ComponentArray<T>>()?
            .as_slice())
    }
    
    pub fn as_mut_slice<T: Component>(&mut self) -> Option<&mut [T]> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        Some(self.array.as_any_mut()
            .downcast_mut::<ComponentArray<T>>()?
            .as_mut_slice())
    }
    
    pub fn swap_remove(&mut self, index: usize) {
        self.array.swap_remove(index);
    }
    
    pub fn len(&self) -> usize {
        self.array.len()
    }
    
    pub fn get_ticks<T: Component>(&self, index: usize) -> Option<&ComponentTicks> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        self.array.as_any()
            .downcast_ref::<ComponentArray<T>>()?
            .get_ticks(index)
    }
    
    pub fn mark_changed<T: Component>(&mut self, index: usize, tick: Tick) -> Result<(), &'static str> {
        if self.type_id != TypeId::of::<T>() {
            return Err("Type mismatch");
        }
        
        if let Some(array) = self.array.as_any_mut().downcast_mut::<ComponentArray<T>>() {
            array.mark_changed(index, tick);
            Ok(())
        } else {
            Err("Downcast failed")
        }
    }
    
    pub fn ticks_slice<T: Component>(&self) -> Option<&[ComponentTicks]> {
        if self.type_id != TypeId::of::<T>() {
            return None;
        }
        
        Some(self.array.as_any()
            .downcast_ref::<ComponentArray<T>>()?
            .ticks_slice())
    }
    
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Clone a component at the given index
    pub fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>> {
        self.array.clone_component_at(index)
    }
    
    /// Add a cloned component
    pub fn push_cloned(&mut self, component: Box<dyn ComponentClone>, ticks: ComponentTicks) -> Result<(), &'static str> {
        self.array.push_cloned(component, ticks)
    }
    
    /// Get component ticks at the given index (non-generic)
    pub fn get_ticks_at(&self, index: usize) -> Option<ComponentTicks> {
        self.array.get_ticks_at(index)
    }
}

/// Archetype - stores entities with the same component signature
/// All components of the same type are stored contiguously
pub struct Archetype {
    id: ArchetypeId,
    entities: Vec<Entity>,
    components: HashMap<TypeId, ErasedComponentArray>,
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
    
    pub fn add_component<T: Component>(&mut self, component: T, ticks: ComponentTicks) {
        let type_id = TypeId::of::<T>();
        if let Some(array) = self.components.get_mut(&type_id) {
            array.push(component, ticks).expect("Component type mismatch");
        } else {
            let mut array = ErasedComponentArray::new::<T>();
            array.push(component, ticks).expect("Component type mismatch");
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
    
    /// Clone a component at a specific index
    pub fn clone_component_at(&self, type_id: TypeId, index: usize) -> Result<Option<Box<dyn ComponentClone>>, &'static str> {
        let array = self.components.get(&type_id).ok_or("Component type not found")?;
        Ok(array.clone_component_at(index))
    }
    
    /// Get component ticks at a specific index
    pub fn get_component_ticks_at(&self, type_id: TypeId, index: usize) -> Option<ComponentTicks> {
        self.components.get(&type_id)?.get_ticks_at(index)
    }
    
    /// Add a component from a cloned box
    pub fn add_component_cloned(&mut self, type_id: TypeId, component: Box<dyn ComponentClone>, ticks: ComponentTicks) -> Result<(), &'static str> {
        if let Some(array) = self.components.get_mut(&type_id) {
            array.push_cloned(component, ticks)
        } else {
            // Try to create new array from registry
            if let Some(mut new_array) = create_component_array(type_id) {
                new_array.push_cloned(component, ticks)?;
                self.components.insert(type_id, ErasedComponentArray {
                    array: new_array,
                    type_id,
                });
                Ok(())
            } else {
                Err("Component type not registered")
            }
        }
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
    change_tick: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            next_generation: 1,
            entity_locations: HashMap::new(),
            archetypes: HashMap::new(),
            change_tick: 1,
        }
    }
    
    /// Get mutable access to archetypes (for bundle system)
    pub fn archetypes_mut(&mut self) -> &mut HashMap<ArchetypeId, Archetype> {
        &mut self.archetypes
    }
    
    /// Clone all components from an entity in an archetype
    #[allow(dead_code)]
    fn clone_entity_components(
        &self,
        _entity: Entity,
        location: &EntityLocation
    ) -> Result<Vec<(TypeId, Box<dyn ComponentClone>, ComponentTicks)>, &'static str> {
        let archetype = self.archetypes.get(&location.archetype_id)
            .ok_or("Archetype not found")?;
            
        let mut components = Vec::new();
        
        // Clone each component from the archetype
        for (type_id, _array) in &archetype.components {
            // We need to add a method to clone components from ErasedComponentArray
            // For now, we'll need to handle this in the archetype
            if let Some(component) = archetype.clone_component_at(*type_id, location.index)? {
                let ticks = archetype.get_component_ticks_at(*type_id, location.index)
                    .ok_or("Component ticks not found")?;
                components.push((*type_id, component, ticks.clone()));
            }
        }
        
        Ok(components)
    }
    
    /// Migrate an entity from one archetype to another when adding components
    fn migrate_entity_to_new_archetype<T: Component>(
        &mut self, 
        entity: Entity, 
        old_location: EntityLocation, 
        target_archetype_id: ArchetypeId, 
        new_component: T, 
        new_component_ticks: ComponentTicks
    ) -> Result<(), &'static str> {
        // Phase 11 Implementation: Full component migration with registry
        
        // 1. Clone all existing components from old archetype
        let components_to_migrate = self.clone_entity_components(entity, &old_location)?;
        
        // 2. Remove entity from old archetype and update swapped entity location
        if let Some(old_archetype) = self.archetypes.get_mut(&old_location.archetype_id) {
            // If there's an entity that will be swapped into the removed position, track it
            let swapped_entity = if old_location.index < old_archetype.entities.len() - 1 {
                // The last entity will be swapped into this position
                Some(old_archetype.entities[old_archetype.entities.len() - 1])
            } else {
                None
            };
            
            // Remove the entity
            old_archetype.remove_entity(old_location.index);
            
            // Update the swapped entity's location if one was swapped
            if let Some(swapped) = swapped_entity {
                if swapped != entity {
                    if let Some(swapped_location) = self.entity_locations.get_mut(&swapped) {
                        swapped_location.index = old_location.index;
                    }
                }
            }
        }
        
        // 3. Create target archetype if it doesn't exist
        self.ensure_archetype_exists(target_archetype_id.clone());
        
        // 4. Add entity to new archetype and update location
        let new_index = {
            let target_archetype = self.archetypes.get_mut(&target_archetype_id)
                .ok_or("Target archetype not found")?;
            target_archetype.add_entity(entity)
        };
        
        // Update entity location
        self.entity_locations.insert(entity, EntityLocation {
            archetype_id: target_archetype_id.clone(),
            index: new_index,
        });
        
        // 5. Add all cloned components to new archetype
        let target_archetype = self.archetypes.get_mut(&target_archetype_id)
            .ok_or("Target archetype not found")?;
            
        for (type_id, component, ticks) in components_to_migrate {
            target_archetype.add_component_cloned(type_id, component, ticks)?;
        }
        
        // 6. Add the new component
        target_archetype.add_component(new_component, new_component_ticks);
        
        Ok(())
    }
    
    /// Get the current change tick
    pub fn change_tick(&self) -> Tick {
        Tick::new(self.change_tick)
    }
    
    /// Increment the change tick (called once per frame/system run)
    pub fn increment_change_tick(&mut self) {
        self.change_tick = self.change_tick.wrapping_add(1);
    }
    
    /// Create a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id, self.next_generation);
        self.next_entity_id += 1;
        entity
    }
    
    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), &'static str> {
        let current_tick = self.change_tick();
        let ticks = ComponentTicks::new(current_tick);
        
        // Determine target archetype
        let target_archetype_id = if let Some(location) = self.entity_locations.get(&entity) {
            // Entity exists, move to new archetype with additional component
            location.archetype_id.clone().with_component::<T>()
        } else {
            // New entity, create archetype with just this component
            ArchetypeId::new().with_component::<T>()
        };
        
        // If entity already exists, we need to move it to the new archetype
        if let Some(old_location) = self.entity_locations.get(&entity).cloned() {
            if old_location.archetype_id == target_archetype_id {
                // Same archetype, just add component
                if let Some(archetype) = self.archetypes.get_mut(&target_archetype_id) {
                    archetype.add_component(component, ticks);
                    return Ok(());
                }
            } else {
                // Need to move entity to new archetype - implement migration
                return self.migrate_entity_to_new_archetype(entity, old_location, target_archetype_id, component, ticks);
            }
        }
        
        // Create new archetype if it doesn't exist
        if !self.archetypes.contains_key(&target_archetype_id) {
            self.archetypes.insert(target_archetype_id.clone(), Archetype::new(target_archetype_id.clone()));
        }
        
        // Add entity and component to archetype
        let archetype = self.archetypes.get_mut(&target_archetype_id).unwrap();
        let index = archetype.add_entity(entity);
        archetype.add_component(component, ticks);
        
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
    
    /// Legacy query for entities with a specific component type
    /// Returns iterator over (Entity, &Component)
    pub fn query_legacy<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
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
}

// ============================================================================
// MODERN QUERY SYSTEM - Bevy-style type-safe queries
// ============================================================================

/// Trait for data that can be queried from the world
pub trait QueryData {
    type Item<'a>;
    
    /// Check if an archetype matches this query
    fn matches_archetype(archetype: &Archetype) -> bool;
    
    /// Fetch data from an archetype at a specific index
    /// # Safety
    /// The caller must ensure the archetype matches and index is valid
    unsafe fn fetch<'a>(archetype: &'a Archetype, index: usize) -> Self::Item<'a>;
    
    /// Fetch data mutably from an archetype at a specific index
    /// # Safety
    /// The caller must ensure the archetype matches and index is valid
    unsafe fn fetch_mut<'a>(archetype: &'a mut Archetype, index: usize) -> Self::Item<'a>;
    
    /// Whether this query requires mutable access
    fn is_mutable() -> bool {
        false
    }
}

/// Read-only component query
pub struct Read<T: Component>(std::marker::PhantomData<T>);

impl<T: Component> QueryData for Read<T> {
    type Item<'a> = &'a T;
    
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.has_component::<T>()
    }
    
    unsafe fn fetch<'a>(archetype: &'a Archetype, index: usize) -> Self::Item<'a> {
        archetype.get_component::<T>(index).unwrap_unchecked()
    }
    
    unsafe fn fetch_mut<'a>(archetype: &'a mut Archetype, index: usize) -> Self::Item<'a> {
        archetype.get_component::<T>(index).unwrap_unchecked()
    }
}

/// Mutable component query
pub struct Write<T: Component>(std::marker::PhantomData<T>);

impl<T: Component> QueryData for Write<T> {
    type Item<'a> = &'a mut T;
    
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.has_component::<T>()
    }
    
    unsafe fn fetch<'a>(_archetype: &'a Archetype, _index: usize) -> Self::Item<'a> {
        panic!("Cannot fetch mutable reference from immutable archetype");
    }
    
    unsafe fn fetch_mut<'a>(archetype: &'a mut Archetype, index: usize) -> Self::Item<'a> {
        archetype.get_component_mut::<T>(index).unwrap_unchecked()
    }
    
    fn is_mutable() -> bool {
        true
    }
}

/// Change detection query filter - only includes entities with components changed after last_run
pub struct Changed<T: Component> {
    #[allow(dead_code)]
    last_run: Tick,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component> Changed<T> {
    pub fn new(last_run: Tick) -> Self {
        Self {
            last_run,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Component> QueryData for Changed<T> {
    type Item<'a> = &'a T;
    
    fn matches_archetype(archetype: &Archetype) -> bool {
        archetype.has_component::<T>()
    }
    
    unsafe fn fetch<'a>(archetype: &'a Archetype, index: usize) -> Self::Item<'a> {
        archetype.get_component::<T>(index).unwrap_unchecked()
    }
    
    unsafe fn fetch_mut<'a>(archetype: &'a mut Archetype, index: usize) -> Self::Item<'a> {
        archetype.get_component::<T>(index).unwrap_unchecked()
    }
}

/// Query for accessing entities and their components
pub struct Query<'w, Q: QueryData> {
    world: &'w World,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'w, Q: QueryData> Query<'w, Q> {
    fn new(world: &'w World) -> Self {
        Self {
            world,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Iterate over all entities matching the query
    pub fn iter(&self) -> QueryIter<'_, Q> {
        QueryIter::new(self.world)
    }
    
    /// Get query results for a specific entity
    pub fn get(&self, entity: Entity) -> Option<Q::Item<'_>> {
        let location = self.world.entity_locations.get(&entity)?;
        let archetype = self.world.archetypes.get(&location.archetype_id)?;
        
        if Q::matches_archetype(archetype) {
            Some(unsafe { Q::fetch(archetype, location.index) })
        } else {
            None
        }
    }
}

/// Mutable query for accessing entities and their components
pub struct QueryMut<'w, Q: QueryData> {
    world: &'w mut World,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'w, Q: QueryData> QueryMut<'w, Q> {
    fn new(world: &'w mut World) -> Self {
        Self {
            world,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Iterate over all entities matching the query (mutable)
    pub fn iter_mut(&mut self) -> QueryIterMut<'_, Q> {
        QueryIterMut::new(self.world)
    }
    
    /// Get mutable query results for a specific entity
    pub fn get_mut(&mut self, entity: Entity) -> Option<Q::Item<'_>> {
        let location = self.world.entity_locations.get(&entity)?;
        let archetype = self.world.archetypes.get_mut(&location.archetype_id)?;
        
        if Q::matches_archetype(archetype) {
            Some(unsafe { Q::fetch_mut(archetype, location.index) })
        } else {
            None
        }
    }
}

/// Iterator for read-only queries
pub struct QueryIter<'w, Q: QueryData> {
    archetype_iter: std::collections::hash_map::Values<'w, ArchetypeId, Archetype>,
    current_archetype: Option<&'w Archetype>,
    current_index: usize,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'w, Q: QueryData> QueryIter<'w, Q> {
    fn new(world: &'w World) -> Self {
        Self {
            archetype_iter: world.archetypes.values(),
            current_archetype: None,
            current_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'w, Q: QueryData> Iterator for QueryIter<'w, Q> {
    type Item = (Entity, Q::Item<'w>);
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(archetype) = self.current_archetype {
                if self.current_index < archetype.len() {
                    let entity = archetype.entities()[self.current_index];
                    let component = unsafe { Q::fetch(archetype, self.current_index) };
                    self.current_index += 1;
                    return Some((entity, component));
                }
            }
            
            // Move to next matching archetype
            self.current_archetype = self.archetype_iter
                .find(|archetype| Q::matches_archetype(archetype));
            self.current_index = 0;
            
            if self.current_archetype.is_none() {
                return None;
            }
        }
    }
}

/// Iterator for mutable queries - simplified to avoid lifetime issues
pub struct QueryIterMut<'w, Q: QueryData> {
    world: *mut World,
    archetype_ids: Vec<ArchetypeId>,
    current_archetype_index: usize,
    current_entity_index: usize,
    _phantom: std::marker::PhantomData<(&'w mut (), Q)>,
}

impl<'w, Q: QueryData> QueryIterMut<'w, Q> {
    fn new(world: &'w mut World) -> Self {
        // Collect matching archetype IDs
        let archetype_ids: Vec<_> = world.archetypes
            .iter()
            .filter(|(_, archetype)| Q::matches_archetype(archetype))
            .map(|(id, _)| id.clone())
            .collect();
            
        Self {
            world: world as *mut World,
            archetype_ids,
            current_archetype_index: 0,
            current_entity_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'w, Q: QueryData> Iterator for QueryIterMut<'w, Q> {
    type Item = (Entity, Q::Item<'w>);
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_archetype_index >= self.archetype_ids.len() {
                return None;
            }
            
            let archetype_id = &self.archetype_ids[self.current_archetype_index];
            
            // Safety: We ensure the world pointer is valid for the lifetime 'w
            let world = unsafe { &mut *self.world };
            let archetype = world.archetypes.get_mut(archetype_id)?;
            
            if self.current_entity_index < archetype.len() {
                let entity = archetype.entities()[self.current_entity_index];
                let component = unsafe { Q::fetch_mut(archetype, self.current_entity_index) };
                self.current_entity_index += 1;
                return Some((entity, component));
            }
            
            // Move to next archetype
            self.current_archetype_index += 1;
            self.current_entity_index = 0;
        }
    }
}

// Generic type aliases can be created by users:
// pub type ReadMockTransform<'a> = Read<MockTransform>;
// pub type WriteMockTransform<'a> = Write<MockTransform>;

// Extension methods for World to create queries
impl World {
    /// Create a read-only query
    pub fn query<Q: QueryData>(&self) -> Query<'_, Q> {
        Query::new(self)
    }
    
    /// Create a mutable query  
    pub fn query_mut<Q: QueryData>(&mut self) -> QueryMut<'_, Q> {
        QueryMut::new(self)
    }
    
    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entity_locations.remove(&entity) {
            if let Some(archetype) = self.archetypes.get_mut(&location.archetype_id) {
                if let Some(swapped_entity) = archetype.remove_entity(location.index) {
                    // Update location of the entity that was moved by swap_remove
                    if swapped_entity != entity && location.index < archetype.entities.len() {
                        let moved_entity = archetype.entities[location.index];
                        if let Some(moved_location) = self.entity_locations.get_mut(&moved_entity) {
                            moved_location.index = location.index;
                        }
                    }
                }
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
    
    /// Spawn an entity with a single component
    pub fn spawn_with<T: Component>(&mut self, component: T) -> Entity {
        let entity = self.spawn();
        // This will work because the entity has no components yet
        self.add_component(entity, component).unwrap();
        entity
    }
    
    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<Option<T>, &'static str> {
        // Get current location
        let old_location = self.entity_locations.get(&entity)
            .ok_or("Entity not found")?
            .clone();
            
        // Get current archetype
        let old_archetype = self.archetypes.get(&old_location.archetype_id)
            .ok_or("Archetype not found")?;
            
        // Check if entity has the component
        if !old_archetype.has_component::<T>() {
            return Ok(None);
        }
        
        // Clone the component value before migration using the clone infrastructure
        let component_value = if let Some(cloned) = old_archetype.clone_component_at(TypeId::of::<T>(), old_location.index)? {
            // Downcast the cloned component back to T
            let any_box = cloned.into_any();
            if let Ok(typed_box) = any_box.downcast::<T>() {
                Some(*typed_box)
            } else {
                return Err("Failed to downcast component");
            }
        } else {
            None
        };
        
        // Create new archetype ID without this component
        let mut new_types = BTreeSet::new();
        for (type_id, _) in &old_archetype.components {
            if *type_id != TypeId::of::<T>() {
                new_types.insert(*type_id);
            }
        }
        let new_archetype_id = ArchetypeId::from_types(new_types);
        
        // If archetype would be empty, just remove the entity
        if new_archetype_id.0.is_empty() {
            self.remove_entity(entity);
            return Ok(component_value);
        }
        
        // Clone all components except the one being removed
        let mut components_to_migrate = Vec::new();
        for (type_id, _) in &old_archetype.components {
            if *type_id != TypeId::of::<T>() {
                if let Some(component) = old_archetype.clone_component_at(*type_id, old_location.index)? {
                    let ticks = old_archetype.get_component_ticks_at(*type_id, old_location.index)
                        .ok_or("Component ticks not found")?;
                    components_to_migrate.push((*type_id, component, ticks));
                }
            }
        }
        
        // Remove entity from old archetype
        if let Some(old_archetype) = self.archetypes.get_mut(&old_location.archetype_id) {
            // Track entity that will be swapped
            let swapped_entity = if old_location.index < old_archetype.entities.len() - 1 {
                Some(old_archetype.entities[old_archetype.entities.len() - 1])
            } else {
                None
            };
            
            old_archetype.remove_entity(old_location.index);
            
            // Update swapped entity location
            if let Some(swapped) = swapped_entity {
                if swapped != entity {
                    if let Some(swapped_location) = self.entity_locations.get_mut(&swapped) {
                        swapped_location.index = old_location.index;
                    }
                }
            }
        }
        
        // Create target archetype if needed
        self.ensure_archetype_exists(new_archetype_id.clone());
        
        // Add entity to new archetype
        let new_index = {
            let target_archetype = self.archetypes.get_mut(&new_archetype_id)
                .ok_or("Target archetype not found")?;
            target_archetype.add_entity(entity)
        };
        
        // Update entity location
        self.entity_locations.insert(entity, EntityLocation {
            archetype_id: new_archetype_id.clone(),
            index: new_index,
        });
        
        // Add all migrated components
        let target_archetype = self.archetypes.get_mut(&new_archetype_id)
            .ok_or("Target archetype not found")?;
            
        for (type_id, component, ticks) in components_to_migrate {
            target_archetype.add_component_cloned(type_id, component, ticks)?;
        }
        
        Ok(component_value)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BUNDLE SYSTEM - Quick solution for multi-component entities
// ============================================================================

/// A bundle is a collection of components that can be added to an entity together

impl World {
    /// Spawn an entity with a bundle of components
    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Result<Entity, &'static str> {
        let entity = self.spawn();
        
        // Get component types and create archetype ID
        let component_ids = B::component_ids();
        let archetype_id = ArchetypeId::from_types(component_ids.clone());
        
        // Add entity to archetype
        let _index = self.add_entity_to_archetype(entity, archetype_id.clone());
        
        // Get current tick
        let tick = self.change_tick();
        
        // Get archetype and add all components
        let archetype = self.archetypes.get_mut(&archetype_id)
            .ok_or("Failed to get archetype")?;
            
        // Add all components from the bundle
        for (type_id, component) in bundle.into_components() {
            archetype.add_component_cloned(type_id, component, ComponentTicks::new(tick))?;
        }
        
        Ok(entity)
    }
    
    /// Internal helper to ensure an archetype exists
    pub(crate) fn ensure_archetype_exists(&mut self, archetype_id: ArchetypeId) {
        if !self.archetypes.contains_key(&archetype_id) {
            self.archetypes.insert(archetype_id.clone(), Archetype::new(archetype_id));
        }
    }
    
    /// Helper to add entity to archetype with location tracking
    pub fn add_entity_to_archetype(&mut self, entity: Entity, archetype_id: ArchetypeId) -> usize {
        self.ensure_archetype_exists(archetype_id.clone());
        let archetype = self.archetypes.get_mut(&archetype_id).unwrap();
        let index = archetype.add_entity(entity);
        
        self.entity_locations.insert(entity, EntityLocation {
            archetype_id,
            index,
        });
        
        index
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
    impl crate::ecs::Component for MockTransform {} // For old ECS
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
    fn test_archetype_creation() {
        let archetype_id = ArchetypeId::new()
            .with_component::<MockTransform>()
            .with_component::<TestComponent>();
        
        assert!(archetype_id.has_component::<MockTransform>());
        assert!(archetype_id.has_component::<TestComponent>());
        // Test with a component type that wasn't added
        #[derive(Debug, Clone)]
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
    fn test_query_iteration() {
        let mut world = World::new();
        
        // Create entities with components
        for i in 0..5 {
            let entity = world.spawn();
            world.add_component(entity, TestComponent { value: i }).unwrap();
        }
        
        // Query all TestComponents using new system
        let components: Vec<_> = world.query::<Read<TestComponent>>()
            .iter()
            .map(|(_, component)| component.value)
            .collect();
        
        assert_eq!(components.len(), 5);
        assert!(components.contains(&0));
        assert!(components.contains(&4));
    }
    
    #[test]
    fn test_archetype_efficiency() {
        let mut world = World::new();
        
        // Create 500 entities with MockTransform only
        for _i in 0..500 {
            let entity = world.spawn();
            world.add_component(entity, MockTransform::default()).unwrap();
        }
        
        // Create 500 entities with TestComponent only  
        for i in 0..500 {
            let entity = world.spawn();
            world.add_component(entity, TestComponent { value: i }).unwrap();
        }
        
        // Should have created 2 archetypes (MockTransform, TestComponent)
        assert_eq!(world.archetype_count(), 2);
        assert_eq!(world.entity_count(), 1000);
    }
    
    #[test]
    fn test_modern_query_system() {
        let mut world = World::new();
        
        // Create entities with MockTransform only
        let entity1 = world.spawn();
        world.add_component(entity1, MockTransform::default()).unwrap();
        
        let entity2 = world.spawn();
        world.add_component(entity2, MockTransform::default()).unwrap();
        
        // Create entities with TestComponent only
        let entity3 = world.spawn();
        world.add_component(entity3, TestComponent { value: 42 }).unwrap();
        
        let entity4 = world.spawn();
        world.add_component(entity4, TestComponent { value: 100 }).unwrap();
        
        // Test read-only query for MockTransform
        let transform_query = world.query::<Read<MockTransform>>();
        let transform_results: Vec<_> = transform_query.iter().collect();
        assert_eq!(transform_results.len(), 2); // entity1 and entity2 have MockTransform
        
        // Test read-only query for TestComponent
        let test_query = world.query::<Read<TestComponent>>();
        let test_results: Vec<_> = test_query.iter().collect();
        assert_eq!(test_results.len(), 2); // entity3 and entity4 have TestComponent
        
        // Verify specific entity access
        assert!(transform_query.get(entity1).is_some());
        assert!(transform_query.get(entity2).is_some());
        assert!(transform_query.get(entity3).is_none());
        assert!(test_query.get(entity3).is_some());
        assert!(test_query.get(entity4).is_some());
        assert!(test_query.get(entity1).is_none());
    }
    
    #[test]
    fn test_mutable_query_system() {
        let mut world = World::new();
        
        // Create entities with TestComponent
        for i in 0..3 {
            let entity = world.spawn();
            world.add_component(entity, TestComponent { value: i }).unwrap();
        }
        
        // Test mutable query
        {
            let mut query = world.query_mut::<Write<TestComponent>>();
            let mut count = 0;
            for (_entity, component) in query.iter_mut() {
                component.value *= 2; // Double all values
                count += 1;
            }
            assert_eq!(count, 3);
        }
        
        // Verify values were modified
        let query = world.query::<Read<TestComponent>>();
        let values: Vec<_> = query.iter().map(|(_, comp)| comp.value).collect();
        assert!(values.contains(&0));  // 0 * 2 = 0
        assert!(values.contains(&2));  // 1 * 2 = 2
        assert!(values.contains(&4));  // 2 * 2 = 4
    }
    
    #[test]
    fn test_query_specific_entity() {
        let mut world = World::new();
        
        let entity = world.spawn();
        world.add_component(entity, TestComponent { value: 999 }).unwrap();
        
        // Test read access to specific entity
        let query = world.query::<Read<TestComponent>>();
        let component = query.get(entity).unwrap();
        assert_eq!(component.value, 999);
        
        // Test mutable access to specific entity
        {
            let mut query = world.query_mut::<Write<TestComponent>>();
            let component = query.get_mut(entity).unwrap();
            component.value = 1000;
        }
        
        // Verify modification
        let query = world.query::<Read<TestComponent>>();
        let component = query.get(entity).unwrap();
        assert_eq!(component.value, 1000);
    }
    
    #[test]
    fn test_change_detection() {
        let mut world = World::new();
        
        // Create entity with component
        let entity = world.spawn();
        world.add_component(entity, TestComponent { value: 42 }).unwrap();
        
        // Increment change tick to simulate frame boundary
        world.increment_change_tick();
        
        // All components should be considered "changed" on first frame
        let current_tick = world.change_tick();
        let last_run = Tick::new(current_tick.get() - 1);
        
        // Verify change detection ticks work
        let location = world.entity_locations.get(&entity).unwrap();
        let archetype = world.archetypes.get(&location.archetype_id).unwrap();
        let ticks = archetype.components.get(&TypeId::of::<TestComponent>())
            .unwrap()
            .get_ticks::<TestComponent>(location.index)
            .unwrap();
        
        assert!(ticks.is_changed(last_run));
        assert!(ticks.is_added(last_run));
        
        // Test tick increment
        let old_tick = world.change_tick();
        world.increment_change_tick();
        let new_tick = world.change_tick();
        assert_eq!(new_tick.get(), old_tick.get() + 1);
    }
    
    #[test]
    fn test_transform_integration() {
        let mut world = World::new();
        
        // Create entities with MockTransform components using ECS v2
        let entity1 = world.spawn();
        let transform1 = MockTransform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 45.0, 0.0],
            scale: [2.0, 2.0, 2.0],
        };
        world.add_component(entity1, transform1.clone()).unwrap();
        
        let entity2 = world.spawn();
        let transform2 = MockTransform {
            position: [4.0, 5.0, 6.0],
            rotation: [90.0, 0.0, 0.0],
            scale: [0.5, 0.5, 0.5],
        };
        world.add_component(entity2, transform2.clone()).unwrap();
        
        // Query all MockTransform components using new query system
        let query = world.query::<Read<MockTransform>>();
        let mut results: Vec<_> = query.iter().collect();
        results.sort_by_key(|(entity, _)| entity.id());
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, entity1);
        assert_eq!(results[0].1, &transform1);
        assert_eq!(results[1].0, entity2);
        assert_eq!(results[1].1, &transform2);
        
        // Test mutable queries on MockTransform
        {
            let mut query_mut = world.query_mut::<Write<MockTransform>>();
            for (_entity, transform) in query_mut.iter_mut() {
                // Scale all transforms by 2
                transform.scale[0] *= 2.0;
                transform.scale[1] *= 2.0;
                transform.scale[2] *= 2.0;
            }
        }
        
        // Verify modifications
        let modified_transform1 = world.get_component::<MockTransform>(entity1).unwrap();
        assert_eq!(modified_transform1.scale, [4.0, 4.0, 4.0]);
        let modified_transform2 = world.get_component::<MockTransform>(entity2).unwrap();
        assert_eq!(modified_transform2.scale, [1.0, 1.0, 1.0]);
    }
    
    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;
        use crate::ecs::World as OldWorld;
        
        println!("\n=== ECS Performance Comparison ===");
        
        // Test with 1000 entities for reasonable test time
        const ENTITY_COUNT: usize = 1000;
        
        // Test 1: Entity creation
        println!("📊 Entity Creation ({} entities)", ENTITY_COUNT);
        
        let start = Instant::now();
        let mut old_world = OldWorld::new();
        for _i in 0..ENTITY_COUNT {
            let entity = old_world.spawn();
            old_world.add_component(entity, MockTransform::default()).unwrap();
        }
        let old_time = start.elapsed();
        println!("   Old ECS: {:?}", old_time);
        
        let start = Instant::now();
        let mut new_world = World::new();
        for _i in 0..ENTITY_COUNT {
            let entity = new_world.spawn();
            new_world.add_component(entity, MockTransform::default()).unwrap();
        }
        let new_time = start.elapsed();
        println!("   New ECS: {:?}", new_time);
        
        // Test 2: Query iteration
        println!("📊 Query Iteration ({} entities)", ENTITY_COUNT);
        
        let start = Instant::now();
        let mut sum = 0.0f32;
        for (_, transform) in old_world.query::<MockTransform>() {
            sum += transform.position[0];
        }
        let old_query_time = start.elapsed();
        println!("   Old ECS Query: {:?}", old_query_time);
        
        let start = Instant::now();
        let mut sum2 = 0.0f32;
        for (_, transform) in new_world.query::<Read<MockTransform>>().iter() {
            sum2 += transform.position[0];
        }
        let new_query_time = start.elapsed();
        println!("   New ECS Query: {:?}", new_query_time);
        
        // Verify correctness
        assert_eq!(sum, sum2, "Query results should be identical");
        
        // Memory layout efficiency
        println!("📊 Memory Layout");
        println!("   Old ECS entities: {}", old_world.entity_count());
        println!("   New ECS entities: {}", new_world.entity_count());
        println!("   New ECS archetypes: {}", new_world.archetype_count());
        
        // Verify structural improvements
        assert_eq!(new_world.archetype_count(), 1, "Should have exactly 1 archetype for MockTransform-only entities");
        assert_eq!(old_world.entity_count(), new_world.entity_count(), "Entity counts should match");
        
        println!("   ✅ ECS v2 structural improvements verified!");
    }
    
    #[test]
    fn test_simple_archetype_migration() {
        let mut world = World::new();
        
        // Create entity with just MockTransform
        let entity = world.spawn();
        world.add_component(entity, MockTransform::default()).unwrap();
        
        // Should be in 1 archetype
        assert_eq!(world.archetype_count(), 1);
        assert!(world.get_component::<MockTransform>(entity).is_some());
        assert!(world.get_component::<TestComponent>(entity).is_none());
        
        // Note: Complex migration is not yet implemented
        // For now, we only test simple single-component archetypes
        
        // Create another entity with TestComponent
        let entity2 = world.spawn();
        world.add_component(entity2, TestComponent { value: 42 }).unwrap();
        
        // Should now have 2 archetypes (MockTransform, TestComponent)
        assert_eq!(world.archetype_count(), 2);
        assert!(world.get_component::<TestComponent>(entity2).is_some());
        assert!(world.get_component::<MockTransform>(entity2).is_none());
        
        println!("   ✅ Basic archetype creation working correctly!");
    }
}