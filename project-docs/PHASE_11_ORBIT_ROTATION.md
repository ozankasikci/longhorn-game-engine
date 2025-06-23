# Phase 11.2: Unity-Style Orbit Rotation

## Status: Completed
**Date:** January 2025

## Problem
After implementing direct camera rotation, the behavior still didn't match Unity's Scene View. The issue was that Unity uses **orbit rotation** (rotating around a pivot point) rather than first-person camera rotation.

## Root Cause Analysis
1. **Initial implementation**: Direct first-person rotation (pitch/yaw applied to camera orientation)
2. **Unity's behavior**: Orbit rotation around a pivot point in the scene
3. **User expectation**: Camera should orbit around objects, not just rotate in place

## Solution
Implemented true Unity-style orbit rotation:

### Orbit Rotation Algorithm
```rust
/// Apply mouse look rotation to the camera - Unity-style orbit rotation
pub fn apply_mouse_look(scene_nav: &mut SceneNavigation, mouse_delta: egui::Vec2) -> Vec<ConsoleMessage> {
    // Calculate rotation deltas
    let pitch_delta = -mouse_delta.y * scene_nav.rotation_sensitivity;
    let yaw_delta = -mouse_delta.x * scene_nav.rotation_sensitivity;
    
    // Update camera rotation angles
    scene_nav.scene_camera_transform.rotation[0] += pitch_delta;
    scene_nav.scene_camera_transform.rotation[1] += yaw_delta;
    
    // Unity-style orbit: rotate camera position around pivot point
    let pivot = [0.0, 2.0, 0.0]; // Pivot at scene center, slightly elevated
    
    // Calculate current offset from pivot
    let mut offset = [
        scene_nav.scene_camera_transform.position[0] - pivot[0],
        scene_nav.scene_camera_transform.position[1] - pivot[1],
        scene_nav.scene_camera_transform.position[2] - pivot[2],
    ];
    
    // Apply yaw rotation (around Y axis)
    let cos_yaw = yaw_delta.cos();
    let sin_yaw = yaw_delta.sin();
    let new_x = offset[0] * cos_yaw - offset[2] * sin_yaw;
    let new_z = offset[0] * sin_yaw + offset[2] * cos_yaw;
    
    // Apply pitch rotation (maintaining distance)
    // ... spherical coordinate calculations ...
    
    // Update camera position
    scene_nav.scene_camera_transform.position = pivot + offset;
}
```

## Key Features
1. **Orbit Center**: Fixed pivot point at (0, 2, 0) - center of scene, slightly elevated
2. **Distance Preservation**: Camera maintains distance from pivot during rotation
3. **Pitch Clamping**: Prevents camera from flipping over at extreme angles
4. **Smooth Movement**: Direct response with no interpolation for Unity-like feel

## Projection Matrix Fixes
Also fixed the world-to-screen projection to use correct rotation order:
```rust
// Apply camera rotation (Unity-style: Y-axis yaw first, then X-axis pitch)
let yaw = -camera_rot[1];  // Negative for correct rotation direction
let pitch = -camera_rot[0];
```

## Files Modified
1. `/crates/application/engine-editor-egui/src/panels/scene_view/navigation.rs`
   - Replaced first-person rotation with orbit rotation
   - Added pivot point and spherical coordinate calculations

2. `/crates/application/engine-editor-egui/src/panels/scene_view/scene_view_impl.rs`
   - Fixed rotation matrix order in `world_to_screen()` functions
   - Corrected rotation direction signs

## Result
Camera now orbits around the scene center exactly like Unity's Scene View:
- Right-click + drag horizontally: Orbits around Y-axis
- Right-click + drag vertically: Orbits up/down while looking at pivot
- Maintains constant distance from pivot point
- Smooth, immediate response with no lag

## Future Improvements
1. **Dynamic Pivot**: Allow focusing on selected objects (F key)
2. **Zoom Control**: Mouse wheel to move closer/further from pivot
3. **Pan Mode**: Middle mouse button for lateral movement
4. **Fly-through Mode**: Optional first-person navigation mode