//! Entity handle with clear ownership and lifetime management

use crate::{ScriptError, api::AccessPermissions};
use engine_ecs_core::{Entity, World};
use engine_component_traits::Component;
use std::marker::PhantomData;

/// Handle to an entity with version tracking and permissions
#[derive(Debug, Clone)]
pub struct EntityHandle {
    entity_id: Entity,
    world_version: u64,
    access_permissions: AccessPermissions,
}

impl EntityHandle {
    /// Create a new entity handle
    pub fn new(entity_id: Entity, world_version: u64, access_permissions: AccessPermissions) -> Self {
        Self {
            entity_id,
            world_version,
            access_permissions,
        }
    }

    /// Check if the handle is still valid
    pub fn is_valid(&self, world: &World) -> bool {
        // For now, we'll check if entity exists
        // In a real implementation, World would have version tracking
        world.contains(self.entity_id)
    }

    /// Get the entity ID
    pub fn entity_id(&self) -> Entity {
        self.entity_id
    }

    /// Get the world version this handle was created with
    pub fn world_version(&self) -> u64 {
        self.world_version
    }

    /// Get component reference with permission check
    pub fn get_component<'a, T: Component>(&self, world: &'a World) -> Result<ComponentRef<'a, T>, ScriptError> {
        // Check validity
        if !self.is_valid(world) {
            return Err(ScriptError::InvalidEntityHandle);
        }

        // Check read permission
        if !self.access_permissions.can_read::<T>() {
            return Err(ScriptError::AccessDenied {
                operation: format!("read component {}", std::any::type_name::<T>()),
            });
        }

        // Get component
        match world.get_component::<T>(self.entity_id) {
            Some(component) => Ok(ComponentRef {
                component,
                _phantom: PhantomData,
            }),
            None => Err(ScriptError::ComponentNotFound {
                entity: self.entity_id,
                component: std::any::type_name::<T>().to_string(),
            }),
        }
    }

    /// Get mutable component reference with permission check
    pub fn get_component_mut<'a, T: Component>(&self, world: &'a mut World) -> Result<ComponentMut<'a, T>, ScriptError> {
        // Check validity
        if !self.is_valid(world) {
            return Err(ScriptError::InvalidEntityHandle);
        }

        // Check write permission
        if !self.access_permissions.can_write::<T>() {
            return Err(ScriptError::AccessDenied {
                operation: format!("write component {}", std::any::type_name::<T>()),
            });
        }

        // Get component
        match world.get_component_mut::<T>(self.entity_id) {
            Some(component) => Ok(ComponentMut {
                component,
                _phantom: PhantomData,
            }),
            None => Err(ScriptError::ComponentNotFound {
                entity: self.entity_id,
                component: std::any::type_name::<T>().to_string(),
            }),
        }
    }

    /// Check if entity has a component
    pub fn has_component<T: Component>(&self, world: &World) -> bool {
        self.is_valid(world) && world.get_component::<T>(self.entity_id).is_some()
    }
}

/// Safe wrapper for component references
pub struct ComponentRef<'a, T> {
    component: &'a T,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> ComponentRef<'a, T> {
    /// Get the inner component reference
    pub fn get(&self) -> &T {
        self.component
    }
}

impl<'a, T> std::ops::Deref for ComponentRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

/// Safe wrapper for mutable component references
pub struct ComponentMut<'a, T> {
    component: &'a mut T,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> ComponentMut<'a, T> {
    /// Get the inner component reference
    pub fn get(&self) -> &T {
        &*self.component
    }

    /// Get the inner mutable component reference
    pub fn get_mut(&mut self) -> &mut T {
        self.component
    }
}

impl<'a, T> std::ops::Deref for ComponentMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.component
    }
}

impl<'a, T> std::ops::DerefMut for ComponentMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

// Extension trait for AccessPermissions to support component-specific permissions
impl AccessPermissions {
    /// Add permission for a specific component type
    pub fn add_component_permission<T>(&mut self, _permission: &str) {
        // In a real implementation, this would track component-specific permissions
        // For now, we'll use the general component permissions
    }
}