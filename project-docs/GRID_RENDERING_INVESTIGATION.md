# Grid Rendering Investigation

## Current Issues

### 1. Inconsistent Line Rendering
- Some grid lines appear and disappear as the camera moves
- Lines near the camera sometimes vanish unexpectedly
- Grid doesn't always render completely

### 2. Root Causes Identified

#### A. Aggressive Culling
```rust
// Skip if both points are behind camera
if start_depth <= 0.1 && end_depth <= 0.1 {
  continue;
}

// Skip lines that would appear too distorted
if start_depth <= 0.5 || end_depth <= 0.5 {
  continue;
}
```
**Problem**: Lines are culled when ANY endpoint has depth <= 0.5, even if one endpoint is visible. This causes lines to disappear when they cross the near plane.

#### B. Fixed Grid Size
```rust
let grid_extent = 50; // Fixed grid size: 50x50 units
```
**Problem**: Fixed grid size doesn't adapt to camera height or viewing distance, causing grid to be too small when zoomed out.

#### C. Render Order
```rust
// Draw grid background (after objects for depth)
self.draw_grid(painter, rect, camera_pos, camera_rot);
```
**Problem**: Grid is drawn after objects, which can cause z-fighting or visibility issues.

#### D. Line Clipping
The current implementation doesn't clip lines at the near plane. When a line crosses from behind to in front of the camera, it should be clipped at the intersection point.

#### E. Depth Testing
No proper depth-based fading or LOD system, causing all lines to render with same intensity regardless of distance.

## Visual Artifacts

1. **Line Popping**: Lines suddenly appear/disappear when crossing depth thresholds
2. **Missing Lines**: Entire grid lines missing when one endpoint is slightly behind camera
3. **Inconsistent Density**: Grid doesn't maintain consistent visual density at different viewing distances
4. **Hard Edges**: Grid abruptly ends at fixed boundaries

## Technical Analysis

### Current Algorithm Flow
1. Fixed 50x50 grid centered at origin
2. Each line tested for visibility with aggressive culling
3. No line clipping or partial rendering
4. Binary visibility (fully visible or fully culled)

### Missing Features
1. **Near-plane clipping**: Lines should be clipped at camera near plane
2. **Distance-based LOD**: Show fewer lines when zoomed out
3. **Infinite grid illusion**: Grid should appear to extend infinitely
4. **Smooth fading**: Lines should fade based on distance
5. **Adaptive sizing**: Grid extent should adapt to viewing conditions

## Comparison with Professional Editors

### modern engines Approach
- Infinite grid that fades with distance
- Dynamic LOD based on camera height
- Smooth transitions between grid levels
- Lines clipped properly at near plane
- Consistent visual density

### Blender Approach
- Multiple grid levels with different scales
- Automatic switching based on zoom level
- Subtle fading at distance
- Always visible floor reference

## Next Steps
Document complete. Moving to research phase for best practices.