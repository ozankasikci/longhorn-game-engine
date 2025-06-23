# Phase 5.5: Core Architecture Consolidation - Project Plan

## Project Overview

**Phase:** 5.5 - Core Architecture Consolidation 
**Timeline:** 30-45 minutes 
**Objective:** Move legacy `engine-core` to `crates/core/` and rename to `engine-ecs-core` for consistency 
**Priority:** High - Completes the core architecture reorganization

---

## Problem Analysis

### Current Architecture Issues:
- ❌ **Inconsistent location**: `engine-core` is in `crates/` while all other core crates are in `crates/core/`
- ❌ **Generic naming**: `engine-core` is too generic compared to domain-specific core crates
- ❌ **Mixed concerns**: Contains ECS, math, memory, and time - should be focused on ECS primarily
- ❌ **Dependency confusion**: Other crates depend on `engine-core` but it's not in the core directory

### Current Core Crate Status:
**✅ Already in `crates/core/` (Domain-specific):**
- `engine-audio-core` - Pure audio abstractions
- `engine-camera-core` - Pure camera abstractions 
- `engine-geometry-core` - Pure geometry abstractions
- `engine-materials-core` - Pure material abstractions
- `engine-physics-core` - Pure physics abstractions
- `engine-renderer-core` - Pure rendering abstractions
- `engine-scene-core` - Pure scene abstractions

**❌ Needs to be moved:**
- `engine-core` (in `crates/`) - Contains ECS, math, memory, time

---

## Proposed Solution: Move and Rename `engine-core`

### **Step 1: Rename and Move**
```
FROM: crates/engine-core/
TO:  crates/core/engine-ecs-core/
```

**Rationale for `engine-ecs-core` name:**
- More specific than generic `engine-core`
- Follows domain-specific naming pattern
- Reflects primary purpose (ECS v1 and v2 systems)
- Math utilities can be re-exported from glam

### **Step 2: Update Dependencies**
All crates currently depending on `engine-core` need to be updated:
- `engine-camera`
- `engine-editor-egui` 
- `engine-renderer-wgpu`
- `engine-runtime`
- `engine-ui`

---

## Implementation Plan

### **Task 1: Move and Rename Core** (10 minutes)
1. Copy `crates/engine-core/` to `crates/core/engine-ecs-core/`
2. Update `Cargo.toml` package name to `engine-ecs-core`
3. Verify content is properly copied

### **Task 2: Update Workspace Configuration** (5 minutes)
1. Remove `engine-core` from workspace members
2. Add `engine-ecs-core` to workspace members
3. Update workspace dependencies section

### **Task 3: Update Dependent Crates** (15 minutes)
Update `Cargo.toml` files in:
- `engine-camera/Cargo.toml`
- `engine-editor-egui/Cargo.toml`
- `engine-renderer-wgpu/Cargo.toml`
- `engine-runtime/Cargo.toml`
- `engine-ui/Cargo.toml`

Change: `engine-core = { workspace = true }` → `engine-ecs-core = { workspace = true }`

### **Task 4: Update Import Statements** (10 minutes)
Update Rust source files to import from `engine_ecs_core` instead of `engine_core`:
- Search and replace `use engine_core::` → `use engine_ecs_core::`
- Update any `extern crate` declarations

### **Task 5: Verification** (5 minutes)
1. Run `cargo check --workspace` to verify compilation
2. Run tests for affected crates
3. Verify no broken dependencies

### **Task 6: Cleanup** (2 minutes)
1. Remove old `crates/engine-core/` directory
2. Verify workspace is clean

---

## Files to Update

### **Workspace Configuration:**
- `Cargo.toml` (root)

### **Crate Dependencies:**
- `crates/engine-camera/Cargo.toml`
- `crates/engine-editor-egui/Cargo.toml`
- `crates/engine-renderer-wgpu/Cargo.toml`
- `crates/engine-runtime/Cargo.toml`
- `crates/engine-ui/Cargo.toml`

### **Source Files to Update:**
- `crates/engine-camera/src/*.rs`
- `crates/engine-editor-egui/src/*.rs`
- `crates/engine-renderer-wgpu/src/*.rs`
- `crates/engine-runtime/src/*.rs`
- `crates/engine-ui/src/*.rs`

---

## Benefits

### **Architectural Consistency:**
- ✅ All core abstractions in `crates/core/`
- ✅ Domain-specific naming throughout core layer
- ✅ Clear separation between core (abstractions) and implementation

### **Better Organization:**
- ✅ ECS functionality clearly identified as `engine-ecs-core`
- ✅ Easier to understand crate purposes
- ✅ Consistent with other core crate naming

### **Development Experience:**
- ✅ Predictable core crate locations
- ✅ Clearer dependency relationships
- ✅ Better IDE navigation and autocomplete

---

## Risk Mitigation

### **Compilation Issues:**
- Test compilation after each major change
- Keep backup of original structure until verified
- Update imports incrementally

### **Missing Dependencies:**
- Search for all references to `engine-core` before cleanup
- Verify all workspace members compile
- Check for any dynamic/string-based references

---

## Success Criteria

- [x] `engine-core` moved to `crates/core/engine-ecs-core/`
- [x] Package renamed to `engine-ecs-core` in Cargo.toml
- [x] Workspace configuration updated
- [x] All dependent crates updated to use `engine-ecs-core`
- [x] All import statements updated
- [x] Workspace compiles successfully
- [x] Tests pass for updated crates
- [x] Old `engine-core` directory removed

**Timeline:** 30-45 minutes 
**Dependencies:** Completed Phase 5.1-5.4 
**Next Phase:** 6.0 - Implementation Layer Updates

---

## Post-Completion State

After completion, the core architecture will be:

```
crates/core/
├── engine-audio-core/   # Audio abstractions
├── engine-camera-core/   # Camera abstractions 
├── engine-ecs-core/    # ECS, math, memory, time
├── engine-geometry-core/  # Geometry abstractions
├── engine-materials-core/ # Material abstractions
├── engine-physics-core/  # Physics abstractions
├── engine-renderer-core/  # Rendering abstractions
└── engine-scene-core/   # Scene abstractions
```

All core crates will be consistently located and named, providing a clean foundation for the implementation layer.