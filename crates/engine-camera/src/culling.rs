//! Frustum culling system for mobile optimization

use crate::Result;
use glam::{Mat4, Vec3};
use serde::{Serialize, Deserialize};

/// Frustum for culling operations
#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [FrustumPlane; 6],
}

/// Single frustum plane
#[derive(Debug, Clone, Copy)]
pub struct FrustumPlane {
    normal: Vec3,
    distance: f32,
}

impl FrustumPlane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }
    
    /// Test point against plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
    
    /// Test if point is in front of plane
    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

impl Frustum {
    /// Create frustum from view-projection matrix
    pub fn from_matrix(view_proj: Mat4) -> Result<Self> {
        let m = view_proj.to_cols_array_2d();
        
        // Extract frustum planes from matrix
        // Left plane
        let left = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]).normalize(),
            m[3][3] + m[3][0]
        );
        
        // Right plane
        let right = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]).normalize(),
            m[3][3] - m[3][0]
        );
        
        // Bottom plane
        let bottom = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]).normalize(),
            m[3][3] + m[3][1]
        );
        
        // Top plane
        let top = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]).normalize(),
            m[3][3] - m[3][1]
        );
        
        // Near plane
        let near = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]).normalize(),
            m[3][3] + m[3][2]
        );
        
        // Far plane
        let far = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]).normalize(),
            m[3][3] - m[3][2]
        );
        
        Ok(Self {
            planes: [left, right, bottom, top, near, far],
        })
    }
    
    /// Test point against frustum
    pub fn test_point(&self, point: Vec3) -> CullingResult {
        for plane in &self.planes {
            if !plane.is_point_in_front(point) {
                return CullingResult::Outside;
            }
        }
        CullingResult::Inside
    }
    
    /// Test sphere against frustum
    pub fn test_sphere(&self, center: Vec3, radius: f32) -> CullingResult {
        let mut intersecting = false;
        
        for plane in &self.planes {
            let distance = plane.distance_to_point(center);
            
            if distance < -radius {
                return CullingResult::Outside;
            } else if distance < radius {
                intersecting = true;
            }
        }
        
        if intersecting {
            CullingResult::Intersecting
        } else {
            CullingResult::Inside
        }
    }
    
    /// Test axis-aligned bounding box against frustum
    pub fn test_aabb(&self, min: Vec3, max: Vec3) -> CullingResult {
        let mut intersecting = false;
        
        for plane in &self.planes {
            // Find the positive vertex (furthest along plane normal)
            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );
            
            // If positive vertex is outside, entire AABB is outside
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return CullingResult::Outside;
            }
            
            // Find the negative vertex (closest along plane normal)
            let negative_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { min.x } else { max.x },
                if plane.normal.y >= 0.0 { min.y } else { max.y },
                if plane.normal.z >= 0.0 { min.z } else { max.z },
            );
            
            // If negative vertex is outside, AABB intersects the plane
            if plane.distance_to_point(negative_vertex) < 0.0 {
                intersecting = true;
            }
        }
        
        if intersecting {
            CullingResult::Intersecting
        } else {
            CullingResult::Inside
        }
    }
    
    /// Get frustum planes
    pub fn planes(&self) -> &[FrustumPlane; 6] {
        &self.planes
    }
}

impl Default for Frustum {
    fn default() -> Self {
        // Create a default frustum that includes everything
        let plane = FrustumPlane::new(Vec3::Y, -1000.0);
        Self {
            planes: [plane; 6],
        }
    }
}

/// Result of culling test
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullingResult {
    /// Object is completely inside the frustum
    Inside,
    /// Object is completely outside the frustum
    Outside,
    /// Object partially intersects the frustum
    Intersecting,
}

impl CullingResult {
    /// Check if object should be rendered
    pub fn should_render(&self) -> bool {
        matches!(self, CullingResult::Inside | CullingResult::Intersecting)
    }
    
    /// Check if object is completely outside
    pub fn is_outside(&self) -> bool {
        matches!(self, CullingResult::Outside)
    }
}

/// Statistics for culling performance monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CullingStats {
    pub total_objects: u32,
    pub inside_count: u32,
    pub outside_count: u32,
    pub intersecting_count: u32,
    pub culling_time_ms: f32,
}

impl CullingStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a culling result
    pub fn record(&mut self, result: CullingResult) {
        self.total_objects += 1;
        match result {
            CullingResult::Inside => self.inside_count += 1,
            CullingResult::Outside => self.outside_count += 1,
            CullingResult::Intersecting => self.intersecting_count += 1,
        }
    }
    
    /// Get culling efficiency (percentage of objects culled)
    pub fn culling_efficiency(&self) -> f32 {
        if self.total_objects == 0 {
            0.0
        } else {
            (self.outside_count as f32 / self.total_objects as f32) * 100.0
        }
    }
    
    /// Get render count (objects that should be rendered)
    pub fn render_count(&self) -> u32 {
        self.inside_count + self.intersecting_count
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Set timing information
    pub fn set_timing(&mut self, time_ms: f32) {
        self.culling_time_ms = time_ms;
    }
}

/// Bounding volume types for culling
#[derive(Debug, Clone)]
pub enum BoundingVolume {
    Point(Vec3),
    Sphere { center: Vec3, radius: f32 },
    AABB { min: Vec3, max: Vec3 },
}

impl BoundingVolume {
    /// Test against frustum
    pub fn test_frustum(&self, frustum: &Frustum) -> CullingResult {
        match self {
            BoundingVolume::Point(point) => frustum.test_point(*point),
            BoundingVolume::Sphere { center, radius } => frustum.test_sphere(*center, *radius),
            BoundingVolume::AABB { min, max } => frustum.test_aabb(*min, *max),
        }
    }
    
    /// Create AABB from center and size
    pub fn aabb_from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self::AABB {
            min: center - half_size,
            max: center + half_size,
        }
    }
    
    /// Create sphere from AABB
    pub fn sphere_from_aabb(min: Vec3, max: Vec3) -> Self {
        let center = (min + max) * 0.5;
        let radius = (max - center).length();
        Self::Sphere { center, radius }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frustum_plane() {
        let plane = FrustumPlane::new(Vec3::Y, -1.0);
        
        // Point above plane should be in front
        assert!(plane.is_point_in_front(Vec3::new(0.0, 1.0, 0.0)));
        
        // Point below plane should be behind
        assert!(!plane.is_point_in_front(Vec3::new(0.0, -2.0, 0.0)));
        
        // Point on plane should be in front (distance = 0)
        assert!(plane.is_point_in_front(Vec3::new(0.0, 1.0, 0.0)));
    }
    
    #[test]
    fn test_culling_result() {
        assert!(CullingResult::Inside.should_render());
        assert!(CullingResult::Intersecting.should_render());
        assert!(!CullingResult::Outside.should_render());
        
        assert!(!CullingResult::Inside.is_outside());
        assert!(!CullingResult::Intersecting.is_outside());
        assert!(CullingResult::Outside.is_outside());
    }
    
    #[test]
    fn test_culling_stats() {
        let mut stats = CullingStats::new();
        
        stats.record(CullingResult::Inside);
        stats.record(CullingResult::Outside);
        stats.record(CullingResult::Outside);
        stats.record(CullingResult::Intersecting);
        
        assert_eq!(stats.total_objects, 4);
        assert_eq!(stats.inside_count, 1);
        assert_eq!(stats.outside_count, 2);
        assert_eq!(stats.intersecting_count, 1);
        assert_eq!(stats.render_count(), 2);
        assert_eq!(stats.culling_efficiency(), 50.0);
    }
    
    #[test]
    fn test_bounding_volume_aabb_creation() {
        let center = Vec3::new(0.0, 0.0, 0.0);
        let size = Vec3::new(2.0, 4.0, 6.0);
        
        if let BoundingVolume::AABB { min, max } = BoundingVolume::aabb_from_center_size(center, size) {
            assert_eq!(min, Vec3::new(-1.0, -2.0, -3.0));
            assert_eq!(max, Vec3::new(1.0, 2.0, 3.0));
        } else {
            panic!("Expected AABB");
        }
    }
}