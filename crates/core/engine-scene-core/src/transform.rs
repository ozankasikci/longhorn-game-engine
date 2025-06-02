//! Transform matrix utilities for efficient rendering

use glam::{Vec3, Quat, Mat4};

/// Precomputed transform matrix for efficient rendering
#[derive(Debug, Clone)]
pub struct TransformMatrix {
    pub local: Mat4,
    pub world: Mat4,
    pub dirty: bool,
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