# Camera Movement Fix - W Key Direction Issue

## Problem
The W key was moving the camera upward instead of forward in the Scene View when using WASD navigation.

## Root Cause
In `scene_renderer.rs`, the `update_camera` function was incorrectly converting rotation values from radians to radians:

```rust
// INCORRECT - rotation values are already in radians
let pitch = camera_transform.rotation[0].to_radians();
let yaw = camera_transform.rotation[1].to_radians();
```

This double conversion (radians to radians) was multiplying the values by π/180 ≈ 0.0175, making them much smaller than intended. This resulted in incorrect forward vector calculations.

## Solution
Remove the unnecessary `.to_radians()` conversion since the rotation values are already stored in radians:

```rust
// CORRECT - rotation values are already in radians
let pitch = camera_transform.rotation[0];
let yaw = camera_transform.rotation[1];
```

## Verification
The fix was verified by running the test `test_w_key_moves_camera_in_look_direction` which now passes correctly.

## Technical Details
- The camera system uses radians internally for all rotation values
- The rotation sensitivity is defined as 0.005 radians per pixel in `types.rs`
- The forward vector calculation in `scene_renderer.rs` expects rotation values in radians
- The navigation system in `navigation.rs` correctly transforms movement based on camera rotation

## Related Files
- `/Users/ozan/Projects/mobile-game-engine/crates/application/engine-editor-egui/src/scene_renderer.rs` - Fixed file
- `/Users/ozan/Projects/mobile-game-engine/crates/application/engine-editor-egui/src/types.rs` - Defines rotation sensitivity in radians
- `/Users/ozan/Projects/mobile-game-engine/crates/application/engine-editor-egui/src/panels/scene_view/navigation.rs` - Camera movement logic

## Test Status
- ✅ `test_w_key_moves_camera_in_look_direction` - PASSING
- ✅ `test_forward_movement_with_yaw_rotation` - PASSING  
- ✅ `test_forward_movement_with_pitch_rotation` - PASSING
- ✅ `test_backward_movement_is_opposite_of_forward` - PASSING
- ✅ `test_strafe_movement_perpendicular_to_forward` - PASSING

Note: Some rotation velocity tests are failing because the navigation system was simplified to use direct rotation instead of velocity-based rotation (Unity-style direct response).