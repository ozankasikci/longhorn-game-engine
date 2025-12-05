use crate::types::EntityId;

/// Entity handle wrapper for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityHandle {
    pub id: EntityId,
}

impl EntityHandle {
    /// Create a new entity handle
    pub fn new(id: EntityId) -> Self {
        Self { id }
    }

    /// Get the raw entity ID
    pub fn id(&self) -> EntityId {
        self.id
    }
}

impl From<EntityId> for EntityHandle {
    fn from(id: EntityId) -> Self {
        Self { id }
    }
}

impl From<EntityHandle> for EntityId {
    fn from(handle: EntityHandle) -> Self {
        handle.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_handle() {
        // Create a world to generate valid entity IDs
        let mut world = hecs::World::new();
        let id = world.spawn(());

        let handle = EntityHandle::new(id);

        assert_eq!(handle.id(), id);

        let handle2: EntityHandle = id.into();
        assert_eq!(handle, handle2);

        let id2: EntityId = handle.into();
        assert_eq!(id, id2);
    }
}
