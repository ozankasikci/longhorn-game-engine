//! Frustum culling abstractions
//! 
//! This module defines core abstractions for frustum culling.
//! Concrete implementations are provided by implementation crates.

use glam::Vec3;
use serde::{Serialize, Deserialize};

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

/// Trait for frustum culling operations
/// 
/// This trait should be implemented by camera implementation crates
/// to provide actual culling algorithms.
pub trait FrustumCuller: Send + Sync {
    /// Test a point against the frustum
    fn test_point(&self, point: Vec3) -> CullingResult;
    
    /// Test a sphere against the frustum
    fn test_sphere(&self, center: Vec3, radius: f32) -> CullingResult;
    
    /// Test an axis-aligned bounding box against the frustum
    fn test_aabb(&self, min: Vec3, max: Vec3) -> CullingResult;
    
    /// Test a bounding volume against the frustum
    fn test_bounding_volume(&self, volume: &BoundingVolume) -> CullingResult {
        match volume {
            BoundingVolume::Point(point) => self.test_point(*point),
            BoundingVolume::Sphere { center, radius } => self.test_sphere(*center, *radius),
            BoundingVolume::AABB { min, max } => self.test_aabb(*min, *max),
        }
    }
}

/// Trait for creating frustum cullers from camera data
pub trait FrustumCullerFactory {
    /// Create a frustum culler from view and projection matrices
    fn create_from_matrices(view: glam::Mat4, projection: glam::Mat4) -> crate::Result<Box<dyn FrustumCuller>>
    where
        Self: Sized;
    
    /// Create a frustum culler from view-projection matrix
    fn create_from_view_projection(view_proj: glam::Mat4) -> crate::Result<Box<dyn FrustumCuller>>
    where
        Self: Sized;
}

/// Configuration for culling operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CullingConfig {
    /// Whether to enable frustum culling
    pub enabled: bool,
    /// Whether to use hierarchical culling
    pub hierarchical: bool,
    /// Whether to collect culling statistics
    pub collect_stats: bool,
    /// Maximum distance for culling (for distance-based culling)
    pub max_distance: Option<f32>,
}

impl Default for CullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hierarchical: false,
            collect_stats: false,
            max_distance: None,
        }
    }
}