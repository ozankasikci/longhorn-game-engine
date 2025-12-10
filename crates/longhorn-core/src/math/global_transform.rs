use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::Transform;

/// World-space transform component (calculated, not directly editable)
///
/// This component stores the final world-space position, rotation, and scale
/// of an entity after applying all parent transforms in the hierarchy.
///
/// You should NOT directly mutate GlobalTransform. Instead, modify the entity's
/// Transform component, and GlobalTransform will be automatically updated by
/// the transform propagation system.
///
/// # Transform Hierarchy
///
/// - `Transform`: Local transform (relative to parent, or world if no parent)
/// - `GlobalTransform`: World-space transform (calculated from hierarchy)
///
/// For entities without a Parent component, GlobalTransform equals Transform.
/// For child entities, GlobalTransform is calculated by combining the parent's
/// GlobalTransform with the child's local Transform.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlobalTransform {
    pub position: Vec2,
    pub rotation: f32, // radians
    pub scale: Vec2,
}

impl GlobalTransform {
    /// Create a new global transform at the origin
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Create from a local Transform (for root entities)
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            position: transform.position,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    /// Combine this global transform with a local transform to produce a new global transform
    ///
    /// This is used to calculate a child's GlobalTransform from:
    /// - parent's GlobalTransform (self)
    /// - child's local Transform
    pub fn mul_transform(&self, local: &Transform) -> Self {
        // Transform the child's local position by the parent's transform
        let world_position = self.transform_point(local.position);

        // Combine rotations (additive in 2D)
        let world_rotation = self.rotation + local.rotation;

        // Combine scales (multiplicative)
        let world_scale = self.scale * local.scale;

        Self {
            position: world_position,
            rotation: world_rotation,
            scale: world_scale,
        }
    }

    /// Transform a point by this global transform
    fn transform_point(&self, point: Vec2) -> Vec2 {
        // Apply scale
        let scaled = point * self.scale;

        // Apply rotation
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );

        // Apply translation
        rotated + self.position
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Transform> for GlobalTransform {
    fn from(transform: Transform) -> Self {
        Self::from_transform(&transform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_transform() {
        let transform = Transform::from_position(Vec2::new(10.0, 20.0));
        let global = GlobalTransform::from_transform(&transform);

        assert_eq!(global.position, Vec2::new(10.0, 20.0));
        assert_eq!(global.rotation, 0.0);
        assert_eq!(global.scale, Vec2::ONE);
    }

    #[test]
    fn test_mul_transform_position_only() {
        // Parent at (10, 10)
        let parent = GlobalTransform {
            position: Vec2::new(10.0, 10.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        };

        // Child at local (5, 5)
        let child_local = Transform::from_position(Vec2::new(5.0, 5.0));

        // Child's world position should be (15, 15)
        let child_global = parent.mul_transform(&child_local);
        assert_eq!(child_global.position, Vec2::new(15.0, 15.0));
    }

    #[test]
    fn test_mul_transform_with_rotation() {
        use std::f32::consts::PI;

        // Parent rotated 90 degrees
        let parent = GlobalTransform {
            position: Vec2::ZERO,
            rotation: PI / 2.0,
            scale: Vec2::ONE,
        };

        // Child at local (1, 0)
        let child_local = Transform::from_position(Vec2::new(1.0, 0.0));

        // After 90 degree rotation, (1, 0) becomes (0, 1)
        let child_global = parent.mul_transform(&child_local);
        assert!((child_global.position.x - 0.0).abs() < 0.0001);
        assert!((child_global.position.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_mul_transform_with_scale() {
        // Parent scaled 2x
        let parent = GlobalTransform {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(2.0, 2.0),
        };

        // Child at local (5, 5) with 3x scale
        let child_local = Transform::from_components(
            Vec2::new(5.0, 5.0),
            0.0,
            Vec2::new(3.0, 3.0),
        );

        let child_global = parent.mul_transform(&child_local);

        // Position: (5, 5) * 2 = (10, 10)
        assert_eq!(child_global.position, Vec2::new(10.0, 10.0));

        // Scale: 2 * 3 = 6
        assert_eq!(child_global.scale, Vec2::new(6.0, 6.0));
    }

    #[test]
    fn test_hierarchy_chain() {
        // Grandparent at (100, 100)
        let grandparent = GlobalTransform {
            position: Vec2::new(100.0, 100.0),
            rotation: 0.0,
            scale: Vec2::ONE,
        };

        // Parent at local (10, 10) relative to grandparent
        let parent_local = Transform::from_position(Vec2::new(10.0, 10.0));
        let parent_global = grandparent.mul_transform(&parent_local);

        // Child at local (5, 5) relative to parent
        let child_local = Transform::from_position(Vec2::new(5.0, 5.0));
        let child_global = parent_global.mul_transform(&child_local);

        // Child's world position should be 100 + 10 + 5 = 115
        assert_eq!(child_global.position, Vec2::new(115.0, 115.0));
    }
}
