# Phase 5.6: Implementation Tier Organization - Project Plan

## Project Overview

**Phase:** 5.6 - Implementation Tier Organization  
**Timeline:** 45-60 minutes  
**Objective:** Organize remaining crates into proper 4-tier architecture (Core → Implementation → Integration → Application)  
**Priority:** High - Completes the architectural reorganization started in Phase 5

---

## Problem Analysis

### Current Architecture Issues:
- ❌ **Mixed tiers**: Implementation and integration crates are mixed in the same directory
- ❌ **Inconsistent organization**: No clear separation between implementation layers
- ❌ **Missing structure**: Implementation crates should be in `crates/implementation/`
- ❌ **Integration unclear**: Editor and runtime should be in higher-tier directories

### Current Crate Distribution Analysis:

**✅ Already in Core (Tier 1):**
```
crates/core/
├── engine-audio-core/      # Audio abstractions
├── engine-camera-core/     # Camera abstractions  
├── engine-ecs-core/        # ECS, math, memory, time
├── engine-geometry-core/   # Geometry abstractions
├── engine-materials-core/  # Material abstractions
├── engine-physics-core/    # Physics abstractions
├── engine-renderer-core/   # Rendering abstractions
└── engine-scene-core/      # Scene abstractions
```

**❌ Needs to be moved to Implementation (Tier 2):**
```
crates/engine-audio/         → crates/implementation/engine-audio/
crates/engine-camera/        → crates/implementation/engine-camera/
crates/engine-physics/       → crates/implementation/engine-physics/
crates/engine-input/         → crates/implementation/engine-input/
crates/engine-assets/        → crates/implementation/engine-assets/
crates/engine-scripting/     → crates/implementation/engine-scripting/
crates/engine-ui/            → crates/implementation/engine-ui/
crates/engine-platform/     → crates/implementation/engine-platform/
crates/engine-renderer-wgpu/ → crates/implementation/engine-renderer-wgpu/
```

**❌ Needs to be moved to Integration (Tier 3):**
```
crates/engine-runtime/       → crates/integration/engine-runtime/
```

**❌ Needs to be moved to Application (Tier 4):**
```
crates/engine-editor-egui/   → crates/application/engine-editor-egui/
```

---

## Proposed 4-Tier Architecture

### **Final Target Structure:**
```
crates/
├── core/                    # Tier 1: Pure abstractions
│   ├── engine-audio-core/
│   ├── engine-camera-core/
│   ├── engine-ecs-core/
│   ├── engine-geometry-core/
│   ├── engine-materials-core/
│   ├── engine-physics-core/
│   ├── engine-renderer-core/
│   └── engine-scene-core/
│
├── implementation/          # Tier 2: Concrete implementations
│   ├── engine-audio/
│   ├── engine-camera/
│   ├── engine-physics/
│   ├── engine-input/
│   ├── engine-assets/
│   ├── engine-scripting/
│   ├── engine-ui/
│   ├── engine-platform/
│   └── engine-renderer-wgpu/
│
├── integration/             # Tier 3: System integration
│   └── engine-runtime/
│
└── application/             # Tier 4: End-user applications
    └── engine-editor-egui/
```

---

## Implementation Plan

### **Step 1: Create Directory Structure** (5 minutes)
```bash
mkdir -p crates/implementation
mkdir -p crates/integration  
mkdir -p crates/application
```

### **Step 2: Move Implementation Crates** (20 minutes)
Move 9 implementation crates:
1. `engine-audio` → `crates/implementation/engine-audio/`
2. `engine-camera` → `crates/implementation/engine-camera/`
3. `engine-physics` → `crates/implementation/engine-physics/`
4. `engine-input` → `crates/implementation/engine-input/`
5. `engine-assets` → `crates/implementation/engine-assets/`
6. `engine-scripting` → `crates/implementation/engine-scripting/`
7. `engine-ui` → `crates/implementation/engine-ui/`
8. `engine-platform` → `crates/implementation/engine-platform/`
9. `engine-renderer-wgpu` → `crates/implementation/engine-renderer-wgpu/`

### **Step 3: Move Integration Crate** (5 minutes)
Move integration crate:
1. `engine-runtime` → `crates/integration/engine-runtime/`

### **Step 4: Move Application Crate** (5 minutes)
Move application crate:
1. `engine-editor-egui` → `crates/application/engine-editor-egui/`

### **Step 5: Update Workspace Configuration** (10 minutes)
Update `Cargo.toml` workspace members to reflect new paths:
- Update all workspace member paths
- Update workspace dependency paths
- Verify no broken references

### **Step 6: Verification** (5 minutes)
1. Run `cargo check --workspace` to verify compilation
2. Verify directory structure is correct
3. Test example builds if needed

---

## Files to Update

### **Root Workspace Configuration:**
- `Cargo.toml` - Update workspace members and dependencies

### **No Source File Changes Needed:**
- Since we're only moving directories (not renaming crates), no import statements need updating
- Package names remain the same
- Only workspace configuration paths change

---

## Detailed Workspace Updates

### **Update Workspace Members:**
```toml
# FROM:
members = [
    "crates/engine-renderer-wgpu", 
    "crates/engine-audio",
    "crates/engine-physics",
    # ... etc
]

# TO:
members = [
    # Core abstractions (Tier 1)
    "crates/core/engine-audio-core",
    "crates/core/engine-ecs-core", 
    "crates/core/engine-physics-core",
    "crates/core/engine-renderer-core",
    "crates/core/engine-geometry-core",
    "crates/core/engine-materials-core",
    "crates/core/engine-scene-core",
    "crates/core/engine-camera-core",
    
    # Implementation layer (Tier 2)
    "crates/implementation/engine-audio",
    "crates/implementation/engine-camera",
    "crates/implementation/engine-physics",
    "crates/implementation/engine-input",
    "crates/implementation/engine-assets",
    "crates/implementation/engine-scripting",
    "crates/implementation/engine-ui",
    "crates/implementation/engine-platform",
    "crates/implementation/engine-renderer-wgpu",
    
    # Integration layer (Tier 3)
    "crates/integration/engine-runtime",
    
    # Application layer (Tier 4)
    "crates/application/engine-editor-egui",
]
```

### **Update Workspace Dependencies:**
```toml
# Update paths for workspace dependencies
[workspace.dependencies.engine-renderer]
path = "crates/implementation/engine-renderer-wgpu"

[workspace.dependencies.engine-audio]
path = "crates/implementation/engine-audio"

# ... etc for all moved crates
```

---

## Benefits

### **Architectural Clarity:**
- ✅ Clear 4-tier separation (Core → Implementation → Integration → Application)
- ✅ Predictable crate locations based on responsibility
- ✅ Easier to understand dependency flow
- ✅ Consistent with industry best practices

### **Development Experience:**
- ✅ Easier to find crates by purpose
- ✅ Clear separation of concerns
- ✅ Better IDE navigation and project structure
- ✅ Reduced cognitive load for new developers

### **Maintenance Benefits:**
- ✅ Logical grouping for code organization
- ✅ Easier to manage dependencies between tiers
- ✅ Clear upgrade paths (Core → Implementation → Integration → Application)
- ✅ Better isolation for testing and development

---

## Risk Mitigation

### **Compilation Issues:**
- Only workspace paths change, not package names or imports
- Test compilation after each major move
- Keep backup of workspace structure

### **Missing References:**
- Systematically update all workspace member paths
- Verify all workspace dependency paths
- Check for any hardcoded path references

---

## Success Criteria

- [x] `crates/implementation/` directory created with 9 crates
- [x] `crates/integration/` directory created with 1 crate  
- [x] `crates/application/` directory created with 1 crate
- [x] All workspace member paths updated
- [x] All workspace dependency paths updated
- [x] Workspace compiles successfully
- [x] Directory structure follows 4-tier architecture
- [x] Old crate directories cleaned up

**Timeline:** 45-60 minutes  
**Dependencies:** Completed Phase 5.1-5.5  
**Next Phase:** 6.0 - Implementation Layer Updates

---

## Post-Completion State

After completion, the workspace will have a clean 4-tier architecture:

```
crates/
├── core/           # 8 core abstraction crates
├── implementation/ # 9 concrete implementation crates  
├── integration/    # 1 system integration crate
└── application/    # 1 end-user application crate
```

This provides a professional, scalable foundation for the mobile game engine with clear separation of concerns and predictable organization patterns.