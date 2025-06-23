# Phase 1: Enhanced ECS Foundation - Project Plan

## Overview
Upgrade the existing `ecs_v2.rs` system to modern archetypal storage architecture, providing Bevy-style performance with cache-friendly memory layout and type-safe queries.

## Goals
- **Performance**: Cache-friendly component storage for mobile optimization
- **Type Safety**: Compile-time query validation preventing data races
- **Rust Integration**: Eliminate borrowing conflicts between systems
- **Scalability**: Handle 10,000+ entities efficiently
- **Compatibility**: Maintain existing Transform/GameObject functionality

## Technical Architecture

### Core Components to Implement

#### 1. Archetypal Storage System
```rust
struct Archetype {
  entities: Vec<Entity>,
  components: HashMap<TypeId, Box<dyn ComponentArray>>,
  component_mask: ComponentMask,
}

trait ComponentArray {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn len(&self) -> usize;
  fn swap_remove(&mut self, index: usize);
  fn push(&mut self, component: Box<dyn Any>);
}
```

#### 2. Query System
```rust
struct Query<'w, Q: QueryData> {
  world: &'w World,
  archetype_filter: ArchetypeFilter,
  _phantom: PhantomData<Q>,
}

trait QueryData {
  type Item<'a>;
  fn fetch<'a>(archetype: &'a Archetype, entity_index: usize) -> Self::Item<'a>;
  fn matches_archetype(archetype: &Archetype) -> bool;
}
```

#### 3. Change Detection
```rust
struct ComponentTicks {
  added: u32,
  changed: u32,
}

struct ChangeDetector {
  current_tick: u32,
  last_run_tick: u32,
}
```

#### 4. World Management
```rust
struct WorldV2 {
  archetypes: Vec<Archetype>,
  entity_to_archetype: HashMap<Entity, (usize, usize)>, // (archetype_index, entity_index)
  archetype_lookup: HashMap<ComponentMask, usize>,
  next_entity_id: u32,
  change_tick: u32,
}
```

## Implementation Tasks

### Task 1: Core Storage Infrastructure (Estimated: 45 minutes)
- [ ] Implement `ComponentArray` trait and concrete implementations
- [ ] Create `Archetype` struct with entity and component storage
- [ ] Build `ComponentMask` for efficient archetype lookup
- [ ] Implement archetype creation and management in `WorldV2`

**Files to modify:**
- `crates/engine-core/src/ecs_v2.rs`

**Success criteria:**
- Can store entities with components in archetypes
- Efficient lookup from entity to component data
- Type-safe component access

### Task 2: Query System Implementation (Estimated: 60 minutes)
- [ ] Implement `QueryData` trait for different query types
- [ ] Create `Query<T>` struct with iterator support
- [ ] Add support for `&T` (read) and `&mut T` (write) queries
- [ ] Implement archetype filtering for query matching
- [ ] Add safety checks to prevent overlapping mutable borrows

**Files to modify:**
- `crates/engine-core/src/ecs_v2.rs`

**Success criteria:**
- `Query<(&Transform,)>` works for read-only access
- `Query<(&mut Transform,)>` works for mutable access
- `Query<(&Transform, &mut Velocity)>` works for mixed access
- Compile-time prevention of data races

### Task 3: Change Detection System (Estimated: 30 minutes)
- [ ] Implement component change tracking
- [ ] Add `Changed<T>` query filter
- [ ] Create system for incrementing change ticks
- [ ] Integrate with existing component modifications

**Files to modify:**
- `crates/engine-core/src/ecs_v2.rs`

**Success criteria:**
- Systems can query only changed components
- Change detection works across frame boundaries
- Minimal performance overhead

### Task 4: Integration and Migration (Estimated: 30 minutes)
- [ ] Update `Transform` to work with new ECS
- [ ] Ensure `ComponentV2` trait compatibility
- [ ] Update exports in `lib.rs`
- [ ] Migrate existing test cases
- [ ] Maintain backward compatibility where possible

**Files to modify:**
- `crates/engine-core/src/lib.rs`
- `crates/engine-core/src/components.rs`
- Test files

**Success criteria:**
- Existing Transform usage continues to work
- All existing tests pass
- New system is exported correctly

### Task 5: Performance Benchmarking (Estimated: 15 minutes)
- [ ] Create benchmarks comparing old vs new ECS
- [ ] Test with 1k, 10k, and 100k entities
- [ ] Measure query performance improvements
- [ ] Document performance characteristics

**Files to modify:**
- `crates/engine-core/benches/ecs_comparison.rs`

**Success criteria:**
- New ECS shows measurable performance improvements
- Memory layout is more cache-friendly
- Query iteration is faster than old system

## Performance Targets

### Entity Capacity
- **1,000 entities**: Sub-millisecond queries
- **10,000 entities**: <5ms for full iteration
- **100,000 entities**: <50ms for full iteration

### Memory Efficiency
- **Component Storage**: Contiguous arrays per archetype
- **Cache Misses**: <10% compared to old system
- **Memory Overhead**: <5% additional overhead for metadata

### Query Performance
- **Simple Queries**: 10x faster than HashMap lookup
- **Complex Queries**: 5x faster than old system
- **Change Detection**: <1% overhead when enabled

## Risk Mitigation

### Technical Risks
- **Borrow Checker Conflicts**: Use explicit lifetime management in queries
- **Type Erasure Complexity**: Comprehensive testing of Any conversions
- **Performance Regression**: Continuous benchmarking during development

### Integration Risks
- **Breaking Changes**: Maintain compatibility layer during transition
- **Test Failures**: Run existing test suite after each major change
- **Editor Compatibility**: Ensure EGUI editor continues working

## Success Metrics

### Functional Requirements
- [ ] All existing ECS functionality preserved
- [ ] Type-safe query system working
- [ ] Change detection operational
- [ ] Performance improvements demonstrated

### Non-Functional Requirements
- [ ] Code coverage >90% for new ECS code
- [ ] Documentation updated for new APIs
- [ ] Benchmarks show measurable improvements
- [ ] No breaking changes to public API

## Next Steps After Completion

1. **System Scheduling**: Implement parallel system execution
2. **Plugin Architecture**: Create trait-based system registration
3. **Editor Integration**: Update EGUI editor to use new queries
4. **Advanced Queries**: Add entity relationships and complex filters

## Dependencies

### Internal Dependencies
- Current `ecs_v2.rs` implementation
- `Transform` and component definitions
- Existing test infrastructure

### External Dependencies
- No new external crates required
- Uses existing `std::collections` and `std::any`
- Compatible with current workspace dependencies