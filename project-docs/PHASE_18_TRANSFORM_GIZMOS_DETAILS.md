# Phase 18: Transform Gizmos Technical Details

## Ray Casting and Mouse Interaction

### Screen to World Ray Conversion
```rust
pub fn screen_to_world_ray(
  screen_pos: Vec2,
  camera: &Camera,
  viewport_size: Vec2,
) -> Ray {
  // 1. Convert to NDC coordinates (-1 to 1)
  let ndc = Vec2::new(
    (screen_pos.x / viewport_size.x) * 2.0 - 1.0,
    1.0 - (screen_pos.y / viewport_size.y) * 2.0, // Y is inverted
  );
  
  // 2. Create ray in clip space
  let ray_clip = Vec4::new(ndc.x, ndc.y, -1.0, 1.0);
  
  // 3. Convert to eye space
  let ray_eye = camera.projection_matrix.inverse() * ray_clip;
  let ray_eye = Vec4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);
  
  // 4. Convert to world space
  let ray_world = (camera.view_matrix.inverse() * ray_eye).xyz();
  let ray_direction = ray_world.normalize();
  
  Ray {
    origin: camera.position,
    direction: ray_direction,
  }
}
```

### Axis-Aligned Plane Projection
When dragging along an axis, we need to project mouse movement onto a plane that contains the axis but faces the camera:

```rust
fn create_axis_plane(axis: Vec3, camera_pos: Vec3, gizmo_pos: Vec3) -> (Vec3, Vec3) {
  let view_dir = (camera_pos - gizmo_pos).normalize();
  
  // Create a plane normal that's perpendicular to both axis and view
  let plane_normal = axis.cross(view_dir).cross(axis).normalize();
  
  // If axis is parallel to view, use alternate calculation
  if plane_normal.length() < 0.001 {
    // Use camera's up or right vector
    plane_normal = axis.cross(camera.up).normalize();
  }
  
  (gizmo_pos, plane_normal)
}
```

### Intersection Tests

#### Ray-Cylinder Intersection (for axes)
```rust
fn ray_cylinder_intersection(
  ray: &Ray,
  cylinder_start: Vec3,
  cylinder_end: Vec3,
  radius: f32,
) -> Option<f32> {
  let cylinder_axis = (cylinder_end - cylinder_start).normalize();
  let rc = ray.origin - cylinder_start;
  
  let d = ray.direction - cylinder_axis * ray.direction.dot(cylinder_axis);
  let rc_perp = rc - cylinder_axis * rc.dot(cylinder_axis);
  
  let a = d.dot(d);
  let b = 2.0 * d.dot(rc_perp);
  let c = rc_perp.dot(rc_perp) - radius * radius;
  
  let discriminant = b * b - 4.0 * a * c;
  if discriminant < 0.0 {
    return None;
  }
  
  let t = (-b - discriminant.sqrt()) / (2.0 * a);
  
  // Check if intersection is within cylinder bounds
  let p = ray.at(t);
  let projection = (p - cylinder_start).dot(cylinder_axis);
  
  if projection >= 0.0 && projection <= (cylinder_end - cylinder_start).length() {
    Some(t)
  } else {
    None
  }
}
```

#### Ray-Torus Intersection (for rotation circles)
```rust
fn ray_torus_intersection(
  ray: &Ray,
  center: Vec3,
  axis: Vec3,
  major_radius: f32,
  minor_radius: f32,
) -> Option<f32> {
  // Complex but well-documented algorithm
  // See: https://www.cl.cam.ac.uk/teaching/1999/AGraphHCI/SMAG/node2.html
}
```

## Constant Screen Size Implementation

### Vertex Shader Approach
```wgsl
struct GizmoUniforms {
  model: mat4x4<f32>,
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  gizmo_position: vec3<f32>,
  gizmo_screen_size: f32, // Desired size in pixels
  viewport_height: f32,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  // Calculate view space position of gizmo center
  let view_pos = uniforms.view * vec4<f32>(uniforms.gizmo_position, 1.0);
  let distance = length(view_pos.xyz);
  
  // Calculate scale factor to maintain constant screen size
  // This accounts for perspective projection
  let fov_scale = 2.0 * tan(radians(60.0) / 2.0); // Assuming 60 degree FOV
  let scale = (uniforms.gizmo_screen_size / uniforms.viewport_height) * distance * fov_scale;
  
  // Apply scale to vertex position
  let scaled_position = vertex.position * scale;
  
  // Transform to world space
  let world_pos = uniforms.model * vec4<f32>(scaled_position, 1.0);
  
  // Final transformation
  output.position = uniforms.projection * uniforms.view * world_pos;
  output.color = vertex.color;
  
  return output;
}
```

## Drag Delta Calculations

### Translation Mode
```rust
fn calculate_translation_delta(
  component: GizmoComponent,
  drag_start_world: Vec3,
  current_ray: &Ray,
  axis_plane: (Vec3, Vec3),
) -> Vec3 {
  // Intersect current ray with the axis plane
  let t = ray_plane_intersection(current_ray, axis_plane.0, axis_plane.1)?;
  let current_world = current_ray.at(t);
  
  // Calculate delta
  let delta = current_world - drag_start_world;
  
  // Constrain to axis if needed
  match component {
    GizmoComponent::AxisX => Vec3::new(delta.x, 0.0, 0.0),
    GizmoComponent::AxisY => Vec3::new(0.0, delta.y, 0.0),
    GizmoComponent::AxisZ => Vec3::new(0.0, 0.0, delta.z),
    GizmoComponent::PlaneXY => Vec3::new(delta.x, delta.y, 0.0),
    GizmoComponent::PlaneXZ => Vec3::new(delta.x, 0.0, delta.z),
    GizmoComponent::PlaneYZ => Vec3::new(0.0, delta.y, delta.z),
    GizmoComponent::Center => delta, // Free movement
  }
}
```

### Rotation Mode
```rust
fn calculate_rotation_delta(
  axis: Vec3,
  drag_start: Vec2,
  current_pos: Vec2,
  gizmo_screen_center: Vec2,
) -> f32 {
  // Calculate angles from screen center
  let start_angle = (drag_start - gizmo_screen_center).angle();
  let current_angle = (current_pos - gizmo_screen_center).angle();
  
  // Return angle difference
  current_angle - start_angle
}
```

### Scale Mode
```rust
fn calculate_scale_delta(
  component: GizmoComponent,
  drag_start: Vec2,
  current_pos: Vec2,
  gizmo_screen_center: Vec2,
) -> Vec3 {
  // Calculate distance change from center
  let start_dist = (drag_start - gizmo_screen_center).length();
  let current_dist = (current_pos - gizmo_screen_center).length();
  
  let scale_factor = current_dist / start_dist;
  
  match component {
    GizmoComponent::AxisX => Vec3::new(scale_factor, 1.0, 1.0),
    GizmoComponent::AxisY => Vec3::new(1.0, scale_factor, 1.0),
    GizmoComponent::AxisZ => Vec3::new(1.0, 1.0, scale_factor),
    GizmoComponent::Center => Vec3::splat(scale_factor), // Uniform scale
  }
}
```

## Visual Feedback States

### Color Definitions
```rust
const AXIS_COLORS: [Vec3; 3] = [
  Vec3::new(1.0, 0.2, 0.2), // X - Red
  Vec3::new(0.2, 1.0, 0.2), // Y - Green
  Vec3::new(0.2, 0.2, 1.0), // Z - Blue
];

const HOVER_COLOR: Vec3 = Vec3::new(1.0, 1.0, 0.2);   // Yellow
const ACTIVE_COLOR: Vec3 = Vec3::new(1.0, 1.0, 0.0);  // Bright Yellow
const INACTIVE_ALPHA: f32 = 0.6;             // Transparency when not hovered
```

### State Management
```rust
pub enum GizmoState {
  Idle,
  Hovered(GizmoComponent),
  Dragging {
    component: GizmoComponent,
    start_pos: Vec3,
    constraint_plane: (Vec3, Vec3),
  },
}
```

## Keyboard Shortcut Implementation

### Input Handling
```rust
fn handle_gizmo_shortcuts(input: &Input) -> Option<GizmoMode> {
  if input.just_pressed(Key::Q) {
    Some(GizmoMode::Select) // No gizmo
  } else if input.just_pressed(Key::W) {
    Some(GizmoMode::Translate)
  } else if input.just_pressed(Key::E) {
    Some(GizmoMode::Rotate)
  } else if input.just_pressed(Key::R) {
    Some(GizmoMode::Scale)
  } else {
    None
  }
}

// Modifier keys
fn get_snap_settings(input: &Input) -> SnapSettings {
  SnapSettings {
    enabled: input.is_pressed(Key::Control),
    position_snap: 0.5, // Units
    rotation_snap: 15.0, // Degrees
    scale_snap: 0.1,   // Factor
  }
}
```

## Performance Optimizations

1. **Frustum Culling**: Skip gizmo rendering if outside view
2. **LOD System**: Simpler geometry at distance
3. **Instanced Rendering**: For multiple selected objects
4. **Depth Pre-pass**: Render gizmos after opaque geometry
5. **State Batching**: Group gizmo components by type

## References
- industry-standard TransformGizmo source analysis
- Three.js TransformControls implementation
- Blender's transform manipulator code