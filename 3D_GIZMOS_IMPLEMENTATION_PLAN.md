# Proper 3D Position Gizmos Implementation Plan

## Current State Analysis
The current "Unity-style" gizmos are actually 2D overlays that:
- Are drawn on top of the 3D scene using egui's 2D drawing API
- Use ray-plane intersection for movement (which works well)
- Always appear on top (no depth testing)
- Don't integrate with the 3D rendering pipeline

## Goal: True 3D Gizmos
Implement gizmos that:
1. Are rendered as actual 3D geometry in world space
2. Have proper depth testing and can be occluded by objects
3. Use the existing 3D renderer's pipeline
4. Support proper 3D input handling with ray casting

## Implementation Plan

### Phase 1: Fix the Existing 3D Gizmo Renderer
The engine already has `GizmoRenderer3D` in `engine-renderer-3d/src/gizmo_3d.rs` but it's not working properly.

**Issues to fix:**
1. **Visibility**: The gizmos might be too small or not rendering at the right position
2. **Shader scaling**: The constant screen-space sizing calculation might be wrong
3. **Depth testing**: Currently set to "Always" which might cause issues

**Steps:**
1. Debug why the 3D gizmos aren't visible
2. Fix the vertex shader's scale calculation
3. Ensure proper transform matrices are being passed
4. Test with different depth compare functions

### Phase 2: Implement 3D Hit Testing
Replace the 2D hit testing with proper 3D ray-object intersection.

**Components needed:**
1. **Ray generation**: Convert mouse position to world-space ray
2. **Geometry intersection**: Test ray against gizmo geometry (cylinders for axes, cones for arrows)
3. **Hit priority**: Handle overlapping gizmos correctly

**Implementation:**
```rust
struct Gizmo3DHitTest {
    // Ray-cylinder intersection for axis shafts
    fn ray_cylinder_intersect(ray: Ray, cylinder: Cylinder) -> Option<f32>;
    
    // Ray-cone intersection for arrow heads
    fn ray_cone_intersect(ray: Ray, cone: Cone) -> Option<f32>;
    
    // Find closest hit
    fn hit_test(ray: Ray, gizmo_transform: Mat4) -> Option<GizmoComponent>;
}
```

### Phase 3: Integrate with Renderer
Properly integrate the 3D gizmos with the scene renderer.

**Tasks:**
1. Ensure gizmo geometry is created correctly (arrows, not just lines)
2. Add gizmo rendering as a separate pass after opaque geometry
3. Implement highlighting for hovered/active components
4. Support different gizmo modes (translate, rotate, scale)

### Phase 4: Implement Drag Handling
Convert the existing drag logic to work with 3D gizmos.

**Components:**
1. Use the existing ray-plane intersection math (it's correct)
2. Update visual feedback during dragging
3. Ensure smooth movement with proper constraints

### Phase 5: Visual Enhancements
Make the gizmos look professional:
1. **Shading**: Add simple shading to show depth
2. **Transparency**: Make gizmos slightly transparent
3. **Outlines**: Add outlines for better visibility
4. **Scaling**: Ensure constant screen size regardless of distance

## Technical Details

### 1. Gizmo Geometry
```rust
// Each axis consists of:
struct AxisGeometry {
    shaft: Cylinder,      // Main axis line
    arrow_head: Cone,     // Directional arrow
    color: Color,         // Axis color (R/G/B)
    highlight_color: Color, // When hovered/active
}
```

### 2. Rendering Pipeline
```
1. Render opaque scene geometry
2. Clear depth buffer (optional)
3. Render gizmos with depth test = Less
4. Apply post-processing (outlines, etc.)
```

### 3. Input Handling Flow
```
1. Mouse down → Generate ray from camera
2. Ray cast against gizmo geometry
3. If hit → Start drag mode
4. Mouse move → Update position along constraint
5. Mouse up → Commit transform
```

## Benefits of True 3D Gizmos
1. **Proper occlusion**: Can see when gizmos are behind objects
2. **Better depth perception**: Shading and perspective help judge position
3. **Performance**: Rendered in same pass as scene (no overlay)
4. **Professional appearance**: Matches industry-standard tools

## Alternative: Hybrid Approach
If full 3D proves too complex, consider:
1. Render gizmos in 3D but in a separate pass
2. Use depth buffer for occlusion testing
3. Combine with 2D overlay for labels/text

## Next Steps
1. Debug why current 3D gizmos aren't visible
2. Fix the shader and rendering issues
3. Implement proper 3D hit testing
4. Test with various camera angles and object positions