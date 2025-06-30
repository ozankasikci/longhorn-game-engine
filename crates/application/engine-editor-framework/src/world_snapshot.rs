use engine_ecs_core::{Entity, World};
use std::collections::HashMap;
use instant::Instant;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub mod entity_memento;
pub use entity_memento::EntityMemento;

#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub entities: HashMap<u32, EntityMemento>,
    pub timestamp: Instant,
    pub snapshot_id: Uuid,
}

impl WorldSnapshot {
    pub fn capture(world: &World) -> Self {
        let mut entities = HashMap::new();
        
        // Capture all entities that have components we can snapshot
        for (entity, _) in world.query_legacy::<engine_components_3d::Transform>() {
            let memento = EntityMemento::capture(world, entity);
            entities.insert(entity.id(), memento);
        }
        
        // Also capture entities with Name components (even if no Transform)
        for (entity, _) in world.query_legacy::<engine_components_ui::Name>() {
            if !entities.contains_key(&entity.id()) {
                let memento = EntityMemento::capture(world, entity);
                entities.insert(entity.id(), memento);
            }
        }
        
        // Also capture entities with TypeScript scripts
        for (entity, _) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            if !entities.contains_key(&entity.id()) {
                let memento = EntityMemento::capture(world, entity);
                entities.insert(entity.id(), memento);
            }
        }
        
        Self {
            entities,
            timestamp: Instant::now(),
            snapshot_id: Uuid::new_v4(),
        }
    }
    
    pub fn restore(&self, world: &mut World) -> Result<(), SnapshotError> {
        for (entity_id, memento) in &self.entities {
            // Find the entity by ID (handle generation correctly)
            let entity = self.find_entity_by_id(world, *entity_id)
                .ok_or_else(|| SnapshotError::EntityNotFound(*entity_id))?;
            
            memento.restore(world, entity)?;
        }
        
        Ok(())
    }
    
    fn find_entity_by_id(&self, world: &World, entity_id: u32) -> Option<Entity> {
        // Try to find the entity by checking for Transform components
        for (entity, _) in world.query_legacy::<engine_components_3d::Transform>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        // If not found via Transform, try Name components
        for (entity, _) in world.query_legacy::<engine_components_ui::Name>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        // If not found via Name, try TypeScriptScript components
        for (entity, _) in world.query_legacy::<engine_scripting::components::TypeScriptScript>() {
            if entity.id() == entity_id {
                return Some(entity);
            }
        }
        
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Entity with ID {0} not found")]
    EntityNotFound(u32),
    #[error("Component serialization failed: {0}")]
    SerializationError(String),
    #[error("Component deserialization failed: {0}")]
    DeserializationError(String),
}