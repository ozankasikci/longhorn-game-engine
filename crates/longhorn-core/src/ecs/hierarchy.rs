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

/// Remove a child entity from its parent
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
pub fn remove_child(world: &mut World, parent: EntityHandle, child: EntityHandle) -> Result<(), HierarchyError> {
    if !world.exists(parent) {
        return Err(HierarchyError::EntityNotFound(parent.id()));
    }
    if !world.exists(child) {
        return Err(HierarchyError::EntityNotFound(child.id()));
    }

    // Remove from parent's Children component
    if let Ok(mut children) = world.get_mut::<Children>(parent) {
        children.remove(child.id());
    }

    // Remove Parent component from child
    world.remove::<Parent>(child).ok();

    Ok(())
}

/// Remove an entity from its parent
///
/// # Errors
/// - `EntityNotFound` if child doesn't exist
pub fn clear_parent(world: &mut World, child: EntityHandle) -> Result<(), HierarchyError> {
    if !world.exists(child) {
        return Err(HierarchyError::EntityNotFound(child.id()));
    }

    // Get parent before removing component
    if let Ok(parent_comp) = world.get::<Parent>(child) {
        let parent_id = parent_comp.get();
        let parent = EntityHandle::new(parent_id);

        // Remove from old parent's Children list
        if let Ok(mut children) = world.get_mut::<Children>(parent) {
            children.remove(child.id());
        }
    }

    // Remove Parent component
    world.remove::<Parent>(child).ok();

    Ok(())
}

/// Check if `ancestor` is an ancestor of `entity`
fn is_ancestor(world: &World, entity: EntityHandle, ancestor: EntityHandle) -> bool {
    if entity == ancestor {
        return true;
    }

    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        let parent = EntityHandle::new(parent_comp.get());
        is_ancestor(world, parent, ancestor)
    } else {
        false
    }
}

/// Change an entity's parent, or set initial parent
///
/// If entity already has a parent, it will be removed from the old parent first.
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
/// - `SelfParenting` if parent == child
/// - `CycleDetected` if new_parent is a descendant of child
pub fn set_parent(world: &mut World, child: EntityHandle, new_parent: EntityHandle) -> Result<(), HierarchyError> {
    if !world.exists(child) {
        return Err(HierarchyError::EntityNotFound(child.id()));
    }
    if !world.exists(new_parent) {
        return Err(HierarchyError::EntityNotFound(new_parent.id()));
    }

    // Prevent self-parenting
    if child == new_parent {
        return Err(HierarchyError::SelfParenting(child.id()));
    }

    // Prevent cycles: new_parent cannot be a descendant of child
    if is_ancestor(world, new_parent, child) {
        return Err(HierarchyError::CycleDetected { child: child.id() });
    }

    // Remove from old parent if exists
    clear_parent(world, child)?;

    // Add to new parent
    add_child(world, new_parent, child)?;

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

    #[test]
    fn test_remove_child() {
        let mut world = World::new();
        let parent = world.spawn().with(Children::new()).build();
        let child = world.spawn().build();

        add_child(&mut world, parent, child).unwrap();

        // Remove child from parent
        remove_child(&mut world, parent, child).unwrap();

        // Verify Parent component removed from child
        assert!(world.get::<Parent>(child).is_err());

        // Verify child removed from parent's Children
        let children = world.get::<Children>(parent).unwrap();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_clear_parent() {
        let mut world = World::new();
        let parent = world.spawn().with(Children::new()).build();
        let child = world.spawn().build();

        add_child(&mut world, parent, child).unwrap();

        // Clear child's parent
        clear_parent(&mut world, child).unwrap();

        // Verify Parent component removed
        assert!(world.get::<Parent>(child).is_err());

        // Verify child removed from old parent's Children
        let children = world.get::<Children>(parent).unwrap();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_set_parent() {
        let mut world = World::new();
        let parent1 = world.spawn().with(Children::new()).build();
        let parent2 = world.spawn().with(Children::new()).build();
        let child = world.spawn().build();

        // Set initial parent
        set_parent(&mut world, child, parent1).unwrap();
        assert_eq!(world.get::<Parent>(child).unwrap().get(), parent1.id());

        // Change parent
        set_parent(&mut world, child, parent2).unwrap();

        // Verify new parent
        assert_eq!(world.get::<Parent>(child).unwrap().get(), parent2.id());

        // Verify removed from old parent
        let children1 = world.get::<Children>(parent1).unwrap();
        assert_eq!(children1.len(), 0);

        // Verify added to new parent
        let children2 = world.get::<Children>(parent2).unwrap();
        assert_eq!(children2.len(), 1);
    }

    #[test]
    fn test_set_parent_cycle_detection() {
        let mut world = World::new();
        let grandparent = world.spawn().with(Children::new()).build();
        let parent = world.spawn().with(Children::new()).build();
        let child = world.spawn().with(Children::new()).build();

        // Create hierarchy: grandparent -> parent -> child
        set_parent(&mut world, parent, grandparent).unwrap();
        set_parent(&mut world, child, parent).unwrap();

        // Try to make grandparent a child of child (creates cycle)
        let result = set_parent(&mut world, grandparent, child);
        assert_eq!(result, Err(HierarchyError::CycleDetected { child: grandparent.id() }));
    }
}
