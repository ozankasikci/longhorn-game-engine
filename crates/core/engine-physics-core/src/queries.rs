//! Physics query abstractions

use glam::{Vec2, Vec3};
use serde::{Serialize, Deserialize};
use crate::{CollisionGroups, BodyHandle, ColliderHandle};

/// Ray for physics queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    /// Ray origin
    pub origin: Vec3,
    /// Ray direction (should be normalized)
    pub direction: Vec3,
    /// Maximum distance to check
    pub max_distance: f32,
}

/// 2D ray for physics queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ray2D {
    /// Ray origin
    pub origin: Vec2,
    /// Ray direction (should be normalized)
    pub direction: Vec2,
    /// Maximum distance to check
    pub max_distance: f32,
}

/// Result of a raycast query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaycastHit {
    /// The entity that was hit
    pub entity: u32,
    /// The collider that was hit
    pub collider: ColliderHandle,
    /// The rigid body that was hit (if any)
    pub body: Option<BodyHandle>,
    /// Point of impact in world space
    pub point: Vec3,
    /// Surface normal at impact point
    pub normal: Vec3,
    /// Distance from ray origin to hit point
    pub distance: f32,
    /// UV coordinates on the hit surface (if available)
    pub uv: Option<Vec2>,
    /// Triangle index (for mesh colliders)
    pub triangle_index: Option<u32>,
}

/// Result of a 2D raycast query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaycastHit2D {
    /// The entity that was hit
    pub entity: u32,
    /// The collider that was hit
    pub collider: ColliderHandle,
    /// The rigid body that was hit (if any)
    pub body: Option<BodyHandle>,
    /// Point of impact in world space
    pub point: Vec2,
    /// Surface normal at impact point
    pub normal: Vec2,
    /// Distance from ray origin to hit point
    pub distance: f32,
}

/// Query filter for physics queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Collision groups to check against
    pub groups: Option<CollisionGroups>,
    /// Entities to exclude from the query
    pub exclude_entities: Vec<u32>,
    /// Whether to include sensors in the query
    pub include_sensors: bool,
    /// Custom predicate for filtering (implementation-specific)
    pub custom_predicate: Option<String>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            groups: None,
            exclude_entities: Vec::new(),
            include_sensors: false,
            custom_predicate: None,
        }
    }
}

/// Shape for overlap queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryShape {
    /// Sphere shape
    Sphere { radius: f32 },
    /// Box shape
    Box { size: Vec3 },
    /// Capsule shape
    Capsule { height: f32, radius: f32 },
}

/// 2D shape for overlap queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryShape2D {
    /// Circle shape
    Circle { radius: f32 },
    /// Box shape
    Box { size: Vec2 },
    /// Capsule shape
    Capsule { height: f32, radius: f32 },
}

/// Overlap query configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlapQuery {
    /// Shape to check for overlaps
    pub shape: QueryShape,
    /// Position of the shape
    pub position: Vec3,
    /// Rotation of the shape
    pub rotation: glam::Quat,
    /// Query filter
    pub filter: QueryFilter,
}

/// 2D overlap query configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlapQuery2D {
    /// Shape to check for overlaps
    pub shape: QueryShape2D,
    /// Position of the shape
    pub position: Vec2,
    /// Rotation of the shape in radians
    pub rotation: f32,
    /// Query filter
    pub filter: QueryFilter,
}

/// Traits for physics world queries
pub trait PhysicsQueries {
    /// Cast a ray and return the first hit
    fn raycast(&self, ray: &Ray, filter: &QueryFilter) -> Option<RaycastHit>;
    
    /// Cast a ray and return all hits
    fn raycast_all(&self, ray: &Ray, filter: &QueryFilter) -> Vec<RaycastHit>;
    
    /// Check if a shape overlaps with any colliders
    fn overlap_test(&self, query: &OverlapQuery) -> bool;
    
    /// Get all entities overlapping with a shape
    fn overlap_all(&self, query: &OverlapQuery) -> Vec<u32>;
    
    /// Get the closest point on any collider to a given point
    fn closest_point(&self, point: Vec3, filter: &QueryFilter) -> Option<(u32, Vec3)>;
    
    /// Check if a point is inside any collider
    fn point_test(&self, point: Vec3, filter: &QueryFilter) -> Option<u32>;
}

/// Trait for 2D physics world queries
pub trait PhysicsQueries2D {
    /// Cast a ray and return the first hit
    fn raycast(&self, ray: &Ray2D, filter: &QueryFilter) -> Option<RaycastHit2D>;
    
    /// Cast a ray and return all hits
    fn raycast_all(&self, ray: &Ray2D, filter: &QueryFilter) -> Vec<RaycastHit2D>;
    
    /// Check if a shape overlaps with any colliders
    fn overlap_test(&self, query: &OverlapQuery2D) -> bool;
    
    /// Get all entities overlapping with a shape
    fn overlap_all(&self, query: &OverlapQuery2D) -> Vec<u32>;
    
    /// Get the closest point on any collider to a given point
    fn closest_point(&self, point: Vec2, filter: &QueryFilter) -> Option<(u32, Vec2)>;
    
    /// Check if a point is inside any collider
    fn point_test(&self, point: Vec2, filter: &QueryFilter) -> Option<u32>;
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            max_distance,
        }
    }
    
    /// Get point along the ray at given distance
    pub fn point_at_distance(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}

impl Ray2D {
    /// Create a new 2D ray
    pub fn new(origin: Vec2, direction: Vec2, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            max_distance,
        }
    }
    
    /// Get point along the ray at given distance
    pub fn point_at_distance(&self, distance: f32) -> Vec2 {
        self.origin + self.direction * distance
    }
}

impl QueryFilter {
    /// Create a filter that checks all collision groups
    pub fn all() -> Self {
        Self::default()
    }
    
    /// Create a filter for specific collision groups
    pub fn groups(groups: CollisionGroups) -> Self {
        Self {
            groups: Some(groups),
            ..Self::default()
        }
    }
    
    /// Create a filter that excludes specific entities
    pub fn exclude_entities(entities: Vec<u32>) -> Self {
        Self {
            exclude_entities: entities,
            ..Self::default()
        }
    }
    
    /// Include sensors in the query
    pub fn with_sensors(mut self) -> Self {
        self.include_sensors = true;
        self
    }
}