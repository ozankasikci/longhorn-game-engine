# Phase 8: Scene Navigation Controls - Research & Best Practices

## Overview
Implement modern engines Engine style scene camera navigation using right-click + WASD controls for intuitive scene exploration and editing workflow.

## Research Findings

### modern game editor Scene Navigation Standards

**Flythrough Mode (Primary Control)**:
- Hold right mouse button to enter flythrough mode
- Mouse movement controls camera rotation (look around)
- WASD keys control camera movement:
 - W: Move forward
 - S: Move backward 
 - A: Move left (strafe)
 - D: Move right (strafe)
- Q/E keys for vertical movement:
 - Q: Move down
 - E: Move up
- Hold Shift to increase movement speed
- Mouse scroll wheel adjusts movement speed during navigation

**Additional Navigation Controls**:
- Alt + Left Mouse: Orbit around selected object
- Alt + Right Mouse: Zoom in/out
- F key: Frame selected object (focus camera on selection)
- Scene Gizmo clicking: Snap to orthographic views

### Unreal Engine Viewport Navigation Standards

**WASD Navigation**:
- Hold right mouse button (RMB) to enable WASD controls
- Mouse movement rotates viewport camera
- WASD keys for movement (same as Unity)
- Mouse wheel up/down: Increase/decrease movement speed
- Navigation only works in Perspective mode (not orthographic)
- Arrow keys and numpad provide alternative movement controls

**Speed Control**:
- Mouse wheel while navigating adjusts speed dynamically
- Holding RMB during zoom preserves FOV settings

### Common Design Patterns

**Input Requirements**:
1. Right mouse button hold is required for navigation mode
2. WASD movement only active during right mouse hold
3. Mouse movement for camera rotation
4. Speed modifiers (Shift for faster, scroll wheel for adjustment)
5. Perspective mode requirement for 3D navigation

**Performance Considerations**:
- Smooth camera interpolation for professional feel
- Configurable sensitivity settings
- Frame rate independent movement using delta time
- Cursor capture/release during navigation

## Technical Implementation Strategy

### Input Handling Architecture
```rust
pub struct SceneNavigation {
  pub enabled: bool,
  pub movement_speed: f32,
  pub rotation_sensitivity: f32,
  pub fast_movement_multiplier: f32,
  pub is_navigating: bool,
  pub last_mouse_pos: Option<Vec2>,
}

pub enum NavigationInput {
  StartNavigation(Vec2), // Right mouse press + position
  UpdateNavigation(Vec2), // Mouse movement during navigation
  EndNavigation,     // Right mouse release
  MoveForward(f32),   // W key + delta time
  MoveBackward(f32),   // S key + delta time
  MoveLeft(f32),     // A key + delta time
  MoveRight(f32),    // D key + delta time
  MoveUp(f32),      // E key + delta time 
  MoveDown(f32),     // Q key + delta time
  AdjustSpeed(f32),   // Mouse wheel
}
```

### Camera Control Integration
- Scene camera separate from game camera
- Transform updates to scene view camera
- Integration with existing gizmo system (disable during navigation)
- Smooth interpolation for professional feel

### Input Event Processing
- EGUI input capture during right mouse hold
- Key state tracking for WASD
- Mouse delta calculation for rotation
- Speed modulation with modifiers

## Platform Considerations

### Desktop Implementation
- Full WASD + mouse navigation
- Shift modifier for speed boost
- Mouse wheel speed adjustment
- Right mouse button capture

### Future Mobile Adaptation
- Touch-based navigation gestures
- Two-finger pan for movement
- Pinch zoom for speed/distance
- Long press for navigation mode

## Integration Points

### Scene View System
- Modify existing scene view camera
- Add navigation state management
- Coordinate with gizmo interaction system
- Maintain separation from game camera

### Input System Integration
- Extend current mouse/keyboard handling
- Priority system (gizmos vs navigation)
- State management for navigation mode
- Smooth transitions between modes

### Visual Feedback
- Cursor changes during navigation
- Speed indicator UI element
- Navigation mode indicators
- Smooth camera movement interpolation

## Success Criteria

### Primary Requirements
1. ✅ Hold right mouse button to enter navigation mode
2. ✅ WASD keys move scene camera during navigation
3. ✅ Mouse movement rotates camera view
4. ✅ Smooth, responsive camera movement
5. ✅ Speed adjustment with mouse wheel
6. ✅ Fast movement with Shift modifier

### Quality Standards
- Frame rate independent movement (60+ FPS)
- Smooth interpolation without jitter
- Intuitive speed scaling
- Proper input priority handling
- No conflicts with gizmo system

### Professional Polish
- modern engines level responsiveness
- Configurable sensitivity settings
- Visual feedback for navigation state
- Seamless transitions in/out of navigation

## Performance Requirements

### Target Metrics
- Sub-16ms frame time during navigation
- Smooth 60 FPS camera movement
- Minimal input latency (<50ms)
- No stuttering during rapid movement

### Optimization Strategies
- Efficient matrix calculations
- Delta time based movement
- Input buffering for smooth response
- Minimal allocations during navigation

## Risk Assessment

### Technical Risks
1. **Input Conflicts**: Navigation vs gizmo interaction
  - Mitigation: Clear priority system and state management
2. **Performance Impact**: Complex camera calculations
  - Mitigation: Efficient math operations and caching
3. **Platform Differences**: Mouse capture behavior
  - Mitigation: Platform abstraction layer

### User Experience Risks
1. **Learning Curve**: Different from other 3D editors
  - Mitigation: Follow modern engines standards exactly
2. **Sensitivity Issues**: Too fast/slow movement
  - Mitigation: Configurable settings with good defaults

## Implementation Phases

### Phase 8.1: Basic Navigation (45 minutes)
- Right mouse button detection
- WASD key handling
- Basic camera movement
- Mouse rotation

### Phase 8.2: Polish & Speed Control (30 minutes)
- Mouse wheel speed adjustment
- Shift modifier for fast movement
- Smooth interpolation
- Input priority management

### Phase 8.3: Integration & Testing (30 minutes)
- Gizmo system coordination
- Edge case handling
- Performance optimization
- User experience polish

**Total Estimated Time**: 1 hour 45 minutes
**Priority**: High - Essential for professional scene editing workflow
**Dependencies**: Current gizmo system, scene view camera