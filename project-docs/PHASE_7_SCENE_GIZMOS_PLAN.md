# Phase 7: Scene Manipulation Gizmos - Plan & Implementation

## Phase Overview
Implement Unity-style 3D manipulation gizmos in the Scene View to enable interactive transformation of objects along three axes (X, Y, Z). This will provide intuitive visual tools for moving, rotating, and scaling objects directly in the scene view, matching Unity and Unreal Engine workflows.

## Research Foundation

Based on Unity's official documentation and industry standards:

### Unity Move Tool Gizmo Design
- **Color-coded axes**: Red (X), Green (Y), Blue (Z) 
- **Interactive highlighting**: Selected axis turns yellow during manipulation
- **Multiple movement modes**:
  - Single-axis movement (drag individual arrows)
  - Planar movement (drag colored squares for 2-axis movement)
  - Screen-space movement (shift+drag center for camera-relative movement)
- **Visual feedback**: Real-time position updates with snap-to-grid options

### Technical Implementation Requirements
- Ray-casting for gizmo interaction detection
- Mouse-to-world space projection for movement calculation
- Visual rendering with proper depth testing and size scaling
- Undo/redo system for transform changes
- Integration with Inspector panel for numeric input

## Implementation Plan

### Phase 7.1: Foundation & Gizmo System (45-60 minutes)
**Objective**: Build the core gizmo rendering and interaction framework

#### Task 7.1.1: Gizmo Data Structures
- Create `Gizmo` trait for different manipulation tools
- Implement `MoveGizmo` struct with axis and plane handles
- Add gizmo state management (selected axis, interaction mode)
- Create hit-testing system for gizmo components

#### Task 7.1.2: Gizmo Rendering System
- Implement 3D arrow rendering for X/Y/Z axes with proper colors
- Add planar squares for two-axis movement
- Implement center handle for screen-space movement
- Add visual feedback (highlighting, scaling based on camera distance)

#### Task 7.1.3: Mouse Interaction Framework
- Implement ray-casting from mouse to world space
- Create hit-testing for gizmo components (arrows, planes, center)
- Add drag state management and mouse capture
- Implement proper depth testing and occlusion handling

### Phase 7.2: Move Tool Implementation (30-45 minutes)
**Objective**: Complete the move tool with full Unity-style functionality

#### Task 7.2.1: Single-Axis Movement
- Implement X-axis movement (red arrow) with constraint to X-only
- Implement Y-axis movement (green arrow) with constraint to Y-only  
- Implement Z-axis movement (blue arrow) with constraint to Z-only
- Add visual feedback and axis highlighting during drag

#### Task 7.2.2: Planar Movement
- Implement XY-plane movement (blue square, Z-locked)
- Implement XZ-plane movement (green square, Y-locked)
- Implement YZ-plane movement (red square, X-locked)
- Add plane highlighting and constraint visualization

#### Task 7.2.3: Screen-Space Movement
- Implement camera-relative movement (center handle)
- Add shift-key modifier for screen-space mode
- Handle perspective and orthographic camera differences
- Maintain object depth during screen-space movement

### Phase 7.3: Tool Selection & UI Integration (30 minutes)
**Objective**: Integrate gizmo tools with editor UI and selection system

#### Task 7.3.1: Tool Selection System
- Add move/rotate/scale tool buttons to toolbar
- Implement tool switching with keyboard shortcuts (Q/W/E/R)
- Create tool state persistence and visual feedback
- Add tool-specific cursor changes

#### Task 7.3.2: Selection Integration
- Connect gizmos to selected entity system
- Position gizmo at selected object's transform center
- Handle multi-selection scenarios (average position)
- Update Inspector panel in real-time during manipulation

#### Task 7.3.3: Grid and Snapping
- Implement optional grid snapping for movement
- Add configurable snap increments (0.1, 0.5, 1.0 units)
- Create visual grid overlay in scene view
- Add snap toggle and increment controls

### Phase 7.4: Rotation and Scale Tools (Optional - Future Phase)
**Objective**: Extend gizmo system with rotation and scale manipulation

#### Task 7.4.1: Rotation Gizmo
- Implement rotation rings for each axis (red/green/blue circles)
- Add free rotation with outer sphere handle
- Create angle snapping and visual angle feedback
- Handle gimbal lock and quaternion math

#### Task 7.4.2: Scale Gizmo
- Implement scale handles for each axis with cubes
- Add uniform scaling with center handle
- Create scale constraint visualization
- Maintain object proportions during uniform scaling

## Technical Specifications

### Gizmo Rendering
```rust
pub struct MoveGizmo {
    position: Vec3,
    scale: f32,           // Scale based on camera distance
    selected_axis: Option<GizmoAxis>,
    selected_plane: Option<GizmoPlane>,
    interaction_state: GizmoInteractionState,
}

pub enum GizmoAxis {
    X, Y, Z
}

pub enum GizmoPlane {
    XY, XZ, YZ
}

pub enum GizmoInteractionState {
    Idle,
    Hovering(GizmoComponent),
    Dragging(GizmoComponent, Vec3), // Component and start position
}
```

### Mouse Interaction
```rust
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

pub fn screen_to_world_ray(mouse_pos: Vec2, camera: &Camera, viewport: &Viewport) -> Ray {
    // Convert screen coordinates to world space ray
}

pub fn intersect_gizmo(ray: &Ray, gizmo: &MoveGizmo) -> Option<GizmoComponent> {
    // Ray-intersection testing for gizmo components
}
```

### Transform Updates
```rust
pub fn update_transform_from_gizmo_drag(
    transform: &mut Transform,
    drag_delta: Vec3,
    constraint: MovementConstraint,
) {
    match constraint {
        MovementConstraint::XAxis => transform.position.x += drag_delta.x,
        MovementConstraint::YAxis => transform.position.y += drag_delta.y,
        MovementConstraint::ZAxis => transform.position.z += drag_delta.z,
        MovementConstraint::XYPlane => {
            transform.position.x += drag_delta.x;
            transform.position.y += drag_delta.y;
        }
        // ... other constraints
    }
}
```

## Visual Design Specifications

### Color Scheme (Unity Standard)
- **X-Axis**: Red (#FF0000) - Right/Left movement
- **Y-Axis**: Green (#00FF00) - Up/Down movement  
- **Z-Axis**: Blue (#0000FF) - Forward/Backward movement
- **Selected/Hover**: Yellow (#FFFF00) - Active manipulation
- **Inactive**: Gray (#808080) - Non-selected tools

### Gizmo Components
- **Arrow Shafts**: Cylindrical with length proportional to camera distance
- **Arrow Heads**: Conical tips for clear direction indication
- **Plane Handles**: Small colored squares at axis intersections
- **Center Handle**: Larger multi-colored cube for screen-space movement
- **Scale Factor**: Gizmos maintain consistent screen size regardless of zoom

### Visual Feedback
- Smooth color transitions during hover/selection
- Subtle animation for tool switching
- Real-time transform value display during manipulation
- Grid overlay with configurable opacity and spacing

## Integration Points

### ECS Integration
- Gizmo system operates on selected entities with Transform components
- Real-time updates reflect in Inspector panel
- Changes are applied through ECS mutation system

### Camera System Integration
- Gizmo size scales with camera distance for consistent visibility
- Ray-casting uses camera projection matrices
- Screen-space movement respects camera orientation

### Scene View Integration
- Gizmos render after scene objects but before UI overlays
- Proper depth testing prevents gizmos from appearing through objects
- Mouse capture prevents scene navigation during gizmo manipulation

## Success Criteria

### Phase 7.1 Complete When:
- Gizmo framework renders correctly in Scene View
- Mouse interaction system detects gizmo components
- Basic hit-testing and selection feedback works

### Phase 7.2 Complete When:
- All three axes support constrained movement
- Planar movement works for all three planes
- Screen-space movement functions with camera orientation
- Visual feedback matches Unity standards

### Phase 7.3 Complete When:
- Tool selection integrates with toolbar and keyboard shortcuts
- Selected objects display appropriate gizmos
- Inspector updates in real-time during manipulation
- Basic grid snapping functions correctly

## Performance Considerations
- Efficient ray-casting algorithms to avoid frame drops
- Optimized gizmo rendering for complex scenes
- Minimal overhead when gizmos are not actively used
- Proper batching of gizmo geometry for rendering

## Risk Mitigation
1. **Mathematical Complexity**: Use proven ray-intersection algorithms and test thoroughly
2. **Performance Impact**: Profile early and optimize rendering pipeline
3. **User Experience**: Follow Unity conventions closely for familiar workflow
4. **Integration Issues**: Design modular system that can be disabled/enabled

## Next Steps After Completion
This phase establishes the foundation for advanced scene editing tools:
- Rotation gizmos with angle constraints
- Scale gizmos with proportional and non-proportional modes
- Custom gizmos for specific component types
- Advanced snapping (vertex, edge, surface)
- Multi-object manipulation tools

**Ready to begin Phase 7.1.1 - Gizmo Data Structures implementation?**