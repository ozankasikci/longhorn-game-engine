# Phase 5: 4-Tier Architecture Implementation Plan

## Project Overview

**Phase:** 5 - Architecture Separation of Concerns 
**Timeline:** 6-8 hours total (4 sub-phases) 
**Objective:** Implement 4-tier architecture separating pure domain logic from implementation details 
**Benefits:** Independent testing, swappable implementations, platform-specific optimizations

---

## 4-Tier Architecture Design

### **Tier 1: Core Abstractions** (Pure Domain Logic)
```
crates/core/
├── engine-core/      # ✅ ECS, math (already good)
├── engine-graphics-core/  # 🆕 Pure graphics concepts, traits
├── engine-audio-core/   # 🆕 Pure audio logic, DSP math
├── engine-physics-core/  # 🆕 Pure physics algorithms
├── engine-assets-core/   # 🆕 Asset system abstractions
└── engine-platform/    # ✅ Platform abstractions (already good)
```

### **Tier 2: Technology Implementations**
```
crates/implementations/
├── engine-renderer-wgpu/  # 🆕 WGPU-specific rendering
├── engine-audio-rodio/   # 🆕 Rodio-specific audio
├── engine-physics-rapier/ # 🆕 Rapier-specific physics
└── engine-assets-fs/    # 🆕 Filesystem asset loading
```

### **Tier 3: System Integration**
```
crates/integration/
├── engine-graphics-integration/ # 🆕 Graphics + Camera + ECS
├── engine-physics-integration/  # 🆕 Physics + ECS + Collision
└── engine-runtime/       # ✅ System orchestration (existing)
```

### **Tier 4: Applications**
```
apps/            # 🆕 Move from crates/
├── editor/         # 📱 professional editor
└── templates/       # 🆕 Game templates

examples/          # 🆕 Move from engine-graphics/examples/
├── basic-rendering/
├── multi-camera/
└── physics-demo/
```

---

## Implementation Phases

### **Phase 5.1: Core Extraction** ⏱️ 2-3 hours

**Objective:** Extract pure domain logic from mixed crates

#### Task 1.1: Create `engine-graphics-core` (60 minutes)
```rust
// crates/core/engine-graphics-core/src/lib.rs
pub mod mesh;      // Pure mesh definitions
pub mod material;    // Material system traits 
pub mod shader;     // Shader abstractions
pub mod geometry;    // Geometric primitives
pub mod color;     // Color management
pub mod camera_traits; // Camera system traits

// Core rendering trait
pub trait Renderer: Send + Sync {
  fn render(&mut self, scene: &Scene) -> Result<()>;
  fn resize(&mut self, width: u32, height: u32);
  fn create_texture(&mut self, data: &[u8]) -> TextureHandle;
}
```

**Extract from `engine-graphics`:**
- [ ] `mesh.rs` → Pure mesh definitions
- [ ] `materials.rs` → Material traits and types
- [ ] `shaders.rs` → Shader abstractions
- [ ] Add `Renderer` trait and core interfaces
- [ ] Move graphics math functions

**Dependencies:** `glam`, `serde`, `thiserror` (no wgpu!)

#### Task 1.2: Create `engine-audio-core` (45 minutes)
```rust
// crates/core/engine-audio-core/src/lib.rs
pub mod source;     // Audio source abstractions
pub mod mixer;     // Audio mixing logic
pub mod effects;    // Audio effects traits
pub mod spatial;    // 3D spatial audio

pub trait AudioSystem: Send + Sync {
  fn play_sound(&mut self, handle: SoundHandle) -> Result<()>;
  fn set_listener(&mut self, transform: Transform);
}
```

**Extract from `engine-audio`:**
- [ ] Pure audio processing logic
- [ ] DSP math functions 
- [ ] Audio system traits
- [ ] Spatial audio calculations

#### Task 1.3: Create `engine-physics-core` (45 minutes)
```rust
// crates/core/engine-physics-core/src/lib.rs
pub mod collision;   // Collision detection algorithms
pub mod dynamics;    // Physics simulation math
pub mod shapes;     // Collision shape definitions

pub trait PhysicsWorld: Send + Sync {
  fn step(&mut self, dt: f32);
  fn add_body(&mut self, body: RigidBody) -> BodyHandle;
  fn raycast(&self, ray: Ray) -> Option<RaycastHit>;
}
```

**Extract from `engine-physics`:**
- [ ] Pure collision detection algorithms
- [ ] Physics math and utilities
- [ ] Physics world traits
- [ ] Body and constraint abstractions

**Deliverables:**
- [ ] 3 new core crates compile independently
- [ ] Zero implementation dependencies (no wgpu, rodio, rapier)
- [ ] Comprehensive trait definitions
- [ ] Basic unit tests for pure logic

---

### **Phase 5.2: Implementation Separation** ⏱️ 2-3 hours

**Objective:** Create technology-specific implementation crates

#### Task 2.1: Create `engine-renderer-wgpu` (90 minutes)
```rust
// crates/implementations/engine-renderer-wgpu/src/lib.rs
use engine_graphics_core::{Renderer, Mesh, Material};
use wgpu::*;

pub struct WgpuRenderer {
  device: Device,
  queue: Queue,
  surface: Surface<'static>,
  // WGPU-specific implementation
}

impl Renderer for WgpuRenderer {
  fn render(&mut self, scene: &Scene) -> Result<()> {
    // WGPU-specific rendering pipeline
  }
}
```

**Move from `engine-graphics`:**
- [ ] `renderer.rs` → WGPU-specific implementation
- [ ] `multi_camera_renderer.rs` → WGPU multi-camera support
- [ ] `basic.wgsl` → WGPU shaders
- [ ] All WGPU dependencies and resource management

#### Task 2.2: Create `engine-audio-rodio` (45 minutes)
```rust
// crates/implementations/engine-audio-rodio/src/lib.rs
use engine_audio_core::AudioSystem;
use rodio::*;

pub struct RodioAudioSystem {
  device: OutputDevice,
  mixer: Mixer,
}

impl AudioSystem for RodioAudioSystem {
  fn play_sound(&mut self, handle: SoundHandle) -> Result<()> {
    // Rodio-specific audio playback
  }
}
```

**Move from `engine-audio`:**
- [ ] All Rodio-specific code
- [ ] Audio device management
- [ ] Audio streaming implementation

#### Task 2.3: Create `engine-physics-rapier` (45 minutes)
```rust
// crates/implementations/engine-physics-rapier/src/lib.rs
use engine_physics_core::PhysicsWorld;
use rapier2d::prelude::*;

pub struct RapierPhysicsWorld {
  rigid_body_set: RigidBodySet,
  collider_set: ColliderSet,
  physics_pipeline: PhysicsPipeline,
}

impl PhysicsWorld for RapierPhysicsWorld {
  fn step(&mut self, dt: f32) {
    // Rapier-specific physics simulation
  }
}
```

**Move from `engine-physics`:**
- [ ] All Rapier-specific code
- [ ] Physics world implementation
- [ ] Rapier integration utilities

**Deliverables:**
- [ ] 3 implementation crates successfully implement core traits
- [ ] All examples compile with new structure
- [ ] Zero reverse dependencies (core → implementation)

---

### **Phase 5.3: Integration Layer** ⏱️ 1-2 hours

**Objective:** Create system coordination and ECS integration

#### Task 3.1: Create `engine-graphics-integration` (60 minutes)
```rust
// crates/integration/engine-graphics-integration/src/lib.rs
use engine_core::{WorldV2, Read};
use engine_camera::CameraComponent;
use engine_graphics_core::Renderer;

pub struct GraphicsSystem<R: Renderer> {
  renderer: R,
}

impl<R: Renderer> GraphicsSystem<R> {
  pub fn render_world(&mut self, world: &WorldV2) -> Result<()> {
    // Extract cameras, meshes, transforms from ECS
    // Coordinate multiple systems
    // Call renderer with processed data
  }
}
```

**Responsibilities:**
- [ ] ECS queries for renderable entities
- [ ] Camera system coordination
- [ ] Multi-camera rendering logic
- [ ] Render pass management

#### Task 3.2: Update existing integration (30 minutes)
- [ ] Move multi-camera demo integration logic
- [ ] Update `engine-runtime` for new architecture
- [ ] Fix cross-system communication

**Deliverables:**
- [ ] Integration layers coordinate multiple systems
- [ ] ECS integration maintains performance
- [ ] Multi-camera demo works with new architecture

---

### **Phase 5.4: Application Reorganization** ⏱️ 1 hour

**Objective:** Move applications and examples to appropriate locations

#### Task 4.1: Reorganize Directory Structure (30 minutes)
```bash
# Create new directory structure
mkdir -p apps/editor
mkdir -p examples/{basic-rendering,multi-camera,physics-demo}

# Move applications
mv crates/engine-editor-egui apps/editor

# Move examples 
mv crates/engine-graphics/examples/* examples/
```

#### Task 4.2: Update Dependencies and Workspace (30 minutes)
```toml
# Cargo.toml
[workspace]
members = [
  # Core crates
  "crates/core/engine-core",
  "crates/core/engine-graphics-core",
  "crates/core/engine-audio-core", 
  "crates/core/engine-physics-core",
  
  # Implementation crates
  "crates/implementations/engine-renderer-wgpu",
  "crates/implementations/engine-audio-rodio",
  "crates/implementations/engine-physics-rapier",
  
  # Integration crates
  "crates/integration/engine-graphics-integration",
  "crates/integration/engine-runtime",
  
  # Applications
  "apps/editor",
]

[features]
default = ["wgpu-renderer", "rodio-audio", "rapier-physics"]
wgpu-renderer = ["engine-renderer-wgpu"]
rodio-audio = ["engine-audio-rodio"]
rapier-physics = ["engine-physics-rapier"]
```

**Deliverables:**
- [ ] Clean directory structure
- [ ] All applications compile and run
- [ ] Examples demonstrate new architecture
- [ ] Workspace builds with all feature combinations

---

## Success Metrics

### **Technical Validation**
- [ ] **Core Crates Independence:** Zero implementation dependencies
- [ ] **Unit Test Coverage:** 100% coverage for core logic using mocks
- [ ] **Integration Tests:** All pass with real implementations
- [ ] **Performance:** <1% overhead vs direct implementation
- [ ] **Build Matrix:** All feature combinations build successfully

### **Architectural Validation**
- [ ] **Swappable Implementations:** Can change renderer without integration code changes
- [ ] **Platform Builds:** Mobile vs desktop select appropriate implementations
- [ ] **Testability:** Core logic testable without heavy dependencies
- [ ] **Maintainability:** Clear boundaries and single responsibilities

### **User Experience**
- [ ] **Editor Functionality:** All editor features work with new architecture
- [ ] **Example Quality:** Examples clearly demonstrate each tier
- [ ] **Developer Experience:** Easy to add new implementations
- [ ] **Documentation:** Clear usage patterns and best practices

---

## Risk Mitigation

### **High-Risk Areas**
1. **Performance Regression** 
  - **Mitigation:** Comprehensive benchmarking, zero-cost abstractions
  - **Contingency:** Roll back if targets not met

2. **Integration Complexity**
  - **Mitigation:** Start simple, add complexity incrementally 
  - **Contingency:** Simplify interfaces if needed

### **Medium-Risk Areas**
1. **Build System Complexity**
  - **Mitigation:** Use proven feature management patterns
  - **Contingency:** Reduce feature combinations

2. **Testing Overhead**
  - **Mitigation:** Invest in good mock implementations
  - **Contingency:** Focus on integration tests if unit tests become unwieldy

---

## Next Steps

### **Immediate Action**
1. **Start Phase 5.1:** Begin with `engine-graphics-core` extraction
2. **Validate Approach:** Ensure core extraction works before proceeding
3. **Iterate:** Complete each phase before moving to next

### **Dependencies**
- **Phase 5.1 → 5.2:** Core traits must be defined before implementations
- **Phase 5.2 → 5.3:** Implementations must exist before integration
- **Phase 5.3 → 5.4:** Integration must work before reorganization

### **Quality Gates**
- **Phase 5.1:** All core crates compile independently
- **Phase 5.2:** Examples work with new implementations 
- **Phase 5.3:** Multi-camera demo functional
- **Phase 5.4:** Complete workspace builds successfully

---

**Phase 5 Status:** Ready for implementation 
**Expected Duration:** 6-8 hours total 
**Expected Outcome:** Production-ready modular architecture with independent testing and swappable implementations