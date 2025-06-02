//! Projection matrix management and calculations

use crate::{CameraError, Result};
use glam::Mat4;
use serde::{Serialize, Deserialize};

/// Projection matrix management
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionMatrix {
    matrix: Mat4,
    projection_type: ProjectionType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectionType {
    Orthographic(OrthographicProjection),
    Perspective(PerspectiveProjection),
    Custom([[f32; 4]; 4]),
}

impl ProjectionMatrix {
    /// Create from orthographic projection
    pub fn orthographic(projection: OrthographicProjection) -> Result<Self> {
        let matrix = projection.to_matrix()?;
        Ok(Self {
            matrix,
            projection_type: ProjectionType::Orthographic(projection),
        })
    }
    
    /// Create from perspective projection
    pub fn perspective(projection: PerspectiveProjection) -> Result<Self> {
        let matrix = projection.to_matrix()?;
        Ok(Self {
            matrix,
            projection_type: ProjectionType::Perspective(projection),
        })
    }
    
    /// Create from custom matrix
    pub fn custom(matrix: Mat4) -> Self {
        Self {
            matrix,
            projection_type: ProjectionType::Custom(matrix.to_cols_array_2d()),
        }
    }
    
    /// Get the matrix
    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }
    
    /// Get projection type
    pub fn projection_type(&self) -> &ProjectionType {
        &self.projection_type
    }
    
    /// Update matrix from current projection parameters
    pub fn update_matrix(&mut self) -> Result<()> {
        self.matrix = match &self.projection_type {
            ProjectionType::Orthographic(proj) => proj.to_matrix()?,
            ProjectionType::Perspective(proj) => proj.to_matrix()?,
            ProjectionType::Custom(matrix) => Mat4::from_cols_array_2d(matrix),
        };
        Ok(())
    }
}

/// Orthographic projection parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicProjection {
    /// Create orthographic projection
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Result<Self> {
        let proj = Self { left, right, bottom, top, near, far };
        proj.validate()?;
        Ok(proj)
    }
    
    /// Create centered orthographic projection
    pub fn centered(width: f32, height: f32, near: f32, far: f32) -> Result<Self> {
        let half_width = width * 0.5;
        let half_height = height * 0.5;
        Self::new(-half_width, half_width, -half_height, half_height, near, far)
    }
    
    /// Create from size and aspect ratio (for 2D cameras)
    pub fn from_size_aspect(size: f32, aspect_ratio: f32, near: f32, far: f32) -> Result<Self> {
        let width = size * aspect_ratio;
        let height = size;
        Self::centered(width, height, near, far)
    }
    
    /// Convert to projection matrix
    pub fn to_matrix(&self) -> Result<Mat4> {
        if self.right <= self.left || self.top <= self.bottom || self.far <= self.near {
            return Err(CameraError::InvalidProjection(
                "Invalid orthographic projection parameters".to_string()
            ));
        }
        
        Ok(Mat4::orthographic_rh(
            self.left, self.right, 
            self.bottom, self.top, 
            self.near, self.far
        ))
    }
    
    /// Validate projection parameters
    pub fn validate(&self) -> Result<()> {
        if self.right <= self.left {
            return Err(CameraError::InvalidProjection(
                format!("Right ({}) must be greater than left ({})", self.right, self.left)
            ));
        }
        if self.top <= self.bottom {
            return Err(CameraError::InvalidProjection(
                format!("Top ({}) must be greater than bottom ({})", self.top, self.bottom)
            ));
        }
        if self.far <= self.near {
            return Err(CameraError::InvalidProjection(
                format!("Far ({}) must be greater than near ({})", self.far, self.near)
            ));
        }
        Ok(())
    }
    
    /// Get projection bounds
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.left, self.right, self.bottom, self.top)
    }
    
    /// Get clipping planes
    pub fn clipping_planes(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self::centered(10.0, 10.0, -10.0, 10.0).unwrap()
    }
}

/// Perspective projection parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerspectiveProjection {
    pub fov_y_radians: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl PerspectiveProjection {
    /// Create perspective projection
    pub fn new(fov_y_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Result<Self> {
        let proj = Self {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            near,
            far,
        };
        proj.validate()?;
        Ok(proj)
    }
    
    /// Convert to projection matrix
    pub fn to_matrix(&self) -> Result<Mat4> {
        if self.near <= 0.0 || self.far <= self.near || self.aspect_ratio <= 0.0 {
            return Err(CameraError::InvalidProjection(
                "Invalid perspective projection parameters".to_string()
            ));
        }
        
        Ok(Mat4::perspective_rh(
            self.fov_y_radians,
            self.aspect_ratio,
            self.near,
            self.far
        ))
    }
    
    /// Validate projection parameters
    pub fn validate(&self) -> Result<()> {
        if self.fov_y_radians <= 0.0 || self.fov_y_radians >= std::f32::consts::PI {
            return Err(CameraError::InvalidProjection(
                format!("FOV ({} radians) must be between 0 and π", self.fov_y_radians)
            ));
        }
        if self.aspect_ratio <= 0.0 {
            return Err(CameraError::InvalidProjection(
                format!("Aspect ratio ({}) must be positive", self.aspect_ratio)
            ));
        }
        if self.near <= 0.0 {
            return Err(CameraError::InvalidProjection(
                format!("Near plane ({}) must be positive", self.near)
            ));
        }
        if self.far <= self.near {
            return Err(CameraError::InvalidProjection(
                format!("Far ({}) must be greater than near ({})", self.far, self.near)
            ));
        }
        Ok(())
    }
    
    /// Get FOV in degrees
    pub fn fov_y_degrees(&self) -> f32 {
        self.fov_y_radians.to_degrees()
    }
    
    /// Set FOV in degrees
    pub fn set_fov_y_degrees(&mut self, degrees: f32) {
        self.fov_y_radians = degrees.to_radians();
    }
    
    /// Get clipping planes
    pub fn clipping_planes(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self::new(60.0, 16.0/9.0, 0.1, 1000.0).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_orthographic_projection() {
        let proj = OrthographicProjection::centered(10.0, 8.0, -5.0, 5.0).unwrap();
        assert_eq!(proj.left, -5.0);
        assert_eq!(proj.right, 5.0);
        assert_eq!(proj.bottom, -4.0);
        assert_eq!(proj.top, 4.0);
        
        let matrix = proj.to_matrix().unwrap();
        assert!(!matrix.is_nan());
    }
    
    #[test]
    fn test_orthographic_validation() {
        // Valid projection
        let valid = OrthographicProjection::new(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
        assert!(valid.is_ok());
        
        // Invalid: right <= left
        let invalid = OrthographicProjection::new(1.0, -1.0, -1.0, 1.0, 0.1, 100.0);
        assert!(invalid.is_err());
        
        // Invalid: top <= bottom
        let invalid = OrthographicProjection::new(-1.0, 1.0, 1.0, -1.0, 0.1, 100.0);
        assert!(invalid.is_err());
        
        // Invalid: far <= near
        let invalid = OrthographicProjection::new(-1.0, 1.0, -1.0, 1.0, 100.0, 0.1);
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_perspective_projection() {
        let proj = PerspectiveProjection::new(60.0, 16.0/9.0, 0.1, 1000.0).unwrap();
        assert!((proj.fov_y_degrees() - 60.0).abs() < 0.001);
        assert!((proj.aspect_ratio - 16.0/9.0).abs() < f32::EPSILON);
        
        let matrix = proj.to_matrix().unwrap();
        assert!(!matrix.is_nan());
    }
    
    #[test]
    fn test_perspective_validation() {
        // Valid projection
        let valid = PerspectiveProjection::new(60.0, 16.0/9.0, 0.1, 1000.0);
        assert!(valid.is_ok());
        
        // Invalid: negative FOV
        let invalid = PerspectiveProjection::new(-60.0, 16.0/9.0, 0.1, 1000.0);
        assert!(invalid.is_err());
        
        // Invalid: FOV >= 180 degrees
        let invalid = PerspectiveProjection::new(180.0, 16.0/9.0, 0.1, 1000.0);
        assert!(invalid.is_err());
        
        // Invalid: negative aspect ratio
        let invalid = PerspectiveProjection::new(60.0, -1.0, 0.1, 1000.0);
        assert!(invalid.is_err());
        
        // Invalid: negative near
        let invalid = PerspectiveProjection::new(60.0, 16.0/9.0, -0.1, 1000.0);
        assert!(invalid.is_err());
        
        // Invalid: far <= near
        let invalid = PerspectiveProjection::new(60.0, 16.0/9.0, 100.0, 0.1);
        assert!(invalid.is_err());
    }
}