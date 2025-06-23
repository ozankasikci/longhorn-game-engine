# Phase 18: Transform Gizmos Implementation Progress

## Current Status: Starting Implementation

### Completed
- [x] Basic gizmo system structure exists
- [x] Move tool selection in toolbar
- [x] Basic 2D gizmo hit testing

### In Progress
- [ ] 3D gizmo rendering system
- [ ] Proper ray casting for 3D interaction

### TODO
- [ ] Rotation gizmo implementation
- [ ] Scale gizmo implementation
- [ ] Keyboard shortcuts
- [ ] Visual polish and feedback

## Implementation Details

### 1. Current Gizmo System Analysis

The existing system has:
- `GizmoSystem` struct managing tool state
- `MoveGizmo` for translation only
- 2D hit testing (needs upgrade to 3D)
- Basic interaction states

Missing:
- 3D rendering of gizmos
- Rotation and scale tools
- Proper 3D ray casting
- Visual feedback states

### 2. 3D Gizmo Renderer Design

```rust
// New structure for 3D gizmo rendering
pub struct Gizmo3D {
  pub transform: Transform,
  pub mode: GizmoMode,
  pub highlighted_component: Option<GizmoComponent>,
  pub active_component: Option<GizmoComponent>,
  pub scale_factor: f32, // For constant screen size
}

pub enum GizmoMode {
  Translation,
  Rotation,
  Scale,
}

pub struct GizmoComponent3D {
  pub axis: Option<Axis>,
  pub plane: Option<Plane>,
  pub component_type: ComponentType,
}

pub enum ComponentType {
  Arrow,   // Translation arrows
  Circle,   // Rotation circles
  Box,    // Scale boxes
  Line,    // Connecting lines
  Center,   // Center shapes
}
```

### 3. Mesh Generation for Gizmos

#### Arrow Mesh (Translation)
```rust
fn generate_arrow_mesh(length: f32, radius: f32) -> MeshData {
  // Cylinder for shaft
  // Cone for tip
  // Total vertices: ~50
}
```

#### Circle Mesh (Rotation)
```rust
fn generate_circle_mesh(radius: f32, segments: u32) -> MeshData {
  // Torus or line loop
  // Total vertices: segments
}
```

#### Box Mesh (Scale)
```rust
fn generate_box_mesh(size: f32) -> MeshData {
  // Simple cube
  // Total vertices: 24
}
```

### 4. Ray Casting Implementation

```rust
pub fn screen_to_world_ray(
  mouse_pos: Vec2,
  camera: &Camera,
  viewport: &Viewport,
) -> Ray {
  // 1. Convert mouse to NDC (-1 to 1)
  let ndc_x = (mouse_pos.x / viewport.width) * 2.0 - 1.0;
  let ndc_y = 1.0 - (mouse_pos.y / viewport.height) * 2.0;
  
  // 2. Create near and far points
  let near = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
  let far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
  
  // 3. Unproject to world space
  let inv_view_proj = (camera.projection * camera.view).inverse();
  let near_world = inv_view_proj * near;
  let far_world = inv_view_proj * far;
  
  // 4. Create ray
  let origin = near_world.xyz() / near_world.w;
  let direction = (far_world.xyz() / far_world.w - origin).normalize();
  
  Ray { origin, direction }
}
```

### 5. Interaction Logic

#### Hit Testing Priority
1. Check center shapes first (highest priority)
2. Check planes (medium priority)
3. Check axes (lowest priority)

#### Drag Calculation
```rust
fn calculate_drag_delta(
  component: &GizmoComponent3D,
  ray_start: &Ray,
  ray_current: &Ray,
  gizmo_transform: &Transform,
) -> TransformDelta {
  match component.component_type {
    ComponentType::Arrow => {
      // Project onto axis line
    },
    ComponentType::Circle => {
      // Calculate rotation angle
    },
    ComponentType::Box => {
      // Calculate scale factor
    },
  }
}
```

## Next Steps

1. **Immediate (Today)**:
  - Create `gizmo_3d.rs` module in renderer
  - Implement arrow mesh generation
  - Add gizmo rendering to render pass

2. **Tomorrow**:
  - Implement ray casting system
  - Add 3D hit testing
  - Connect to transform updates

3. **This Week**:
  - Add rotation and scale modes
  - Implement keyboard shortcuts
  - Polish visual feedback

## Technical Decisions

1. **Rendering Approach**: Render gizmos in a separate pass after scene objects but before grid
2. **Depth Testing**: Use depth test but render with slight bias to appear on top
3. **Constant Size**: Scale gizmos based on camera distance in vertex shader
4. **Color Scheme**: Follow industry-standard standard (X=Red, Y=Green, Z=Blue)

## Known Issues

1. Current 2D hit testing needs complete replacement
2. No rotation or scale tools implemented yet
3. Missing visual feedback for hover states
4. No keyboard shortcut handling

## Performance Considerations

1. Use instanced rendering for repeated gizmo components
2. Frustum cull gizmos when not visible
3. LOD system for distant gizmos (simpler geometry)
4. Minimize state changes between gizmo draws