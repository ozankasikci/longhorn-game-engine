# Phase 13.2: Scene View Refactoring Progress

## Goal
Refactor the 1,018-line `scene_view_impl.rs` into focused, maintainable modules.

## Current State
- Monolithic file mixing grid, mesh, entity, and sprite rendering
- Contains projection math, culling logic, and rendering orchestration
- Difficult to modify individual rendering systems

## Target Structure
```
crates/application/engine-editor-egui/src/panels/scene_view/
├── scene_view_impl.rs   # Main orchestrator (200 lines)
├── rendering/
│  ├── mod.rs       # Rendering module exports
│  ├── grid_renderer.rs  # Grid rendering logic
│  ├── mesh_renderer.rs  # 3D mesh rendering
│  ├── entity_renderer.rs # Generic entity rendering
│  ├── sprite_renderer.rs # 2D sprite rendering
│  └── projection.rs    # World-to-screen projection
├── camera/         # (existing)
├── interaction/      # (existing)
└── overlays/
  ├── mod.rs
  ├── scene_info.rs   # Scene statistics overlay
  └── debug_info.rs   # Debug information overlay
```

## Tasks Checklist

### Setup
- [ ] Create `rendering/` directory structure
- [ ] Create `overlays/` directory structure
- [ ] Plan module interfaces

### Grid Renderer Module
- [ ] Extract `draw_grid` method
- [ ] Move grid-specific imports
- [ ] Create `GridRenderer` struct if needed
- [ ] Extract grid configuration
- [ ] Add grid-specific tests

### Mesh Renderer Module
- [ ] Extract `render_mesh_entity_enhanced`
- [ ] Extract `render_3d_cube`
- [ ] Extract `render_enhanced_cube`
- [ ] Move mesh-specific rendering logic
- [ ] Create mesh rendering utilities

### Entity Renderer Module
- [ ] Extract `render_entity`
- [ ] Extract non-mesh entity rendering
- [ ] Move camera indicator rendering
- [ ] Consolidate entity type detection

### Sprite Renderer Module
- [ ] Extract `render_sprites`
- [ ] Move 2D sprite logic
- [ ] Add sprite batching preparation

### Projection Module
- [ ] Extract `world_to_screen`
- [ ] Extract `world_to_screen_enhanced`
- [ ] Create projection utilities
- [ ] Add clipping functions
- [ ] Document coordinate systems

### Scene Overlays
- [ ] Extract `draw_scene_overlay`
- [ ] Extract `draw_scene_camera_indicator`
- [ ] Create modular overlay system
- [ ] Add overlay configuration

### Main Refactoring
- [ ] Reduce `scene_view_impl.rs` to orchestrator
- [ ] Create clean module interfaces
- [ ] Update `draw_scene` to use modules
- [ ] Maintain performance characteristics

### Integration
- [ ] Update all imports
- [ ] Test each rendering component
- [ ] Verify visual output unchanged
- [ ] Profile performance
- [ ] Update documentation

## Progress Tracking
- **Started**: Not yet
- **Completed**: 0/8 modules
- **Tests Passing**: N/A
- **Visual Regression**: None
- **Performance Impact**: TBD

## Considerations
- Maintain rendering order (grid last for transparency)
- Keep performance optimizations
- Preserve culling logic
- Document coordinate system assumptions
- Consider future GPU-based rendering