use crate::ecs::{Children, Parent};
use crate::math::{GlobalTransform, Transform};
use crate::world::World;

/// Propagate transforms through the entity hierarchy
///
/// This system updates GlobalTransform components based on the Transform hierarchy.
/// - For root entities (no Parent): GlobalTransform = Transform
/// - For child entities: GlobalTransform = parent's GlobalTransform * child's Transform
///
/// This should be called before rendering to ensure all entities have up-to-date
/// world-space transforms.
pub fn propagate_transforms(world: &mut World) {
    // First pass: Update root entities (entities without Parent component)
    let root_entities: Vec<_> = world
        .query::<()>()
        .iter()
        .filter_map(|(entity_id, _)| {
            let entity_handle = crate::ecs::EntityHandle::new(entity_id);

            // Check if this entity has a Parent component
            let has_parent = world.get::<Parent>(entity_handle).is_ok();

            if !has_parent {
                Some(entity_id)
            } else {
                None
            }
        })
        .collect();

    // Update GlobalTransform for all root entities
    for entity_id in root_entities {
        let entity_handle = crate::ecs::EntityHandle::new(entity_id);

        // Get Transform (if it exists) and copy it to avoid borrow issues
        let transform_copy = match world.get::<Transform>(entity_handle) {
            Ok(t) => *t, // Copy the Transform
            Err(_) => continue,
        };

        let global = GlobalTransform::from_transform(&transform_copy);

        // Set or update GlobalTransform (set handles both insert and replace)
        let _ = world.set(entity_handle, global);

        // Get children (copy the list to avoid borrow issues)
        let children_copy: Vec<hecs::Entity> = world
            .get::<Children>(entity_handle)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default();

        // Recursively propagate to children
        for child_id in children_copy {
            propagate_to_child(world, &global, child_id);
        }
    }
}

/// Recursively propagate transform to a child and its descendants
fn propagate_to_child(world: &mut World, parent_global: &GlobalTransform, child_id: hecs::Entity) {
    let child_handle = crate::ecs::EntityHandle::new(child_id);

    // Get child's local Transform (copy it to avoid borrow issues)
    let child_transform = match world.get::<Transform>(child_handle) {
        Ok(t) => *t, // Copy the Transform
        Err(_) => return, // No Transform component, skip
    };

    // Calculate child's GlobalTransform
    let child_global = parent_global.mul_transform(&child_transform);

    // Set or update child's GlobalTransform (set handles both insert and replace)
    let _ = world.set(child_handle, child_global);

    // Get grandchildren (copy the list to avoid borrow issues)
    let grandchildren_copy: Vec<hecs::Entity> = world
        .get::<Children>(child_handle)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();

    // Recursively propagate to grandchildren
    for grandchild_id in grandchildren_copy {
        propagate_to_child(world, &child_global, grandchild_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;
    use glam::Vec2;

    #[test]
    fn test_propagate_root_entity() {
        let mut world = World::new();

        // Create a root entity with Transform
        let entity = world
            .spawn()
            .with(Name::new("Root"))
            .with(Transform::from_position(Vec2::new(10.0, 20.0)))
            .build();

        // Run propagation
        propagate_transforms(&mut world);

        // Check GlobalTransform was created and matches Transform
        let global = world.get::<GlobalTransform>(entity).unwrap();

        assert_eq!(global.position, Vec2::new(10.0, 20.0));
        assert_eq!(global.rotation, 0.0);
        assert_eq!(global.scale, Vec2::ONE);
    }

    #[test]
    fn test_propagate_parent_child() {
        let mut world = World::new();

        // Create parent at (100, 100)
        let parent = world
            .spawn()
            .with(Name::new("Parent"))
            .with(Transform::from_position(Vec2::new(100.0, 100.0)))
            .with(Children::new())
            .build();

        // Create child at local (10, 10)
        let child = world
            .spawn()
            .with(Name::new("Child"))
            .with(Transform::from_position(Vec2::new(10.0, 10.0)))
            .with(Parent::new(parent.id))
            .build();

        // Add child to parent's Children component
        {
            let mut children = (*world.get::<Children>(parent).unwrap()).clone();
            children.add(child.id);
            let _ = world.set(parent, children);
        }

        // Run propagation
        propagate_transforms(&mut world);

        // Check parent's GlobalTransform
        assert_eq!(world.get::<GlobalTransform>(parent).unwrap().position, Vec2::new(100.0, 100.0));

        // Check child's GlobalTransform (should be 100 + 10 = 110)
        assert_eq!(world.get::<GlobalTransform>(child).unwrap().position, Vec2::new(110.0, 110.0));
    }

    #[test]
    fn test_propagate_three_level_hierarchy() {
        let mut world = World::new();

        // Grandparent at (100, 100)
        let grandparent = world
            .spawn()
            .with(Name::new("Grandparent"))
            .with(Transform::from_position(Vec2::new(100.0, 100.0)))
            .with(Children::new())
            .build();

        // Parent at local (10, 10)
        let parent = world
            .spawn()
            .with(Name::new("Parent"))
            .with(Transform::from_position(Vec2::new(10.0, 10.0)))
            .with(Parent::new(grandparent.id))
            .with(Children::new())
            .build();

        // Child at local (1, 1)
        let child = world
            .spawn()
            .with(Name::new("Child"))
            .with(Transform::from_position(Vec2::new(1.0, 1.0)))
            .with(Parent::new(parent.id))
            .build();

        // Set up hierarchy
        {
            let mut gp_children = (*world.get::<Children>(grandparent).unwrap()).clone();
            gp_children.add(parent.id);
            let _ = world.set(grandparent, gp_children);
        }

        {
            let mut p_children = (*world.get::<Children>(parent).unwrap()).clone();
            p_children.add(child.id);
            let _ = world.set(parent, p_children);
        }

        // Run propagation
        propagate_transforms(&mut world);

        // Check child's GlobalTransform (should be 100 + 10 + 1 = 111)
        assert_eq!(world.get::<GlobalTransform>(child).unwrap().position, Vec2::new(111.0, 111.0));
    }
}
