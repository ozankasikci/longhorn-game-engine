# Camera Integration Plan: Making Camera Work in modern game editor

## Current State Analysis

### ✅ What's Already Working
1. **Camera Components Available**:
  - Basic `Camera` component (from engine-ecs-core)
  - `Camera2D` component (from engine-ecs-core) 
  - Advanced `CameraComponent` (from engine-camera-core)

2. **Editor Integration**:
  - Camera components can be added to entities
  - Camera properties displayed in Inspector panel
  - Camera entities visible in Hierarchy panel
  - Default camera entity spawned on startup

3. **Core Architecture**:
  - `engine-camera-core` provides advanced camera abstractions
  - `engine-renderer-core` provides rendering pipeline interfaces
  - ECS v2 system for component management

### ❌ What's Missing
1. **No Visual Camera Rendering**: Scene View panel shows empty space
2. **No Camera-to-Renderer Connection**: Camera data not fed to renderer
3. **No Scene Geometry**: Nothing for camera to render
4. **No Viewport Integration**: Camera viewport not connected to Scene View panel

## Implementation Plan

### Phase 1: Basic Scene View Rendering (1-2 hours)
**Objective**: Get a simple rendered scene visible in Scene View panel

#### Task 1.1: Create Basic Scene Geometry
- Add simple 3D primitives (cube, sphere, plane) to scene
- Use `engine-geometry-core` for mesh data
- Create default materials with `engine-materials-core`

#### Task 1.2: Connect Camera to Scene View Panel
- Integrate camera viewport with Scene View egui panel 
- Set up render surface for Scene View
- Connect camera projection matrices to rendering

#### Task 1.3: Implement Basic Renderer
- Create minimal WGPU renderer using `engine-renderer-wgpu`
- Implement basic vertex/fragment shaders
- Render simple colored geometry

### Phase 2: Camera Controls (30-45 minutes)
**Objective**: Interactive camera movement in Scene View

#### Task 2.1: Camera Movement
- Implement orbit controls (rotate around point)
- Add pan and zoom functionality
- WASD + mouse controls for free-look mode

#### Task 2.2: Camera Frustum Visualization
- Show camera frustum bounds in Scene View
- Visual indicators for near/far planes
- FOV visualization for perspective cameras

### Phase 3: Inspector Integration (30 minutes)
**Objective**: Live camera property editing

#### Task 3.1: Real-time Camera Updates
- Update camera rendering when properties change in Inspector
- Live preview of FOV, near/far plane changes
- Viewport size adjustments

#### Task 3.2: Camera Selection in Scene View
- Click to select camera entities in Scene View
- Visual camera gizmos for selection
- Camera icon rendering in 3D space

### Phase 4: Enhanced Features (45 minutes)
**Objective**: Professional camera workflow

#### Task 4.1: Multiple Camera Support
- Switch between different cameras in scene
- Camera priority system
- Picture-in-picture for multiple camera views

#### Task 4.2: Camera Presets
- Quick camera position presets (Front, Back, Left, Right, Top, Bottom)
- Save/restore custom camera positions
- Animation between camera positions

## Technical Implementation Details

### Core Integration Points

1. **Scene View Panel Enhancement**:
```rust
// In show_scene_view()
let camera_entity = self.get_active_camera();
if let Some(camera) = self.world.get_component::<CameraComponent>(camera_entity) {
  let render_target = self.create_render_target_for_panel(response.rect);
  self.renderer.render_scene(&camera, &render_target, &self.world);
  
  // Display rendered texture in egui
  ui.image(render_target.texture_id(), response.rect.size());
}
```

2. **Renderer Integration**:
```rust
// Create basic mesh entities
let cube_entity = world.spawn();
world.add_component(cube_entity, Transform::default());
world.add_component(cube_entity, MeshRenderer {
  mesh: cube_mesh_handle,
  material: default_material_handle,
});
```

3. **Camera Controls**:
```rust
// In Scene View input handling
if response.dragged_by(egui::PointerButton::Middle) {
  self.orbit_camera(response.drag_delta());
}
if input.scroll_delta.y != 0.0 {
  self.zoom_camera(input.scroll_delta.y);
}
```

### Dependencies Needed

1. **Renderer Setup**:
  - `engine-renderer-wgpu` for WGPU implementation
  - Basic shader compilation system
  - Render target creation for egui integration

2. **Scene Content**:
  - Default meshes (cube, sphere, plane) in `engine-geometry-core`
  - Basic materials in `engine-materials-core`
  - MeshRenderer component for ECS

3. **Input System**:
  - Mouse/keyboard input handling in Scene View
  - Camera controller component
  - Input state management

## Success Criteria

### ✅ Phase 1 Complete When:
- Scene View panel shows rendered 3D content
- Camera can view basic geometry (cube/sphere)
- No crashes or errors in rendering pipeline

### ✅ Phase 2 Complete When:
- Mouse controls camera movement in Scene View
- Smooth orbit, pan, zoom controls working
- Camera frustum visible and updates correctly

### ✅ Phase 3 Complete When:
- Inspector camera changes immediately visible in Scene View
- Can select and manipulate camera entities
- Live preview of all camera properties

### ✅ Phase 4 Complete When:
- Multiple cameras work seamlessly
- Camera presets provide professional workflow
- Overall camera experience matches modern game editor

## Risk Mitigation

1. **WGPU Integration Complexity**: Start with simplest possible renderer
2. **egui Texture Integration**: Use existing egui-wgpu examples as reference
3. **Performance Concerns**: Profile early, optimize camera updates
4. **Math/Matrix Issues**: Leverage glam crate, test matrix calculations

## Next Steps

1. **Immediate**: Start with Phase 1, Task 1.1 - Create basic scene geometry
2. **Priority Order**: Focus on visual results first, polish controls later
3. **Testing Strategy**: Use existing modern game editor as reference for expected behavior
4. **Documentation**: Update this plan based on implementation discoveries