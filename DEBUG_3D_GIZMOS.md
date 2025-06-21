# 3D Gizmos Debug Guide

## Current Status
The 3D gizmos are enabled but not working. Here's what we need to check:

## 1. Check if Gizmos are Visible
Run the editor and look for:
- Red arrow (X axis)
- Green arrow (Y axis)  
- Blue arrow (Z axis)

If you don't see them, the issue is with rendering.

## 2. Check Console Output
When you select an object, you should see:
```
3D GIZMO DEBUG: Enabled gizmo at position Vec3(...), transform: Mat4(...)
```

When you click and drag, you should see:
```
3D GIZMO INPUT: Drag started at mouse pos Pos2(...)
3D GIZMO HIT TEST: Testing at gizmo pos Vec3(...), mouse pos Pos2(...)
```

## 3. Possible Issues

### A. Gizmos Not Visible
- The gizmo shader might have issues with the axis rotation
- The gizmo size might be too small
- The depth test might be hiding them

### B. Hit Testing Failing
- The screen projection might be incorrect
- The hit threshold might be too small
- The gizmo scale calculation might not match

### C. Input Not Working
- The input might be consumed by navigation
- The drag plane calculation might be wrong

## 4. Quick Fix to Try
In `gizmo_3d.rs`, change the depth test from "Always" to "Less":
```rust
depth_compare: wgpu::CompareFunction::Less, // Instead of Always
```

This will make gizmos respect depth, which might help visibility.

## 5. Alternative: Re-enable 2D Gizmos
If 3D gizmos aren't working, you can re-enable the 2D overlay gizmos by uncommenting the code in `scene_view/mod.rs`.

## Debug Steps
1. Check if gizmos are visible at all
2. Check console for debug messages
3. Try clicking where the gizmos should be
4. Check if the gizmo scale is correct