# Phase 6: Play Button and Game View Progress

## Research Complete ✅
- **Research Document**: `PHASE_6_PLAY_BUTTON_RESEARCH.md` 
- **Key Findings**: Unity state management patterns, camera matrix mathematics, frustum culling techniques
- **Technical Foundation**: glam library for matrix calculations, atomic state transitions, visual feedback systems

## Overview
Implement the Play button functionality to switch from editor Scene View to runtime Game View, showing the scene from the main camera's perspective during gameplay. This creates the classic Unity workflow of Scene/Game view switching based on research findings.

## Current State Analysis

### ✅ What We Have
1. **Working Scene View**: Top-down editor view showing all objects, sprites, and camera
2. **Functional Camera System**: Main camera entity with proper transforms and properties
3. **Sprite Rendering**: 2D sprites visible and selectable in Scene View
4. **Play Button UI**: Play button exists in toolbar but doesn't do anything
5. **Game View Panel**: GameView panel type exists but shows same content as Scene View

### ❌ What's Missing
1. **Play State Management**: No tracking of play/edit modes
2. **Game View Camera Rendering**: Game View doesn't show camera perspective
3. **Runtime Logic**: No game loop or update systems
4. **View Switching**: No actual difference between Scene and Game views
5. **Play Controls**: Play/Pause/Stop buttons don't affect anything

## Implementation Plan

### Phase 6.1: Play State System (30-45 minutes)
**Objective**: Add play/edit mode state management and UI feedback

#### Task 6.1.1: Add Play State Management
- Create `PlayState` enum (Editing, Playing, Paused)
- Add play state to editor struct
- Implement state transitions with validation

#### Task 6.1.2: Update Play Button UI
- Show different icons for play states (▶️ Play, ⏸️ Pause, ⏹️ Stop)
- Add visual feedback when in play mode
- Implement play/pause/stop button logic

#### Task 6.1.3: Add Play Mode Indicators
- Show play state in editor title bar or status area
- Add subtle UI changes when in play mode (different background color)
- Console messages for state transitions

### Phase 6.2: Game View Camera Rendering (45-60 minutes)
**Objective**: Show scene from main camera perspective in Game View

#### Task 6.2.1: Implement Camera Perspective Rendering
- Create camera projection matrix calculations
- Implement 3D to 2D projection using camera transform
- Render objects from camera's point of view

#### Task 6.2.2: Game View Panel Implementation
- Separate Game View from Scene View rendering
- Show camera viewport with proper field of view
- Handle camera movement and rotation in game view

#### Task 6.2.3: Camera Frustum Culling
- Only render objects visible to camera
- Implement basic frustum culling for performance
- Handle near/far plane clipping

### Phase 6.3: Runtime Game Loop (45 minutes)
**Objective**: Basic game logic and update systems when playing

#### Task 6.3.1: Game Update Loop
- Implement basic game tick system
- Add delta time calculation
- Update game objects during play mode

#### Task 6.3.2: Input Handling in Play Mode
- Capture input events during play mode
- Basic WASD camera movement (optional)
- Mouse look controls (optional)

#### Task 6.3.3: Editor vs Runtime Separation
- Prevent editing while in play mode
- Lock Inspector properties during play
- Save/restore scene state between play sessions

## Technical Implementation Details

### Play State Management

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    Editing,   // Normal editor mode
    Playing,   // Game is running
    Paused,    // Game is paused
}

struct UnityEditor {
    play_state: PlayState,
    game_start_time: Option<std::time::Instant>,
    delta_time: f32,
    // ... existing fields
}
```

### Game View Rendering

```rust
fn show_game_view(&mut self, ui: &mut egui::Ui) {
    if self.play_state == PlayState::Editing {
        // Show message to press Play
        ui.centered_and_justified(|ui| {
            ui.label("🎮 Press Play to see Game View");
        });
        return;
    }
    
    // Render from main camera perspective
    if let Some(camera_entity) = self.get_main_camera() {
        self.render_camera_view(ui, camera_entity);
    }
}

fn render_camera_view(&mut self, ui: &mut egui::Ui, camera_entity: EntityV2) {
    let camera_transform = self.world.get_component::<Transform>(camera_entity).unwrap();
    let camera = self.world.get_component::<Camera>(camera_entity).unwrap();
    
    // Create projection matrix from camera
    let projection = create_perspective_matrix(camera.fov, ui.available_width() / ui.available_height(), camera.near, camera.far);
    let view = create_view_matrix(&camera_transform);
    
    // Render all visible objects from camera perspective
    self.render_scene_from_camera(ui, &view, &projection);
}
```

### Camera Projection Math

```rust
fn create_perspective_matrix(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let fov_rad = fov_degrees.to_radians();
    glam::Mat4::perspective_rh(fov_rad, aspect, near, far)
}

fn create_view_matrix(transform: &Transform) -> Mat4 {
    let position = Vec3::from_array(transform.position);
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        transform.rotation[0].to_radians(),
        transform.rotation[1].to_radians(),
        transform.rotation[2].to_radians()
    );
    
    Mat4::from_rotation_translation(rotation, position).inverse()
}

fn world_to_screen(world_pos: Vec3, view_matrix: &Mat4, projection_matrix: &Mat4, screen_size: Vec2) -> Option<Vec2> {
    let clip_pos = projection_matrix * view_matrix * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    
    if clip_pos.w <= 0.0 { return None; } // Behind camera
    
    let ndc = clip_pos.xyz() / clip_pos.w;
    if ndc.z < -1.0 || ndc.z > 1.0 { return None; } // Outside depth range
    
    // Convert to screen coordinates
    let screen_x = (ndc.x + 1.0) * 0.5 * screen_size.x;
    let screen_y = (1.0 - ndc.y) * 0.5 * screen_size.y; // Flip Y for screen space
    
    Some(Vec2::new(screen_x, screen_y))
}
```

## Expected Workflow

### 1. Editor Mode (Default)
- **Scene View**: Top-down editor perspective with gizmos and selection
- **Game View**: Shows "Press Play to start" message
- **Play Button**: Shows ▶️ icon, clicking starts game

### 2. Play Mode (After clicking Play)
- **Scene View**: Same editor view but with play mode indicator
- **Game View**: Shows scene from main camera's perspective
- **Play Button**: Shows ⏸️ (Pause) and ⏹️ (Stop) options
- **Inspector**: Properties locked/grayed out during play

### 3. Camera Perspective in Game View
- **3D Objects**: Rendered with proper perspective, depth, and occlusion
- **Sprites**: Billboard toward camera or maintain world orientation
- **Camera Movement**: If enabled, WASD/mouse controls move camera
- **Performance**: Only render objects within camera frustum

## Success Criteria

### ✅ Phase 6.1 Complete When:
- Play/Pause/Stop buttons work and show correct states
- Editor UI provides clear feedback about current play state
- Console shows state transition messages

### ✅ Phase 6.2 Complete When:
- Game View shows scene from main camera perspective
- Objects appear correctly positioned relative to camera
- Camera FOV, near/far planes affect rendering correctly

### ✅ Phase 6.3 Complete When:
- Basic game update loop runs during play mode
- Can optionally move camera with WASD during play
- Editor properties are protected during play mode

## Integration with Existing Systems

### Camera System
- Use existing Camera and Transform components
- Leverage camera frustum and projection calculations
- Main camera detection for Game View rendering

### Sprite System
- Sprites render correctly in camera perspective
- Billboard behavior for camera-facing sprites
- Proper depth sorting from camera position

### ECS Integration
- Game loop updates Transform components
- Query systems work during play mode
- Component changes reflected in real-time

## Risk Mitigation

1. **Math Complexity**: Use proven glam library for matrix calculations
2. **Performance Issues**: Implement basic frustum culling early
3. **State Management**: Keep play state simple with clear transitions
4. **Camera Controls**: Make camera movement optional and well-documented

## Next Steps

This phase transforms the editor from a static tool into a dynamic game development environment. The Play button becomes the gateway between editing and testing, just like Unity.

**Ready to start with Phase 6.1, Task 6.1.1 - Add Play State Management?**