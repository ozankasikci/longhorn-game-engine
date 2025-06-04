//! World container for ECS V2
//! 
//! The World is the main container for entities and their components, organizing them
//! into archetypes for cache-efficient access.

use std::collections::HashMap;
use std::any::TypeId;
use rayon::prelude::*;
use engine_component_traits::{Component, ComponentClone, ComponentTicks, Tick};
use crate::error::{EcsError, EcsResult};
use crate::ecs_v2::{
    entity::{Entity, EntityLocation},
    archetype::{Archetype, ArchetypeId},
};

/// Data-Oriented World - stores entities in archetypes for cache efficiency
/// 
/// The World manages:
/// - Entity allocation and lifecycle
/// - Component storage in archetypes
/// - Efficient queries across entities
/// - Change tracking for components
pub struct World {
    next_entity_id: u32,
    next_generation: u32,
    entity_locations: HashMap<Entity, EntityLocation>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    change_tick: u32,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            next_generation: 1,
            entity_locations: HashMap::new(),
            archetypes: HashMap::new(),
            change_tick: 1,
        }
    }
    
    /// Get the number of entities in the world
    pub fn entity_count(&self) -> usize {
        self.entity_locations.len()
    }
    
    /// Get mutable access to archetypes (for bundle system)
    pub fn archetypes_mut(&mut self) -> &mut HashMap<ArchetypeId, Archetype> {
        &mut self.archetypes
    }
    
    /// Get the current change tick
    pub fn change_tick(&self) -> Tick {
        Tick::new(self.change_tick)
    }
    
    /// Increment the change tick (called once per frame/system run)
    pub fn increment_change_tick(&mut self) {
        self.change_tick = self.change_tick.wrapping_add(1);
    }
    
    /// Create a new entity with no components
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id, self.next_generation);
        self.next_entity_id += 1;
        entity
    }
    
    /// Check if an entity exists in the world
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_locations.contains_key(&entity)
    }
    
    /// Despawn an entity and all its components
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entity_locations.remove(&entity) {
            if let Some(archetype) = self.archetypes.get_mut(&location.archetype_id) {
                // Handle entity swapping during removal
                if location.index < archetype.len() - 1 {
                    // The last entity will be swapped into this position
                    if let Some(swapped_entity) = archetype.entities().last().copied() {
                        if swapped_entity != entity {
                            if let Some(swapped_location) = self.entity_locations.get_mut(&swapped_entity) {
                                swapped_location.index = location.index;
                            }
                        }
                    }
                }
                archetype.remove_entity(location.index);
                
                // Remove empty archetypes
                if archetype.is_empty() {
                    self.archetypes.remove(&location.archetype_id);
                }
            }
            true
        } else {
            false
        }
    }
    
    /// Ensure an archetype exists in the world
    fn ensure_archetype_exists(&mut self, archetype_id: ArchetypeId) {
        if !self.archetypes.contains_key(&archetype_id) {
            let mut archetype = Archetype::new(archetype_id.clone());
            let _ = archetype.initialize_components(); // Ignore errors for now
            self.archetypes.insert(archetype_id, archetype);
        }
    }
    
    /// Clone all components from an entity in an archetype
    fn clone_entity_components(
        &self,
        entity: Entity,
        location: &EntityLocation
    ) -> EcsResult<Vec<(TypeId, Box<dyn ComponentClone>, ComponentTicks)>> {
        let archetype = self.archetypes.get(&location.archetype_id)
            .ok_or(EcsError::ArchetypeNotFound)?;
            
        let mut components = Vec::new();
        
        // Clone each component from the archetype
        for type_id in location.archetype_id.type_ids() {
            if let Some(component) = archetype.clone_component_at(*type_id, location.index)? {
                let ticks = archetype.get_component_ticks_at(*type_id, location.index)
                    .ok_or(EcsError::ComponentNotFound {
                        entity,
                        component_type: "ComponentTicks",
                    })?;
                components.push((*type_id, component, ticks));
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
    ) -> EcsResult<()> {
        // 1. Clone all existing components from old archetype
        let components_to_migrate = self.clone_entity_components(entity, &old_location)?;
        
        // 2. Remove entity from old archetype and update swapped entity location
        if let Some(old_archetype) = self.archetypes.get_mut(&old_location.archetype_id) {
            // If there's an entity that will be swapped into the removed position, track it
            let swapped_entity = if old_location.index < old_archetype.entities().len() - 1 {
                // The last entity will be swapped into this position
                Some(old_archetype.entities()[old_archetype.entities().len() - 1])
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
            
            // Remove empty archetype
            if old_archetype.is_empty() {
                self.archetypes.remove(&old_location.archetype_id);
            }
        }
        
        // 3. Create target archetype if it doesn't exist
        self.ensure_archetype_exists(target_archetype_id.clone());
        
        // 4. Add entity to new archetype and update location
        let new_index = {
            let target_archetype = self.archetypes.get_mut(&target_archetype_id)
                .ok_or(EcsError::ArchetypeNotFound)?;
            target_archetype.add_entity(entity)
        };
        
        // Update entity location
        self.entity_locations.insert(entity, EntityLocation::new(
            target_archetype_id.clone(),
            new_index,
        ));
        
        // 5. Add all cloned components to new archetype
        let target_archetype = self.archetypes.get_mut(&target_archetype_id)
            .ok_or(EcsError::ArchetypeNotFound)?;
            
        for (type_id, component, ticks) in components_to_migrate {
            target_archetype.add_component_cloned(type_id, component, ticks)?;
        }
        
        // 6. Add the new component
        target_archetype.add_component(new_component, new_component_ticks)?;
        
        Ok(())
    }
    
    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> EcsResult<()> {
        let current_tick = self.change_tick();
        let ticks = ComponentTicks::new(current_tick);
        
        // Determine target archetype
        let target_archetype_id = if let Some(location) = self.entity_locations.get(&entity) {
            // Entity exists, move to new archetype with additional component
            location.archetype_id().clone().with_component::<T>()
        } else {
            // New entity, create archetype with just this component
            ArchetypeId::new().with_component::<T>()
        };
        
        // If entity already exists, we need to move it to the new archetype
        if let Some(old_location) = self.entity_locations.get(&entity).cloned() {
            if old_location.archetype_id() == &target_archetype_id {
                // Same archetype, component already exists - update it
                if let Some(archetype) = self.archetypes.get_mut(&target_archetype_id) {
                    if let Some(comp) = archetype.get_component_mut::<T>(old_location.index()) {
                        *comp = component;
                        return Ok(());
                    }
                }
            } else {
                // Need to move entity to new archetype
                return self.migrate_entity_to_new_archetype(entity, old_location, target_archetype_id, component, ticks);
            }
        }
        
        // Create new archetype if it doesn't exist
        self.ensure_archetype_exists(target_archetype_id.clone());
        
        // Add entity and component to archetype
        let archetype = self.archetypes.get_mut(&target_archetype_id).unwrap();
        let index = archetype.add_entity(entity);
        archetype.add_component(component, ticks)?;
        
        // Update entity location
        self.entity_locations.insert(entity, EntityLocation::new(
            target_archetype_id,
            index,
        ));
        
        Ok(())
    }
    
    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> EcsResult<()> {
        let location = self.entity_locations.get(&entity)
            .ok_or(EcsError::EntityNotFound(entity))?
            .clone();
            
        if !location.archetype_id().has_component::<T>() {
            return Err(EcsError::ComponentNotFound {
                entity,
                component_type: std::any::type_name::<T>(),
            });
        }
        
        // Create new archetype ID without the component
        let mut new_archetype_id = ArchetypeId::new();
        for type_id in location.archetype_id().type_ids() {
            if *type_id != TypeId::of::<T>() {
                new_archetype_id = new_archetype_id.with_type_id(*type_id);
            }
        }
        
        // Clone components except the one being removed
        let mut components_to_keep = Vec::new();
        if let Some(archetype) = self.archetypes.get(&location.archetype_id()) {
            for type_id in location.archetype_id().type_ids() {
                if *type_id != TypeId::of::<T>() {
                    if let Some(component) = archetype.clone_component_at(*type_id, location.index())? {
                        let ticks = archetype.get_component_ticks_at(*type_id, location.index())
                            .unwrap_or_else(|| ComponentTicks::new(self.change_tick()));
                        components_to_keep.push((*type_id, component, ticks));
                    }
                }
            }
        }
        
        // Remove from old archetype
        self.despawn(entity);
        
        // If there are components left, re-add entity with remaining components
        if !components_to_keep.is_empty() {
            self.ensure_archetype_exists(new_archetype_id.clone());
            
            let archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
            let index = archetype.add_entity(entity);
            
            for (type_id, component, ticks) in components_to_keep {
                archetype.add_component_cloned(type_id, component, ticks)?;
            }
            
            self.entity_locations.insert(entity, EntityLocation::new(new_archetype_id, index));
        }
        
        Ok(())
    }
    
    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let location = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get(location.archetype_id())?;
        archetype.get_component::<T>(location.index())
    }
    
    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let location = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get_mut(location.archetype_id())?;
        archetype.get_component_mut::<T>(location.index())
    }
    
    /// Check if an entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if let Some(location) = self.entity_locations.get(&entity) {
            location.archetype_id().has_component::<T>()
        } else {
            false
        }
    }
    
    /// Legacy query for entities with a specific component type
    /// Returns iterator over (Entity, &Component)
    pub fn query_legacy<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.archetypes.values()
            .filter(|archetype| archetype.has_component::<T>())
            .flat_map(|archetype| {
                let component_array = archetype.get_component_array::<T>();
                archetype.entities().iter()
                    .enumerate()
                    .filter_map(move |(idx, entity)| {
                        component_array.and_then(|arr| arr.get(idx))
                            .map(|comp| (*entity, comp))
                    })
            })
    }
    
    /// Parallel query for high-performance iteration
    pub fn par_query<T: Component + Sync>(&self) -> impl ParallelIterator<Item = (Entity, &T)> {
        self.archetypes.par_iter()
            .filter(|(_, archetype)| archetype.has_component::<T>())
            .flat_map(|(_, archetype)| {
                let entities = archetype.entities();
                let component_array = archetype.get_component_array::<T>();
                
                (0..entities.len()).into_par_iter()
                    .filter_map(move |idx| {
                        component_array.and_then(|arr| arr.get(idx))
                            .map(|comp| (entities[idx], comp))
                    })
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

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs_v2::test_utils::*;
    
    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.change_tick().get(), 1);
    }
    
    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        let e2 = world.spawn();
        
        assert_ne!(e1, e2);
        assert_eq!(e1.id(), 1);
        assert_eq!(e2.id(), 2);
        assert_eq!(e1.generation(), 1);
        assert_eq!(e2.generation(), 1);
    }
    
    #[test]
    fn test_spawn_and_add_component() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        let result = world.add_component(entity, Position::new(10.0, 20.0, 30.0));
        assert!(result.is_ok());
        
        assert!(world.contains(entity));
        assert_eq!(world.entity_count(), 1);
        
        let pos = world.get_component::<Position>(entity);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().x, 10.0);
        assert_eq!(pos.unwrap().y, 20.0);
        assert_eq!(pos.unwrap().z, 30.0);
    }
    
    #[test]
    fn test_add_multiple_components() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        world.add_component(entity, Velocity::new(4.0, 5.0, 6.0)).unwrap();
        world.add_component(entity, Health::new(100.0)).unwrap();
        
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));
        assert!(world.has_component::<Health>(entity));
        assert!(!world.has_component::<Name>(entity));
    }
    
    #[test]
    fn test_component_mutation() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        
        // Mutate component
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            pos.x = 10.0;
            pos.y = 20.0;
            pos.z = 30.0;
        }
        
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
        assert_eq!(pos.z, 30.0);
    }
    
    #[test]
    fn test_despawn_entity() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        
        assert!(world.contains(entity));
        assert_eq!(world.entity_count(), 1);
        
        let despawned = world.despawn(entity);
        assert!(despawned);
        assert!(!world.contains(entity));
        assert_eq!(world.entity_count(), 0);
        
        // Try to despawn again
        let despawned_again = world.despawn(entity);
        assert!(!despawned_again);
    }
    
    #[test]
    fn test_archetype_migration() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        // Start with Position
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        
        // Add Velocity - should migrate to new archetype
        world.add_component(entity, Velocity::new(4.0, 5.0, 6.0)).unwrap();
        
        // Both components should still be accessible
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        
        let vel = world.get_component::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 4.0);
    }
    
    #[test]
    fn test_remove_component() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        world.add_component(entity, Velocity::new(4.0, 5.0, 6.0)).unwrap();
        
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));
        
        // Remove Velocity
        let result = world.remove_component::<Velocity>(entity);
        assert!(result.is_ok());
        
        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_component::<Velocity>(entity));
        
        // Try to remove non-existent component
        let result = world.remove_component::<Health>(entity);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_query_legacy() {
        register_test_components();
        
        let mut world = World::new();
        
        // Create entities with different component combinations
        let e1 = world.spawn();
        world.add_component(e1, Position::new(1.0, 0.0, 0.0)).unwrap();
        
        let e2 = world.spawn();
        world.add_component(e2, Position::new(2.0, 0.0, 0.0)).unwrap();
        world.add_component(e2, Velocity::new(1.0, 1.0, 1.0)).unwrap();
        
        let e3 = world.spawn();
        world.add_component(e3, Position::new(3.0, 0.0, 0.0)).unwrap();
        
        let e4 = world.spawn();
        world.add_component(e4, Velocity::new(2.0, 2.0, 2.0)).unwrap();
        
        // Query all entities with Position
        let positions: Vec<_> = world.query_legacy::<Position>().collect();
        assert_eq!(positions.len(), 3);
        
        // Query all entities with Velocity
        let velocities: Vec<_> = world.query_legacy::<Velocity>().collect();
        assert_eq!(velocities.len(), 2);
    }
    
    #[test]
    fn test_change_tick_tracking() {
        let mut world = World::new();
        
        assert_eq!(world.change_tick().get(), 1);
        
        world.increment_change_tick();
        assert_eq!(world.change_tick().get(), 2);
        
        world.increment_change_tick();
        assert_eq!(world.change_tick().get(), 3);
    }
    
    #[test]
    fn test_entity_swapping_on_removal() {
        register_test_components();
        
        let mut world = World::new();
        
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        
        world.add_component(e1, Position::new(1.0, 0.0, 0.0)).unwrap();
        world.add_component(e2, Position::new(2.0, 0.0, 0.0)).unwrap();
        world.add_component(e3, Position::new(3.0, 0.0, 0.0)).unwrap();
        
        // Remove e2 - e3 should be swapped into its place
        world.despawn(e2);
        
        // e1 and e3 should still exist
        assert!(world.contains(e1));
        assert!(!world.contains(e2));
        assert!(world.contains(e3));
        
        // Check that positions are still correct
        assert_eq!(world.get_component::<Position>(e1).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Position>(e3).unwrap().x, 3.0);
    }
    
    #[test]
    fn test_add_component_to_non_existent_entity() {
        register_test_components();
        
        let mut world = World::new();
        let fake_entity = Entity::new(999, 1);
        
        let result = world.add_component(fake_entity, Position::new(1.0, 2.0, 3.0));
        assert!(result.is_ok()); // Should create the entity
        assert!(world.contains(fake_entity));
    }
    
    #[test]
    fn test_update_existing_component() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        
        // Add same component type again - should update
        world.add_component(entity, Position::new(10.0, 20.0, 30.0)).unwrap();
        
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
        assert_eq!(pos.z, 30.0);
    }
    
    #[test]
    fn test_archetype_cleanup() {
        register_test_components();
        
        let mut world = World::new();
        
        // Create entity with unique component combination
        let entity = world.spawn();
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        world.add_component(entity, Name::new("Unique")).unwrap();
        
        // Archetype should exist
        let archetype_count_before = world.archetypes.len();
        assert!(archetype_count_before > 0);
        
        // Remove entity
        world.despawn(entity);
        
        // Empty archetype should be cleaned up
        let archetype_count_after = world.archetypes.len();
        assert_eq!(archetype_count_after, archetype_count_before - 1);
    }
    
    #[test]
    fn test_zero_sized_components() {
        register_test_components();
        
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Player).unwrap();
        world.add_component(entity, Position::new(1.0, 2.0, 3.0)).unwrap();
        
        assert!(world.has_component::<Player>(entity));
        assert!(world.has_component::<Position>(entity));
        
        // Query for players
        let players: Vec<_> = world.query_legacy::<Player>().collect();
        assert_eq!(players.len(), 1);
        assert_eq!(players[0].0, entity);
    }
}