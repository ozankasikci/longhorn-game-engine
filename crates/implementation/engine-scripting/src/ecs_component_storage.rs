//! ECS-based component storage to replace global SHARED_COMPONENT_STATE
//! This implements proper component storage through ECS resources

use crate::components::Transform;
use engine_ecs_core::Entity;
use std::collections::HashMap;

/// ECS Resource for component storage instead of global variable
#[derive(Debug, Default)]
pub struct ScriptComponentStorage {
    transforms: HashMap<Entity, Transform>,
    // Add other component types as needed
}

impl ScriptComponentStorage {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
        }
    }

    pub fn insert_transform(&mut self, entity: Entity, transform: Transform) {
        self.transforms.insert(entity, transform);
    }

    pub fn get_transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(&entity)
    }

    pub fn get_transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.transforms.get_mut(&entity)
    }

    pub fn remove_transform(&mut self, entity: Entity) -> Option<Transform> {
        self.transforms.remove(&entity)
    }

    pub fn has_transform(&self, entity: Entity) -> bool {
        self.transforms.contains_key(&entity)
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

/// Non-global component handler that can be passed to script engines
pub struct ScriptComponentHandler {
    storage: ScriptComponentStorage,
}

impl ScriptComponentHandler {
    pub fn new() -> Self {
        Self {
            storage: ScriptComponentStorage::new(),
        }
    }

    pub fn init_entity_transform(&mut self, entity: Entity, transform: Transform) {
        self.storage.insert_transform(entity, transform);
    }

    pub fn get_entity_transform(&self, entity: Entity) -> Option<&Transform> {
        self.storage.get_transform(entity)
    }

    pub fn update_entity_transform(&mut self, entity: Entity, transform: Transform) {
        self.storage.insert_transform(entity, transform);
    }

    pub fn has_entity_transform(&self, entity: Entity) -> bool {
        self.storage.has_transform(entity)
    }

    pub fn clear_all(&mut self) {
        self.storage.clear();
    }
}

/// Trait for script engines to use component storage without global state
pub trait ComponentProvider {
    fn set_component_transform(&mut self, entity: Entity, transform: Transform);
    fn get_component_transform(&self, entity: Entity) -> Option<&Transform>;
    fn has_component_transform(&self, entity: Entity) -> bool;
}

impl ComponentProvider for ScriptComponentHandler {
    fn set_component_transform(&mut self, entity: Entity, transform: Transform) {
        self.init_entity_transform(entity, transform);
    }

    fn get_component_transform(&self, entity: Entity) -> Option<&Transform> {
        self.get_entity_transform(entity)
    }

    fn has_component_transform(&self, entity: Entity) -> bool {
        self.has_entity_transform(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_handler_no_globals() {
        let mut handler = ScriptComponentHandler::new();
        
        let entity1 = Entity::new(1, 0);
        let entity2 = Entity::new(2, 0);
        
        let transform1 = Transform::identity();
        let transform2 = Transform::identity();
        
        // Add components
        handler.init_entity_transform(entity1, transform1.clone());
        handler.init_entity_transform(entity2, transform2);
        
        // Verify components are stored locally, not globally
        assert!(handler.has_entity_transform(entity1));
        assert!(handler.has_entity_transform(entity2));
        assert_eq!(handler.get_entity_transform(entity1).unwrap().position, [0.0, 0.0, 0.0]);
        assert_eq!(handler.get_entity_transform(entity2).unwrap().position, [0.0, 0.0, 0.0]);
        
        // Test isolation
        handler.clear_all();
        assert!(!handler.has_entity_transform(entity1));
        assert!(!handler.has_entity_transform(entity2));
    }

    #[test]
    fn test_multiple_component_handlers_isolated() {
        let mut handler1 = ScriptComponentHandler::new();
        let mut handler2 = ScriptComponentHandler::new();
        
        let entity = Entity::new(1, 0);
        let transform = Transform::identity();
        
        handler1.init_entity_transform(entity, transform.clone());
        
        // Each handler should have its own components
        assert!(handler1.has_entity_transform(entity));
        assert!(!handler2.has_entity_transform(entity));
        
        // Components should not interfere
        handler2.init_entity_transform(entity, transform);
        assert!(handler1.has_entity_transform(entity));
        assert!(handler2.has_entity_transform(entity));
        
        handler1.clear_all();
        assert!(!handler1.has_entity_transform(entity));
        assert!(handler2.has_entity_transform(entity)); // Should still exist in handler2
    }
}