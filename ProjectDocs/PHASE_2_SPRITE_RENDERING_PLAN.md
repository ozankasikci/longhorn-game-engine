# Phase 2: Sprite Rendering in Camera View

## Overview
Now that we have a working camera system and Scene View, the next step is to implement sprite rendering so 2D sprites can be displayed in the 3D camera view. This enables 2D game development within the 3D engine architecture.

## Current State Analysis

### ✅ What We Have
1. **Working Camera System**: Camera entities with proper transforms and frustum visualization
2. **Scene View Panel**: Visual representation of 3D space with object positioning
3. **ECS Components**: `Sprite`, `SpriteRenderer`, and `Canvas` components already defined
4. **Transform System**: Full 3D positioning, rotation, and scaling for all entities
5. **Material System**: Basic PBR materials for textures and colors

### ❌ What's Missing
1. **Sprite Texture Loading**: No texture assets or loading system
2. **Sprite Rendering Pipeline**: No visual rendering of sprite components
3. **2D/3D Integration**: Sprites not positioned in 3D camera space
4. **Sprite Materials**: No sprite-specific shaders or materials
5. **Billboard/Camera Facing**: Sprites need to face camera properly

## Implementation Plan

### Phase 2.1: Basic Sprite Texture System (45-60 minutes)
**Objective**: Load and display simple colored sprites in Scene View

#### Task 2.1.1: Create Basic Texture Asset System
- Add simple colored texture generation (red, green, blue, white squares)
- Implement basic texture handles and references
- Create default sprite materials

#### Task 2.1.2: Extend Scene View Sprite Rendering
- Add sprite visualization to Scene View (colored rectangles)
- Implement sprite scale and rotation display
- Show sprite pivot points and bounds

#### Task 2.1.3: Add Sprite Entities to Test Scene
- Create sprite entities with SpriteRenderer components
- Position sprites in 3D space visible to camera
- Test different sprite sizes, colors, and rotations

### Phase 2.2: 3D Billboard Sprites (30-45 minutes)
**Objective**: Make sprites properly face the camera in 3D space

#### Task 2.2.1: Implement Billboard Behavior
- Calculate camera-facing rotation for sprites
- Update sprite transforms to always face active camera
- Handle sprite orientation (world-space vs screen-space)

#### Task 2.2.2: Sprite Depth and Layering
- Implement sprite sorting by distance from camera
- Add sprite layer system for rendering order
- Handle transparency and alpha blending

### Phase 2.3: Advanced Sprite Features (45 minutes)
**Objective**: Professional sprite workflow and editing

#### Task 2.3.1: Sprite Inspector Integration
- Add sprite editing in Inspector panel
- Live preview of sprite changes in Scene View
- Sprite texture, color, and size editing

#### Task 2.3.2: Sprite Gizmos and Selection
- Visual sprite bounds and pivot gizmos in Scene View
- Click-to-select sprites in 3D space
- Transform handles for sprite manipulation

#### Task 2.3.3: Canvas and UI Sprite Support
- Implement world-space Canvas rendering
- Screen-space overlay sprites for UI
- Camera-relative sprite positioning

## Technical Implementation Details

### Sprite Data Structures

```rust
// Enhanced Sprite component for 3D integration
#[derive(Debug, Clone)]
pub struct Sprite3D {
    pub sprite: Sprite,              // Base sprite data
    pub billboard_mode: BillboardMode, // How sprite faces camera
    pub depth_offset: f32,           // Z-fighting prevention
    pub world_size: Vec2,            // Size in world units
}

#[derive(Debug, Clone)]
pub enum BillboardMode {
    None,                    // Fixed orientation
    FaceCamera,              // Always face camera
    CameraPlane,             // Align with camera plane
    ConstrainedAxis(Vec3),   // Rotate around fixed axis
}
```

### Scene View Integration

```rust
// In draw_simple_scene_view()
fn draw_sprites(&mut self, ui: &mut egui::Ui, center: egui::Pos2, scale: f32) {
    for (entity, (transform, sprite_renderer)) in self.world.query::<(Read<Transform>, Read<SpriteRenderer>)>().iter() {
        // Project sprite position to screen
        let screen_pos = project_to_screen(transform.position, center, scale);
        
        // Calculate sprite size in screen space
        let sprite_size = calculate_sprite_screen_size(&sprite_renderer.sprite, transform.scale, scale);
        
        // Draw sprite rectangle with color
        let sprite_rect = egui::Rect::from_center_size(screen_pos, sprite_size);
        let color = get_sprite_color(&sprite_renderer.sprite);
        
        ui.painter().rect_filled(sprite_rect, egui::Rounding::ZERO, color);
        
        // Draw sprite bounds if selected
        if self.selected_entity == Some(entity) {
            ui.painter().rect_stroke(sprite_rect, egui::Rounding::ZERO, 
                egui::Stroke::new(2.0, egui::Color32::YELLOW));
        }
    }
}
```

### Billboard Camera Facing

```rust
// System to update sprite rotations to face camera
fn update_billboard_sprites(world: &mut WorldV2) {
    let camera_transform = get_main_camera_transform(world);
    
    for (entity, (mut transform, sprite_3d)) in world.query_mut::<(Write<Transform>, Read<Sprite3D>)>().iter() {
        match sprite_3d.billboard_mode {
            BillboardMode::FaceCamera => {
                let look_dir = (camera_transform.position - transform.position).normalize();
                transform.rotation = look_at_rotation(look_dir);
            }
            BillboardMode::CameraPlane => {
                transform.rotation = camera_transform.rotation;
            }
            _ => {} // No billboard behavior
        }
    }
}
```

## Expected Workflow

### 1. Creating Sprites
```rust
// In editor initialization
let sprite_entity = world.spawn();
world.add_component(sprite_entity, Transform {
    position: [0.0, 1.0, 0.0],
    rotation: [0.0, 0.0, 0.0],
    scale: [2.0, 2.0, 1.0], // 2x2 world units
}).unwrap();

world.add_component(sprite_entity, SpriteRenderer {
    sprite: Sprite::new().with_color(1.0, 0.0, 0.0, 1.0), // Red sprite
    layer: 0,
    enabled: true,
}).unwrap();

world.add_component(sprite_entity, Name::new("Red Sprite")).unwrap();
```

### 2. Sprite Visibility in Scene View
- **Red rectangles** representing sprites in 3D space
- **Proper scaling** based on transform and camera distance
- **Selection highlights** with yellow outline
- **Billboard rotation** arrows showing facing direction

### 3. Inspector Integration
- **Sprite Properties**: Color, texture, flip X/Y, pivot point
- **Transform Properties**: Position, rotation, scale (world units)
- **Renderer Properties**: Layer, enabled state, material override

## Success Criteria

### ✅ Phase 2.1 Complete When:
- Colored sprite rectangles visible in Scene View
- Sprites positioned correctly in 3D space relative to camera
- Can create and edit sprite entities in editor

### ✅ Phase 2.2 Complete When:
- Sprites automatically face camera (billboard behavior)
- Proper depth sorting and layering
- Sprites render correctly at different distances from camera

### ✅ Phase 2.3 Complete When:
- Full sprite editing workflow in Inspector
- Visual sprite manipulation gizmos in Scene View
- Professional Unity-like sprite development experience

## Integration with Existing Systems

### Camera System
- Sprites use camera transform for billboard calculations
- Camera frustum culling for sprite visibility
- Distance-based scaling and LOD

### ECS Integration
- Sprites work seamlessly with existing Transform components
- SpriteRenderer components display in Inspector
- Visibility and Name components work with sprites

### Material System
- Sprite materials use engine-materials-core
- Texture handles and UV mapping
- Alpha blending and transparency support

## Risk Mitigation

1. **Texture Loading Complexity**: Start with simple colored rectangles before implementing full texture system
2. **Billboard Math Issues**: Use proven quaternion/matrix libraries (glam)
3. **Performance Concerns**: Profile sprite rendering early, implement culling
4. **3D/2D Coordinate Confusion**: Clear documentation of coordinate spaces and transforms

## Next Steps

This phase builds directly on our working camera system and Scene View. The first task is to add basic sprite visualization to the existing `draw_simple_scene_view` function, then gradually add more sophisticated features.

**Ready to start with Phase 2.1, Task 2.1.1 - Create Basic Texture Asset System?**