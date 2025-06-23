# Fisheye Distortion Fix

## Problem
The scene view in the Longhorn editor had significant fisheye distortion caused by:
1. Hardcoded screen dimensions (600px height, 200.0 screen scale)
2. No aspect ratio consideration
3. Simplified projection that didn't use proper NDC (Normalized Device Coordinates) transformation
4. Grid and object rendering using different projection methods

## Solution
Implemented proper perspective projection with the following changes:

### 1. Updated Projection Functions
Modified both `world_to_screen` and `world_to_screen_enhanced` functions in `scene_view_impl.rs` to:
- Accept viewport rect as a parameter instead of using hardcoded values
- Calculate aspect ratio from `viewport_rect.width() / viewport_rect.height()`
- Use standard perspective projection formula with a 60-degree FOV
- Project to NDC space before converting to screen coordinates

### 2. Key Changes Made

#### Before (Simplified Projection):
```rust
fn world_to_screen(&self, world_pos: [f32; 3], camera_pos: [f32; 3], camera_rot: [f32; 3], view_center: egui::Pos2) -> (egui::Pos2, f32) {
  // ... rotation calculations ...
  let depth = final_z;
  let fov_scale = 100.0; // Hardcoded value
  let perspective_scale = fov_scale / depth.max(0.1);
  let screen_x = view_center.x + (rotated_x * perspective_scale);
  let screen_y = view_center.y - (final_y * perspective_scale);
  (egui::pos2(screen_x, screen_y), depth)
}
```

#### After (Proper Perspective Projection):
```rust
fn world_to_screen(&self, world_pos: [f32; 3], camera_pos: [f32; 3], camera_rot: [f32; 3], view_center: egui::Pos2, viewport_rect: egui::Rect) -> (egui::Pos2, f32) {
  // ... rotation calculations ...
  let depth = final_z;
  
  // Proper perspective projection with FOV and aspect ratio
  let fov_radians = 60.0_f32.to_radians(); // 60 degree FOV
  let aspect_ratio = viewport_rect.width() / viewport_rect.height();
  let projection_scale = viewport_rect.height() / (2.0 * (fov_radians / 2.0).tan());
  
  // Project to NDC space
  let ndc_x = (rotated_x / depth.max(0.1)) * (1.0 / aspect_ratio);
  let ndc_y = final_y / depth.max(0.1);
  
  // Convert to screen coordinates
  let screen_x = view_center.x + ndc_x * projection_scale;
  let screen_y = view_center.y - ndc_y * projection_scale;
  
  (egui::pos2(screen_x, screen_y), depth)
}
```

### 3. Updated All Function Calls
Updated all calls to `world_to_screen` and `world_to_screen_enhanced` to pass the viewport rect:
- Grid rendering (draw_grid)
- Object rendering (render_3d_cube, render_mesh_entity_enhanced)
- Entity rendering (render_entity)
- Sprite rendering (render_sprites)

## Technical Details

### Perspective Projection Formula
The standard perspective projection formula used:
1. **Field of View (FOV)**: 60 degrees (converted to radians)
2. **Aspect Ratio**: viewport.width / viewport.height
3. **Projection Scale**: viewport.height / (2 * tan(FOV/2))
4. **NDC Conversion**: 
  - X: (world_x / depth) * (1 / aspect_ratio)
  - Y: (world_y / depth)
5. **Screen Conversion**: 
  - X: center_x + ndc_x * projection_scale
  - Y: center_y - ndc_y * projection_scale

### Benefits
1. **Correct Perspective**: Objects now appear with proper perspective, eliminating fisheye distortion
2. **Responsive**: Projection adapts to viewport size changes
3. **Consistent**: All rendering (grid, objects, sprites) uses the same projection
4. **Professional**: Matches standard 3D graphics projection techniques

## Testing
The fix ensures:
- Grid lines converge properly toward vanishing points
- Objects maintain correct proportions at different distances
- No fisheye distortion at the edges of the viewport
- Proper depth sorting and culling
- Responsive to viewport resizing

## Future Improvements
Consider adding:
- Configurable FOV setting in editor preferences
- Near/far clipping planes for better depth precision
- Orthographic projection mode option
- Camera frustum culling optimization