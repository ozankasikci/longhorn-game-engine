use crate::ecs::{Children, Parent, World};
use crate::math::Transform;
use crate::types::EntityId;
use thiserror::Error;

/// Errors that can occur during hierarchy operations
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HierarchyError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

    #[error("Cycle detected: entity {child:?} cannot be a descendant of itself")]
    CycleDetected { child: EntityId },

    #[error("Self-parenting not allowed: entity {0:?}")]
    SelfParenting(EntityId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_error_display() {
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        let err = HierarchyError::EntityNotFound(entity);
        assert!(err.to_string().contains("Entity not found"));

        let err = HierarchyError::CycleDetected { child: entity };
        assert!(err.to_string().contains("Cycle detected"));

        let err = HierarchyError::SelfParenting(entity);
        assert!(err.to_string().contains("Self-parenting"));
    }
}
