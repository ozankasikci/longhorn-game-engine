# Engine Components 3D

Core 3D components for the Longhorn Game Engine's ECS system.

## Overview

This crate provides fundamental 3D components that can be attached to entities in the ECS world. All components implement the `Component` trait from `engine-component-traits`.

## Components

### Transform
- **Position**: 3D position in world space `[f32; 3]`
- **Rotation**: Euler angles in radians `[f32; 3]` (pitch, yaw, roll)
- **Scale**: Non-uniform scale `[f32; 3]`

### Mesh Components
- **MeshFilter**: References which mesh to render
- **MeshRenderer**: Rendering properties and material reference
- **Mesh**: Mesh type (Cube, Sphere, Plane, Cylinder, Cone, Torus)

### Rendering
- **Material**: PBR material properties (albedo, metallic, roughness, emissive)
- **Light**: Light sources (Directional, Point, Spot)
- **Camera**: Camera component for rendering viewpoints

### Physics (Placeholders)
- **Collider**: Physics collision shapes
- **Rigidbody**: Physics simulation properties

### Other
- **Visibility**: Controls entity visibility
- **Bounds**: Axis-aligned bounding box for culling

## Usage Example

```rust
use engine_components_3d::{Transform, MeshFilter, MeshRenderer, Material};
use engine_ecs_core::World;

let mut world = World::new();

// Create a cube entity
let cube = world.spawn((
    Transform::default(),
    MeshFilter { mesh_type: MeshType::Cube },
    MeshRenderer { material_id: 0 },
    Material::default(),
));
```

## Dependencies

- `engine-component-traits`: Component trait definition
- `serde`: Serialization support
- `glam`: Math types (through re-exports)