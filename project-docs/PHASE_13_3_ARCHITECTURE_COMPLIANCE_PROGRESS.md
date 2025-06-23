# Phase 13.3: Architecture Compliance Progress

## Goal
Fix violations of the 4-tier architecture by moving implementation details out of core crates.

## Current Violations

### Core Crates with Implementation Details
1. **engine-camera-core**
  - `optimization.rs` (522 lines) - Should be in impl
  - `culling.rs` - Contains implementation, not just traits

2. **engine-resource-core**
  - `cache.rs` (502 lines) - Caching is implementation
  - `metadata.rs` (625 lines) - Too detailed for core
  - `loader.rs` (463 lines) - Loading logic is implementation

3. **engine-geometry-core**
  - `mesh.rs` (918 lines) - Too much implementation
  - `bounds.rs` (464 lines) - Complex calculations

## New Crates to Create

### engine-resource-impl
```
crates/implementation/engine-resource-impl/
├── Cargo.toml
├── README.md
├── src/
│  ├── lib.rs
│  ├── cache/
│  │  ├── mod.rs
│  │  ├── memory_cache.rs
│  │  └── disk_cache.rs
│  ├── loader/
│  │  ├── mod.rs
│  │  ├── async_loader.rs
│  │  └── sync_loader.rs
│  └── metadata/
│    ├── mod.rs
│    └── metadata_store.rs
```

### engine-math-impl
```
crates/implementation/engine-math-impl/
├── Cargo.toml
├── README.md
├── src/
│  ├── lib.rs
│  ├── simd/
│  │  ├── mod.rs
│  │  └── operations.rs
│  ├── geometry/
│  │  ├── mod.rs
│  │  └── algorithms.rs
│  └── interpolation/
│    ├── mod.rs
│    └── curves.rs
```

## Tasks Checklist

### Camera Core Cleanup
- [ ] Move `optimization.rs` to `engine-camera-impl`
- [ ] Extract culling implementations
- [ ] Leave only traits and basic types in core
- [ ] Update camera-impl dependencies
- [ ] Fix import paths

### Create engine-resource-impl
- [ ] Create new crate structure
- [ ] Move `cache.rs` from core
- [ ] Move implementation from `metadata.rs`
- [ ] Move loader implementations
- [ ] Keep only traits in core
- [ ] Add implementation tests

### Create engine-math-impl
- [ ] Create new crate structure
- [ ] Move complex geometry calculations
- [ ] Add SIMD optimizations
- [ ] Move interpolation implementations
- [ ] Platform-specific math (future)

### Geometry Core Cleanup
- [ ] Reduce `mesh.rs` to traits and basic types
- [ ] Move mesh building to geometry-impl
- [ ] Move bounds calculations to impl
- [ ] Keep only data structures in core

### Update Dependencies
- [ ] Update Cargo.toml files
- [ ] Fix circular dependencies
- [ ] Update all import paths
- [ ] Ensure proper layering

### Standardize Naming
- [ ] Rename `engine-assets` → `engine-assets-impl`
- [ ] Rename `engine-input` → `engine-input-impl`
- [ ] Rename `engine-platform` → `engine-platform-impl`
- [ ] Rename `engine-ui` → `engine-ui-impl`
- [ ] Update all references

## Progress Tracking
- **Started**: Not yet
- **Crates Created**: 0/2
- **Files Moved**: 0/6
- **Naming Fixed**: 0/4
- **Tests Passing**: N/A

## Architecture Rules

### Core Crates Should Only Contain:
- Trait definitions
- Basic data structures
- Constants and enums
- No implementation logic
- No dependencies on impl crates

### Implementation Crates Should:
- Depend on core traits
- Provide concrete implementations
- Handle platform-specific code
- Contain optimizations
- Include heavy algorithms

### Integration Crates Should:
- Wire together implementations
- Handle system coordination
- Manage runtime behavior
- Provide high-level APIs

## Validation
- [ ] Run architecture linter (create if needed)
- [ ] Verify no core→impl dependencies
- [ ] Check compilation times improved
- [ ] Ensure clean module boundaries
- [ ] Document architecture decisions