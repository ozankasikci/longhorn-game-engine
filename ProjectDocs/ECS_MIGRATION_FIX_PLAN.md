# ECS Component Migration Fix Plan

## Problem Summary

The current ECS v2 implementation has a critical limitation: entities can only have a single component type. When trying to add additional components to an entity, the migration fails because the `migrate_entity_to_new_archetype` function is not implemented. This prevents creating game objects with multiple components (Transform + Mesh + Material + etc).

## Current State

- Entities with single components work correctly
- `query_legacy<T>()` successfully finds entities with component T
- Rendering pipeline works (entities render as magenta debug cubes)
- But we cannot create realistic game objects that need multiple components

## Root Cause

The archetype-based ECS requires migrating entities between archetypes when components are added/removed. The migration function needs to:
1. Copy all existing components from the old archetype
2. Add the new component
3. Move the entity to the new archetype
4. Update entity location tracking

However, Rust's type system makes it challenging to generically copy components of unknown types at runtime.

## Solution Options

### Option 1: Type-Erased Component Cloning (Recommended)
Add a `clone_boxed()` method to the Component trait that returns a type-erased clone.

**Pros:**
- Maintains archetype-based performance benefits
- Allows dynamic component addition/removal
- Works with Rust's type system

**Cons:**
- Requires modifying the Component trait
- Slightly more complex than other options

### Option 2: Pre-Registered Component Types
Register all component types at startup and use a registry for cloning.

**Pros:**
- More explicit about supported components
- Can optimize for known types

**Cons:**
- Less flexible - must register all types upfront
- More boilerplate code

### Option 3: Hybrid Storage (Archetype + Sparse)
Use archetypes for common component combinations and sparse storage for rare components.

**Pros:**
- Best of both worlds for performance
- No migration needed for sparse components

**Cons:**
- More complex implementation
- Two different storage systems to maintain

### Option 4: Code Generation
Use macros or build scripts to generate migration code for all component combinations.

**Pros:**
- Zero runtime overhead
- Type-safe

**Cons:**
- Compile-time complexity
- Explosion of generated code

## Recommended Implementation Plan

### Phase 1: Add Component Cloning (2-3 hours)

1. **Update Component Trait**
   ```rust
   pub trait Component: 'static + Send + Sync {
       fn type_id() -> TypeId where Self: Sized {
           TypeId::of::<Self>()
       }
       
       // New method for cloning
       fn clone_boxed(&self) -> Box<dyn Component>;
   }
   ```

2. **Implement for all components**
   ```rust
   impl Component for Transform {
       fn clone_boxed(&self) -> Box<dyn Component> {
           Box::new(self.clone())
       }
   }
   ```

3. **Update ErasedComponentArray**
   - Add method to clone component at index
   - Return type-erased Box<dyn Component>

### Phase 2: Implement Generic Migration (3-4 hours)

1. **Create Component Migration Logic**
   ```rust
   fn migrate_entity_to_new_archetype<T: Component>(
       &mut self,
       entity: Entity,
       old_location: EntityLocation,
       target_archetype_id: ArchetypeId,
       new_component: T,
       new_component_ticks: ComponentTicks
   ) -> Result<(), &'static str>
   ```

2. **Migration Steps:**
   - Get old archetype
   - For each component type in old archetype:
     - Clone component at entity's index
     - Store in temporary collection
   - Remove entity from old archetype
   - Add entity to new archetype
   - Add all cloned components to new archetype
   - Add the new component
   - Update entity location

3. **Handle Edge Cases:**
   - Entity that was swapped during removal
   - Empty archetypes cleanup
   - Component tick preservation

### Phase 3: Testing & Validation (2 hours)

1. **Unit Tests**
   - Test single component entities
   - Test adding components progressively
   - Test removing components
   - Test complex entities (5+ components)

2. **Integration Tests**
   - Create full game objects (Transform + Mesh + Material + etc)
   - Verify queries work correctly
   - Test performance with many entities

3. **Editor Validation**
   - Remove Transform-only workarounds
   - Create proper 3D objects with all components
   - Verify rendering with actual Mesh components

### Phase 4: Optimization (Optional, 2 hours)

1. **Archetype Caching**
   - Cache frequently used archetypes
   - Pre-allocate space for common patterns

2. **Migration Batching**
   - Detect multiple component additions
   - Migrate once instead of multiple times

3. **Component Storage**
   - Use dense storage for common components
   - Optimize memory layout

## Implementation Priority

1. **Immediate Fix** (This Sprint)
   - Implement Phase 1 & 2
   - Get basic multi-component entities working
   - Unblock 3D rendering development

2. **Polish** (Next Sprint)
   - Complete Phase 3 testing
   - Add error handling and logging
   - Document the system

3. **Performance** (Future)
   - Profile and optimize if needed
   - Consider Phase 4 optimizations

## Alternative Quick Fix

If we need entities working immediately without implementing full migration:

1. **Bundle Pattern**: Create component bundles that are added atomically
   ```rust
   let entity = world.spawn_bundle(GameObjectBundle {
       transform: Transform::default(),
       mesh: Mesh::default(),
       material: Material::default(),
       visibility: Visibility::default(),
   });
   ```

2. **Fixed Archetypes**: Pre-define common component combinations
   - Transform-only
   - Transform + Mesh + Material + Visibility (standard 3D object)
   - Transform + SpriteRenderer + Visibility (2D sprite)
   - Transform + Camera (camera entity)

This would allow immediate progress while the proper solution is implemented.

## Success Criteria

- [ ] Entities can have multiple components added dynamically
- [ ] No components are lost during migration
- [ ] Queries return correct results after migration
- [ ] Performance remains acceptable (< 1ms for 1000 migrations)
- [ ] All existing tests pass
- [ ] Editor can create proper 3D objects

## Risks & Mitigation

1. **Performance Impact**
   - Risk: Cloning components during migration could be slow
   - Mitigation: Profile and optimize hot paths, consider pooling

2. **Memory Fragmentation**
   - Risk: Frequent migrations could fragment memory
   - Mitigation: Archetype pooling, periodic defragmentation

3. **Type Safety**
   - Risk: Type-erased cloning could hide errors
   - Mitigation: Comprehensive testing, debug assertions

## Timeline Estimate

- Research & Design: 1 hour
- Implementation: 5-7 hours  
- Testing: 2 hours
- Documentation: 1 hour

**Total: 9-11 hours**

## Next Steps

1. Review and approve this plan
2. Create feature branch `fix/ecs-component-migration`
3. Implement Phase 1 (Component trait update)
4. Test with simple cases
5. Implement Phase 2 (Migration logic)
6. Full testing and validation
7. Update editor to use proper components
8. Merge and celebrate! 🎉