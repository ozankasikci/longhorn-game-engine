//! ECS Camera Components - Unified camera system for all camera types

use crate::{CameraType, Viewport};
use engine_ecs_core::{Component, ComponentV2};
use serde::{Serialize, Deserialize};

/// Basic 3D Camera Component - simple ECS camera for perspective rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub fov: f32,      // Field of view in degrees
    pub near: f32,     // Near clipping plane
    pub far: f32,      // Far clipping plane
    pub is_main: bool, // Is this the main camera?
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            is_main: false,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn main_camera() -> Self {
        Self {
            is_main: true,
            ..Default::default()
        }
    }
    
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }
    
    pub fn with_near_far(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }
}

// ECS component implementations
impl Component for Camera {}
impl ComponentV2 for Camera {}

/// 2D Camera Component - specialized for 2D rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    pub size: f32,                      // Orthographic size (world units from center to top)
    pub aspect_ratio: f32,              // Width/height ratio (auto-calculated if 0.0)
    pub near: f32,                      // Near clipping plane
    pub far: f32,                       // Far clipping plane
    pub background_color: [f32; 4],     // Clear color RGBA
    pub viewport_rect: [f32; 4],        // [x, y, width, height] normalized (0.0-1.0)
    pub is_main: bool,                  // Whether this is the main 2D camera
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            size: 5.0,                  // 5 world units from center to top
            aspect_ratio: 0.0,          // Auto-calculate from screen
            near: -10.0,                // Behind the camera
            far: 10.0,                  // In front of the camera
            background_color: [0.2, 0.2, 0.3, 1.0], // Dark blue-gray
            viewport_rect: [0.0, 0.0, 1.0, 1.0],    // Full screen
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
    
    pub fn with_viewport(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.viewport_rect = [x, y, width, height];
        self
    }
}

// ECS component implementations
impl Component for Camera2D {}
impl ComponentV2 for Camera2D {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert_eq!(camera.fov, 60.0);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
        assert!(!camera.is_main);
    }

    #[test]
    fn test_camera_main() {
        let camera = Camera::main_camera();
        assert!(camera.is_main);
        assert_eq!(camera.fov, 60.0); // Should still have default values
    }

    #[test]
    fn test_camera_builder() {
        let camera = Camera::new()
            .with_fov(90.0)
            .with_near_far(0.5, 500.0);
        
        assert_eq!(camera.fov, 90.0);
        assert_eq!(camera.near, 0.5);
        assert_eq!(camera.far, 500.0);
    }

    #[test]
    fn test_camera_2d_default() {
        let camera = Camera2D::default();
        assert_eq!(camera.size, 5.0);
        assert_eq!(camera.aspect_ratio, 0.0);
        assert_eq!(camera.near, -10.0);
        assert_eq!(camera.far, 10.0);
        assert_eq!(camera.background_color, [0.2, 0.2, 0.3, 1.0]);
        assert_eq!(camera.viewport_rect, [0.0, 0.0, 1.0, 1.0]);
        assert!(!camera.is_main);
    }

    #[test]
    fn test_camera_2d_main() {
        let camera = Camera2D::main_camera();
        assert!(camera.is_main);
        assert_eq!(camera.size, 5.0); // Should still have default size
    }

    #[test]
    fn test_camera_2d_builder() {
        let camera = Camera2D::new()
            .with_size(10.0)
            .with_background_color(1.0, 0.0, 0.0, 1.0)
            .with_viewport(0.1, 0.1, 0.8, 0.8);
        
        assert_eq!(camera.size, 10.0);
        assert_eq!(camera.background_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(camera.viewport_rect, [0.1, 0.1, 0.8, 0.8]);
    }
}