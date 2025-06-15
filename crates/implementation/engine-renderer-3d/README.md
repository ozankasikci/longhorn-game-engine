# Engine Renderer 3D

WGPU-based 3D renderer implementation for the Longhorn Game Engine.

## Overview

This crate provides a standalone 3D renderer that can render to textures for integration with UI frameworks like egui. It follows a retained-mode architecture with efficient batching and culling.

## Key Features

- **Render-to-texture**: Renders scenes to textures that can be displayed in UI
- **ECS integration**: Bridges ECS world to rendering through `EcsRenderBridge`
- **Resource management**: Efficient mesh and material management
- **Camera system**: Support for multiple cameras with different projections
- **Basic lighting**: Directional and point lights
- **Culling**: Frustum culling for performance

## Main Types

- `Renderer3D`: Main renderer managing GPU resources and render pipeline
- `Camera`: 3D camera with view and projection matrices
- `Mesh`: Vertex and index buffer management
- `Material`: PBR material properties
- `RenderScene`: Scene description for rendering
- `EcsRenderBridge`: Converts ECS world to render scene
- `EguiRenderWidget`: egui widget for displaying rendered scenes

## Usage Example

```rust
use engine_renderer_3d::{Renderer3D, Camera, EguiRenderWidget, EcsRenderBridge};

// Initialize renderer
let renderer = Renderer3D::new(device, queue, width, height).await?;

// Create camera
let camera = Camera::new(aspect_ratio);

// Create ECS bridge
let bridge = EcsRenderBridge::new(mesh_mappings, material_mappings);

// Convert ECS world to render scene
let render_scene = bridge.world_to_render_scene(&ecs_world, camera);

// Render and display in egui
let widget = EguiRenderWidget::new(Arc::new(Mutex::new(renderer)));
widget.render_scene(&render_scene)?;
ui.add(&widget);
```

## Dependencies

- `wgpu`: GPU rendering
- `bytemuck`: Zero-copy buffer uploads
- `glam`: Math operations
- `egui`: UI integration