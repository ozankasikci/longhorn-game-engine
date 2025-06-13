//! Material system for rendering

use glam::Vec3;

/// Material properties for rendering
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub albedo: Vec3,
    pub metallic: f32,
    pub roughness: f32,
}

impl Material {
    /// Create a new material
    pub fn new(name: String) -> Self {
        Self {
            name,
            albedo: Vec3::new(0.8, 0.8, 0.8), // Light gray
            metallic: 0.0,
            roughness: 0.5,
        }
    }
    
    /// Create a material with specific albedo color
    pub fn with_color(name: String, color: Vec3) -> Self {
        Self {
            name,
            albedo: color,
            metallic: 0.0,
            roughness: 0.5,
        }
    }
    
    /// Default red material
    pub fn red() -> Self {
        Self::with_color("Red".to_string(), Vec3::new(1.0, 0.0, 0.0))
    }
    
    /// Default green material
    pub fn green() -> Self {
        Self::with_color("Green".to_string(), Vec3::new(0.0, 1.0, 0.0))
    }
    
    /// Default blue material
    pub fn blue() -> Self {
        Self::with_color("Blue".to_string(), Vec3::new(0.0, 0.0, 1.0))
    }
}