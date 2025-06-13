# Phase 15: Renderer Implementation Guide

## Quick Start Implementation

Based on the research, here's a practical approach to implement a working renderer quickly:

## 1. Create the Renderer Crate

```bash
cargo new --lib crates/implementation/engine-renderer-3d
```

### Cargo.toml
```toml
[package]
name = "engine-renderer-3d"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "0.20"
bytemuck = { version = "1.13", features = ["derive"] }
glam = "0.24"
egui = "0.28"
egui-wgpu = "0.28"
anyhow = "1.0"
log = "0.4"

# Internal deps
engine-ecs-core = { path = "../../core/engine-ecs-core" }
engine-components-3d = { path = "../../core/engine-components-3d" }
```

## 2. Core Renderer Structure

### src/lib.rs
```rust
pub mod renderer;
pub mod mesh;
pub mod material;
pub mod camera;
pub mod scene;
pub mod integration;

pub use renderer::Renderer3D;
pub use scene::{RenderScene, RenderObject};
pub use integration::egui::EguiRenderWidget;
```

### src/renderer.rs
```rust
use std::sync::Arc;
use wgpu::util::DeviceExt;
use glam::Mat4;

pub struct Renderer3D {
    // Core resources
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    
    // Render targets
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    
    // Pipeline
    render_pipeline: wgpu::RenderPipeline,
    
    // Bind groups
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    
    // Mesh data
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Renderer3D {
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        width: u32,
        height: u32,
    ) -> Result<Self, anyhow::Error> {
        // Implementation
    }
    
    pub fn render(&mut self, scene: &RenderScene) -> Result<(), anyhow::Error> {
        // Render to internal texture
    }
    
    pub fn get_render_texture(&self) -> &wgpu::TextureView {
        &self.render_view
    }
}
```

## 3. Scene Representation

### src/scene.rs
```rust
use glam::{Mat4, Vec3};

pub struct RenderScene {
    pub camera: Camera,
    pub objects: Vec<RenderObject>,
}

pub struct Camera {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

pub struct RenderObject {
    pub transform: Mat4,
    pub mesh_id: usize,
    pub material_id: usize,
}
```

## 4. egui Integration (Texture-based)

### src/integration/egui.rs
```rust
use egui::{Widget, Response, Ui, TextureId};
use std::sync::{Arc, Mutex};

pub struct EguiRenderWidget {
    renderer: Arc<Mutex<Renderer3D>>,
    texture_id: Option<TextureId>,
}

impl EguiRenderWidget {
    pub fn new(renderer: Arc<Mutex<Renderer3D>>) -> Self {
        Self {
            renderer,
            texture_id: None,
        }
    }
    
    pub fn render_scene(&mut self, scene: &RenderScene) {
        let mut renderer = self.renderer.lock().unwrap();
        renderer.render(scene).ok();
    }
}

impl Widget for &mut EguiRenderWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.available_rect();
        
        if let Some(texture_id) = self.texture_id {
            ui.image(texture_id, rect.size())
        } else {
            ui.label("Renderer not initialized")
        }
    }
}
```

## 5. Simple Working Example

### Basic Shader (shader.wgsl)
```wgsl
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

## 6. Integration with Editor

### In engine-editor-egui
```rust
use engine_renderer_3d::{Renderer3D, RenderScene, EguiRenderWidget};

pub struct SceneView3D {
    renderer: Arc<Mutex<Renderer3D>>,
    widget: EguiRenderWidget,
}

impl SceneView3D {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let renderer = Renderer3D::new(device, queue, 800, 600)
            .await
            .expect("Failed to create renderer");
        let renderer = Arc::new(Mutex::new(renderer));
        let widget = EguiRenderWidget::new(renderer.clone());
        
        Self { renderer, widget }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, world: &World) {
        // Convert ECS world to RenderScene
        let scene = self.create_render_scene(world);
        
        // Render scene
        self.widget.render_scene(&scene);
        
        // Display in UI
        ui.add(&mut self.widget);
    }
}
```

## 7. Key Implementation Tips

1. **Start Simple**: Get a triangle rendering first
2. **Use Existing Examples**: wgpu has great examples to reference
3. **Handle Errors Gracefully**: GPU operations can fail
4. **Profile Early**: Add timing measurements from the start
5. **Test Incrementally**: Build features one at a time

## 8. Common Pitfalls to Avoid

1. **Don't Abstract Too Early**: Use wgpu directly first
2. **Avoid Global State**: Pass resources explicitly
3. **Don't Block on GPU**: Use async operations
4. **Handle Resize**: Update render targets on window resize
5. **Validate Shaders**: Check shader compilation errors

## 9. Performance Considerations

1. **Batch Draw Calls**: Group similar objects
2. **Use Instancing**: For repeated objects
3. **Minimize State Changes**: Sort by material/pipeline
4. **Update Buffers Efficiently**: Use staging buffers
5. **Cull Invisible Objects**: Implement frustum culling

## 10. Testing Approach

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_renderer_creation() {
        // Test renderer can be created
    }
    
    #[test]
    fn test_render_empty_scene() {
        // Test rendering works with no objects
    }
    
    #[test]
    fn test_render_single_cube() {
        // Test basic mesh rendering
    }
}
```

## 11. Migration from Current Code

### Phase 1: Parallel Implementation
- Keep existing scene_view_impl.rs working
- Create new renderer in parallel
- Add feature flag to switch between them

### Phase 2: Integration
- Replace scene view rendering with new system
- Migrate existing features (grid, overlays)
- Ensure all functionality preserved

### Phase 3: Cleanup
- Remove old rendering code
- Clean up unused modules
- Update documentation

## 12. Expected Timeline

- **Week 1**: Core renderer setup and basic triangle
- **Week 2**: Resource management and mesh rendering
- **Week 3**: Scene integration and materials
- **Week 4**: Optimization and polish

This implementation guide provides a practical, working approach that avoids the callback issues we encountered and follows best practices from successful Rust renderers.