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

/// Set parent and insert child at specific index in parent's children list
///
/// Similar to `set_parent` but allows controlling where in the sibling order
/// the child appears.
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
/// - `SelfParenting` if parent == child
/// - `CycleDetected` if new_parent is a descendant of child
pub fn set_parent_at_index(
    world: &mut World,
    child: EntityHandle,
    new_parent: EntityHandle,
    index: usize,
) -> Result<(), HierarchyError> {
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

    // Check if moving within same parent - need to adjust index
    let old_index_in_new_parent = if let Ok(parent_comp) = world.get::<Parent>(child) {
        if parent_comp.get() == new_parent.id() {
            // Same parent - get current index for adjustment
            world.get::<Children>(new_parent)
                .ok()
                .and_then(|children| children.index_of(child.id()))
        } else {
            None
        }
    } else {
        None
    };

    // Remove from old parent if exists
    clear_parent(world, child)?;

    // Adjust index if child was before target position in the same parent
    let adjusted_index = match old_index_in_new_parent {
        Some(old_idx) if old_idx < index => index.saturating_sub(1),
        _ => index,
    };

    // Add Parent component to child
    world.set(child, Parent::new(new_parent.id())).ok();

    // Add child to parent's Children component at adjusted index
    let has_children = world.has::<Children>(new_parent);
    if has_children {
        if let Ok(mut children) = world.get_mut::<Children>(new_parent) {
            children.insert_at(child.id(), adjusted_index);
        }
    } else {
        // Parent doesn't have Children component, create it
        world.set(new_parent, Children::with_children(vec![child.id()])).ok();
    }

    Ok(())
}

/// Reorder a child within its parent's children list
///
/// Moves the entity to a new index among its siblings.
/// If the entity has no parent, this is a no-op.
///
/// # Errors
/// - `EntityNotFound` if entity doesn't exist
pub fn reorder_sibling(
    world: &mut World,
    entity: EntityHandle,
    new_index: usize,
) -> Result<(), HierarchyError> {
    if !world.exists(entity) {
        return Err(HierarchyError::EntityNotFound(entity.id()));
    }

    // Get parent
    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        let parent = EntityHandle::new(parent_comp.get());

        if let Ok(mut children) = world.get_mut::<Children>(parent) {
            children.move_to(entity.id(), new_index);
        }
    }

    Ok(())
}

/// Get the index of an entity within its parent's children list
///
/// Returns None if the entity has no parent or is not in the parent's children list.
pub fn get_sibling_index(world: &World, entity: EntityHandle) -> Option<usize> {
    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        let parent = EntityHandle::new(parent_comp.get());

        if let Ok(children) = world.get::<Children>(parent) {
            return children.index_of(entity.id());
        }
    }
    None
}

/// Recursively collect all descendants of an entity
///
/// Returns a Vec of all descendants in depth-first order.
/// Used for cascade deletion.
pub fn collect_descendants(world: &World, entity: EntityHandle) -> Vec<EntityId> {
    let mut descendants = Vec::new();

    if let Ok(children) = world.get::<Children>(entity) {
        for &child in children.iter() {
            descendants.push(child);
            // Recursively collect grandchildren
            let child_handle = EntityHandle::new(child);
            descendants.extend(collect_descendants(world, child_handle));
        }
    }

    descendants
}

/// Compute the world-space transform for an entity
///
/// Walks up the parent chain and combines transforms.
/// If entity has no parent, returns its local transform.
pub fn compute_global_transform(world: &World, entity: EntityHandle) -> Transform {
    // Get entity's local transform
    let local_transform = world.get::<Transform>(entity)
        .map(|t| *t)
        .unwrap_or_else(|_| Transform::new());

    // If no parent, local == global
    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        let parent = EntityHandle::new(parent_comp.get());
        let parent_global = compute_global_transform(world, parent);

        // Combine parent's global with this entity's local
        combine_transforms(&parent_global, &local_transform)
    } else {
        local_transform
    }
}

/// Combine two transforms (parent * child)
fn combine_transforms(parent: &Transform, child: &Transform) -> Transform {
    use glam::Vec2;

    // Apply parent's scale to child's position
    let scaled_child_pos = child.position * parent.scale;

    // Rotate child's position by parent's rotation
    let cos = parent.rotation.cos();
    let sin = parent.rotation.sin();
    let rotated_pos = Vec2::new(
        scaled_child_pos.x * cos - scaled_child_pos.y * sin,
        scaled_child_pos.x * sin + scaled_child_pos.y * cos,
    );

    Transform {
        position: parent.position + rotated_pos,
        rotation: parent.rotation + child.rotation,
        scale: parent.scale * child.scale,
    }
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

    #[test]
    fn test_collect_descendants() {
        let mut world = World::new();
        let root = world.spawn().with(Children::new()).build();
        let child1 = world.spawn().with(Children::new()).build();
        let child2 = world.spawn().with(Children::new()).build();
        let grandchild = world.spawn().build();

        // Create hierarchy:
        //   root
        //   ├── child1
        //   │   └── grandchild
        //   └── child2
        set_parent(&mut world, child1, root).unwrap();
        set_parent(&mut world, child2, root).unwrap();
        set_parent(&mut world, grandchild, child1).unwrap();

        let descendants = collect_descendants(&world, root);

        // Should return all descendants in depth-first order
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&child1.id()));
        assert!(descendants.contains(&child2.id()));
        assert!(descendants.contains(&grandchild.id()));
    }

    #[test]
    fn test_compute_global_transform() {
        use glam::Vec2;

        let mut world = World::new();

        // Parent at (100, 0) with no rotation/scale
        let parent = world.spawn()
            .with(Children::new())
            .with(Transform::from_position(Vec2::new(100.0, 0.0)))
            .build();

        // Child at local position (50, 0)
        let child = world.spawn()
            .with(Transform::from_position(Vec2::new(50.0, 0.0)))
            .build();

        set_parent(&mut world, child, parent).unwrap();

        let global = compute_global_transform(&world, child);

        // Global position should be (150, 0)
        assert_eq!(global.position, Vec2::new(150.0, 0.0));
    }

    #[test]
    fn test_compute_global_transform_no_parent() {
        use glam::Vec2;

        let mut world = World::new();
        let entity = world.spawn()
            .with(Transform::from_position(Vec2::new(100.0, 50.0)))
            .build();

        let global = compute_global_transform(&world, entity);

        // Without parent, global == local
        assert_eq!(global.position, Vec2::new(100.0, 50.0));
    }
}
