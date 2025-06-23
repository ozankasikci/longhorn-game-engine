- **Mobile-First Design**: Optimize for tile-based mobile GPUs from day one - **ECS v2 Integration**: Leverage existing archetypal storage and query system - **Editor-Native Experience**: All features feel natural in professional editor - **Performance-Driven**: Built-in thermal management and quality scaling

### Technology Stack
- **Rendering**: WGPU for cross-platform mobile compatibility
- **ECS Integration**: Existing ECS v2 with archetypal storage
- **Editor**: EGUI with existing dockable panel system
- **Assets**: Handle-based system with async streaming
- **Mobile Optimization**: Sprite batching, texture atlasing, quality scaling

---

## Task 1: Core 2D Components & Systems (Week 1)

### 1.1 Essential 2D Components

**File: `engine-core/src/components.rs`**

```rust
// 2D Sprite Component
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
  pub texture_handle: Option<u64>,  // Asset handle for texture
  pub uv_rect: [f32; 4],       // [x, y, width, height] in texture space (0.0-1.0)
  pub color: [f32; 4],        // RGBA tint multiplier (1.0 = no tint)
  pub flip_x: bool,         // Horizontal flip
  pub flip_y: bool,         // Vertical flip
  pub pivot: [f32; 2],        // Local pivot point [0.0-1.0, 0.0-1.0]
}

impl Default for Sprite {
  fn default() -> Self {
    Self {
      texture_handle: None,
      uv_rect: [0.0, 0.0, 1.0, 1.0],  // Full texture
      color: [1.0, 1.0, 1.0, 1.0],   // White, no tint
      flip_x: false,
      flip_y: false,
      pivot: [0.5, 0.5],        // Center pivot
    }
  }
}

impl Sprite {
  pub fn new() -> Self {
    Self::default()
  }
  
  pub fn with_texture(mut self, handle: u64) -> Self {
    self.texture_handle = Some(handle);
    self
  }
  
  pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
    self.color = [r, g, b, a];
    self
  }
  
  pub fn with_uv_rect(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
    self.uv_rect = [x, y, width, height];
    self
  }
}

// Sprite Renderer Component
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteRenderer {
  pub sprite: Sprite,         // Sprite data
  pub layer: i32,           // Z-order for sorting (-32768 to 32767)
  pub material_override: Option<u64>, // Custom material handle (optional)
  pub enabled: bool,         // Whether to render this sprite
}

impl Default for SpriteRenderer {
  fn default() -> Self {
    Self {
      sprite: Sprite::default(),
      layer: 0,          // Default layer
      material_override: None,
      enabled: true,
    }
  }
}

impl SpriteRenderer {
  pub fn new(sprite: Sprite) -> Self {
    Self {
      sprite,
      layer: 0,
      material_override: None,
      enabled: true,
    }
  }
  
  pub fn with_layer(mut self, layer: i32) -> Self {
    self.layer = layer;
    self
  }
  
  pub fn with_material(mut self, material_handle: u64) -> Self {
    self.material_override = Some(material_handle);
    self
  }
}

// Canvas Component for UI rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
  pub render_mode: CanvasRenderMode, // How the canvas is rendered
  pub sorting_layer: i32,       // Global sorting layer
  pub order_in_layer: i32,      // Order within the sorting layer
  pub pixel_perfect: bool,      // Snap to pixel boundaries
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasRenderMode {
  WorldSpace,             // Rendered in 3D world space
  ScreenSpaceOverlay,         // Rendered as overlay on top of everything
  ScreenSpaceCamera,         // Rendered relative to a specific camera
}

impl Default for Canvas {
  fn default() -> Self {
    Self {
      render_mode: CanvasRenderMode::WorldSpace,
      sorting_layer: 0,
      order_in_layer: 0,
      pixel_perfect: true,
    }
  }
}

// 2D Camera Component
#[derive(Debug, Clone, PartialEq)]
pub struct Camera2D {
  pub size: f32,           // Orthographic size (world units from center to top)
  pub aspect_ratio: f32,       // Width/height ratio (auto-calculated if 0.0)
  pub near: f32,           // Near clipping plane
  pub far: f32,            // Far clipping plane
  pub background_color: [f32; 4],   // Clear color RGBA
  pub viewport_rect: [f32; 4],    // [x, y, width, height] normalized (0.0-1.0)
  pub is_main: bool,         // Whether this is the main 2D camera
}

impl Default for Camera2D {
  fn default() -> Self {
    Self {
      size: 5.0,         // 5 world units from center to top
      aspect_ratio: 0.0,     // Auto-calculate from screen
      near: -10.0,        // Behind the camera
      far: 10.0,         // In front of the camera
      background_color: [0.2, 0.2, 0.3, 1.0], // Dark blue-gray
      viewport_rect: [0.0, 0.0, 1.0, 1.0],  // Full screen
      is_main: false,
    }
  }
}

impl Camera2D {
  pub fn new() -> Self {
    Self::default()
  }
  
  pub fn main_camera() -> Self {
    Self {
      is_main: true,
      ..Default::default()
    }
  }
  
  pub fn with_size(mut self, size: f32) -> Self {
    self.size = size;
    self
  }
  
  pub fn with_background_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
    self.background_color = [r, g, b, a];
    self
  }
}

// Component trait implementations for ECS v2
impl crate::ecs_v2::Component for Sprite {}
impl crate::ecs_v2::Component for SpriteRenderer {}
impl crate::ecs_v2::Component for Canvas {}
impl crate::ecs_v2::Component for Camera2D {}

// Legacy ECS compatibility (if needed)
impl crate::ecs::Component for Sprite {}
impl crate::ecs::Component for SpriteRenderer {}
impl crate::ecs::Component for Canvas {}
impl crate::ecs::Component for Camera2D {}
```

**File: `engine-core/src/lib.rs` (additions)**

```rust
// Re-export 2D components
pub use components::{Sprite, SpriteRenderer, Canvas, CanvasRenderMode, Camera2D};
```

### 1.2 ECS v2 Integration Tests

**File: `engine-core/src/ecs_v2.rs` (test additions)**

```rust
#[cfg(test)]
mod tests_2d {
  use super::*;
  use crate::{Sprite, SpriteRenderer, Camera2D};
  
  #[test]
  fn test_2d_component_storage() {
    let mut world = World::new();
    
    // Create entity with 2D components
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();
    world.add_component(entity, SpriteRenderer::default()).unwrap();
    world.add_component(entity, Camera2D::default()).unwrap();
    
    // Verify components exist
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<SpriteRenderer>(entity).is_some());
    assert!(world.get_component::<Camera2D>(entity).is_some());
    
    // Test archetype creation
    assert_eq!(world.archetype_count(), 1);
  }
  
  #[test]
  fn test_2d_query_system() {
    let mut world = World::new();
    
    // Create multiple sprite entities
    for i in 0..5 {
      let entity = world.spawn();
      world.add_component(entity, Transform::default()).unwrap();
      world.add_component(entity, SpriteRenderer {
        sprite: Sprite::default(),
        layer: i,
        material_override: None,
        enabled: true,
      }).unwrap();
    }
    
    // Query sprites with transforms
    let query = world.query::<(Read<Transform>, Read<SpriteRenderer>)>();
    let results: Vec<_> = query.iter().collect();
    
    assert_eq!(results.len(), 5);
    
    // Test layer sorting
    let layers: Vec<_> = results.iter().map(|(_, sprite_renderer)| sprite_renderer.layer).collect();
    assert!(layers.contains(&0));
    assert!(layers.contains(&4));
  }
  
  #[test]
  fn test_sprite_archetype_migration() {
    let mut world = World::new();
    
    // Create entity with just Transform
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();
    assert_eq!(world.archetype_count(), 1);
    
    // Add SpriteRenderer - should trigger archetype migration
    world.add_component(entity, SpriteRenderer::default()).unwrap();
    assert_eq!(world.archetype_count(), 2);
    
    // Add Camera2D - another migration
    world.add_component(entity, Camera2D::default()).unwrap();
    assert_eq!(world.archetype_count(), 3);
    
    // Verify all components still exist
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<SpriteRenderer>(entity).is_some());
    assert!(world.get_component::<Camera2D>(entity).is_some());
  }
}
```

---

## Task 2: Mobile-Optimized Rendering Backend (Week 1-2)

### 2.1 WGPU Setup and Core Structures

**File: `engine-graphics/Cargo.toml`**

```toml
[package]
name = "engine-graphics"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core engine integration
engine-core = { path = "../engine-core" }

# Graphics rendering
wgpu = "0.18"
winit = "0.29"
bytemuck = { version = "1.14", features = ["derive"] }

# Image loading and processing
image = "0.24"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Math library
glam = "0.24"

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"

[features]
default = []
mobile = []
desktop = []
```

**File: `engine-graphics/src/lib.rs`**

```rust
//! Mobile-first 2D graphics rendering engine
//! 
//! This crate provides high-performance 2D rendering optimized for mobile devices,
//! with features like sprite batching, texture atlasing, and thermal management.

pub mod renderer_2d;
pub mod sprite_batcher;
pub mod texture_atlas;
pub mod mobile_optimizer;
pub mod shaders;

// Core exports
pub use renderer_2d::Renderer2D;
pub use sprite_batcher::{SpriteBatcher, SpriteVertex, SpriteBatch};
pub use texture_atlas::{TextureAtlas, AtlasSprite};
pub use mobile_optimizer::{QualityScaler, QualityLevel, ThermalMonitor};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphicsError {
  #[error("WGPU adapter creation failed")]
  AdapterCreationFailed,
  
  #[error("WGPU device creation failed: {0}")]
  DeviceCreationFailed(String),
  
  #[error("Shader compilation failed: {0}")]
  ShaderCompilationFailed(String),
  
  #[error("Texture loading failed: {0}")]
  TextureLoadingFailed(String),
  
  #[error("Buffer creation failed: {0}")]
  BufferCreationFailed(String),
}

pub type Result<T> = std::result::Result<T, GraphicsError>;
```

**File: `engine-graphics/src/renderer_2d.rs`**

```rust
//! Core 2D renderer with mobile optimizations
//!
//! This module provides the main 2D rendering system optimized for mobile GPUs,
//! featuring sprite batching, texture atlasing, and dynamic quality scaling.

use crate::{SpriteBatcher, TextureAtlas, QualityScaler, GraphicsError, Result};
use engine_core::{Transform, SpriteRenderer, Camera2D};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// Main 2D renderer optimized for mobile devices
pub struct Renderer2D {
  // Core WGPU resources
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: Option<wgpu::Surface>,
  surface_config: Option<wgpu::SurfaceConfiguration>,
  
  // Mobile optimizations
  sprite_batcher: SpriteBatcher,
  texture_atlas: TextureAtlas,
  quality_scaler: QualityScaler,
  
  // Rendering pipelines
  sprite_pipeline: wgpu::RenderPipeline,
  
  // Resource buffers
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  uniform_buffer: wgpu::Buffer,
  
  // Bind groups
  texture_bind_group_layout: wgpu::BindGroupLayout,
  camera_bind_group_layout: wgpu::BindGroupLayout,
  camera_bind_group: Option<wgpu::BindGroup>,
  
  // Performance tracking
  frame_count: u64,
  last_render_time: std::time::Instant,
}

/// Camera uniform data for shaders
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
  view_proj: [[f32; 4]; 4], // 4x4 matrix
  resolution: [f32; 2],   // Screen resolution
  _padding: [f32; 2],    // Padding for alignment
}

impl Renderer2D {
  /// Create a new 2D renderer
  pub async fn new(
    window: Option<&winit::window::Window>,
    width: u32,
    height: u32,
  ) -> Result<Self> {
    // Create WGPU instance with optimal settings for mobile
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });
    
    // Create surface if window is provided
    let surface = if let Some(window) = window {
      Some(unsafe { instance.create_surface(window) }.map_err(|_| GraphicsError::AdapterCreationFailed)?)
    } else {
      None
    };
    
    // Request adapter with mobile-optimized settings
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::LowPower, // Mobile optimization
      compatible_surface: surface.as_ref(),
      force_fallback_adapter: false,
    }).await.ok_or(GraphicsError::AdapterCreationFailed)?;
    
    // Create device with mobile-friendly features
    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(), // Minimal features for mobile compatibility
        limits: wgpu::Limits::downlevel_webgl2_defaults(), // Mobile-compatible limits
        label: None,
      },
      None,
    ).await.map_err(|e| GraphicsError::DeviceCreationFailed(e.to_string()))?;
    
    // Configure surface if available
    let surface_config = if let Some(ref surface) = surface {
      let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo, // VSync for mobile battery life
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
      };
      surface.configure(&device, &config);
      Some(config)
    } else {
      None
    };
    
    // Create bind group layouts
    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
      label: Some("camera_bind_group_layout"),
    });
    
    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
      label: Some("texture_bind_group_layout"),
    });
    
    // Create render pipeline
    let sprite_pipeline = Self::create_sprite_pipeline(
      &device,
      &camera_bind_group_layout,
      &texture_bind_group_layout,
      surface_config.as_ref().map(|c| c.format).unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb),
    )?;
    
    // Create buffers
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("sprite_vertex_buffer"),
      size: (std::mem::size_of::<crate::SpriteVertex>() * 10000) as u64, // 10k sprites max
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("sprite_index_buffer"),
      size: (std::mem::size_of::<u16>() * 60000) as u64, // 6 indices per sprite * 10k sprites
      usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("camera_uniform_buffer"),
      contents: bytemuck::cast_slice(&[CameraUniform {
        view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        resolution: [width as f32, height as f32],
        _padding: [0.0, 0.0],
      }]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    
    // Create camera bind group
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: uniform_buffer.as_entire_binding(),
        },
      ],
      label: Some("camera_bind_group"),
    });
    
    Ok(Self {
      device,
      queue,
      surface,
      surface_config,
      sprite_batcher: SpriteBatcher::new(),
      texture_atlas: TextureAtlas::new(),
      quality_scaler: QualityScaler::new(),
      sprite_pipeline,
      vertex_buffer,
      index_buffer,
      uniform_buffer,
      texture_bind_group_layout,
      camera_bind_group_layout,
      camera_bind_group: Some(camera_bind_group),
      frame_count: 0,
      last_render_time: std::time::Instant::now(),
    })
  }
  
  /// Create the sprite rendering pipeline
  fn create_sprite_pipeline(
    device: &wgpu::Device,
    camera_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> Result<wgpu::RenderPipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("sprite_shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
    });
    
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("sprite_pipeline_layout"),
      bind_group_layouts: &[camera_layout, texture_layout],
      push_constant_ranges: &[],
    });
    
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("sprite_pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[crate::SpriteVertex::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });
    
    Ok(render_pipeline)
  }
  
  /// Update camera uniform buffer
  pub fn update_camera(&mut self, camera: &Camera2D, transform: &Transform, screen_width: f32, screen_height: f32) {
    let aspect = if camera.aspect_ratio > 0.0 { 
      camera.aspect_ratio 
    } else { 
      screen_width / screen_height 
    };
    
    // Create orthographic projection matrix
    let left = -camera.size * aspect;
    let right = camera.size * aspect;
    let bottom = -camera.size;
    let top = camera.size;
    
    let projection = glam::Mat4::orthographic_rh(left, right, bottom, top, camera.near, camera.far);
    
    // Create view matrix from transform
    let view = glam::Mat4::from_translation(glam::Vec3::new(
      -transform.position[0],
      -transform.position[1],
      -transform.position[2],
    ));
    
    let view_proj = projection * view;
    
    let uniform = CameraUniform {
      view_proj: view_proj.to_cols_array_2d(),
      resolution: [screen_width, screen_height],
      _padding: [0.0, 0.0],
    };
    
    self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
  }
  
  /// Begin a new frame
  pub fn begin_frame(&mut self) -> Result<Option<wgpu::SurfaceTexture>> {
    self.frame_count += 1;
    self.sprite_batcher.clear();
    
    // Update quality scaling based on performance
    self.quality_scaler.update();
    
    if let Some(ref surface) = self.surface {
      let output = surface.get_current_texture()
        .map_err(|_| GraphicsError::DeviceCreationFailed("Failed to acquire surface texture".to_string()))?;
      Ok(Some(output))
    } else {
      Ok(None)
    }
  }
  
  /// Submit sprite for rendering
  pub fn submit_sprite(&mut self, sprite_renderer: &SpriteRenderer, transform: &Transform) {
    if !sprite_renderer.enabled {
      return;
    }
    
    self.sprite_batcher.add_sprite(sprite_renderer, transform);
  }
  
  /// End frame and submit for rendering
  pub fn end_frame(&mut self, output: Option<wgpu::SurfaceTexture>) -> Result<()> {
    // Sort and batch sprites for optimal rendering
    self.sprite_batcher.finalize_batches();
    
    if let Some(output) = output {
      let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
      
      let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("sprite_render_encoder"),
      });
      
      {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          label: Some("sprite_render_pass"),
          color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.2,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: true,
            },
          })],
          depth_stencil_attachment: None,
        });
        
        // Set pipeline and bind groups
        render_pass.set_pipeline(&self.sprite_pipeline);
        if let Some(ref camera_bind_group) = self.camera_bind_group {
          render_pass.set_bind_group(0, camera_bind_group, &[]);
        }
        
        // Render sprite batches
        self.sprite_batcher.render(&mut render_pass, &self.queue, &self.vertex_buffer, &self.index_buffer);
      }
      
      self.queue.submit(std::iter::once(encoder.finish()));
      output.present();
    }
    
    Ok(())
  }
  
  /// Resize the renderer
  pub fn resize(&mut self, new_width: u32, new_height: u32) {
    if let (Some(ref surface), Some(ref mut config)) = (&self.surface, &mut self.surface_config) {
      config.width = new_width;
      config.height = new_height;
      surface.configure(&self.device, config);
    }
  }
  
  /// Get rendering statistics
  pub fn get_stats(&self) -> RenderStats {
    RenderStats {
      frame_count: self.frame_count,
      sprite_count: self.sprite_batcher.sprite_count(),
      batch_count: self.sprite_batcher.batch_count(),
      quality_level: self.quality_scaler.current_quality(),
    }
  }
}

/// Rendering statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct RenderStats {
  pub frame_count: u64,
  pub sprite_count: usize,
  pub batch_count: usize,
  pub quality_level: crate::QualityLevel,
}
```

This comprehensive plan provides detailed implementation guidance for creating a production-grade 2D rendering system. Each task builds upon the previous one and integrates seamlessly with your existing ECS v2 and editor architecture.
