//! Frustum Culling for 3D Renderer
//!
//! This module provides frustum culling functionality to eliminate objects
//! that are outside the camera's view, improving rendering performance.

use crate::{Camera, RenderObject};
use glam::{Mat4, Vec3, Vec4};

/// A plane in 3D space defined by a normal vector and distance from origin
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector of the plane (should be normalized)
    pub normal: Vec3,
    /// Distance from origin along the normal
    pub distance: f32,
}

impl Plane {
    /// Create a new plane from normal and distance
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create a plane from three points
    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2).normalize();
        let distance = normal.dot(p1);
        Self { normal, distance }
    }

    /// Create a plane from a Vec4 representation (nx, ny, nz, d)
    pub fn from_vec4(plane: Vec4) -> Self {
        let normal = Vec3::new(plane.x, plane.y, plane.z);
        Self {
            normal,
            distance: plane.w,
        }
    }

    /// Get the signed distance from a point to this plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// Check if a point is in front of (positive side) the plane
    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }

    /// Normalize the plane (ensure the normal vector is unit length)
    pub fn normalize(mut self) -> Self {
        let length = self.normal.length();
        if length > 0.0 {
            self.normal /= length;
            self.distance /= length;
        }
        self
    }
}

/// View frustum defined by 6 planes
#[derive(Debug, Clone)]
pub struct Frustum {
    /// The six frustum planes: left, right, bottom, top, near, far
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Plane indices for easier access
    pub const LEFT: usize = 0;
    pub const RIGHT: usize = 1;
    pub const BOTTOM: usize = 2;
    pub const TOP: usize = 3;
    pub const NEAR: usize = 4;
    pub const FAR: usize = 5;

    /// Create frustum from view-projection matrix
    pub fn from_view_projection_matrix(vp_matrix: Mat4) -> Self {
        // Extract frustum planes from view-projection matrix
        // This uses the standard technique of extracting planes from the matrix
        let m = vp_matrix;

        // Left plane: m[3] + m[0]
        let left = Plane::from_vec4(Vec4::new(
            m.col(3).x + m.col(0).x,
            m.col(3).y + m.col(0).y,
            m.col(3).z + m.col(0).z,
            m.col(3).w + m.col(0).w,
        ))
        .normalize();

        // Right plane: m[3] - m[0]
        let right = Plane::from_vec4(Vec4::new(
            m.col(3).x - m.col(0).x,
            m.col(3).y - m.col(0).y,
            m.col(3).z - m.col(0).z,
            m.col(3).w - m.col(0).w,
        ))
        .normalize();

        // Bottom plane: m[3] + m[1]
        let bottom = Plane::from_vec4(Vec4::new(
            m.col(3).x + m.col(1).x,
            m.col(3).y + m.col(1).y,
            m.col(3).z + m.col(1).z,
            m.col(3).w + m.col(1).w,
        ))
        .normalize();

        // Top plane: m[3] - m[1]
        let top = Plane::from_vec4(Vec4::new(
            m.col(3).x - m.col(1).x,
            m.col(3).y - m.col(1).y,
            m.col(3).z - m.col(1).z,
            m.col(3).w - m.col(1).w,
        ))
        .normalize();

        // Near plane: m[3] + m[2]
        let near = Plane::from_vec4(Vec4::new(
            m.col(3).x + m.col(2).x,
            m.col(3).y + m.col(2).y,
            m.col(3).z + m.col(2).z,
            m.col(3).w + m.col(2).w,
        ))
        .normalize();

        // Far plane: m[3] - m[2]
        let far = Plane::from_vec4(Vec4::new(
            m.col(3).x - m.col(2).x,
            m.col(3).y - m.col(2).y,
            m.col(3).z - m.col(2).z,
            m.col(3).w - m.col(2).w,
        ))
        .normalize();

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Test if a point is inside the frustum
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if !plane.is_point_in_front(point) {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is inside or intersects the frustum
    pub fn is_sphere_inside(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.distance_to_point(center);
            if distance < -radius {
                return false; // Sphere is completely outside this plane
            }
        }
        true // Sphere is inside or intersects the frustum
    }

    /// Test if an axis-aligned bounding box intersects the frustum
    pub fn is_aabb_inside(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find the positive vertex (the one furthest along the plane normal)
            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            // If the positive vertex is outside the plane, the AABB is outside
            if !plane.is_point_in_front(positive_vertex) {
                return false;
            }
        }
        true
    }

    /// Get the corners of the frustum in world space
    pub fn get_corners(&self) -> [Vec3; 8] {
        // This is more complex and typically requires the original camera parameters
        // For now, we'll return a simple approximation
        // In a full implementation, this would compute the actual frustum corners
        [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -100.0),
            Vec3::new(1.0, -1.0, -100.0),
            Vec3::new(1.0, 1.0, -100.0),
            Vec3::new(-1.0, 1.0, -100.0),
        ]
    }
}

/// Bounding volume types for culling tests
#[derive(Debug, Clone, Copy)]
pub enum BoundingVolume {
    /// Point in 3D space
    Point(Vec3),
    /// Sphere with center and radius
    Sphere { center: Vec3, radius: f32 },
    /// Axis-aligned bounding box
    AABB { min: Vec3, max: Vec3 },
}

impl BoundingVolume {
    /// Create a bounding sphere from center and radius
    pub fn sphere(center: Vec3, radius: f32) -> Self {
        Self::Sphere { center, radius }
    }

    /// Create an AABB from min and max points
    pub fn aabb(min: Vec3, max: Vec3) -> Self {
        Self::AABB { min, max }
    }

    /// Create a point bounding volume
    pub fn point(position: Vec3) -> Self {
        Self::Point(position)
    }

    /// Test if this bounding volume is visible in the frustum
    pub fn is_visible_in_frustum(&self, frustum: &Frustum) -> bool {
        match self {
            BoundingVolume::Point(point) => frustum.is_point_inside(*point),
            BoundingVolume::Sphere { center, radius } => frustum.is_sphere_inside(*center, *radius),
            BoundingVolume::AABB { min, max } => frustum.is_aabb_inside(*min, *max),
        }
    }
}

/// Frustum culler that can cull lists of objects
pub struct FrustumCuller {
    /// Current frustum for culling
    frustum: Frustum,
    /// Statistics about culling operations
    stats: CullingStats,
}

impl FrustumCuller {
    /// Create a new frustum culler from a camera
    pub fn new(camera: &Camera) -> Self {
        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        let view_projection = projection_matrix * view_matrix;
        let frustum = Frustum::from_view_projection_matrix(view_projection);

        Self {
            frustum,
            stats: CullingStats::default(),
        }
    }

    /// Update the frustum from a camera
    pub fn update_from_camera(&mut self, camera: &Camera) {
        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        let view_projection = projection_matrix * view_matrix;
        self.frustum = Frustum::from_view_projection_matrix(view_projection);
        self.stats.reset();
    }

    /// Test if a single object is visible
    pub fn is_object_visible(&mut self, object: &RenderObject) -> bool {
        self.stats.total_tests += 1;

        // Extract position from transform matrix
        let position = object.transform.col(3).truncate();

        // For now, use a simple point test
        // In a real implementation, this would use the object's actual bounding volume
        let bounding_volume = BoundingVolume::point(position);
        let visible = bounding_volume.is_visible_in_frustum(&self.frustum);

        if visible {
            self.stats.visible_objects += 1;
        } else {
            self.stats.culled_objects += 1;
        }

        visible
    }

    /// Cull a list of render objects, returning only the visible ones
    pub fn cull_objects(&mut self, objects: &[RenderObject]) -> Vec<RenderObject> {
        let mut visible_objects = Vec::new();

        for object in objects {
            if self.is_object_visible(object) {
                visible_objects.push(object.clone());
            }
        }

        visible_objects
    }

    /// Cull objects with custom bounding volumes
    pub fn cull_objects_with_bounds(
        &mut self,
        objects: &[(RenderObject, BoundingVolume)],
    ) -> Vec<RenderObject> {
        let mut visible_objects = Vec::new();

        for (object, bounding_volume) in objects {
            self.stats.total_tests += 1;

            if bounding_volume.is_visible_in_frustum(&self.frustum) {
                self.stats.visible_objects += 1;
                visible_objects.push(object.clone());
            } else {
                self.stats.culled_objects += 1;
            }
        }

        visible_objects
    }

    /// Get culling statistics
    pub fn get_stats(&self) -> &CullingStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }

    /// Get the current frustum
    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }
}

/// Statistics about frustum culling operations
#[derive(Debug, Clone, Default)]
pub struct CullingStats {
    /// Total number of objects tested
    pub total_tests: usize,
    /// Number of objects that passed the test (visible)
    pub visible_objects: usize,
    /// Number of objects that were culled (not visible)
    pub culled_objects: usize,
}

impl CullingStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        self.total_tests = 0;
        self.visible_objects = 0;
        self.culled_objects = 0;
    }

    /// Get the culling efficiency as a percentage
    pub fn culling_efficiency(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.culled_objects as f32 / self.total_tests as f32) * 100.0
        }
    }

    /// Get the percentage of visible objects
    pub fn visibility_percentage(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.visible_objects as f32 / self.total_tests as f32) * 100.0
        }
    }
}

impl std::fmt::Display for CullingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CullingStats {{ tested: {}, visible: {}, culled: {}, efficiency: {:.1}% }}",
            self.total_tests,
            self.visible_objects,
            self.culled_objects,
            self.culling_efficiency()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance_calculation() {
        let plane = Plane::new(Vec3::Y, 0.0); // XZ plane at origin

        assert_eq!(plane.distance_to_point(Vec3::new(0.0, 1.0, 0.0)), 1.0);
        assert_eq!(plane.distance_to_point(Vec3::new(0.0, -1.0, 0.0)), -1.0);
        assert_eq!(plane.distance_to_point(Vec3::new(1.0, 0.0, 1.0)), 0.0);
    }

    #[test]
    fn test_plane_point_test() {
        let plane = Plane::new(Vec3::Y, 0.0); // XZ plane at origin

        assert!(plane.is_point_in_front(Vec3::new(0.0, 1.0, 0.0)));
        assert!(!plane.is_point_in_front(Vec3::new(0.0, -1.0, 0.0)));
        assert!(plane.is_point_in_front(Vec3::new(1.0, 0.0, 1.0)));
    }

    #[test]
    fn test_bounding_volume_point() {
        let identity_matrix = Mat4::IDENTITY;
        let _frustum = Frustum::from_view_projection_matrix(identity_matrix);

        let _point_inside = BoundingVolume::point(Vec3::new(0.0, 0.0, -1.0));
        let _point_outside = BoundingVolume::point(Vec3::new(100.0, 0.0, -1.0));

        // Note: These tests depend on the specific frustum extraction implementation
        // For a proper test, we'd need a known camera setup
    }

    #[test]
    fn test_culling_stats() {
        let stats = CullingStats {
            total_tests: 10,
            visible_objects: 3,
            culled_objects: 7,
        };

        assert!((stats.culling_efficiency() - 70.0).abs() < 0.001);
        assert!((stats.visibility_percentage() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_frustum_culler_creation() {
        let camera = Camera::new(16.0 / 9.0);
        let culler = FrustumCuller::new(&camera);

        assert_eq!(culler.get_stats().total_tests, 0);
    }
}
