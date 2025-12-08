# Hierarchical Scene Tree Design

**Date:** 2025-12-09
**Status:** Design Complete
**Approach:** Simplified hierarchy system inspired by Bevy

## Overview

This design implements a hierarchical entity system for the Longhorn game engine, allowing game objects to have parent-child relationships. The design is inspired by Bevy's proven approach but simplified for a mobile-focused 2D engine.

## Design Decisions

### Architecture Choice
- **Simplified manual synchronization** over Bevy's automatic component hooks
- **Single Transform component** instead of dual Transform/GlobalTransform system
- **Cascade delete** on parent removal (matching Unity/Unreal behavior)
- **Drag & drop** for creating parent-child relationships in editor

### Rationale
For mobile-focused 2D games, scenes are typically smaller and hierarchy changes less frequent than complex 3D games. Manual synchronization is easier to debug and understand, with negligible performance difference for typical use cases. Compute world-space positions on-the-fly by walking the parent chain rather than maintaining cached global transforms.

## Part 1: Core Components & Data Model

### New Components

**File:** `crates/longhorn-core/src/ecs/component.rs`

```rust
/// Stores reference to parent entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parent(pub Entity);

/// Stores list of child entities in order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Children(pub Vec<Entity>);
```

**Key Properties:**
- `Parent` lives on child entities, points to parent
- `Children` lives on parent entities, stores all children
- Both components manually synchronized (simpler than automatic hooks)
- Transform remains relative to parent (or world if no parent)

## Part 2: Hierarchy Manipulation API

### New Module

**File:** `crates/longhorn-core/src/ecs/hierarchy.rs`

```rust
/// Add child to parent, maintaining bidirectional sync
pub fn add_child(world: &mut World, parent: Entity, child: Entity)
    -> Result<(), HierarchyError>;

/// Remove child from parent
pub fn remove_child(world: &mut World, parent: Entity, child: Entity)
    -> Result<(), HierarchyError>;

/// Change entity's parent (detach from old, attach to new)
pub fn set_parent(world: &mut World, child: Entity, new_parent: Entity)
    -> Result<(), HierarchyError>;

/// Remove entity from its parent
pub fn clear_parent(world: &mut World, child: Entity)
    -> Result<(), HierarchyError>;

/// Recursively collect all descendants for cascade delete
pub fn collect_descendants(world: &World, entity: Entity) -> Vec<Entity>;

/// Compute world-space transform by walking up parent chain
pub fn compute_global_transform(world: &World, entity: Entity) -> Transform;
```

### Error Handling

```rust
pub enum HierarchyError {
    EntityNotFound,
    CycleDetected,      // Prevent making ancestor a child
    SelfParenting,      // Prevent entity being its own parent
}
```

**Safety Guarantees:**
- Cycle detection prevents circular hierarchies
- Both `Parent` and `Children` always stay in sync
- Invalid entity references rejected with clear errors

## Part 3: Scene Tree UI - Hierarchical Display

### Updates to Scene Tree Panel

**File:** `crates/longhorn-editor/src/panels/scene_tree.rs`

Transform from flat list to tree view (similar to existing `project_panel/tree_view.rs`):

```rust
/// Represents entity hierarchy for display
struct EntityNode {
    entity: Entity,
    name: String,
    children: Vec<EntityNode>,
    is_expanded: bool,  // Track collapse/expand state
}

/// Recursively render tree with indentation
fn show_entity_node(
    ui: &mut Ui,
    node: &EntityNode,
    world: &mut World,
    state: &mut EditorState,
    depth: usize,
) {
    ui.horizontal(|ui| {
        // Indent based on depth
        ui.add_space(depth as f32 * 16.0);

        // Expand/collapse arrow if has children
        if !node.children.is_empty() {
            if ui.button(if node.is_expanded { "▼" } else { "▶" }).clicked() {
                // Toggle expansion
            }
        }

        // Entity name (selectable)
        if ui.selectable_label(
            state.selected_entity == Some(node.entity),
            &node.name
        ).clicked() {
            state.selected_entity = Some(node.entity);
        }
    });

    // Recursively show children if expanded
    if node.is_expanded {
        for child in &node.children {
            show_entity_node(ui, child, world, state, depth + 1);
        }
    }
}
```

### Drag & Drop for Reparenting

Using egui's drag-and-drop system:

```rust
// On drag source
let response = ui.selectable_label(...);
if response.hovered() {
    response.dnd_set_drag_payload(dragged_entity);
}

// On drop target (when hovering another entity)
if let Some(dragged) = response.dnd_release_payload::<Entity>() {
    // Call hierarchy::set_parent(world, dragged, drop_target)
    // Includes cycle detection
}
```

**Visual Feedback:**
- Highlight drop target when dragging
- Show insertion indicator line
- Prevent dropping onto descendants (cycle detection)

## Part 4: Scene Serialization with Hierarchy

### Updated Serialization Structures

**File:** `crates/longhorn-core/src/scene/scene.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub id: u64,
    pub components: SerializedComponents,
    pub children: Vec<SerializedEntity>,  // NEW: Nested structure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedComponents {
    pub name: Option<String>,
    pub transform: Option<SerializedTransform>,
    pub sprite: Option<SerializedSprite>,
    pub script: Option<Script>,
    pub enabled: Option<bool>,
    // Note: No Parent/Children components - derived from nesting
}
```

### Key Insight from Bevy

Store hierarchy as nested JSON structure, not as `Parent`/`Children` components. More intuitive and prevents desync between entity IDs and relationships.

### Serialization Process

1. Find all root entities (entities without `Parent` component)
2. For each root, recursively serialize its children
3. Create nested `SerializedEntity` tree
4. Write to JSON

### Deserialization Process

1. Spawn entities depth-first
2. As each entity spawns, automatically set `Parent` on children and `Children` on parents
3. Build hierarchy while creating entities

### JSON Example

```json
{
  "name": "Game Scene",
  "entities": [
    {
      "id": 1,
      "components": {
        "name": "Player",
        "transform": { "position": [0, 0], "rotation": 0, "scale": [1, 1] }
      },
      "children": [
        {
          "id": 2,
          "components": { "name": "Weapon" },
          "children": []
        }
      ]
    }
  ]
}
```

## Part 5: World Integration & Cascade Delete

### Update World::despawn()

**File:** `crates/longhorn-core/src/ecs/world.rs`

```rust
pub fn despawn(&mut self, entity: Entity) -> Result<(), EcsError> {
    // Collect all descendants before deleting anything
    let descendants = hierarchy::collect_descendants(self, entity);

    // Remove from parent's Children list if has parent
    if let Ok(parent_comp) = self.get::<Parent>(entity) {
        hierarchy::remove_child(self, parent_comp.0, entity)?;
    }

    // Despawn entity and all descendants
    for e in std::iter::once(entity).chain(descendants) {
        self.world.despawn(e)?;
    }

    Ok(())
}
```

This ensures cascade delete works transparently - users don't need to think about it.

### Rendering with Hierarchy

Update rendering system to use world-space transforms:

```rust
// In render system
for (entity, (sprite, transform)) in world.query::<(&Sprite, &Transform)>() {
    let global_transform = hierarchy::compute_global_transform(world, entity);

    // Render sprite at global_transform position
    render_sprite(sprite, &global_transform);
}
```

**Note:** This computes global transforms on-demand. For performance-critical cases, could cache results, but start simple.

## Part 6: Testing Strategy & Migration Path

### Testing Approach

**Unit Tests** (`crates/longhorn-core/src/ecs/hierarchy.rs`):
- Test cycle detection
- Test cascade delete collection
- Test parent/children synchronization
- Test world transform computation
- Test self-parenting prevention

**Integration Tests** (scene serialization):
- Save and load hierarchical scenes
- Verify entity IDs preserved
- Verify parent-child relationships restored
- Test backward compatibility with flat scenes

**Manual Editor Testing:**
- Drag-drop reparenting
- Expand/collapse nodes
- Delete entities with children
- Save/load scenes with hierarchy
- Verify visual rendering with nested transforms

### Migration Path for Existing Scenes

Existing flat scenes will load correctly because:
- Entities without `Parent` component are treated as roots
- Missing `children` arrays default to empty
- Backward compatible with current JSON format

### Implementation Order

1. **Add components** - `Parent`/`Children` to `component.rs`
2. **Implement helpers** - `hierarchy.rs` with tests
3. **Update despawn** - Cascade delete in `world.rs`
4. **Update serialization** - Nested structure in `scene.rs` (breaking change - commit!)
5. **Update scene tree UI** - Hierarchical display with expand/collapse
6. **Add drag-drop** - Reparenting interaction
7. **Update rendering** - Use global transforms
8. **Physics integration** - If/when needed

### Commit Strategy

Small, focused commits at each step:
- Each logical change in its own commit
- Commits are reviewable and revertible
- Tests committed with implementation
- Breaking changes clearly marked in commit message

## Files to Modify

| File | Changes |
|------|---------|
| `crates/longhorn-core/src/ecs/component.rs` | Add `Parent` and `Children` components |
| `crates/longhorn-core/src/ecs/hierarchy.rs` | New module with hierarchy helpers |
| `crates/longhorn-core/src/ecs/mod.rs` | Export hierarchy module |
| `crates/longhorn-core/src/ecs/world.rs` | Update `despawn()` for cascade delete |
| `crates/longhorn-core/src/scene/scene.rs` | Update serialization for nested structure |
| `crates/longhorn-editor/src/panels/scene_tree.rs` | Tree view with drag-drop |
| Rendering systems | Use `compute_global_transform()` |

## Future Enhancements

**Not in initial scope, but possible later:**

1. **Cached GlobalTransform** - If performance becomes an issue
2. **Component hooks** - Automatic sync like Bevy (adds complexity)
3. **Prefab/template system** - Reusable hierarchies
4. **Hierarchy queries** - "Find all descendants with Sprite"
5. **Inspector parent field** - Alternative to drag-drop
6. **Right-click menu** - Advanced operations
7. **Keyboard shortcuts** - Power user features

## References

- Bevy hierarchy implementation: `bevy/crates/bevy_ecs/src/hierarchy.rs`
- Existing project panel tree: `longhorn-editor/src/panels/project_panel/tree_view.rs`
- Current scene serialization: `longhorn-core/src/scene/scene.rs`
