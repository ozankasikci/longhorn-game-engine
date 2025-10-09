//! DEPRECATED: Legacy shared state management for Lua script ECS integration
//! 
//! ⚠️  WARNING: This module is deprecated and will be removed in a future version.
//! The new secure scripting system uses `ecs_component_storage` instead.
//! 
//! This module is kept temporarily for backward compatibility with existing tests.

use crate::components::Transform;
use engine_ecs_core::Entity;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// DEPRECATED: Global shared component state for testing
/// 
/// This is a legacy solution that has been replaced by the new secure scripting system.
/// Use `ecs_component_storage::ScriptComponentHandler` instead.
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
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
    
    pub fn remove_transform(&mut self, entity: Entity) -> Option<Transform> {
        self.transforms.remove(&entity)
    }
    
    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

/// DEPRECATED: Global shared state instance
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
static SHARED_COMPONENT_STATE: Lazy<Arc<Mutex<SharedComponentState>>> = 
    Lazy::new(|| Arc::new(Mutex::new(SharedComponentState::new())));

/// DEPRECATED: Initialize entity transform in shared state
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
pub fn init_entity_transform(entity: Entity, transform: Transform) {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.set_transform(entity, transform);
}

/// DEPRECATED: Get entity transform from shared state
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
pub fn get_entity_transform(entity: Entity) -> Option<Transform> {
    let state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.get_transform(entity)
}

/// DEPRECATED: Update entity transform in shared state
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
pub fn update_entity_transform(entity: Entity, transform: Transform) {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.set_transform(entity, transform);
}

/// DEPRECATED: Clear all shared state
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage::ScriptComponentHandler instead")]
pub fn clear_shared_state() {
    let mut state = SHARED_COMPONENT_STATE.lock().unwrap();
    state.clear();
}