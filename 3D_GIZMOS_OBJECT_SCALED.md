# 3D Gizmos with Object-Based Scaling

## Summary
The 3D gizmos now scale based on the object's size while maintaining reasonable screen-space sizing.

## Changes Made:

### 1. Updated Shader to Use Object Scale
- **gizmo_3d.wgsl**: 
  - Added object scale to viewport_size.w component
  - Combined screen-space scaling with object scale
  - Formula: `screen_scale * object_scale * 0.3`

### 2. Updated Gizmo Renderer
- **gizmo_3d.rs**:
  - Extract scale from transform matrix by measuring basis vector lengths
  - Calculate average scale from X, Y, Z scales
  - Pass object scale in uniforms

### 3. Updated Input Handler
- **gizmo_3d_input.rs**:
  - Calculate object scale from transform
  - Use object scale in hit testing to match visual size
  - Ensures mouse interaction aligns with rendered gizmos

## How It Works:

The gizmo size is now calculated as:
```
final_scale = screen_space_scale * object_scale * balance_factor
```

Where:
- `screen_space_scale` maintains consistent pixel size regardless of distance
- `object_scale` is the average of the object's X, Y, Z scale values
- `balance_factor` (0.3) balances between screen and object sizing

## Benefits:
- Small objects get smaller gizmos
- Large objects get larger gizmos
- Gizmos remain visible at all distances
- Hit testing accurately matches visual size
- More intuitive editing experience

## Example:
- Object with scale [1, 1, 1] → Normal sized gizmos
- Object with scale [2, 2, 2] → 2x larger gizmos
- Object with scale [0.5, 0.5, 0.5] → Half-sized gizmos