//! Camera system for scene viewing

use glam::{Vec3, Mat4, Vec4Swizzles};
use serde::{Serialize, Deserialize};

/// Camera component for scene viewing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub projection: CameraProjection,
    pub view: CameraView,
    pub active: bool,
    pub clear_color: Option<[f32; 4]>,
    pub clear_depth: f32,
    pub render_layers: u32,
}

/// Camera projection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraProjection {
    Perspective {
        fov_y: f32,      // Field of view in radians
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

/// Camera view settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraView {
    pub target: Option<Vec3>,  // Look-at target (None = use transform rotation)
    pub up: Vec3,              // Up vector
    pub viewport: Viewport,
}

/// Viewport settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: CameraProjection::perspective(
                45.0_f32.to_radians(),
                16.0 / 9.0,
                0.1,
                1000.0
            ),
            view: CameraView::default(),
            active: true,
            clear_color: Some([0.2, 0.3, 0.4, 1.0]),
            clear_depth: 1.0,
            render_layers: 0xFFFFFFFF, // All layers
        }
    }
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            target: None,
            up: Vec3::Y,
            viewport: Viewport::default(),
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

impl Camera {
    /// Create a perspective camera
    pub fn perspective(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            projection: CameraProjection::perspective(fov_y, aspect_ratio, near, far),
            ..Default::default()
        }
    }
    
    /// Create an orthographic camera
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            projection: CameraProjection::orthographic(left, right, bottom, top, near, far),
            ..Default::default()
        }
    }
    
    /// Set look-at target
    pub fn look_at(mut self, target: Vec3) -> Self {
        self.view.target = Some(target);
        self
    }
    
    /// Set up vector
    pub fn with_up(mut self, up: Vec3) -> Self {
        self.view.up = up;
        self
    }
    
    /// Set viewport
    pub fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.view.viewport = viewport;
        self
    }
    
    /// Set clear color
    pub fn with_clear_color(mut self, color: [f32; 4]) -> Self {
        self.clear_color = Some(color);
        self
    }
    
    /// Set render layers
    pub fn with_layers(mut self, layers: u32) -> Self {
        self.render_layers = layers;
        self
    }
    
    /// Update aspect ratio for perspective projection
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        if let CameraProjection::Perspective { aspect_ratio: ref mut ar, .. } = self.projection {
            *ar = aspect_ratio;
        }
    }
    
    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.matrix()
    }
    
    /// Get view matrix from camera position and view settings
    pub fn view_matrix(&self, camera_position: Vec3) -> Mat4 {
        if let Some(target) = self.view.target {
            Mat4::look_at_lh(camera_position, target, self.view.up)
        } else {
            // Use identity if no target (transform rotation will be applied)
            Mat4::IDENTITY
        }
    }
    
    /// Get view-projection matrix
    pub fn view_projection_matrix(&self, camera_position: Vec3) -> Mat4 {
        self.projection_matrix() * self.view_matrix(camera_position)
    }
    
    /// Convert screen coordinates to world ray
    pub fn screen_to_ray(&self, screen_pos: Vec3, camera_position: Vec3, screen_size: (f32, f32)) -> Ray {
        let (screen_width, screen_height) = screen_size;
        
        // Normalize screen coordinates to [-1, 1]
        let ndc_x = (2.0 * screen_pos.x) / screen_width - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_height;
        
        let view_proj = self.view_projection_matrix(camera_position);
        let inv_view_proj = view_proj.inverse();
        
        // Near and far points in homogeneous coordinates
        let near_point = inv_view_proj * Vec3::new(ndc_x, ndc_y, -1.0).extend(1.0);
        let far_point = inv_view_proj * Vec3::new(ndc_x, ndc_y, 1.0).extend(1.0);
        
        // Convert to 3D coordinates
        let near_point = near_point.xyz() / near_point.w;
        let far_point = far_point.xyz() / far_point.w;
        
        Ray {
            origin: near_point,
            direction: (far_point - near_point).normalize(),
        }
    }
}

impl CameraProjection {
    /// Create perspective projection
    pub fn perspective(fov_y: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }
    
    /// Create orthographic projection
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }
    
    /// Get projection matrix
    pub fn matrix(&self) -> Mat4 {
        match self {
            Self::Perspective { fov_y, aspect_ratio, near, far } => {
                Mat4::perspective_lh(*fov_y, *aspect_ratio, *near, *far)
            }
            Self::Orthographic { left, right, bottom, top, near, far } => {
                Mat4::orthographic_lh(*left, *right, *bottom, *top, *near, *far)
            }
        }
    }
    
    /// Get near plane distance
    pub fn near(&self) -> f32 {
        match self {
            Self::Perspective { near, .. } => *near,
            Self::Orthographic { near, .. } => *near,
        }
    }
    
    /// Get far plane distance
    pub fn far(&self) -> f32 {
        match self {
            Self::Perspective { far, .. } => *far,
            Self::Orthographic { far, .. } => *far,
        }
    }
    
    /// Check if this is a perspective projection
    pub fn is_perspective(&self) -> bool {
        matches!(self, Self::Perspective { .. })
    }
    
    /// Check if this is an orthographic projection
    pub fn is_orthographic(&self) -> bool {
        matches!(self, Self::Orthographic { .. })
    }
}

impl Viewport {
    /// Create a new viewport
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
    
    /// Create a full-screen viewport
    pub fn fullscreen() -> Self {
        Self::default()
    }
    
    /// Set depth range
    pub fn with_depth_range(mut self, min_depth: f32, max_depth: f32) -> Self {
        self.min_depth = min_depth;
        self.max_depth = max_depth;
        self
    }
    
    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        if self.height != 0.0 {
            self.width / self.height
        } else {
            1.0
        }
    }
    
    /// Check if point is inside viewport
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
}

/// Ray for camera picking and intersection tests
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    /// Get point along ray at distance t
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}