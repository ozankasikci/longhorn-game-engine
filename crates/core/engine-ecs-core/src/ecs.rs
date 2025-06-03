// Legacy ECS implementation - used for performance comparison tests
// This is a simplified version for benchmarking purposes

use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::error::EcsResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub trait Component: 'static + Send + Sync {}

pub struct World {
    next_entity_id: u32,
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any + Send + Sync>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            components: HashMap::new(),
        }
    }
    
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        entity
    }
    
    pub fn add_component<T: Component + Send + Sync + 'static>(&mut self, entity: Entity, component: T) -> EcsResult<()> {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
        Ok(())
    }
    
    pub fn query<T: Component + 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|storage| {
                storage.iter().filter_map(|(entity, component)| {
                    component.downcast_ref::<T>().map(|c| (*entity, c))
                })
            })
            .into_iter()
            .flatten()
    }
    
    pub fn entity_count(&self) -> usize {
        // Count unique entities across all component storages
        let mut entities = std::collections::HashSet::new();
        for storage in self.components.values() {
            for entity in storage.keys() {
                entities.insert(*entity);
            }
        }
        entities.len()
    }
}