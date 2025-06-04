# Phase 13.1: ECS V2 Testing Plan

## Overview
Comprehensive testing strategy for the modularized ECS V2 system using Test-Driven Development (TDD) approach.

## Testing Philosophy
1. **Unit Tests**: Test each module in isolation
2. **Integration Tests**: Test module interactions
3. **Performance Tests**: Ensure no regression from monolithic version
4. **Property-Based Tests**: Test invariants and edge cases

## Test Coverage Goals
- Minimum 80% code coverage per module
- 100% coverage for critical paths (entity allocation, component storage)
- Performance benchmarks to compare with original implementation

## Module Testing Strategy

### 1. Entity Module Tests
**File**: `src/ecs_v2/entity/tests.rs`

- [ ] Entity creation and properties
- [ ] Entity equality and ordering
- [ ] EntityAllocator allocation
- [ ] EntityAllocator recycling
- [ ] Entity generation safety
- [ ] EntityLocation manipulation
- [ ] Edge cases (max entities, overflow)

### 2. Component Module Tests
**File**: `src/ecs_v2/component/tests.rs`

- [ ] Component registration
- [ ] Component array creation
- [ ] Type-erased storage operations
- [ ] Component array push/get/remove
- [ ] Component cloning
- [ ] Change tick tracking
- [ ] Registry thread safety
- [ ] Unregistered component handling

### 3. Archetype Module Tests
**File**: `src/ecs_v2/archetype/tests.rs`

- [ ] ArchetypeId creation and comparison
- [ ] ArchetypeId component operations
- [ ] Archetype entity management
- [ ] Archetype component storage
- [ ] Component array initialization
- [ ] Entity removal and swapping
- [ ] Archetype cloning operations

### 4. World Module Tests
**File**: `src/ecs_v2/world/tests.rs`

- [ ] World creation
- [ ] Entity spawning and despawning
- [ ] Component addition/removal
- [ ] Entity migration between archetypes
- [ ] Component queries
- [ ] Parallel query safety
- [ ] Change tick management
- [ ] Edge cases (empty world, missing entities)

### 5. Query Module Tests (TDD)
**File**: `src/ecs_v2/query/tests.rs`

Before implementation:
- [ ] Define query matching behavior
- [ ] Define query iteration behavior
- [ ] Define mutable query constraints
- [ ] Define filter behavior
- [ ] Define changed component detection

### 6. Bundle Module Tests (TDD)
**File**: `src/ecs_v2/bundle/tests.rs`

Before implementation:
- [ ] Define bundle spawning behavior
- [ ] Define bundle component ordering
- [ ] Define nested bundle behavior
- [ ] Define bundle query interaction

## Integration Tests
**File**: `tests/ecs_v2_integration.rs`

- [ ] Complete entity lifecycle
- [ ] Complex archetype migrations
- [ ] Multi-component queries
- [ ] System-like update patterns
- [ ] Concurrent access patterns

## Performance Benchmarks
**File**: `benches/ecs_v2_benchmarks.rs`

- [ ] Entity spawn performance
- [ ] Component add/remove performance
- [ ] Query iteration performance
- [ ] Archetype migration cost
- [ ] Memory usage comparison
- [ ] Cache efficiency metrics

## Property-Based Tests
Using `proptest` crate:

- [ ] Entity ID uniqueness
- [ ] Archetype consistency
- [ ] Component storage integrity
- [ ] Query result correctness
- [ ] World state consistency

## Test Utilities
**File**: `src/ecs_v2/test_utils.rs`

Common test components and helpers:
- Position, Velocity, Health components
- World setup functions
- Assertion helpers
- Performance measurement utilities

## TDD Process for Remaining Modules

### Query Module TDD Steps
1. Write failing tests for basic query matching
2. Implement minimal QueryData trait
3. Write tests for tuple queries
4. Implement tuple query support
5. Write tests for filters
6. Implement filter support
7. Write tests for change detection
8. Implement change detection

### Bundle Module TDD Steps
1. Write failing tests for single component bundles
2. Implement basic Bundle trait
3. Write tests for multi-component bundles
4. Implement tuple bundle support
5. Write tests for nested bundles
6. Implement bundle composition
7. Write tests for bundle queries
8. Ensure query compatibility

## Continuous Testing
- Run tests on every file save during development
- Use `cargo watch -x test` for automatic test running
- Profile test execution time
- Monitor test coverage with `cargo tarpaulin`

## Success Criteria
- All tests pass
- No performance regression (within 5% of original)
- Code coverage > 80%
- All edge cases handled
- Documentation includes test examples