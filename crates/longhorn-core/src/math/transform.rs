use glam::{Mat4, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 2D transform component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32, // radians
    pub scale: Vec2,
}

impl Transform {
    /// Create a new transform at the origin
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Create a transform at the given position
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Create a transform with position and rotation
    pub fn from_position_rotation(position: Vec2, rotation: f32) -> Self {
        Self {
            position,
            rotation,
            scale: Vec2::ONE,
        }
    }

    /// Create a transform with all components
    pub fn from_components(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Get the forward direction (right in 2D)
    pub fn forward(&self) -> Vec2 {
        Vec2::new(self.rotation.cos(), self.rotation.sin())
    }

    /// Get the right direction (up in 2D)
    pub fn right(&self) -> Vec2 {
        Vec2::new(-self.rotation.sin(), self.rotation.cos())
    }

    /// Convert to a 4x4 transformation matrix
    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let rotation = Mat4::from_quat(Quat::from_rotation_z(self.rotation));
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0));

        translation * rotation * scale
    }

    /// Transform a point by this transform
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        let scaled = point * self.scale;
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );
        rotated + self.position
    }

    /// Inverse transform a point
    pub fn inverse_transform_point(&self, point: Vec2) -> Vec2 {
        let translated = point - self.position;
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotated = Vec2::new(
            translated.x * cos + translated.y * sin,
            -translated.x * sin + translated.y * cos,
        );
        rotated / self.scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_transform_new() {
        let t = Transform::new();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_from_position() {
        let t = Transform::from_position(Vec2::new(10.0, 20.0));
        assert_eq!(t.position, Vec2::new(10.0, 20.0));
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_forward_right() {
        let t = Transform::from_position_rotation(Vec2::ZERO, 0.0);
        let forward = t.forward();
        assert!((forward.x - 1.0).abs() < 0.0001);
        assert!(forward.y.abs() < 0.0001);

        let t = Transform::from_position_rotation(Vec2::ZERO, PI / 2.0);
        let forward = t.forward();
        assert!(forward.x.abs() < 0.0001);
        assert!((forward.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_point() {
        let t = Transform::from_position(Vec2::new(10.0, 20.0));
        let p = Vec2::new(5.0, 5.0);
        let transformed = t.transform_point(p);
        assert_eq!(transformed, Vec2::new(15.0, 25.0));
    }

    #[test]
    fn test_transform_point_with_rotation() {
        let t = Transform::from_position_rotation(Vec2::ZERO, PI / 2.0);
        let p = Vec2::new(1.0, 0.0);
        let transformed = t.transform_point(p);
        assert!(transformed.x.abs() < 0.0001);
        assert!((transformed.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_point_with_scale() {
        let t = Transform::from_components(Vec2::ZERO, 0.0, Vec2::new(2.0, 2.0));
        let p = Vec2::new(1.0, 1.0);
        let transformed = t.transform_point(p);
        assert_eq!(transformed, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_inverse_transform() {
        let t = Transform::from_components(
            Vec2::new(10.0, 20.0),
            PI / 4.0,
            Vec2::new(2.0, 2.0),
        );
        let p = Vec2::new(5.0, 5.0);
        let transformed = t.transform_point(p);
        let inverse = t.inverse_transform_point(transformed);

        assert!((inverse.x - p.x).abs() < 0.0001);
        assert!((inverse.y - p.y).abs() < 0.0001);
    }
}
