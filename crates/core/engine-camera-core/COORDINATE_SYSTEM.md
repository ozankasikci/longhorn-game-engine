# Coordinate System Convention

## Overview

The Longhorn Game Engine uses a **right-handed coordinate system** consistent with OpenGL, Vulkan, and most modern graphics APIs.

## World Space Conventions

### Axes
- **+X**: Points to the right
- **+Y**: Points up (vertical axis)
- **+Z**: Points towards the viewer (out of the screen)
- **-Z**: Points into the screen (forward direction)

### Camera Default Orientation
- Cameras look down the **-Z axis** by default
- Up vector is **+Y**
- Right vector is **+X**

## Matrix Conventions

### View Matrix
- Transforms from world space to view space
- Created using either:
  - Position + Rotation (quaternion)
  - Look-at (eye, target, up)

### Projection Matrix
- Transforms from view space to clip space
- Uses right-handed perspective projection
- Near/far planes use positive distances

### Matrix Multiplication Order
```
Clip Space = Projection * View * Model * Vertex
MVP = Projection * View * Model
```

## Rotation Conventions

### Euler Angles
- Order: YXZ (Yaw, Pitch, Roll)
- **Pitch** (X-axis): Look up/down
- **Yaw** (Y-axis): Turn left/right  
- **Roll** (Z-axis): Tilt sideways

### Positive Rotations (Right-Hand Rule)
- **+Pitch**: Look up
- **+Yaw**: Turn left
- **+Roll**: Tilt counter-clockwise

## Screen Space

### NDC (Normalized Device Coordinates)
- X: [-1, 1] left to right
- Y: [-1, 1] bottom to top
- Z: [-1, 1] near to far (after projection)

### Viewport
- Origin: Top-left corner
- X: Increases to the right
- Y: Increases downward

## Common Pitfalls

1. **Z-Direction**: Remember cameras look down -Z, not +Z
2. **Handedness**: Ensure all math libraries use right-handed conventions
3. **Euler Order**: Always use YXZ for camera rotations
4. **Near Plane**: Must be positive for perspective projection

## Conversion to Other Systems

### To Left-Handed Systems
- Negate Z coordinates
- Adjust winding order
- Flip projection matrix Z
- May need to adjust clip space Z range [0,1] vs [-1,1]