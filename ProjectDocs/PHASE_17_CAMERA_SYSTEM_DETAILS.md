# Phase 17: Camera System Implementation Details

## Current State Analysis

### Existing Camera Implementation
1. **Location**: `engine-renderer-3d/src/camera.rs`
   - Basic Camera struct with position, target, up vectors
   - Simple view/projection matrix calculations
   - Limited to look-at style camera

2. **Editor Integration**: `engine-editor-egui/src/panels/scene_view/`
   - Camera movement in `camera_movement.rs`
   - Navigation in `navigation.rs`
   - Currently tightly coupled to editor

3. **Issues**:
   - No proper camera component in ECS
   - Camera logic mixed with renderer
   - Limited camera types (only perspective)
   - No frustum culling implementation
   - Editor camera and game camera not properly separated

## Implementation Strategy

### 1. Mathematical Foundation
```rust
// Proper view matrix calculation
pub fn calculate_view_matrix(position: Vec3, rotation: Quat) -> Mat4 {
    let forward = rotation * Vec3::Z;
    let right = rotation * Vec3::X;
    let up = rotation * Vec3::Y;
    
    Mat4::look_at_rh(position, position + forward, up)
}

// Perspective projection with proper parameters
pub fn calculate_projection_matrix(
    fov_radians: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32
) -> Mat4 {
    Mat4::perspective_rh(fov_radians, aspect_ratio, near_plane, far_plane)
}
```

### 2. Component Architecture
```rust
// In engine-components-3d
#[derive(Component)]
pub struct Camera {
    pub projection_type: ProjectionType,
    pub fov_degrees: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub orthographic_size: f32,
    pub viewport: Viewport,
    pub render_target: Option<RenderTargetId>,
    pub priority: i32,
    pub active: bool,
}

#[derive(Component)]
pub struct MainCamera; // Tag component

pub enum ProjectionType {
    Perspective,
    Orthographic,
}
```

### 3. Camera Controller System
```rust
// In engine-camera-impl
pub trait CameraController: Send + Sync {
    fn update(
        &mut self,
        transform: &mut Transform,
        input: &InputState,
        delta_time: f32
    );
    
    fn get_config(&self) -> CameraControllerConfig;
}

pub struct FPSCameraController {
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
    pub sprint_multiplier: f32,
    pub smoothing: f32,
}

pub struct OrbitCameraController {
    pub target: Vec3,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub rotation_speed: f32,
}
```

### 4. Frustum Culling Implementation
```rust
pub struct Frustum {
    planes: [FrustumPlane; 6],
}

impl Frustum {
    pub fn from_view_projection(vp_matrix: &Mat4) -> Self {
        // Extract planes from VP matrix
        let planes = [
            extract_left_plane(vp_matrix),
            extract_right_plane(vp_matrix),
            extract_bottom_plane(vp_matrix),
            extract_top_plane(vp_matrix),
            extract_near_plane(vp_matrix),
            extract_far_plane(vp_matrix),
        ];
        
        Self { planes }
    }
    
    pub fn test_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }
    
    pub fn test_aabb(&self, aabb: &AABB) -> bool {
        // Optimized AABB test
        for plane in &self.planes {
            if plane.distance_to_aabb(aabb) < 0.0 {
                return false;
            }
        }
        true
    }
}
```

### 5. System Integration

#### Camera System (runs each frame)
```rust
pub fn camera_system(world: &World) {
    // Find active cameras
    let cameras = world.query::<(&Camera, &Transform)>()
        .filter(|(camera, _)| camera.active)
        .sorted_by_key(|(camera, _)| -camera.priority);
    
    // Update view and projection matrices
    for (entity, (camera, transform)) in cameras {
        let view_matrix = calculate_view_matrix(
            transform.position,
            transform.rotation
        );
        
        let projection_matrix = match camera.projection_type {
            ProjectionType::Perspective => {
                calculate_perspective_matrix(camera)
            }
            ProjectionType::Orthographic => {
                calculate_orthographic_matrix(camera)
            }
        };
        
        // Store matrices for renderer
        world.add_component(entity, ViewMatrix(view_matrix));
        world.add_component(entity, ProjectionMatrix(projection_matrix));
    }
}
```

#### Culling System (before rendering)
```rust
pub fn frustum_culling_system(
    world: &World,
    render_queue: &mut RenderQueue
) {
    // Get main camera frustum
    if let Some((_, (view, proj))) = world.query::<(&MainCamera, &ViewMatrix, &ProjectionMatrix)>().next() {
        let vp_matrix = proj.0 * view.0;
        let frustum = Frustum::from_view_projection(&vp_matrix);
        
        // Cull objects
        for (entity, (transform, bounds)) in world.query::<(&Transform, &Bounds)>() {
            let world_bounds = bounds.transform(transform);
            
            if frustum.test_aabb(&world_bounds) {
                render_queue.add_visible(entity);
            }
        }
    }
}
```

## Migration Plan

### Step 1: Create New Camera Components
1. Add Camera component to `engine-components-3d`
2. Add ViewMatrix and ProjectionMatrix components
3. Add Bounds component for culling

### Step 2: Implement Camera Controllers
1. Create controller trait in `engine-camera-impl`
2. Port existing editor camera logic to FPSCameraController
3. Add new controller types

### Step 3: Integrate with Renderer
1. Update `engine-renderer-3d` to use camera components
2. Implement frustum culling in render pipeline
3. Add debug visualization for frustums

### Step 4: Update Editor
1. Refactor scene view to use new camera system
2. Add camera component to editor camera entity
3. Support multiple viewports

### Step 5: Testing and Optimization
1. Benchmark frustum culling performance
2. Add unit tests for all matrix calculations
3. Test different camera controllers
4. Optimize with SIMD where appropriate

## Performance Considerations

1. **Matrix Caching**: Only recalculate when transform/camera changes
2. **Culling Granularity**: Balance between culling accuracy and overhead
3. **Spatial Partitioning**: Prepare for future octree integration
4. **SIMD Optimization**: Use for plane-sphere/AABB tests
5. **Multi-threading**: Cull in parallel for multiple cameras

## Debug Features

1. **Frustum Visualization**: Draw frustum lines in scene
2. **Culling Statistics**: Show objects culled/rendered
3. **Camera Info Overlay**: Display FOV, position, matrices
4. **Performance Metrics**: Frame time impact of culling
5. **Camera Path Visualization**: Show camera animation paths