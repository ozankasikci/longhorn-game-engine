# Phase 13.1: ECS V2 Modularization Progress

## Goal
Split the 1,619-line `ecs_v2.rs` file into logical, maintainable modules.

## Current State
- Single monolithic file containing all ECS v2 implementation
- Difficult to navigate and maintain
- Slow compilation when any part changes

## Target Structure
```
crates/core/engine-ecs-core/src/
├── ecs_v2/
│   ├── mod.rs              # Public API and re-exports
│   ├── entity.rs           # Entity, EntityLocation, EntityAllocator
│   ├── component.rs        # ComponentArray, ComponentArrayTrait, Registry
│   ├── archetype.rs        # Archetype, ArchetypeId, ArchetypeStorage
│   ├── world.rs            # World struct and core operations
│   ├── query.rs            # Query, QueryData, QueryIter, filters
│   ├── bundle.rs           # Bundle trait and implementations
│   └── system.rs           # System traits and scheduling (future)
└── ecs_v2.rs              # Compatibility re-exports (deprecated)
```

## Tasks Checklist

### Setup
- [ ] Create `ecs_v2/` directory
- [ ] Create `mod.rs` with module declarations
- [ ] Set up re-export structure

### Entity Module (entity.rs)
- [ ] Move `Entity` struct
- [ ] Move `EntityLocation` struct
- [ ] Move entity allocation logic
- [ ] Move entity generation handling
- [ ] Add entity-specific tests

### Component Module (component.rs)
- [ ] Move `ComponentArrayTrait` trait
- [ ] Move `ComponentArray<T>` implementation
- [ ] Move `ErasedComponentArray` struct
- [ ] Move component registry (global static)
- [ ] Move `register_component` function
- [ ] Move `create_component_array` function

### Archetype Module (archetype.rs)
- [ ] Move `ArchetypeId` struct
- [ ] Move `Archetype` struct
- [ ] Move archetype storage logic
- [ ] Move archetype matching functions
- [ ] Move archetype creation/modification

### World Module (world.rs)
- [ ] Move `World` struct
- [ ] Move entity management methods
- [ ] Move component add/remove methods
- [ ] Move archetype management
- [ ] Move resource management

### Query Module (query.rs)
- [ ] Move `QueryData` trait
- [ ] Move `Read<T>` and `Write<T>` types
- [ ] Move `Changed<T>` filter
- [ ] Move `Query` and `QueryMut` structs
- [ ] Move query iterators
- [ ] Move query filtering logic

### Bundle Module (bundle.rs)
- [ ] Move `Bundle` trait
- [ ] Move bundle implementations
- [ ] Move bundle spawning logic
- [ ] Move component tuple implementations

### Integration
- [ ] Update all imports in the codebase
- [ ] Create deprecated re-exports in `ecs_v2.rs`
- [ ] Run all ECS tests
- [ ] Update documentation
- [ ] Performance benchmark comparison

## Progress Tracking
- **Started**: Not yet
- **Completed**: 0/8 modules
- **Tests Passing**: N/A
- **Blockers**: None identified

## Notes
- Maintain backward compatibility during transition
- Each module should be under 400 lines
- Add comprehensive documentation to each module
- Consider future system scheduling needs