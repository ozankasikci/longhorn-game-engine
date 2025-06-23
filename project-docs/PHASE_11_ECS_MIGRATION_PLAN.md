# Phase 11: ECS Component Migration Implementation

## Overview

Implement proper component migration in the ECS v2 system to enable dynamic component addition and removal at runtime. This will transform our static bundle-based ECS into a fully dynamic system following industry standards/Godot.

## Current State

- ✅ Bundle system works for creating entities with multiple components
- ❌ Cannot add components to existing entities
- ❌ Cannot remove components from entities
- ❌ Migration fails with "Component migration not yet implemented"

## Goals

1. Enable dynamic component addition: `world.add_component(entity, component)`
2. Enable component removal: `world.remove_component::<T>(entity)`
3. Support runtime entity evolution (status effects, equipment, etc.)
4. Enable full inspector functionality (add/remove components in editor)

## Technical Approach

### 1. Component Trait Enhancement (2 hours)

Update the Component trait to support type-erased cloning:

```rust
pub trait Component: 'static + Send + Sync {
  fn type_id() -> TypeId where Self: Sized {
    TypeId::of::<Self>()
  }
  
  /// Clone this component as a type-erased box
  fn clone_boxed(&self) -> Box<dyn ComponentClone>;
}

/// Helper trait for component cloning
pub trait ComponentClone: Component {
  fn clone_box(&self) -> Box<dyn ComponentClone>;
  fn as_any(&self) -> &dyn Any;
  fn into_any_box(self: Box<Self>) -> Box<dyn Any>;
}
```

### 2. Component Implementations (1 hour)

Implement cloning for all components using a macro:

```rust
#[macro_export]
macro_rules! impl_component_clone {
  ($type:ty) => {
    impl Component for $type {
      fn clone_boxed(&self) -> Box<dyn ComponentClone> {
        Box::new(self.clone())
      }
    }
    
    impl ComponentClone for $type {
      fn clone_box(&self) -> Box<dyn ComponentClone> {
        Box::new(self.clone())
      }
      
      fn as_any(&self) -> &dyn Any {
        self
      }
      
      fn into_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
      }
    }
  };
}

// Apply to all components
impl_component_clone!(Transform);
impl_component_clone!(Mesh);
impl_component_clone!(Material);
impl_component_clone!(Visibility);
impl_component_clone!(Camera);
impl_component_clone!(Name);
impl_component_clone!(Light);
impl_component_clone!(SpriteRenderer);
```

### 3. ErasedComponentArray Enhancement (2 hours)

Add cloning capability to type-erased storage:

```rust
impl ErasedComponentArray {
  /// Clone a component at the given index
  pub fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>>;
  
  /// Add a cloned component
  pub fn push_cloned(&mut self, component: Box<dyn ComponentClone>, ticks: ComponentTicks) -> Result<(), &'static str>;
}
```

### 4. Migration Implementation (3 hours)

Implement the full migration logic:

```rust
fn migrate_entity_to_new_archetype<T: Component>(
  &mut self,
  entity: Entity,
  old_location: EntityLocation,
  target_archetype_id: ArchetypeId,
  new_component: T,
  new_component_ticks: ComponentTicks
) -> Result<(), &'static str> {
  // 1. Clone all existing components from old archetype
  let components_to_migrate = self.clone_entity_components(entity, &old_location)?;
  
  // 2. Remove entity from old archetype
  self.remove_entity_from_archetype(entity, &old_location)?;
  
  // 3. Create/get target archetype
  self.ensure_archetype_exists(target_archetype_id.clone());
  
  // 4. Add entity to new archetype
  let new_index = self.add_entity_to_archetype(entity, target_archetype_id.clone());
  
  // 5. Add all cloned components
  for (component, ticks) in components_to_migrate {
    self.add_cloned_component_to_archetype(&target_archetype_id, new_index, component, ticks)?;
  }
  
  // 6. Add the new component
  self.add_component_to_archetype(&target_archetype_id, new_index, new_component, new_component_ticks)?;
  
  Ok(())
}
```

### 5. Component Removal (2 hours)

Implement remove_component functionality:

```rust
impl World {
  pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<Option<T>, &'static str> {
    // 1. Get current location
    let location = self.entity_locations.get(&entity).ok_or("Entity not found")?;
    
    // 2. Check if entity has the component
    if !self.has_component::<T>(entity) {
      return Ok(None);
    }
    
    // 3. Create new archetype ID without this component
    let new_archetype_id = self.create_archetype_without::<T>(&location.archetype_id);
    
    // 4. Migrate entity (excluding the removed component)
    self.migrate_entity_excluding::<T>(entity, location, new_archetype_id)?;
    
    Ok(Some(removed_component))
  }
}
```

## Implementation Steps

### Phase 11.1: Core Infrastructure (4 hours)
- [ ] Update Component trait with clone_boxed
- [ ] Create ComponentClone helper trait
- [ ] Implement macro for easy component implementation
- [ ] Update all existing components

### Phase 11.2: Storage Enhancement (2 hours)
- [ ] Add cloning to ErasedComponentArray
- [ ] Add component copying utilities
- [ ] Implement archetype cloning helpers

### Phase 11.3: Migration Logic (3 hours)
- [ ] Implement full migrate_entity_to_new_archetype
- [ ] Add component collection before migration
- [ ] Handle entity location updates
- [ ] Clean up empty archetypes

### Phase 11.4: Remove Component (2 hours)
- [ ] Implement remove_component method
- [ ] Add archetype creation without component
- [ ] Test component removal

### Phase 11.5: Testing & Validation (2 hours)
- [ ] Unit tests for component cloning
- [ ] Integration tests for migration
- [ ] Performance benchmarks
- [ ] Editor integration testing

## Success Criteria

1. **Dynamic Component Addition**
  ```rust
  let entity = world.spawn_with(Transform::default());
  world.add_component(entity, Mesh::default()); // Works!
  world.add_component(entity, Material::default()); // Works!
  ```

2. **Component Removal**
  ```rust
  world.remove_component::<Visibility>(entity); // Works!
  ```

3. **Inspector Functionality**
  - Add Component button works
  - Remove Component button works
  - Components can be added/removed at runtime

4. **Performance**
  - Migration < 1ms for typical entities
  - No memory leaks
  - Efficient archetype management

## Testing Plan

### Unit Tests
```rust
#[test]
fn test_add_component_to_existing_entity() {
  let mut world = World::new();
  let entity = world.spawn_with(Transform::default());
  
  // Should work now!
  world.add_component(entity, Mesh::default()).unwrap();
  world.add_component(entity, Material::default()).unwrap();
  
  assert!(world.get_component::<Transform>(entity).is_some());
  assert!(world.get_component::<Mesh>(entity).is_some());
  assert!(world.get_component::<Material>(entity).is_some());
}

#[test]
fn test_remove_component() {
  let mut world = World::new();
  let entity = world.spawn_bundle(GameObject3DBundle::default()).unwrap();
  
  // Remove mesh component
  let removed = world.remove_component::<Mesh>(entity).unwrap();
  assert!(removed.is_some());
  assert!(world.get_component::<Mesh>(entity).is_none());
  assert!(world.get_component::<Transform>(entity).is_some());
}
```

### Integration Test
```rust
// Test complex entity evolution
let entity = world.spawn_with(Transform::default());
world.add_component(entity, Mesh::default()).unwrap();
world.add_component(entity, Material::default()).unwrap();
world.remove_component::<Mesh>(entity).unwrap();
world.add_component(entity, Light::default()).unwrap();
```

## Risks & Mitigation

1. **Performance Impact**
  - Risk: Cloning overhead during migration
  - Mitigation: Profile and optimize hot paths, pool allocations

2. **Memory Fragmentation**
  - Risk: Many migrations cause fragmentation
  - Mitigation: Archetype cleanup, periodic defragmentation

3. **Type Safety**
  - Risk: Type-erased operations could fail
  - Mitigation: Comprehensive testing, debug assertions

## Timeline

- Phase 11.1: 4 hours
- Phase 11.2: 2 hours 
- Phase 11.3: 3 hours
- Phase 11.4: 2 hours
- Phase 11.5: 2 hours

**Total: 13 hours**

## Next Steps

1. Start with Component trait update
2. Implement cloning for one component as proof of concept
3. Extend to all components
4. Implement migration logic
5. Add remove functionality
6. Comprehensive testing

This phase will complete the ECS implementation and enable full dynamic entity composition!