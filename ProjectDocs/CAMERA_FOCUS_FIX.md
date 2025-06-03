# Camera Focus Fix (F Key)

## Problem
The F key (focus on selected object) was not working correctly. When pressing F to focus on a cube, the camera would move to an incorrect position, requiring the user to navigate backwards to see the object.

## Root Cause
The original focus implementation had several issues:
1. Still checking for old `Mesh` component instead of new `MeshFilter`
2. Incorrect camera positioning calculation
3. Improper rotation angles for looking at the object

## Solution
Fixed the `focus_on_selected_object` function in `scene_view/mod.rs`:

### Key Changes:
1. **Updated component check**: Now checks for `MeshFilter` instead of old `Mesh` component
2. **Improved camera positioning**:
   - Camera is positioned behind and above the object for a nice 3/4 view
   - Uses positive Z offset (behind) since camera looks down -Z axis
   - Added slight X offset for better viewing angle
   - View distance is proportional to object size (3x object radius, clamped between 2-10 units)

3. **Fixed rotation calculation**:
   - Proper pitch calculation to look down at object
   - Correct yaw calculation using `atan2(dx, -dz)` for our coordinate system
   - Camera now properly faces the selected object

### Camera Positioning Formula:
```rust
let angle = PI / 6.0; // 30 degrees
let height_offset = view_distance * sin(angle) * 2.0;
let horizontal_offset = view_distance * cos(angle);

camera_position = [
    object.x + horizontal_offset * 0.5,  // Slight X offset
    object.y + height_offset,             // Above object
    object.z + horizontal_offset          // Behind object
];
```

### Testing
- Press F with a cube selected
- Camera moves to a position that gives a clear view of the cube
- No need to navigate backwards to see the object
- Works with objects at different positions and scales

## Coordinate System Notes
- +Y is up
- Camera looks down -Z axis when rotation is [0, 0, 0]
- Positive Z positions are behind objects (for camera placement)
- Rotation order: Yaw (Y-axis) first, then Pitch (X-axis)