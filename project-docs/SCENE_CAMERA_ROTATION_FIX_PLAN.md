# Scene Camera Rotation Fix Plan

## Problem Statement
When right-clicking and dragging in the Scene View, the camera appears to move position instead of just rotating in place. This differs from Unity's behavior where the camera stays in the same position and only rotates to look around.

## Investigation Results

### Root Cause
The `handle_wasd_movement()` function is being called every frame during navigation, potentially applying movement even when no movement keys are pressed.

### Current Behavior Flow
1. User right-clicks and drags mouse
2. `is_navigating` is set to true
3. Every frame while navigating:
   - `apply_mouse_look()` correctly updates rotation only
   - `handle_wasd_movement()` is ALSO called
   - Movement might be applied due to incorrect input state

### Key Code Locations
- `scene_input.rs:68` - Calls `handle_scene_navigation()`
- `navigation.rs:285` - Calls `handle_wasd_movement()` unconditionally
- `navigation.rs:207-299` - WASD movement logic

## Fix Plan

### Step 1: Add Debug Logging
First, add detailed logging to understand what's happening:
1. Log when `handle_wasd_movement()` is called
2. Log which keys are detected as pressed
3. Log any movement being applied
4. Log camera position changes

### Step 2: Fix Movement Detection
Modify `handle_wasd_movement()` to:
1. Check if ANY movement key is actually pressed before proceeding
2. Return early if no movement keys are pressed
3. Add explicit key state validation

### Step 3: Separate Navigation Modes
Consider separating:
1. Pure rotation mode (right-click only)
2. Movement mode (right-click + WASD)
3. Make movement require explicit key presses

### Step 4: Test and Verify
1. Test that right-click drag only rotates
2. Test that WASD keys work when intended
3. Verify camera position stays constant during rotation-only

## Implementation Priority
1. **Quick Fix**: Add movement key check in `handle_wasd_movement()`
2. **Better Fix**: Separate rotation and movement logic more clearly
3. **Best Fix**: Implement proper input state management

## Expected Outcome
After the fix:
- Right-click and drag will ONLY rotate the camera
- Camera position will remain unchanged during rotation
- WASD movement will only occur when keys are actually pressed
- Behavior will match Unity's scene navigation