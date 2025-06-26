//! Shared state management for Lua script ECS integration

use crate::components::Transform;
use engine_ecs_core::Entity;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global shared component state for testing
/// This is a temporary solution to enable TDD - in a real implementation,
/// we would integrate more deeply with the ECS system
#[derive(Debug, Default)]
pub struct SharedComponentState {
    transforms: HashMap<Entity, Transform>,
}

impl SharedComponentState {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
        }
    }
    
    pub fn get_transform(&self, entity: Entity) -> Option<Transform> {
        self.transforms.get(&entity).cloned()
    }
    
    pub fn set_transform(&mut self, entity: Entity, transform: Transform) {
        self.transforms.insert(entity, transform);
    }
    
    pub fn has_transform(&self, entity: Entity) -> bool {
        self.transforms.contains_key(&entity)
    }
    
    pub fn remove_transform(&mut self, entity: Entity) -> Option<Transform> {
        self.transforms.remove(&entity)
    }
}

/// Global component state instance
pub static SHARED_COMPONENT_STATE: Lazy<Arc<Mutex<SharedComponentState>>> = 
    Lazy::new(|| Arc::new(Mutex::new(SharedComponentState::new())));

/// Initialize a Transform component for an entity in shared state
pub fn init_entity_transform(entity: Entity, transform: Transform) {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.set_transform(entity, transform);
}

/// Get Transform component from shared state
pub fn get_entity_transform(entity: Entity) -> Option<Transform> {
    let state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.get_transform(entity)
}

/// Update Transform component in shared state
pub fn update_entity_transform(entity: Entity, transform: Transform) {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.set_transform(entity, transform);
}

/// Check if entity has Transform in shared state
pub fn entity_has_transform(entity: Entity) -> bool {
    let state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.has_transform(entity)
}

/// Clear all shared state (for tests)
pub fn clear_shared_state() {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    *state = SharedComponentState::new();
}