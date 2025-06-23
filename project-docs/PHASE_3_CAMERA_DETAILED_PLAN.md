# Phase 3: Advanced Camera System - Detailed Implementation Plan

## Overview

This phase implements a comprehensive camera system crate for our mobile-first game engine, building upon the ECS v2 foundation and 2D rendering pipeline established in previous phases. The camera system will provide advanced viewport management, efficient culling, and mobile-optimized rendering capabilities.

## Phase 3 Architecture

### Core Principles
- **Mobile-First Design**: Aggressive culling and quality scaling for mobile devices
- **ECS v2 Integration**: Seamless integration with existing archetypal storage system
- **Performance-Driven**: GPU-optimized matrix calculations and efficient culling
- **Modular Design**: Clean separation between camera logic and rendering systems

### Technology Stack
- **Graphics**: WGPU for cross-platform mobile compatibility
- **Math**: glam for matrix calculations and transformations
- **ECS**: Existing ECS v2 with archetypal storage
- **Memory**: bytemuck for safe GPU buffer transmutation
- **Editor**: EGUI integration for visual camera manipulation

---

## Task 1: Core Camera Components (Week 1)

### 1.1 Basic Camera Component

**File: `crates/engine-camera/Cargo.toml`**

```toml
[package]
name = "engine-camera"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core engine integration
engine-core = { path = "../engine-core" }

# Math and graphics
glam = "0.24"
bytemuck = { version = "1.14", features = ["derive"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"

[features]
default = []
debug = []
```

**File: `crates/engine-camera/src/lib.rs`**

```rust
//! Advanced camera system for mobile-first game engine
//! 
//! This crate provides sophisticated camera management with viewport control,
//! efficient culling, and mobile-optimized rendering capabilities.

pub mod camera;
pub mod viewport;
pub mod projection;
pub mod culling;
pub mod controllers;

// Core exports
pub use camera::{Camera, CameraType, CameraComponent};
pub use viewport::{Viewport, ViewportTransform};
pub use projection::{ProjectionMatrix, OrthographicProjection, PerspectiveProjection};
pub use culling::{Frustum, CullingResult, CullingStats};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraError {
  #[error("Invalid viewport dimensions: width={0}, height={1}")]
  InvalidViewport(u32, u32),
  
  #[error("Invalid projection parameters: {0}")]
  InvalidProjection(String),
  
  #[error("Matrix calculation failed: {0}")]
  MatrixCalculationFailed(String),
  
  #[error("Culling operation failed: {0}")]
  CullingFailed(String),
}

pub type Result<T> = std::result::Result<T, CameraError>;
```

**File: `crates/engine-camera/src/camera.rs`**

```rust
//! Core camera implementation with ECS v2 integration

use crate::{Viewport, ProjectionMatrix, Frustum, CameraError, Result};
use engine_core::{Transform, Component, ecs_v2};
use glam::{Mat4, Vec3, Vec4};
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

/// Camera types supported by the engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CameraType {
  /// 2D orthographic camera for 2D games
  Orthographic2D {
    size: f32,
    near: f32,
    far: f32,
  },
  /// 3D perspective camera for 3D games
  Perspective3D {
    fov_degrees: f32,
    near: f32,
    far: f32,
  },
  /// Custom projection matrix
  Custom {
    projection_matrix: [[f32; 4]; 4],
  },
}

impl Default for CameraType {
  fn default() -> Self {
    Self::Orthographic2D {
      size: 5.0,
      near: -10.0,
      far: 10.0,
    }
  }
}

/// Core camera with view and projection matrix management
#[derive(Debug, Clone)]
pub struct Camera {
  camera_type: CameraType,
  viewport: Viewport,
  view_matrix: Mat4,
  projection_matrix: Mat4,
  view_projection_matrix: Mat4,
  frustum: Frustum,
  
  // Rendering properties
  clear_color: [f32; 4],
  clear_depth: f32,
  render_order: i32,
  enabled: bool,
  
  // Performance tracking
  last_update_frame: u64,
  dirty_flags: CameraDirtyFlags,
}

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy)]
  struct CameraDirtyFlags: u8 {
    const VIEW_MATRIX = 0b0001;
    const PROJECTION_MATRIX = 0b0010;
    const FRUSTUM = 0b0100;
    const ALL = 0b0111;
  }
}

impl Camera {
  /// Create a new camera with specified type and viewport
  pub fn new(camera_type: CameraType, viewport: Viewport) -> Self {
    let mut camera = Self {
      camera_type,
      viewport,
      view_matrix: Mat4::IDENTITY,
      projection_matrix: Mat4::IDENTITY,
      view_projection_matrix: Mat4::IDENTITY,
      frustum: Frustum::default(),
      clear_color: [0.2, 0.2, 0.3, 1.0],
      clear_depth: 1.0,
      render_order: 0,
      enabled: true,
      last_update_frame: 0,
      dirty_flags: CameraDirtyFlags::ALL,
    };
    
    camera.update_projection_matrix().ok();
    camera
  }
  
  /// Create a 2D orthographic camera
  pub fn orthographic_2d(size: f32, viewport: Viewport) -> Self {
    Self::new(
      CameraType::Orthographic2D {
        size,
        near: -10.0,
        far: 10.0,
      },
      viewport,
    )
  }
  
  /// Create a 3D perspective camera
  pub fn perspective_3d(fov_degrees: f32, viewport: Viewport) -> Self {
    Self::new(
      CameraType::Perspective3D {
        fov_degrees,
        near: 0.1,
        far: 1000.0,
      },
      viewport,
    )
  }
  
  /// Update view matrix from transform
  pub fn update_view_matrix(&mut self, transform: &Transform) -> Result<()> {
    let position = Vec3::from_array(transform.position);
    let rotation = Vec3::from_array(transform.rotation);
    
    // Convert Euler angles to direction vector
    let yaw = rotation.y.to_radians();
    let pitch = rotation.x.to_radians();
    
    let direction = Vec3::new(
      yaw.cos() * pitch.cos(),
      pitch.sin(),
      yaw.sin() * pitch.cos(),
    ).normalize();
    
    let up = Vec3::Y;
    let target = position + direction;
    
    self.view_matrix = Mat4::look_at_rh(position, target, up);
    self.dirty_flags.insert(CameraDirtyFlags::VIEW_MATRIX);
    
    Ok(())
  }
  
  /// Update projection matrix based on camera type
  pub fn update_projection_matrix(&mut self) -> Result<()> {
    let aspect_ratio = self.viewport.aspect_ratio();
    
    self.projection_matrix = match &self.camera_type {
      CameraType::Orthographic2D { size, near, far } => {
        let width = size * aspect_ratio;
        let height = *size;
        Mat4::orthographic_rh(-width, width, -height, height, *near, *far)
      }
      CameraType::Perspective3D { fov_degrees, near, far } => {
        Mat4::perspective_rh(fov_degrees.to_radians(), aspect_ratio, *near, *far)
      }
      CameraType::Custom { projection_matrix } => {
        Mat4::from_cols_array_2d(projection_matrix)
      }
    };
    
    self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
    Ok(())
  }
  
  /// Update combined view-projection matrix and frustum
  pub fn update_derived_data(&mut self) -> Result<()> {
    if self.dirty_flags.intersects(CameraDirtyFlags::VIEW_MATRIX | CameraDirtyFlags::PROJECTION_MATRIX) {
      self.view_projection_matrix = self.projection_matrix * self.view_matrix;
      self.dirty_flags.insert(CameraDirtyFlags::FRUSTUM);
    }
    
    if self.dirty_flags.contains(CameraDirtyFlags::FRUSTUM) {
      self.frustum = Frustum::from_matrix(self.view_projection_matrix)?;
    }
    
    self.dirty_flags = CameraDirtyFlags::empty();
    Ok(())
  }
  
  /// Get world-to-screen projection
  pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec3> {
    let clip_pos = self.view_projection_matrix * world_pos.extend(1.0);
    
    if clip_pos.w <= 0.0 {
      return None; // Behind camera
    }
    
    let ndc = clip_pos.xyz() / clip_pos.w;
    
    // Check if point is within NDC bounds
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
      return None;
    }
    
    // Convert to screen coordinates
    let screen_x = (ndc.x + 1.0) * 0.5 * self.viewport.width as f32;
    let screen_y = (1.0 - ndc.y) * 0.5 * self.viewport.height as f32;
    
    Some(Vec3::new(screen_x, screen_y, ndc.z))
  }
  
  /// Get screen-to-world ray
  pub fn screen_to_world_ray(&self, screen_pos: Vec3) -> Option<(Vec3, Vec3)> {
    let x = (screen_pos.x / self.viewport.width as f32) * 2.0 - 1.0;
    let y = 1.0 - (screen_pos.y / self.viewport.height as f32) * 2.0;
    
    let inv_view_proj = self.view_projection_matrix.inverse();
    
    let near_point = inv_view_proj * Vec4::new(x, y, -1.0, 1.0);
    let far_point = inv_view_proj * Vec4::new(x, y, 1.0, 1.0);
    
    if near_point.w == 0.0 || far_point.w == 0.0 {
      return None;
    }
    
    let near_world = near_point.xyz() / near_point.w;
    let far_world = far_point.xyz() / far_point.w;
    
    let origin = near_world;
    let direction = (far_world - near_world).normalize();
    
    Some((origin, direction))
  }
  
  // Getters
  pub fn camera_type(&self) -> &CameraType { &self.camera_type }
  pub fn viewport(&self) -> &Viewport { &self.viewport }
  pub fn view_matrix(&self) -> Mat4 { self.view_matrix }
  pub fn projection_matrix(&self) -> Mat4 { self.projection_matrix }
  pub fn view_projection_matrix(&self) -> Mat4 { self.view_projection_matrix }
  pub fn frustum(&self) -> &Frustum { &self.frustum }
  pub fn clear_color(&self) -> [f32; 4] { self.clear_color }
  pub fn clear_depth(&self) -> f32 { self.clear_depth }
  pub fn render_order(&self) -> i32 { self.render_order }
  pub fn enabled(&self) -> bool { self.enabled }
  
  // Setters
  pub fn set_camera_type(&mut self, camera_type: CameraType) {
    self.camera_type = camera_type;
    self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
  }
  
  pub fn set_viewport(&mut self, viewport: Viewport) {
    self.viewport = viewport;
    self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
  }
  
  pub fn set_clear_color(&mut self, color: [f32; 4]) {
    self.clear_color = color;
  }
  
  pub fn set_render_order(&mut self, order: i32) {
    self.render_order = order;
  }
  
  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }
}

/// Camera component for ECS integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraComponent {
  pub camera: Camera,
  pub is_main: bool,
  pub target_texture: Option<u64>, // Handle to render target texture
}

impl CameraComponent {
  pub fn new(camera: Camera) -> Self {
    Self {
      camera,
      is_main: false,
      target_texture: None,
    }
  }
  
  pub fn main_camera(camera: Camera) -> Self {
    Self {
      camera,
      is_main: true,
      target_texture: None,
    }
  }
  
  pub fn with_render_target(mut self, texture_handle: u64) -> Self {
    self.target_texture = Some(texture_handle);
    self
  }
  
  /// Update camera matrices from transform
  pub fn update(&mut self, transform: &Transform, frame: u64) -> Result<()> {
    if self.camera.last_update_frame != frame {
      self.camera.update_view_matrix(transform)?;
      self.camera.update_derived_data()?;
      self.camera.last_update_frame = frame;
    }
    Ok(())
  }
}

impl Default for CameraComponent {
  fn default() -> Self {
    Self::new(Camera::orthographic_2d(5.0, Viewport::new(800, 600)))
  }
}

// ECS v2 integration
impl engine_core::Component for CameraComponent {}
impl engine_core::ecs_v2::Component for CameraComponent {}

/// Camera uniform data for GPU shaders
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
  pub view_matrix: [[f32; 4]; 4],
  pub projection_matrix: [[f32; 4]; 4],
  pub view_projection_matrix: [[f32; 4]; 4],
  pub camera_position: [f32; 3],
  pub _padding1: f32,
  pub viewport_size: [f32; 2],
  pub near_far: [f32; 2], // [near, far]
  pub clear_color: [f32; 4],
}

impl CameraUniform {
  pub fn from_camera(camera: &Camera, camera_position: Vec3) -> Self {
    let (near, far) = match &camera.camera_type {
      CameraType::Orthographic2D { near, far, .. } => (*near, *far),
      CameraType::Perspective3D { near, far, .. } => (*near, *far),
      CameraType::Custom { .. } => (0.1, 1000.0), // Default values
    };
    
    Self {
      view_matrix: camera.view_matrix.to_cols_array_2d(),
      projection_matrix: camera.projection_matrix.to_cols_array_2d(),
      view_projection_matrix: camera.view_projection_matrix.to_cols_array_2d(),
      camera_position: camera_position.to_array(),
      _padding1: 0.0,
      viewport_size: [camera.viewport.width as f32, camera.viewport.height as f32],
      near_far: [near, far],
      clear_color: camera.clear_color,
    }
  }
}
```

This comprehensive plan provides the foundation for an advanced camera system. The implementation focuses on mobile optimization, ECS v2 integration, and clean separation of concerns between camera logic and rendering systems.

Would you like me to continue with the remaining tasks (viewport management, culling systems, etc.) or would you prefer to start implementing this core camera component first?