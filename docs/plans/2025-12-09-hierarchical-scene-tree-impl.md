# Hierarchical Scene Tree Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use @superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement parent-child entity relationships for the Longhorn game engine with manual synchronization, cascade delete, and hierarchical scene tree UI.

**Architecture:** Add `Parent` and `Children` components with manual bidirectional sync via helper functions. Keep single `Transform` component, compute global transforms on-demand. Store hierarchy as nested JSON for serialization.

**Tech Stack:** Rust, hecs ECS, serde, egui, glam

---

## Task 1: Add Parent and Children Components

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/component.rs:167`

**Step 1: Add Parent component with test**

In `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/component.rs`, after the `Sprite` component (line 115), add:

```rust
/// Parent component - stores reference to parent entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parent(pub Entity);

impl Parent {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

/// Children component - stores list of child entities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Children(pub Vec<Entity>);

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_children(children: Vec<Entity>) -> Self {
        Self(children)
    }

    pub fn add(&mut self, entity: Entity) {
        if !self.0.contains(&entity) {
            self.0.push(entity);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(pos) = self.0.iter().position(|&e| e == entity) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
```

**Step 2: Add tests for Parent and Children**

At the end of the `tests` module (after line 166):

```rust
#[test]
fn test_parent_component() {
    let mut world = hecs::World::new();
    let parent_id = world.spawn(());
    let child_id = world.spawn(());

    let parent = Parent::new(parent_id);
    assert_eq!(parent.get(), parent_id);
}

#[test]
fn test_children_component() {
    let mut world = hecs::World::new();
    let child1 = world.spawn(());
    let child2 = world.spawn(());

    let mut children = Children::new();
    assert!(children.is_empty());
    assert_eq!(children.len(), 0);

    children.add(child1);
    assert_eq!(children.len(), 1);

    children.add(child2);
    assert_eq!(children.len(), 2);

    // Adding same child twice should not duplicate
    children.add(child1);
    assert_eq!(children.len(), 2);

    assert!(children.remove(child1));
    assert_eq!(children.len(), 1);

    assert!(!children.remove(child1)); // Already removed
}
```

**Step 3: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::component::tests
```

Expected: All tests pass including new hierarchy component tests

**Step 4: Commit**

```bash
git add crates/longhorn-core/src/ecs/component.rs
git commit -m "feat(ecs): add Parent and Children hierarchy components

Add foundational components for entity parent-child relationships:
- Parent: stores single parent entity reference
- Children: stores list of child entities with add/remove helpers
- Manual synchronization approach (no automatic hooks)

Related to hierarchical scene tree implementation."
```

---

## Task 2: Create Hierarchy Module with Error Types

**Files:**
- Create: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/mod.rs:4`

**Step 1: Create hierarchy module with error types**

Create `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`:

```rust
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
```

**Step 2: Export hierarchy module**

In `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/mod.rs`, add `hierarchy` to the module list:

```rust
pub mod component;
pub mod entity;
pub mod hierarchy;  // NEW
pub mod script;
pub mod world;

pub use component::*;
pub use entity::*;
pub use hierarchy::*;  // NEW
pub use script::*;
pub use world::*;
```

**Step 3: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: Test passes

**Step 4: Commit**

```bash
git add crates/longhorn-core/src/ecs/hierarchy.rs crates/longhorn-core/src/ecs/mod.rs
git commit -m "feat(ecs): create hierarchy module with error types

Add hierarchy module foundation:
- HierarchyError enum with EntityNotFound, CycleDetected, SelfParenting
- Export from ecs module

Next: implement hierarchy manipulation functions."
```

---

## Task 3: Implement add_child with Tests

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs:11`

**Step 1: Write failing test for add_child**

Add to the `tests` module in `hierarchy.rs`:

```rust
#[test]
fn test_add_child() {
    let mut world = World::new();
    let parent = world.spawn().insert(Children::new()).id();
    let child = world.spawn().id();

    // Add child to parent
    add_child(&mut world, parent, child).unwrap();

    // Verify Parent component on child
    let parent_comp = world.get::<Parent>(child).unwrap();
    assert_eq!(parent_comp.get(), parent);

    // Verify Children component on parent
    let children = world.get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children.iter().next().unwrap(), &child);
}

#[test]
fn test_add_child_entity_not_found() {
    let mut world = World::new();
    let parent = world.spawn().insert(Children::new()).id();

    // Create and immediately despawn an entity to get invalid ID
    let invalid = world.spawn().id();
    world.despawn(invalid).unwrap();

    let result = add_child(&mut world, parent, invalid);
    assert_eq!(result, Err(HierarchyError::EntityNotFound(invalid)));
}

#[test]
fn test_add_child_self_parenting() {
    let mut world = World::new();
    let entity = world.spawn().insert(Children::new()).id();

    let result = add_child(&mut world, entity, entity);
    assert_eq!(result, Err(HierarchyError::SelfParenting(entity)));
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests::test_add_child
```

Expected: FAIL - `add_child` function not found

**Step 3: Implement add_child**

Add before the `tests` module:

```rust
/// Add a child entity to a parent entity
///
/// This function maintains bidirectional synchronization:
/// - Adds Parent component to child
/// - Adds child to parent's Children component (creates if missing)
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
/// - `SelfParenting` if parent == child
pub fn add_child(world: &mut World, parent: EntityId, child: EntityId) -> Result<(), HierarchyError> {
    // Validate entities exist
    if !world.contains(parent) {
        return Err(HierarchyError::EntityNotFound(parent));
    }
    if !world.contains(child) {
        return Err(HierarchyError::EntityNotFound(child));
    }

    // Prevent self-parenting
    if parent == child {
        return Err(HierarchyError::SelfParenting(child));
    }

    // Add Parent component to child
    world.set(child, Parent::new(parent)).ok();

    // Add child to parent's Children component
    if let Ok(mut children) = world.get_mut::<Children>(parent) {
        children.add(child);
    } else {
        // Parent doesn't have Children component, create it
        world.set(parent, Children::with_children(vec![child])).ok();
    }

    Ok(())
}
```

**Step 4: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/longhorn-core/src/ecs/hierarchy.rs
git commit -m "feat(ecs): implement add_child with bidirectional sync

Add add_child function with:
- Validation for entity existence and self-parenting
- Sets Parent component on child
- Updates Children component on parent (creates if missing)
- Comprehensive tests for success and error cases

Manual synchronization keeps Parent and Children in sync."
```

---

## Task 4: Implement remove_child and clear_parent

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`

**Step 1: Write failing tests**

Add to tests module:

```rust
#[test]
fn test_remove_child() {
    let mut world = World::new();
    let parent = world.spawn().insert(Children::new()).id();
    let child = world.spawn().id();

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
    let parent = world.spawn().insert(Children::new()).id();
    let child = world.spawn().id();

    add_child(&mut world, parent, child).unwrap();

    // Clear child's parent
    clear_parent(&mut world, child).unwrap();

    // Verify Parent component removed
    assert!(world.get::<Parent>(child).is_err());

    // Verify child removed from old parent's Children
    let children = world.get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 0);
}
```

**Step 2: Run tests to verify failure**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests::test_remove_child
```

Expected: FAIL - functions not found

**Step 3: Implement remove_child and clear_parent**

Add after `add_child`:

```rust
/// Remove a child entity from its parent
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
pub fn remove_child(world: &mut World, parent: EntityId, child: EntityId) -> Result<(), HierarchyError> {
    if !world.contains(parent) {
        return Err(HierarchyError::EntityNotFound(parent));
    }
    if !world.contains(child) {
        return Err(HierarchyError::EntityNotFound(child));
    }

    // Remove from parent's Children component
    if let Ok(mut children) = world.get_mut::<Children>(parent) {
        children.remove(child);
    }

    // Remove Parent component from child
    world.remove_one::<Parent>(child).ok();

    Ok(())
}

/// Remove an entity from its parent
///
/// # Errors
/// - `EntityNotFound` if child doesn't exist
pub fn clear_parent(world: &mut World, child: EntityId) -> Result<(), HierarchyError> {
    if !world.contains(child) {
        return Err(HierarchyError::EntityNotFound(child));
    }

    // Get parent before removing component
    if let Ok(parent_comp) = world.get::<Parent>(child) {
        let parent = parent_comp.get();

        // Remove from old parent's Children list
        if let Ok(mut children) = world.get_mut::<Children>(parent) {
            children.remove(child);
        }
    }

    // Remove Parent component
    world.remove_one::<Parent>(child).ok();

    Ok(())
}
```

**Step 4: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs
git commit -m "feat(ecs): implement remove_child and clear_parent

Add functions to break parent-child relationships:
- remove_child: removes specific child from parent
- clear_parent: detaches entity from its parent
- Both maintain bidirectional synchronization
- Tests verify proper cleanup of components"
```

---

## Task 5: Implement set_parent with Cycle Detection

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`

**Step 1: Write failing tests**

Add to tests module:

```rust
#[test]
fn test_set_parent() {
    let mut world = World::new();
    let parent1 = world.spawn().insert(Children::new()).id();
    let parent2 = world.spawn().insert(Children::new()).id();
    let child = world.spawn().id();

    // Set initial parent
    set_parent(&mut world, child, parent1).unwrap();
    assert_eq!(world.get::<Parent>(child).unwrap().get(), parent1);

    // Change parent
    set_parent(&mut world, child, parent2).unwrap();

    // Verify new parent
    assert_eq!(world.get::<Parent>(child).unwrap().get(), parent2);

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
    let grandparent = world.spawn().insert(Children::new()).id();
    let parent = world.spawn().insert(Children::new()).id();
    let child = world.spawn().insert(Children::new()).id();

    // Create hierarchy: grandparent -> parent -> child
    set_parent(&mut world, parent, grandparent).unwrap();
    set_parent(&mut world, child, parent).unwrap();

    // Try to make grandparent a child of child (creates cycle)
    let result = set_parent(&mut world, grandparent, child);
    assert_eq!(result, Err(HierarchyError::CycleDetected { child: grandparent }));
}
```

**Step 2: Run tests to verify failure**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests::test_set_parent
```

Expected: FAIL - function not found

**Step 3: Implement is_ancestor helper**

Add before `set_parent`:

```rust
/// Check if `ancestor` is an ancestor of `entity`
fn is_ancestor(world: &World, entity: EntityId, ancestor: EntityId) -> bool {
    if entity == ancestor {
        return true;
    }

    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        is_ancestor(world, parent_comp.get(), ancestor)
    } else {
        false
    }
}
```

**Step 4: Implement set_parent**

```rust
/// Change an entity's parent, or set initial parent
///
/// If entity already has a parent, it will be removed from the old parent first.
///
/// # Errors
/// - `EntityNotFound` if parent or child doesn't exist
/// - `SelfParenting` if parent == child
/// - `CycleDetected` if new_parent is a descendant of child
pub fn set_parent(world: &mut World, child: EntityId, new_parent: EntityId) -> Result<(), HierarchyError> {
    if !world.contains(child) {
        return Err(HierarchyError::EntityNotFound(child));
    }
    if !world.contains(new_parent) {
        return Err(HierarchyError::EntityNotFound(new_parent));
    }

    // Prevent self-parenting
    if child == new_parent {
        return Err(HierarchyError::SelfParenting(child));
    }

    // Prevent cycles: new_parent cannot be a descendant of child
    if is_ancestor(world, new_parent, child) {
        return Err(HierarchyError::CycleDetected { child });
    }

    // Remove from old parent if exists
    clear_parent(world, child)?;

    // Add to new parent
    add_child(world, new_parent, child)?;

    Ok(())
}
```

**Step 5: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/longhorn-core/src/ecs/hierarchy.rs
git commit -m "feat(ecs): implement set_parent with cycle detection

Add set_parent function with:
- Detaches from old parent before attaching to new parent
- Cycle detection prevents making ancestor a child
- is_ancestor helper recursively walks up parent chain
- Tests verify reparenting and cycle prevention"
```

---

## Task 6: Implement collect_descendants for Cascade Delete

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`

**Step 1: Write failing test**

Add to tests module:

```rust
#[test]
fn test_collect_descendants() {
    let mut world = World::new();
    let root = world.spawn().insert(Children::new()).id();
    let child1 = world.spawn().insert(Children::new()).id();
    let child2 = world.spawn().insert(Children::new()).id();
    let grandchild = world.spawn().id();

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
    assert!(descendants.contains(&child1));
    assert!(descendants.contains(&child2));
    assert!(descendants.contains(&grandchild));
}
```

**Step 2: Run test to verify failure**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests::test_collect_descendants
```

Expected: FAIL - function not found

**Step 3: Implement collect_descendants**

Add after `set_parent`:

```rust
/// Recursively collect all descendants of an entity
///
/// Returns a Vec of all descendants in depth-first order.
/// Used for cascade deletion.
pub fn collect_descendants(world: &World, entity: EntityId) -> Vec<EntityId> {
    let mut descendants = Vec::new();

    if let Ok(children) = world.get::<Children>(entity) {
        for &child in children.iter() {
            descendants.push(child);
            // Recursively collect grandchildren
            descendants.extend(collect_descendants(world, child));
        }
    }

    descendants
}
```

**Step 4: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/longhorn-core/src/ecs/hierarchy.rs
git commit -m "feat(ecs): implement collect_descendants for cascade delete

Add collect_descendants function:
- Recursively walks hierarchy depth-first
- Returns all descendants for an entity
- Will be used by World::despawn for cascade deletion
- Test verifies multi-level hierarchy collection"
```

---

## Task 7: Implement compute_global_transform

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/hierarchy.rs`

**Step 1: Write failing test**

Add to tests module:

```rust
#[test]
fn test_compute_global_transform() {
    use glam::Vec2;

    let mut world = World::new();

    // Parent at (100, 0) with no rotation/scale
    let parent = world.spawn()
        .insert(Children::new())
        .insert(Transform::from_position(Vec2::new(100.0, 0.0)))
        .id();

    // Child at local position (50, 0)
    let child = world.spawn()
        .insert(Transform::from_position(Vec2::new(50.0, 0.0)))
        .id();

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
        .insert(Transform::from_position(Vec2::new(100.0, 50.0)))
        .id();

    let global = compute_global_transform(&world, entity);

    // Without parent, global == local
    assert_eq!(global.position, Vec2::new(100.0, 50.0));
}
```

**Step 2: Run test to verify failure**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests::test_compute_global_transform
```

Expected: FAIL - function not found

**Step 3: Implement compute_global_transform**

Add after `collect_descendants`:

```rust
/// Compute the world-space transform for an entity
///
/// Walks up the parent chain and combines transforms.
/// If entity has no parent, returns its local transform.
pub fn compute_global_transform(world: &World, entity: EntityId) -> Transform {
    // Get entity's local transform
    let local_transform = world.get::<Transform>(entity)
        .map(|t| *t)
        .unwrap_or_else(|_| Transform::new());

    // If no parent, local == global
    if let Ok(parent_comp) = world.get::<Parent>(entity) {
        let parent_global = compute_global_transform(world, parent_comp.get());

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
```

**Step 4: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::hierarchy::tests
```

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/longhorn-core/src/ecs/hierarchy.rs
git commit -m "feat(ecs): implement compute_global_transform

Add compute_global_transform function:
- Recursively walks up parent chain
- Combines transforms using proper 2D transform math
- combine_transforms helper applies TRS order
- Tests verify with and without parents

On-demand computation keeps implementation simple."
```

---

## Task 8: Update World::despawn for Cascade Delete

**Files:**
- Modify: `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/world.rs`

**Step 1: Read current despawn implementation**

```bash
cargo test --package longhorn-core --lib ecs::world -- --nocapture
```

Check existing World tests to understand current despawn behavior.

**Step 2: Write integration test**

Add test in `/Users/ozan/Projects/longhorn-game-engine/crates/longhorn-core/src/ecs/world.rs` at end of tests module:

```rust
#[test]
fn test_despawn_cascade_delete() {
    use crate::ecs::hierarchy::{set_parent, Children};

    let mut world = World::new();
    let parent = world.spawn().insert(Children::new()).id();
    let child1 = world.spawn().insert(Children::new()).id();
    let grandchild = world.spawn().id();

    set_parent(&mut world, child1, parent).unwrap();
    set_parent(&mut world, grandchild, child1).unwrap();

    // Despawn parent should cascade to descendants
    world.despawn(parent).unwrap();

    // All should be despawned
    assert!(!world.contains(parent));
    assert!(!world.contains(child1));
    assert!(!world.contains(grandchild));
}
```

**Step 3: Run test to verify current behavior**

```bash
cargo test --package longhorn-core --lib ecs::world::tests::test_despawn_cascade_delete
```

Expected: Likely PASS (but descendants NOT deleted - will fix)

**Step 4: Update despawn implementation**

Find the `despawn` method in `world.rs` and update it:

```rust
pub fn despawn(&mut self, entity: EntityId) -> Result<(), EcsError> {
    use crate::ecs::hierarchy::{collect_descendants, clear_parent};

    if !self.contains(entity) {
        return Err(EcsError::EntityNotFound);
    }

    // Collect all descendants before deleting
    let descendants = collect_descendants(self, entity);

    // Remove from parent's Children list if has parent
    clear_parent(self, entity).ok();

    // Despawn the entity
    self.world.despawn(entity)
        .map_err(|_| EcsError::EntityNotFound)?;

    // Despawn all descendants
    for descendant in descendants {
        if self.contains(descendant) {
            self.world.despawn(descendant).ok();
        }
    }

    Ok(())
}
```

**Step 5: Run tests**

```bash
cargo test --package longhorn-core --lib ecs::world
```

Expected: All tests pass, cascade delete working

**Step 6: Commit**

```bash
git add crates/longhorn-core/src/ecs/world.rs
git commit -m "feat(ecs): add cascade delete to World::despawn

Update despawn to:
- Collect all descendants before deletion
- Remove entity from parent's children list
- Despawn entity and all descendants
- Test verifies multi-level cascade

Transparent cascade delete - users don't need to handle it manually."
```

---

## Verification

After completing all tasks, run full test suite:

```bash
cargo test --package longhorn-core
```

Expected: All tests pass

---

## Next Steps

This completes the core hierarchy system implementation. Remaining work (in separate plan):

1. Update scene serialization for nested structure
2. Update scene tree UI for hierarchical display
3. Add drag-drop reparenting
4. Update rendering to use global transforms

Related design doc: `/Users/ozan/Projects/longhorn-game-engine/docs/plans/2025-12-09-hierarchical-scene-tree-design.md`
