# 3D Gizmo Input Implementation

## Overview
I've implemented a complete input handling system for the 3D gizmos that converts 2D mouse input into 3D object transformations.

## Key Components

### 1. Gizmo3DInput Module
**File**: `crates/application/engine-editor-egui/src/panels/scene_view/gizmo_3d_input.rs`

This module handles:
- **Hit Testing**: Detects which gizmo axis the mouse is clicking on
- **Drag Plane Calculation**: Creates an appropriate plane for dragging along each axis
- **Ray-Plane Intersection**: Converts 2D mouse movement to 3D world movement
- **Transform Updates**: Applies the calculated movement to the selected entity

### 2. Integration with Scene View
The input handler is integrated into the scene view panel with proper priority:
1. **Gizmo input is handled first** (before navigation)
2. **Navigation only happens if gizmo doesn't handle the input**
3. **Gizmos are disabled during scene navigation**

## How It Works

### Hit Testing
1. Projects gizmo position and axis endpoints to screen space
2. Calculates distance from mouse to each axis line
3. Returns the closest axis within the hit threshold (20 pixels)

### Dragging
1. **Start Drag**: 
   - Calculates a drag plane that contains the selected axis
   - The plane faces the camera as much as possible
   - Handles edge cases when camera looks along an axis

2. **Update Drag**:
   - Converts mouse position to a world ray
   - Intersects the ray with the drag plane
   - Projects the intersection onto the selected axis
   - Updates the entity's transform

3. **End Drag**: Cleans up the drag state

## Key Features

1. **Camera-Aware Drag Planes**: The drag plane adapts based on camera angle for intuitive movement
2. **Axis-Constrained Movement**: Objects only move along the selected axis
3. **Proper 3D Projection**: Uses the same matrices as the renderer for accurate hit testing
4. **Edge Case Handling**: Works even when the camera is looking along an axis

## Usage
1. Click and drag on a gizmo axis (red/green/blue arrow)
2. The object will move along that world axis
3. Movement is constrained to the selected axis only
4. Release mouse to complete the movement

## Technical Details

### Ray Casting
```rust
// Convert screen position to world ray
let ndc_x = (screen_pos.x - rect.left()) / rect.width() * 2.0 - 1.0;
let ndc_y = 1.0 - (screen_pos.y - rect.top()) / rect.height() * 2.0;
let clip_pos = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
let eye_pos = projection_matrix.inverse() * clip_pos;
let eye_dir = Vec4::new(eye_pos.x, eye_pos.y, -1.0, 0.0);
let world_dir = view_matrix.inverse() * eye_dir;
```

### Plane Selection
The drag plane is chosen to:
1. Contain the movement axis
2. Face the camera as much as possible
3. Use fallback planes for degenerate cases

This ensures smooth and predictable gizmo behavior from any viewing angle.