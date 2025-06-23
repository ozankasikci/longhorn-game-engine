# Camera 3D Movement Implementation

## Overview
This document describes the implementation of full 3D camera movement in the Longhorn Game Engine editor, allowing the camera to move in the direction it's looking, including vertical movement when pitched up or down.

## Problem
The camera movement was initially constrained to horizontal movement only. When pressing W to move forward, the camera would only move along the XZ plane, ignoring the camera's pitch (vertical look angle). This made it impossible to fly up or down by looking in that direction and moving forward.

## Solution
Implemented proper 3D movement by including pitch in the forward vector calculation:

```rust
// Calculate forward direction including pitch for full 3D movement
let forward_x = -yaw.sin() * pitch.cos();
let forward_y = pitch.sin();
let forward_z = yaw.cos() * pitch.cos();

// Transform movement from camera space to world space
let world_x = movement[0] * right_x + movement[1] * up_x + movement[2] * forward_x;
let world_y = movement[0] * right_y + movement[1] * up_y + movement[2] * forward_y;
let world_z = movement[0] * right_z + movement[1] * up_z + movement[2] * forward_z;
```

## Key Components

### 1. Camera Movement Module (`camera_movement.rs`)
- Transforms movement input from camera space to world space
- Takes into account both yaw (horizontal rotation) and pitch (vertical rotation)
- Provides full 3D movement capabilities

### 2. Scene Navigation (`navigation.rs`)
- Handles mouse input for camera rotation
- Processes WASD+QE keyboard input for movement
- Applies the transformed movement to the camera position

### 3. Debug Overlay (`debug_overlay.rs`)
- Visual debugging tool showing camera position and orientation
- Displays forward direction vector
- Shows a compass indicating camera look direction

## Controls
- **Right Mouse + Drag**: Rotate camera (yaw and pitch)
- **W/S**: Move forward/backward in look direction
- **A/D**: Strafe left/right
- **Q/E**: Move down/up (world space)
- **Shift**: Fast movement
- **Mouse Wheel**: Adjust movement speed

## Coordinate System
- **+X**: Right
- **+Y**: Up
- **+Z**: Forward (default camera look direction)
- **Yaw**: Rotation around Y axis (positive = turn left)
- **Pitch**: Rotation around X axis (positive = look up)

## Implementation Details

### Forward Vector Calculation
When pitch is included, the forward vector becomes 3D:
- `forward_x = -sin(yaw) * cos(pitch)`
- `forward_y = sin(pitch)`
- `forward_z = cos(yaw) * cos(pitch)`

This ensures that:
- When pitch = 0°, movement is horizontal
- When pitch > 0° (looking up), forward movement includes upward component
- When pitch < 0° (looking down), forward movement includes downward component

### Movement Types
1. **Forward/Back (W/S)**: Moves along the camera's look direction
2. **Strafe (A/D)**: Moves perpendicular to look direction, always horizontal
3. **Vertical (Q/E)**: Moves up/down in world space, regardless of camera orientation

## Testing
The implementation was verified with various camera orientations:
- Looking straight ahead: Movement is horizontal
- Looking up 45°: Forward movement goes up at 45° angle
- Looking down: Forward movement descends
- Various yaw angles: Movement follows the horizontal look direction correctly

## Files Modified
- `crates/application/engine-editor-egui/src/panels/scene_view/camera_movement.rs` - Core movement transformation
- `crates/application/engine-editor-egui/src/panels/scene_view/navigation.rs` - Input handling
- `crates/application/engine-editor-egui/src/panels/scene_view/debug_overlay.rs` - Visual debugging
- `crates/application/engine-editor-egui/src/panels/scene_view/mod.rs` - Module organization

## Result
The camera now provides full 3D movement capabilities, allowing users to fly through the scene in any direction by looking where they want to go and pressing W. This creates an intuitive, first-person style navigation system suitable for 3D scene editing.