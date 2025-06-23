# Phase 8: Scene Navigation Controls - Implementation Plan

## Phase Overview
Implement modern engines Engine style scene camera navigation using right-click + WASD controls for professional scene editing workflow.

## Implementation Strategy

### Architecture Design
Based on research, we'll implement a navigation system that follows modern engines standards:
- Right mouse button hold to enter navigation mode
- WASD for movement, mouse for rotation
- Speed control via mouse wheel and Shift modifier
- Integration with existing gizmo system

## Implementation Tasks

### Phase 8.1: Core Navigation System (45 minutes)

#### Task 8.1.1: Navigation State Management (15 minutes)
**Objective**: Add navigation state tracking to editor

**Implementation**:
- Add `SceneNavigation` struct to `UnityEditor`
- Track navigation mode state (idle/navigating)
- Store last mouse position for delta calculation
- Add navigation configuration (speeds, sensitivity)

**Code Structure**:
```rust
#[derive(Debug, Clone)]
pub struct SceneNavigation {
  pub enabled: bool,
  pub is_navigating: bool,
  pub movement_speed: f32,
  pub rotation_sensitivity: f32,
  pub fast_movement_multiplier: f32,
  pub last_mouse_pos: Option<egui::Pos2>,
  pub scene_camera_transform: Transform,
}
```

#### Task 8.1.2: Right Mouse Button Detection (15 minutes)
**Objective**: Detect right mouse button press/release for navigation mode

**Implementation**:
- Modify scene view input handling
- Detect right mouse button press to start navigation
- Track mouse position for rotation calculation
- Release navigation mode on right mouse release

**Key Points**:
- Use `response.secondary_clicked()` for right mouse detection
- Capture mouse position when navigation starts
- Set navigation state flags appropriately

#### Task 8.1.3: WASD Movement Implementation (15 minutes)
**Objective**: Implement keyboard-based camera movement

**Implementation**:
- Add keyboard input detection in scene view
- Map WASD keys to camera movement directions
- Calculate movement vectors relative to camera orientation
- Apply delta time for frame-rate independent movement

**Movement Mapping**:
- W: Forward (camera's -Z direction)
- S: Backward (camera's +Z direction)
- A: Left (camera's -X direction)
- D: Right (camera's +X direction)
- Q: Down (world -Y direction)
- E: Up (world +Y direction)

### Phase 8.2: Mouse Rotation & Speed Control (30 minutes)

#### Task 8.2.1: Mouse Look Implementation (15 minutes)
**Objective**: Rotate camera based on mouse movement

**Implementation**:
- Calculate mouse delta during navigation
- Convert mouse movement to rotation angles
- Apply rotation to scene camera transform
- Smooth rotation with proper sensitivity

**Math Implementation**:
```rust
fn update_camera_rotation(&mut self, mouse_delta: egui::Vec2) {
  let sensitivity = self.scene_navigation.rotation_sensitivity;
  
  // Horizontal rotation (Y-axis)
  let yaw_delta = -mouse_delta.x * sensitivity;
  
  // Vertical rotation (X-axis) 
  let pitch_delta = -mouse_delta.y * sensitivity;
  
  // Apply rotations to camera transform
  self.apply_camera_rotation(yaw_delta, pitch_delta);
}
```

#### Task 8.2.2: Speed Control System (15 minutes)
**Objective**: Implement dynamic speed adjustment

**Implementation**:
- Mouse wheel detection for speed adjustment
- Shift modifier for fast movement
- Speed multiplier application
- Visual feedback for current speed

**Features**:
- Mouse wheel up/down adjusts base movement speed
- Shift key applies fast movement multiplier (2x-5x)
- Speed persistence during navigation session
- Configurable speed ranges and defaults

### Phase 8.3: Integration & Polish (30 minutes)

#### Task 8.3.1: Gizmo System Coordination (15 minutes)
**Objective**: Ensure navigation doesn't conflict with gizmo interaction

**Implementation**:
- Priority system for input handling
- Disable navigation when gizmo is active
- Proper state transitions between modes
- Visual feedback for current mode

**Conflict Resolution**:
- Gizmo interaction takes priority over navigation
- Right-click on gizmo components should not start navigation
- Clear visual indication of active mode

#### Task 8.3.2: Scene Camera Integration (15 minutes)
**Objective**: Integrate navigation with existing scene view system

**Implementation**:
- Modify scene view camera positioning
- Update gizmo rendering with new camera position
- Maintain camera state between navigation sessions
- Reset camera to default position option

**Integration Points**:
- Scene view rendering uses navigation camera transform
- Gizmo positioning updates with camera movement
- Object selection and highlighting work with new camera

## Technical Implementation Details

### Input Event Flow
```rust
fn handle_scene_navigation(&mut self, ui: &egui::Ui, response: &egui::Response) {
  // 1. Check for right mouse button to start/stop navigation
  if response.secondary_clicked() {
    self.start_scene_navigation(response.hover_pos());
  } else if response.secondary_released() {
    self.end_scene_navigation();
  }
  
  // 2. Handle navigation input during active navigation
  if self.scene_navigation.is_navigating {
    self.handle_navigation_input(ui, response);
  }
}
```

### Camera Transform Updates
```rust
fn update_scene_camera(&mut self, delta_time: f32) {
  let nav = &mut self.scene_navigation;
  
  // Apply WASD movement
  let movement = self.calculate_movement_vector(delta_time);
  nav.scene_camera_transform.position[0] += movement.x;
  nav.scene_camera_transform.position[1] += movement.y;
  nav.scene_camera_transform.position[2] += movement.z;
  
  // Apply mouse rotation
  if let Some(mouse_delta) = self.get_mouse_delta() {
    self.apply_camera_rotation(mouse_delta);
  }
}
```

### Performance Optimizations
- Cache frequently calculated values
- Use efficient vector math operations
- Minimize allocations during navigation
- Frame rate independent calculations

## Integration with Existing Systems

### Scene View Rendering
- Use navigation camera transform for scene projection
- Update gizmo positioning relative to new camera
- Maintain object selection with camera movement

### Input Priority System
```rust
enum InputMode {
  Gizmo,    // Gizmo interaction active
  Navigation, // Scene navigation active 
  Selection,  // Object selection mode
}
```

### State Management
- Navigation state persists during editing session
- Camera position saved/restored appropriately
- Smooth transitions between navigation and editing

## Success Criteria

### Functional Requirements
- ✅ Right-click + drag rotates scene camera
- ✅ WASD keys move camera during right-click hold
- ✅ Mouse wheel adjusts movement speed
- ✅ Shift modifier increases movement speed
- ✅ Smooth, responsive camera movement
- ✅ No conflicts with gizmo system

### Quality Standards
- Frame-rate independent movement (60+ FPS)
- modern engines level responsiveness and feel
- Intuitive speed scaling and sensitivity
- Professional visual feedback

### User Experience Goals
- Familiar workflow for modern engines users
- Smooth learning curve for new users
- Efficient scene exploration and editing
- No disruptive mode switching

## Testing Strategy

### Core Functionality Tests
1. Right mouse button detection accuracy
2. WASD movement in all directions
3. Mouse rotation smoothness and precision
4. Speed adjustment responsiveness
5. Gizmo interaction priority

### Edge Case Testing
1. Rapid input switching
2. Multiple modifier keys
3. Mouse leaving window during navigation
4. Extreme camera positions/rotations
5. Very fast/slow movement speeds

### Performance Testing
1. Frame rate during rapid navigation
2. Input latency measurement
3. Memory usage during navigation
4. Smooth operation with large scenes

## Risk Mitigation

### Input Conflicts
**Risk**: Navigation interfering with gizmo interaction
**Mitigation**: Clear priority system and state management

### Performance Issues
**Risk**: Camera calculations impacting frame rate
**Mitigation**: Efficient math and delta time usage

### User Experience Issues
**Risk**: Confusing or unintuitive navigation
**Mitigation**: Follow modern engines standards exactly

## Next Steps

1. **Phase 8.1**: Implement core navigation system
2. **Phase 8.2**: Add mouse rotation and speed control
3. **Phase 8.3**: Polish integration and testing
4. **Future**: Advanced features (orbit mode, focus selected)

**Ready to begin Phase 8.1.1 - Navigation State Management?**