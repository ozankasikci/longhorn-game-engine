//! Camera system for 3D rendering

use glam::{Mat4, Vec3};
use serde::{Serialize, Deserialize};

/// 3D camera for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub is_main: bool,
}

impl Camera {
    /// Create a new camera with default values
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(2.0, 2.0, 3.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 60.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100.0,
            is_main: false,
        }
    }
    
    /// Create a camera from position and rotation (euler angles)
    pub fn from_position_rotation(position: [f32; 3], _rotation: [f32; 3], aspect: f32) -> Self {
        let pos = Vec3::from(position);
        
        // For now, always look at origin where the cube is
        let target = Vec3::ZERO;
        
        Self {
            position: pos,
            target,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100.0,
            is_main: false,
        }
    }
    
    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
    
    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
    
    /// Get the combined view-projection matrix
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    /// Update the aspect ratio (call when window resizes)
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
    
    /// Set the field of view
    pub fn with_fov(mut self, fov_degrees: f32) -> Self {
        self.fov = fov_degrees.to_radians();
        self
    }
    
    /// Create a main camera
    pub fn main_camera() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
            is_main: true,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(16.0 / 9.0) // Default aspect ratio
    }
}

// Component trait implementation - this allows Camera to be used in ECS
impl engine_component_traits::Component for Camera {}