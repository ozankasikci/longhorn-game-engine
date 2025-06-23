# Camera Movement Investigation Report

## Problem Description
The scene camera appears to be moving instead of just rotating when the user right-clicks and drags. The expected behavior (following industry standards) is that the camera should stay in the same position and only rotate to look around.

## Investigation Findings

### 1. Mouse Input Handling Path
The right-click drag input is handled through the following path:
- `/crates/application/engine-editor-egui/src/panels/scene_view/scene_input.rs` - Main input handler
- `handle_scene_input()` calls `SceneNavigator::handle_scene_navigation()`
- `/crates/application/engine-editor-egui/src/panels/scene_view/navigation.rs` - Navigation logic

### 2. Mouse Delta to Camera Transformation
In `navigation.rs`:
- `handle_scene_navigation()` (lines 207-299) detects right-click drag
- When dragging, it calculates mouse delta (lines 251-276)
- Calls `apply_mouse_look()` (lines 41-74) which ONLY modifies rotation:
 ```rust
 scene_nav.scene_camera_transform.rotation[0] += pitch_delta;
 scene_nav.scene_camera_transform.rotation[1] += yaw_delta;
 ```

### 3. The Root Cause: Unintended WASD Movement
The issue is in `handle_scene_navigation()` at line 285:
```rust
// Handle WASD movement
let delta_time = ui.input(|i| i.stable_dt);
let messages = Self::handle_wasd_movement(scene_navigation, ui, delta_time);
```

**The problem**: `handle_wasd_movement()` is being called EVERY frame during navigation, not just when WASD keys are pressed.

Looking at `handle_wasd_movement()` (lines 78-143):
- Line 81 checks: `if !scene_nav.is_navigating || !ui.ctx().wants_keyboard_input()`
- The issue is likely that `ui.ctx().wants_keyboard_input()` returns true during navigation

### 4. Why Camera Moves
The movement calculation in lines 93-112 checks for individual key presses:
```rust
if ui.input(|i| i.key_down(egui::Key::W)) { movement[2] -= 1.0; }
if ui.input(|i| i.key_down(egui::Key::S)) { movement[2] += 1.0; }
// etc...
```

However, if any of these conditions are incorrectly evaluated as true (due to input state issues), the camera will move.

### 5. Rendering Side Analysis
The rendering code in `scene_view_impl.rs` is correct:
- `world_to_screen()` (lines 727-767) properly applies rotation transformations
- It does NOT modify camera position, only uses it for view calculations

## Suspected Issues

1. **Input State Contamination**: The UI context might be reporting keyboard input availability incorrectly during mouse navigation.

2. **Key State Persistence**: EGUI might be retaining key press states from before navigation started.

3. **Focus Issues**: The scene view might not have proper keyboard focus, causing `wants_keyboard_input()` to return unexpected values.

## Recommended Fixes

### Fix 1: Add Debug Logging
Add logging to understand what's triggering movement:
```rust
// In handle_wasd_movement, before movement calculation:
console_messages.push(ConsoleMessage::info(&format!(
  "WASD Check: navigating={}, wants_keyboard={}, W={}, A={}, S={}, D={}",
  scene_nav.is_navigating,
  ui.ctx().wants_keyboard_input(),
  ui.input(|i| i.key_down(egui::Key::W)),
  ui.input(|i| i.key_down(egui::Key::A)),
  ui.input(|i| i.key_down(egui::Key::S)),
  ui.input(|i| i.key_down(egui::Key::D)),
)));
```

### Fix 2: Stricter WASD Control
Only process WASD when explicitly intended:
```rust
// In handle_wasd_movement, change the guard condition:
if !scene_nav.is_navigating {
  return messages;
}

// Remove the wants_keyboard_input check, or make it more specific
```

### Fix 3: Separate Navigation Modes
Consider separating "look around" mode from "fly around" mode:
- Right-click drag = look around only (no WASD)
- Right-click hold + WASD = fly around

### Fix 4: Check for Actual Key Presses
Before applying any movement, verify at least one movement key is actually pressed:
```rust
let any_movement_key = ui.input(|i| 
  i.key_down(egui::Key::W) || i.key_down(egui::Key::A) || 
  i.key_down(egui::Key::S) || i.key_down(egui::Key::D) ||
  i.key_down(egui::Key::Q) || i.key_down(egui::Key::E)
);

if !any_movement_key {
  return messages;
}
```

## Next Steps

1. Add the debug logging to confirm the exact cause
2. Test with stricter WASD controls
3. Consider implementing separate navigation modes for better user control
4. Verify EGUI input state management during docked panel interactions