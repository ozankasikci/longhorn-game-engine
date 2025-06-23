# Phase 15: Standalone 3D Renderer Implementation

## Overview

After extensive investigation into GPU rendering issues with egui paint callbacks, we've determined that the best approach is to implement a standalone renderer as a separate component, following best practices from successful Rust game engines like Bevy and rend3.

## Duration

3-4 weeks

## Goals

1. Create a dedicated `engine-renderer-3d` crate
2. Implement texture-based rendering (render to texture, display in egui)
3. Establish clean separation between rendering and UI
4. Build a performant, maintainable renderer architecture

## Problem Statement

Current issues with the existing approach:
- Paint callbacks not executing properly in egui-wgpu integration
- Tight coupling between renderer and scene view UI
- No clear renderer abstraction
- Scattered rendering logic across multiple files

## Sub-phases

### Phase 15.1: Core Renderer Setup (1 week)
- [ ] Create `engine-renderer-3d` crate with proper structure
- [ ] Implement basic WGPU initialization and device management
- [ ] Create render pipeline for simple triangle/cube
- [ ] Set up render-to-texture pipeline
- [ ] Implement basic shader system (WGSL)

### Phase 15.2: Resource Management (3-4 days)
- [ ] Implement mesh buffer management system
- [ ] Create material system with uniform buffers
- [ ] Add texture loading and management
- [ ] Handle dynamic buffer updates efficiently
- [ ] Implement resource pooling

### Phase 15.3: Scene Integration (3-4 days)
- [ ] Bridge ECS world to render scene representation
- [ ] Implement render queue and sorting
- [ ] Add basic frustum culling
- [ ] Support multiple objects with transforms
- [ ] Create camera system with MVP matrices

### Phase 15.4: egui Integration (2-3 days)
- [ ] Implement texture-based rendering widget
- [ ] Create egui widget wrapper for display
- [ ] Handle input pass-through to renderer
- [ ] Add debug overlay support
- [ ] Ensure proper resize handling

### Phase 15.5: Lighting and Materials (3-4 days)
- [ ] Implement basic lighting (directional, point)
- [ ] Add Phong shading model
- [ ] Create material property system
- [ ] Support multiple material types
- [ ] Add shadow mapping (basic)

### Phase 15.6: Optimization and Polish (2-3 days)
- [ ] Implement batching for similar objects
- [ ] Add GPU timing and profiling
- [ ] Optimize draw call submission
- [ ] Add LOD support
- [ ] Performance testing and tuning

## Architecture Design

### Crate Structure
```
crates/implementation/engine-renderer-3d/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── renderer.rs         # Main renderer struct
│   ├── pipeline.rs         # Render pipeline management
│   ├── resources.rs        # GPU resource management
│   ├── mesh.rs            # Mesh data structures
│   ├── material.rs        # Material system
│   ├── camera.rs          # Camera matrices
│   ├── lighting.rs        # Lighting calculations
│   ├── scene.rs           # Scene representation
│   ├── frame.rs           # Frame rendering logic
│   └── integration/
│       ├── mod.rs
│       └── egui.rs        # egui-specific integration
```

### Key Components

1. **Renderer3D**: Main renderer managing WGPU resources
2. **RenderScene**: Intermediate representation of ECS world
3. **ResourceManager**: Handles GPU buffer allocation
4. **PipelineCache**: Manages render pipelines
5. **EguiRenderWidget**: Integration widget for editor

## Technical Approach

### Rendering Strategy
- **Retained Mode**: Objects persist between frames
- **Forward Rendering**: Simple, efficient for our use case
- **Texture Target**: Render to texture for egui display
- **Double Buffering**: Smooth frame presentation

### Integration Pattern
```rust
// In scene view
let scene = convert_world_to_render_scene(&world);
renderer.render(&scene);
let texture_id = renderer.get_texture_id();
ui.image(texture_id, available_size);
```

## Success Criteria

1. Stable 60 FPS rendering of 1000+ objects
2. Clean separation from UI code
3. No paint callback issues
4. Proper GPU resource management
5. Easy integration with existing editor

## Migration Strategy

1. Implement new renderer in parallel with existing code
2. Add feature flag to switch between renderers
3. Test thoroughly with simple scenes
4. Gradually migrate features
5. Remove old renderer code once stable

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Performance regression | Profile early and often |
| Integration complexity | Start with simple texture display |
| Platform differences | Test on all target platforms |
| Memory leaks | Use RAII and proper cleanup |

## Dependencies

- wgpu 0.20
- bytemuck 1.13
- glam 0.24
- egui 0.28
- egui-wgpu 0.28

## References

- [wgpu examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
- [Bevy renderer](https://github.com/bevyengine/bevy/tree/main/crates/bevy_render)
- [rend3](https://github.com/BVE-Reborn/rend3)

## Next Steps

1. Create the `engine-renderer-3d` crate
2. Implement minimal triangle rendering
3. Add render-to-texture support
4. Create egui integration widget
5. Test with simple cube scene