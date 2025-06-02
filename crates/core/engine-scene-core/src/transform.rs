//! Transform component for position, rotation, and scale

use glam::{Vec3, Quat, Mat4};
use serde::{Serialize, Deserialize};

/// Transform component storing position, rotation, and scale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

/// Precomputed transform matrix for efficient rendering
#[derive(Debug, Clone)]
pub struct TransformMatrix {
    pub local: Mat4,
    pub world: Mat4,
    pub dirty: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Create a new transform at origin
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a transform with position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    
    /// Create a transform with rotation
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }
    
    /// Create a transform with scale
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }
    
    /// Create a transform with uniform scale
    pub fn from_uniform_scale(scale: f32) -> Self {
        Self {
            scale: Vec3::splat(scale),
            ..Default::default()
        }
    }
    
    /// Set position
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }
    
    /// Set rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
    
    /// Set scale
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
    
    /// Translate by offset
    pub fn translate(&mut self, offset: Vec3) {
        self.position += offset;
    }
    
    /// Rotate by quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }
    
    /// Rotate around axis by angle (radians)
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis, angle);
        self.rotate(rotation);
    }
    
    /// Rotate around X axis
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate_axis(Vec3::X, angle);
    }
    
    /// Rotate around Y axis
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Y, angle);
    }
    
    /// Rotate around Z axis
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Z, angle);
    }
    
    /// Scale uniformly
    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale *= factor;
    }
    
    /// Scale non-uniformly
    pub fn scale_by(&mut self, scale: Vec3) {
        self.scale *= scale;
    }
    
    /// Look at target position (sets rotation)
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        self.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        
        // Adjust for up vector
        let right = forward.cross(up).normalize();
        let actual_up = right.cross(forward);
        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            actual_up.extend(0.0),
            (-forward).extend(0.0),
            Vec3::ZERO.extend(1.0),
        );
        self.rotation = Quat::from_mat4(&rotation_matrix);
    }
    
    /// Get forward direction (negative Z in local space)
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }
    
    /// Get right direction (positive X in local space)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    
    /// Get up direction (positive Y in local space)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    
    /// Calculate local transformation matrix
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
    
    /// Calculate inverse transformation matrix
    pub fn inverse_matrix(&self) -> Mat4 {
        self.matrix().inverse()
    }
    
    /// Transform a point from local to world space
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix().transform_point3(point)
    }
    
    /// Transform a vector from local to world space
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.matrix().transform_vector3(vector)
    }
    
    /// Combine with another transform (this transform applied first)
    pub fn combine(&self, other: &Transform) -> Transform {
        let combined_matrix = other.matrix() * self.matrix();
        Transform::from_matrix(combined_matrix)
    }
    
    /// Create transform from matrix (decompose)
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, position) = matrix.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }
    
    /// Linearly interpolate between transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            local: Mat4::IDENTITY,
            world: Mat4::IDENTITY,
            dirty: true,
        }
    }
}

impl TransformMatrix {
    /// Create new transform matrix
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update local matrix from transform
    pub fn update_local(&mut self, transform: &Transform) {
        self.local = transform.matrix();
        self.dirty = true;
    }
    
    /// Update world matrix from parent world matrix
    pub fn update_world(&mut self, parent_world: Option<&Mat4>) {
        if self.dirty {
            self.world = if let Some(parent) = parent_world {
                *parent * self.local
            } else {
                self.local
            };
            self.dirty = false;
        }
    }
    
    /// Mark as dirty (needs recalculation)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Check if needs update
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    /// Get local matrix
    pub fn local(&self) -> &Mat4 {
        &self.local
    }
    
    /// Get world matrix
    pub fn world(&self) -> &Mat4 {
        &self.world
    }
    
    /// Get normal matrix (inverse transpose of world matrix)
    pub fn normal_matrix(&self) -> Mat4 {
        self.world.inverse().transpose()
    }
}