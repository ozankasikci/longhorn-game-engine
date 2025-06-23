# Phase 18: Transform Gizmos Implementation Plan

## Overview
Implement professional transform manipulation gizmos (handles) for position, rotation, and scale with proper 3D rendering, mouse interaction, and keyboard shortcuts.

## Goals
1. Create visually accurate 3D gizmos that render in world space
2. Implement proper mouse picking and dragging for all three transform modes
3. Add Unity-standard keyboard shortcuts (W: Move, E: Rotate, R: Scale)
4. Support axis constraints and plane manipulation
5. Include visual feedback for hover and active states

## Technical Architecture

### 1. Gizmo Rendering System
```rust
// In engine-renderer-3d/src/gizmo_3d.rs
pub struct GizmoRenderer3D {
  // Separate pipelines for each gizmo type
  translation_pipeline: wgpu::RenderPipeline,
  rotation_pipeline: wgpu::RenderPipeline,
  scale_pipeline: wgpu::RenderPipeline,
  
  // Vertex buffers for gizmo geometry
  arrow_mesh: GizmoMesh,   // For translation
  circle_mesh: GizmoMesh,   // For rotation
  box_mesh: GizmoMesh,    // For scale
  
  // Uniforms
  gizmo_uniforms: GizmoUniforms,
  
  // Interaction state
  hover_component: Option<GizmoComponent>,
  active_component: Option<GizmoComponent>,
}
```

### 2. Gizmo Components

#### Translation Gizmo (Move - W key)
- 3 arrows for X, Y, Z axes (Red, Green, Blue)
- 3 planes for XY, XZ, YZ constrained movement
- Center sphere for screen-space movement
- Visual: Arrows with cones at tips

#### Rotation Gizmo (Rotate - E key)
- 3 circles for pitch, yaw, roll (Red, Green, Blue)
- Outer circle for screen-space rotation
- Visual: Torus/circle meshes

#### Scale Gizmo (Scale - R key)
- 3 axes with boxes at ends (Red, Green, Blue)
- Center cube for uniform scaling
- Visual: Lines with cubes at tips

### 3. Mouse Interaction System

#### Ray Casting
```rust
pub struct GizmoRaycaster {
  pub fn screen_to_ray(mouse_pos: Vec2, camera: &Camera, viewport: Rect) -> Ray;
  pub fn ray_intersect_plane(ray: &Ray, plane_normal: Vec3, plane_point: Vec3) -> Option<f32>;
  pub fn ray_intersect_sphere(ray: &Ray, center: Vec3, radius: f32) -> Option<f32>;
  pub fn ray_intersect_cylinder(ray: &Ray, axis: Vec3, radius: f32) -> Option<f32>;
}
```

#### Interaction States
1. **Idle**: No interaction, gizmo at default colors
2. **Hover**: Component highlighted (brighter color)
3. **Active**: Component being dragged (yellow highlight)

### 4. Transform Calculation

#### Position Delta Calculation
```rust
fn calculate_translation_delta(
  drag_start: Vec3,
  current_mouse: Vec2,
  constraint: GizmoConstraint,
  camera: &Camera,
) -> Vec3 {
  match constraint {
    GizmoConstraint::Axis(axis) => {
      // Project mouse movement onto world axis
    },
    GizmoConstraint::Plane(normal) => {
      // Calculate intersection with plane
    },
    GizmoConstraint::Screen => {
      // Move parallel to camera plane
    },
  }
}
```

#### Rotation Delta Calculation
```rust
fn calculate_rotation_delta(
  drag_start: Vec3,
  current_mouse: Vec2,
  rotation_axis: Vec3,
  camera: &Camera,
) -> f32 {
  // Calculate angle based on mouse movement around axis
}
```

### 5. Shader Implementation

#### Gizmo Vertex Shader
```wgsl
struct GizmoUniforms {
  model: mat4x4<f32>,
  view_proj: mat4x4<f32>,
  gizmo_scale: f32, // For constant screen size
  highlight_color: vec4<f32>,
  camera_pos: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  // Scale gizmo based on distance to camera
  let world_pos = model * vec4<f32>(vertex.position, 1.0);
  let distance = length(camera_pos - world_pos.xyz);
  let scale = gizmo_scale * distance * 0.1; // Constant screen size
  
  // Apply scaling and transform
  let scaled_pos = vertex.position * scale;
  let final_pos = model * vec4<f32>(scaled_pos, 1.0);
  
  out.position = view_proj * final_pos;
  out.color = mix(vertex.color, highlight_color, highlight_factor);
}
```

## Implementation Steps

### Phase 18.1: 3D Gizmo Rendering (2 days)
1. Create gizmo mesh generators for arrows, circles, and boxes
2. Implement gizmo shader with constant screen-size scaling
3. Add gizmo rendering pass to main renderer
4. Support color highlighting for hover/active states

### Phase 18.2: Mouse Interaction (2 days)
1. Implement ray casting from screen to world
2. Add intersection tests for each gizmo component
3. Create dragging logic with proper constraints
4. Handle mouse state transitions (idle → hover → drag)

### Phase 18.3: Transform Integration (1 day)
1. Connect gizmo dragging to transform component updates
2. Add undo/redo support for transform changes
3. Implement snapping (hold Ctrl/Cmd)
4. Add visual feedback during manipulation

### Phase 18.4: Keyboard Shortcuts (1 day)
1. Implement W/E/R shortcuts for tool switching
2. Add Q for selection tool (no gizmo)
3. Support T for rect transform tool (2D UI)
4. Add visual indicators in toolbar

### Phase 18.5: Polish and Optimization (1 day)
1. Add gizmo size slider in viewport settings
2. Implement multi-selection support
3. Add pivot/center toggle for multi-selection
4. Optimize rendering with instancing

## Key Challenges

1. **Constant Screen Size**: Gizmos should maintain consistent size regardless of camera distance
2. **Depth Testing**: Gizmos should be visible through objects but still respect depth
3. **Precision**: Small mouse movements should allow precise adjustments
4. **Performance**: Gizmos should not impact scene rendering performance

## Success Criteria

1. Gizmos visually match industry-standard implementation
2. All three transform modes work correctly
3. Mouse interaction feels responsive and precise
4. Keyboard shortcuts work as expected
5. Performance impact is negligible
6. Multi-selection works properly

## References

- standard transform Tools: https://industry references/Manual/PositioningGameObjects.html
- Blender Gizmo System: https://docs.blender.org/manual/en/latest/editors/3dview/controls/gizmos.html
- Three.js TransformControls: https://threejs.org/examples/#misc_controls_transform

## Timeline
Total: 7 days
- Days 1-2: 3D Gizmo Rendering
- Days 3-4: Mouse Interaction
- Day 5: Transform Integration
- Day 6: Keyboard Shortcuts
- Day 7: Polish and Optimization