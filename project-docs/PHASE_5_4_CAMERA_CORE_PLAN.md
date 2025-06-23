# Phase 5.4: Create Camera Core - Project Plan

## Project Overview

**Phase:** 5.4 - Camera Core Creation 
**Timeline:** 1-2 hours 
**Objective:** Create `engine-camera-core` with pure camera abstractions to complete the core architecture 
**Priority:** High - Completes the domain-driven core architecture (implementation updates deferred to later phase)

---

## Problem Analysis

### Current Camera Architecture Issues:
- ❌ **Mixed abstractions**: `engine-scene-core` has basic camera, `engine-camera` has advanced features
- ❌ **Implementation coupling**: Advanced camera features tied to specific ECS implementation
- ❌ **Missing core abstractions**: Frustum culling, mobile optimizations lack pure interfaces
- ❌ **Inconsistent patterns**: Camera doesn't follow the 4-tier architecture like other domains

### Current Camera Functionality Distribution:

**In `engine-scene-core` (Tier 1: Basic Abstractions):**
- Basic `Camera` struct
- Simple `CameraProjection` (Perspective/Orthographic)
- Basic `Viewport` 
- Camera matrix calculations

**In `engine-camera` (Tier 2: Implementation):**
- Advanced `CameraType` with mobile optimizations
- `Frustum` culling system
- ECS v2 integration (`CameraComponent`)
- Performance optimization flags
- Advanced viewport transformations

---

## Proposed Solution: Create `engine-camera-core`

### **New Tier 1: Core Camera Abstractions**
```
crates/core/engine-camera-core/  # Pure camera abstractions
├── src/
│  ├── camera.rs      # Advanced camera traits and interfaces
│  ├── projection.rs    # Advanced projection abstractions
│  ├── viewport.rs     # Advanced viewport management
│  ├── culling.rs     # Frustum culling abstractions
│  ├── optimization.rs   # Mobile optimization interfaces
│  └── lib.rs       # Module exports
└── Cargo.toml
```

### **Future Tier 2: Camera Implementation** (Later Phase)
```
crates/engine-camera/       # Concrete camera implementation
├── src/
│  ├── camera.rs     # Implements camera-core traits
│  ├── frustum.rs     # Concrete frustum culling
│  ├── mobile.rs     # Mobile-specific optimizations
│  └── ecs.rs       # ECS v2 integration
└── Cargo.toml       # Will depend on engine-camera-core
```
*Note: Implementation updates deferred to focus on completing core architecture*

---

## Architecture Integration

### **Dependency Flow:**
```
engine-camera-core (Tier 1)
├─ No dependencies (pure abstractions)
├─ Integrated with engine-scene-core for basic camera types
└─ Provides interfaces for advanced features

engine-camera (Tier 2)
├─ depends on → engine-camera-core (implements traits)
├─ depends on → engine-scene-core (basic camera integration)
├─ depends on → engine-core (ECS integration)
└─ Uses → glam, bytemuck (implementation libs)
```

### **Relationship with Scene-Core:**
- `engine-scene-core` - Basic camera for scene graphs
- `engine-camera-core` - Advanced camera abstractions
- `engine-camera` - Concrete implementation of both

---

## Implementation Plan

### **Step 1: Create `engine-camera-core` Structure** (15 minutes)
- Create directory structure
- Set up Cargo.toml with pure dependencies
- Create module structure

### **Step 2: Extract Camera Abstractions** (30 minutes)
- Move advanced camera traits from `engine-camera`
- Create pure culling interfaces
- Define mobile optimization abstractions
- Extract viewport management traits

### **Step 3: Advanced Projection System** (20 minutes)
- Extract advanced projection abstractions
- Create projection builder patterns
- Mobile-specific projection optimizations

### **Step 4: Culling Abstractions** (20 minutes)
- Extract frustum culling interfaces
- Create culling result types
- Define performance optimization traits

### **Step 5: Integration & Testing** (15 minutes)
- Add to workspace configuration
- Create comprehensive tests
- Verify compilation

---

## Key Abstractions to Extract

### **Camera Management:**
```rust
pub trait AdvancedCamera: Send + Sync {
  fn frustum_culling(&self) -> &dyn FrustumCuller;
  fn mobile_optimizations(&self) -> &dyn MobileOptimizer;
  fn performance_settings(&self) -> CameraPerformanceSettings;
}

pub trait FrustumCuller: Send + Sync {
  fn cull_objects(&self, objects: &[BoundingBox]) -> CullingResult;
  fn update_frustum(&mut self, view_proj: Mat4);
}

pub trait MobileOptimizer: Send + Sync {
  fn optimize_for_device(&mut self, device_info: &DeviceInfo);
  fn get_recommended_settings(&self) -> CameraSettings;
}
```

### **Advanced Projections:**
```rust
pub trait ProjectionBuilder: Send + Sync {
  fn build_perspective(&self, params: PerspectiveParams) -> ProjectionMatrix;
  fn build_orthographic(&self, params: OrthographicParams) -> ProjectionMatrix;
  fn build_mobile_optimized(&self, device: &DeviceInfo) -> ProjectionMatrix;
}
```

### **Performance Abstractions:**
```rust
pub struct CameraPerformanceSettings {
  pub frustum_culling_enabled: bool,
  pub lod_bias: f32,
  pub render_distance: f32,
  pub mobile_optimizations: bool,
}
```

---

## Benefits

### **Architectural Consistency:**
- ✅ Completes 4-tier architecture for camera domain
- ✅ Separates concerns (abstractions vs implementation)
- ✅ Enables multiple camera implementations

### **Mobile Performance:**
- ✅ Pure mobile optimization interfaces
- ✅ Device-agnostic performance abstractions
- ✅ Testable optimization strategies

### **Development Experience:**
- ✅ Clear separation of camera concerns
- ✅ Easier testing of camera algorithms
- ✅ Better documentation structure

### **Future Extensibility:**
- ✅ Easy to add new camera types
- ✅ Platform-specific implementations
- ✅ VR/AR camera support preparation

---

## Implementation Notes

### **Core Crate Dependencies:**
```toml
[dependencies]
glam = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
# NO implementation dependencies
```

### **Integration Points:**
- Basic camera integration with `engine-scene-core`
- Advanced features in `engine-camera-core`
- Concrete implementation in `engine-camera`

### **Testing Strategy:**
- Comprehensive trait testing in core
- Integration tests for camera implementations
- Mobile optimization benchmarks

---

## Success Criteria

- [x] `engine-camera-core` created with pure abstractions
- [x] All camera traits properly defined
- [x] Frustum culling interfaces extracted
- [x] Mobile optimization abstractions created
- [x] Comprehensive test coverage
- [x] Workspace compilation successful
- [x] Documentation complete

**Timeline:** 1-2 hours 
**Dependencies:** Completed Phase 5.1-5.3 
**Next Phase:** 6.0 - Implementation Layer Updates (Future)