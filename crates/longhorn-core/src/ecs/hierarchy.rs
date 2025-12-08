use crate::ecs::{Children, EntityHandle, Parent, World};
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

/// Add a child entity to a parent entity
///
/// This function maintains bidirectional synchronization:
/// - Adds Parent component to child
/// - Adds child to parent's Children component (creates if missing)
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
/// - `SelfParenting` if parent == child
pub fn add_child(world: &mut World, parent: EntityHandle, child: EntityHandle) -> Result<(), HierarchyError> {
    // Validate entities exist
    if !world.exists(parent) {
        return Err(HierarchyError::EntityNotFound(parent.id()));
    }
    if !world.exists(child) {
        return Err(HierarchyError::EntityNotFound(child.id()));
    }

    // Prevent self-parenting
    if parent == child {
        return Err(HierarchyError::SelfParenting(child.id()));
    }

    // Add Parent component to child
    world.set(child, Parent::new(parent.id())).ok();

    // Add child to parent's Children component
    let has_children = world.has::<Children>(parent);
    if has_children {
        if let Ok(mut children) = world.get_mut::<Children>(parent) {
            children.add(child.id());
        }
    } else {
        // Parent doesn't have Children component, create it
        world.set(parent, Children::with_children(vec![child.id()])).ok();
    }

    Ok(())
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

    #[test]
    fn test_add_child() {
        let mut world = World::new();
        let parent = world.spawn().with(Children::new()).build();
        let child = world.spawn().build();

        // Add child to parent
        add_child(&mut world, parent, child).unwrap();

        // Verify Parent component on child
        let parent_comp = world.get::<Parent>(child).unwrap();
        assert_eq!(parent_comp.get(), parent.id());

        // Verify Children component on parent
        let children = world.get::<Children>(parent).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children.iter().next().unwrap(), &child.id());
    }

    #[test]
    fn test_add_child_entity_not_found() {
        let mut world = World::new();
        let parent = world.spawn().with(Children::new()).build();

        // Create and immediately despawn an entity to get invalid ID
        let invalid = world.spawn().build();
        world.despawn(invalid).unwrap();

        let result = add_child(&mut world, parent, invalid);
        assert_eq!(result, Err(HierarchyError::EntityNotFound(invalid.id())));
    }

    #[test]
    fn test_add_child_self_parenting() {
        let mut world = World::new();
        let entity = world.spawn().with(Children::new()).build();

        let result = add_child(&mut world, entity, entity);
        assert_eq!(result, Err(HierarchyError::SelfParenting(entity.id())));
    }
}
