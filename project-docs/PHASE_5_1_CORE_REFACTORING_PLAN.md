# Phase 5.1.1: Core Graphics Refactoring Plan

## Project Overview

**Phase:** 5.1.1 - Core Graphics Domain Separation 
**Timeline:** 2-3 hours 
**Objective:** Refactor monolithic `engine-graphics-core` into focused domain-specific core crates 
**Trigger:** User feedback that "graphics" is too generic and should be separated into logical domains

---

## Problem Analysis

The current `engine-graphics-core` crate contains multiple distinct domains that should be separated:

### Current Mixed Domains in `engine-graphics-core`:
1. **Rendering abstractions** - Renderer trait, render commands, pipelines
2. **Mesh/geometry data** - Vertices, primitives, spatial math, bounding boxes
3. **Materials/shaders** - PBR materials, textures, shader definitions
4. **Scene management** - Lights, culling, render queues, cameras
5. **Color management** - Color spaces, conversions

### Issues with Current Approach:
- ❌ **Too generic**: "graphics" doesn't communicate specific responsibility
- ❌ **Mixed concerns**: Data structures mixed with abstractions
- ❌ **Unclear dependencies**: Hard to understand what depends on what
- ❌ **Testing complexity**: Hard to test individual domains in isolation

---

## Proposed Domain-Based Separation

### **Core Crate Structure:**
```
crates/core/
├── engine-renderer-core/   # Pure rendering abstractions & traits
├── engine-geometry-core/   # Mesh, primitives, spatial math
├── engine-materials-core/  # Materials, shaders, textures
├── engine-scene-core/    # Scene graph, lighting, culling, cameras
├── engine-audio-core/    # ✅ Already implemented
└── engine-physics-core/   # ✅ Already implemented
```

### **Dependency Flow:**
```
engine-scene-core
├─ depends on → engine-renderer-core (for RenderCommand)
├─ depends on → engine-geometry-core (for BoundingBox, Mesh)
└─ depends on → engine-materials-core (for Material)

engine-materials-core
└─ depends on → engine-renderer-core (for texture traits)

engine-geometry-core
└─ depends on → glam, serde (no internal dependencies)

engine-renderer-core 
└─ depends on → glam, serde (no internal dependencies)
```

---

## Detailed Breakdown

### **1. `engine-renderer-core`** (Pure rendering abstractions)

**Purpose:** Core rendering traits and abstractions with zero implementation dependencies

**Contents:**
- `renderer.rs` - Core Renderer trait, RenderCommand, RenderState
- `pipeline.rs` - Render pipeline abstractions
- `resources.rs` - Handle types, resource management traits
- `capabilities.rs` - Renderer capabilities and limits
- `errors.rs` - Rendering error types

**Dependencies:** `glam`, `serde`, `thiserror`

**Key Types:**
```rust
pub trait Renderer: Send + Sync {
  fn render(&mut self, commands: &[RenderCommand]) -> Result<()>;
  fn create_texture(&mut self, desc: &TextureDescriptor) -> TextureHandle;
  fn capabilities(&self) -> RendererCapabilities;
}

pub enum RenderCommand {
  DrawMesh { mesh: MeshHandle, material: MaterialHandle, transform: Mat4 },
  SetViewport { viewport: Viewport },
  BeginRenderPass { clear_color: Option<Color> },
}
```

### **2. `engine-geometry-core`** (Mesh and spatial data)

**Purpose:** Pure geometric data structures and spatial math

**Contents:**
- `mesh.rs` - Vertex, Mesh, BoundingBox (current implementation)
- `primitives.rs` - Geometric primitive generators
- `spatial.rs` - Spatial queries, ray casting, frustum culling
- `geometry.rs` - Geometric utilities and math

**Dependencies:** `glam`, `serde`, `bytemuck`

**Key Types:**
```rust
pub struct Mesh {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u32>,
  pub bounds: BoundingBox,
}

pub struct BoundingBox {
  pub min: Vec3,
  pub max: Vec3,
}

pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3,
}
```

### **3. `engine-materials-core`** (Materials and shaders)

**Purpose:** Material system and shader abstractions

**Contents:**
- `material.rs` - Material, PBR properties (current implementation)
- `shader.rs` - Shader, ShaderProgram, vertex layouts (current implementation)
- `texture.rs` - Texture descriptors and formats
- `color.rs` - Color management (current implementation)

**Dependencies:** `glam`, `serde`, `engine-renderer-core`

**Key Types:**
```rust
pub struct Material {
  pub albedo: Color,
  pub metallic: f32,
  pub roughness: f32,
  pub textures: MaterialTextures,
}

pub struct Shader {
  pub shader_type: ShaderType,
  pub source: ShaderSource,
  pub entry_point: String,
}
```

### **4. `engine-scene-core`** (Scene management)

**Purpose:** High-level scene organization and management

**Contents:**
- `scene.rs` - Scene, RenderObject (current implementation)
- `lighting.rs` - Light types, environment settings
- `camera.rs` - Camera abstractions, view matrices (current camera_traits.rs)
- `culling.rs` - Scene culling, render queues

**Dependencies:** `glam`, `serde`, `engine-renderer-core`, `engine-geometry-core`, `engine-materials-core`

**Key Types:**
```rust
pub struct Scene {
  pub objects: Vec<RenderObject>,
  pub lights: Vec<Light>,
  pub environment: Environment,
}

pub struct Camera {
  pub projection: CameraProjection,
  pub transform: CameraTransform,
}
```

---

## Implementation Plan

### **Phase 5.1.1a: Create New Core Crates** ⏱️ 45 minutes

#### Task 1: Create `engine-renderer-core` (15 minutes)
- [ ] Create crate structure with Cargo.toml
- [ ] Extract pure renderer abstractions from current graphics-core
- [ ] Move: `renderer.rs` (traits only), handle types, errors
- [ ] Remove all data structure dependencies

#### Task 2: Create `engine-geometry-core` (10 minutes) 
- [ ] Create crate structure with Cargo.toml
- [ ] Move: `mesh.rs`, `geometry.rs` (primitives, spatial math)
- [ ] Keep: All geometric data structures and algorithms
- [ ] Dependencies: glam, serde, bytemuck only

#### Task 3: Create `engine-materials-core` (10 minutes)
- [ ] Create crate structure with Cargo.toml 
- [ ] Move: `material.rs`, `shader.rs`, `color.rs`
- [ ] Add dependency on `engine-renderer-core` for texture handles
- [ ] Keep: All material and shader definitions

#### Task 4: Create `engine-scene-core` (10 minutes)
- [ ] Create crate structure with Cargo.toml
- [ ] Move: `scene.rs`, `camera_traits.rs` → `camera.rs`
- [ ] Add dependencies on other core crates
- [ ] Keep: High-level scene management

### **Phase 5.1.1b: Update Dependencies and Workspace** ⏱️ 30 minutes

#### Task 5: Update Workspace Configuration (10 minutes)
- [ ] Add new crates to workspace members
- [ ] Add workspace dependencies for new crates
- [ ] Remove old `engine-graphics-core` from workspace

#### Task 6: Update Existing Crate Dependencies (20 minutes)
- [ ] Update `engine-graphics` to depend on new core crates
- [ ] Update `engine-camera` to use `engine-scene-core`
- [ ] Update any other crates that used `engine-graphics-core`
- [ ] Verify all imports and re-exports work correctly

### **Phase 5.1.1c: Validation and Testing** ⏱️ 45 minutes

#### Task 7: Compile and Test (30 minutes)
- [ ] Verify all new core crates compile independently
- [ ] Verify existing crates compile with new dependencies
- [ ] Run basic functionality tests
- [ ] Check for circular dependencies

#### Task 8: Clean Up (15 minutes)
- [ ] Remove old `engine-graphics-core` crate
- [ ] Update documentation and comments
- [ ] Verify workspace builds successfully
- [ ] Update CLAUDE.md with new architecture

---

## Success Criteria

### **Technical Validation**
- [ ] **Independent Compilation**: All 4 new core crates compile with zero dependencies on each other (except declared dependencies)
- [ ] **Clear Separation**: Each crate has a single, well-defined responsibility
- [ ] **Dependency Flow**: Dependencies flow in one direction (scene → materials → renderer)
- [ ] **Zero Implementation Dependencies**: No wgpu, rodio, rapier, etc. in any core crate

### **Architectural Validation** 
- [ ] **Domain Clarity**: Each crate name clearly communicates its purpose
- [ ] **Testability**: Each domain can be tested in isolation
- [ ] **Reusability**: Core crates can be used independently in other projects
- [ ] **Extensibility**: Easy to add new implementations without changing core

### **Integration Validation**
- [ ] **Existing Code Works**: All current functionality continues to work
- [ ] **Editor Integration**: professional editor continues to function
- [ ] **Examples Function**: Multi-camera demo and other examples still work
- [ ] **Build Performance**: No significant increase in build times

---

## Risk Assessment

### **Medium Risk Areas**

1. **Circular Dependencies**
  - **Risk:** Scene depends on renderer, materials depend on renderer
  - **Mitigation:** Careful interface design, use of handle types instead of concrete types
  - **Contingency:** Merge problematic crates if circular dependencies can't be resolved

2. **Interface Complexity**
  - **Risk:** Too many small crates create integration complexity
  - **Mitigation:** Keep interfaces minimal and focused
  - **Contingency:** Merge related crates if interfaces become unwieldy

### **Low Risk Areas**

1. **Build Time Impact**
  - **Risk:** More crates might increase build time
  - **Mitigation:** Rust's incremental compilation handles this well
  - **Contingency:** Profile build times and optimize if needed

---

## Expected Outcomes

### **Immediate Benefits**
1. **Clear Naming**: `engine-renderer-core` clearly communicates rendering abstractions
2. **Better Testing**: Each domain can be tested independently with focused unit tests
3. **Reduced Coupling**: Cleaner separation between data structures and abstractions
4. **Easier Navigation**: Developers can find relevant code faster

### **Long-term Benefits**
1. **Modular Reuse**: Individual core crates can be used in other projects
2. **Parallel Development**: Teams can work on different domains independently
3. **Cleaner Implementations**: Implementation crates have clearer interfaces to implement
4. **Better Documentation**: Each crate can have focused, domain-specific documentation

---

**Status:** Ready for implementation 
**Next Phase:** Phase 5.2 - Implementation Separation (after completion) 
**Dependencies:** Completion of this phase before proceeding with WGPU/Rodio/Rapier implementation crates