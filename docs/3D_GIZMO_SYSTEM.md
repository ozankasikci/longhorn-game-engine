# 3D Gizmo System Documentation

## Overview

The Longhorn Game Engine uses a Unity-style 3D gizmo system for object manipulation in the scene view. The system provides visual handles for transforming objects in 3D space with intuitive mouse-based interaction.

## Recent Updates

The 3D gizmo system has been cleaned up and simplified:
- Removed redundant gizmo implementations (gizmos.rs, simple_gizmos.rs, unity_style_gizmos.rs)
- Consolidated all gizmo logic into two core files:
  - `gizmo_3d.rs` - Renderer-side implementation
  - `gizmo_3d_input.rs` - Input handling
- Removed unused test files and scene input modules
- Simplified GizmoSystem to just track the active tool state

## Architecture

### Core Components

1. **Renderer-side (crates/implementation/engine-renderer-3d/src/gizmo_3d.rs)**
   - `GizmoRenderer3D`: Handles the visual rendering of 3D gizmos
   - Renders arrows for single-axis movement (X, Y, Z)
   - Renders plane handles for two-axis movement (XY, XZ, YZ)
   - Uses WGPU for GPU-accelerated rendering

2. **Input Handler (crates/application/engine-editor-egui/src/panels/scene_view/gizmo_3d_input.rs)**
   - `Gizmo3DInput`: Processes mouse input for gizmo interaction
   - Handles hit detection for arrows and plane handles
   - Manages drag operations with proper constraint calculations

### Features

#### Visual Design
- **Axis Arrows**: 
  - Red (X-axis), Green (Y-axis), Blue (Z-axis)
  - Cylindrical shafts with conical tips for professional appearance
  - 12-segment geometry for smooth rendering

- **Plane Handles**:
  - Yellow (XY plane), Cyan (XZ plane), Magenta (YZ plane)
  - Semi-transparent squares at the origin
  - Size: 0.3 world units for good visibility

#### Rendering Features
- **Always-on-top rendering**: Gizmos render without depth testing to remain visible
- **Transparent object support**: Selected objects render with 50% transparency
- **World-space scaling**: Gizmos scale naturally with camera zoom
- **Proper render order**: Objects → Plane handles → Arrows

#### Input Features
- **Forgiving hit detection**: 10-pixel padding on plane handles
- **Smart priority**: Axes have priority over planes when overlapping
- **Dead zone**: 30-pixel radius around center prevents axis stealing
- **Smooth dragging**: Constrained movement along axes or planes

### Implementation Details

#### Gizmo Scaling
```wgsl
// From gizmo_3d.wgsl
let base_scale = uniforms.viewport_size.z * 0.01; // Fixed world-space size
let final_scale = base_scale;
```

#### Hit Detection
```rust
// Plane handle hit testing with padding
let padding = 10.0; // pixels
let hit_rect = egui::Rect::from_min_max(
    egui::pos2(min_x - padding, min_y - padding),
    egui::pos2(max_x + padding, max_y + padding)
);
```

#### Movement Constraints
```rust
match axis {
    Axis::X => start_pos + Vec3::new(delta.x, 0.0, 0.0),
    Axis::Y => start_pos + Vec3::new(0.0, delta.y, 0.0),
    Axis::Z => start_pos + Vec3::new(0.0, 0.0, delta.z),
    Axis::XY => start_pos + Vec3::new(delta.x, delta.y, 0.0),
    Axis::XZ => start_pos + Vec3::new(delta.x, 0.0, delta.z),
    Axis::YZ => start_pos + Vec3::new(0.0, delta.y, delta.z),
}
```

## Usage

### Basic Operation
1. Select an object in the scene
2. The gizmo appears at the object's position
3. Click and drag:
   - **Arrows**: Move along single axis
   - **Colored squares**: Move along two axes simultaneously
4. Object position updates in real-time

### Input Controls
- **Left Mouse Button**: Drag gizmo handles
- **Right Mouse Button**: Camera navigation (doesn't interfere with gizmos)

## Integration Points

### Scene View Integration
The gizmo system integrates with the scene view through:
- `SceneViewPanel::show()`: Main rendering loop
- Hit testing occurs before camera navigation
- Transform updates write directly to ECS components

### ECS Integration
- Reads `Transform` components for object positions
- Updates positions through `World::get_component_mut()`
- Selection state passed through rendering pipeline

## Future Enhancements

Potential improvements for the gizmo system:
1. **Rotation gizmos**: Circular handles for rotation
2. **Scale gizmos**: Corner handles for scaling
3. **Multi-select**: Transform multiple objects
4. **Snapping**: Grid-based movement snapping
5. **Local/World space toggle**: Switch coordinate systems

## Code Structure

```
crates/
├── implementation/
│   └── engine-renderer-3d/
│       └── src/
│           ├── gizmo_3d.rs        # Rendering logic
│           └── shaders/
│               └── gizmo_3d.wgsl  # Gizmo shader
└── application/
    └── engine-editor-egui/
        └── src/
            └── panels/
                └── scene_view/
                    ├── gizmo_3d_input.rs  # Input handling
                    └── mod.rs             # Integration
```

## Performance Considerations

- Gizmos use instanced rendering for efficiency
- Hit detection uses screen-space calculations to avoid unnecessary 3D math
- Minimal state tracking reduces memory overhead
- No depth buffer writes improve rendering performance