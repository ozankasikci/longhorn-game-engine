//! Collider abstractions

use glam::{Vec2, Vec3};
use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;

/// Handle to a physics collider
pub type ColliderHandle = u32;

/// Collider component for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    /// Shape of the collider
    pub shape: ColliderShape,
    
    /// Offset from the entity's transform
    pub offset: Vec3,
    
    /// Whether this collider is a sensor (no collision response)
    pub is_sensor: bool,
    
    /// Collision groups for filtering
    pub collision_groups: CollisionGroups,
    
    /// Physics material handle
    pub material: Option<u32>,
    
    /// Density for mass calculation (kg/m³)
    pub density: f32,
    
    /// User data for identification
    pub user_data: Option<u64>,
    
    /// Whether this collider is enabled
    pub enabled: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box { size: Vec3::ONE },
            offset: Vec3::ZERO,
            is_sensor: false,
            collision_groups: CollisionGroups::default(),
            material: None,
            density: 1000.0, // Water density
            user_data: None,
            enabled: true,
        }
    }
}

impl Component for Collider {}


/// 2D collider component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collider2D {
    /// Shape of the collider
    pub shape: ColliderShape2D,
    
    /// Offset from the entity's transform
    pub offset: Vec2,
    
    /// Whether this collider is a sensor
    pub is_sensor: bool,
    
    /// Collision groups for filtering
    pub collision_groups: CollisionGroups,
    
    /// Physics material handle
    pub material: Option<u32>,
    
    /// Density for mass calculation (kg/m²)
    pub density: f32,
    
    /// User data for identification
    pub user_data: Option<u64>,
    
    /// Whether this collider is enabled
    pub enabled: bool,
}

impl Default for Collider2D {
    fn default() -> Self {
        Self {
            shape: ColliderShape2D::Box { size: Vec2::ONE },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_groups: CollisionGroups::default(),
            material: None,
            density: 1000.0,
            user_data: None,
            enabled: true,
        }
    }
}

impl Component for Collider2D {}


/// 3D collider shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// Box collider
    Box {
        size: Vec3,
    },
    
    /// Sphere collider
    Sphere {
        radius: f32,
    },
    
    /// Capsule collider
    Capsule {
        height: f32,
        radius: f32,
    },
    
    /// Cylinder collider
    Cylinder {
        height: f32,
        radius: f32,
    },
    
    /// Cone collider
    Cone {
        height: f32,
        radius: f32,
    },
    
    /// Convex hull from points
    ConvexHull {
        points: Vec<Vec3>,
    },
    
    /// Triangle mesh (for static bodies only)
    TriangleMesh {
        vertices: Vec<Vec3>,
        indices: Vec<[u32; 3]>,
    },
    
    /// Heightfield for terrain
    Heightfield {
        width: u32,
        height: u32,
        heights: Vec<f32>,
        scale: Vec3,
    },
    
    /// Compound shape (multiple shapes combined)
    Compound {
        shapes: Vec<(Vec3, ColliderShape)>, // (offset, shape)
    },
}

/// 2D collider shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape2D {
    /// Box collider
    Box {
        size: Vec2,
    },
    
    /// Circle collider
    Circle {
        radius: f32,
    },
    
    /// Capsule collider
    Capsule {
        height: f32,
        radius: f32,
    },
    
    /// Convex polygon
    ConvexPolygon {
        points: Vec<Vec2>,
    },
    
    /// Triangle mesh
    TriangleMesh {
        vertices: Vec<Vec2>,
        indices: Vec<[u32; 3]>,
    },
    
    /// Compound shape
    Compound {
        shapes: Vec<(Vec2, ColliderShape2D)>,
    },
}

/// Collision group configuration for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionGroups {
    /// Which groups this collider belongs to (bit mask)
    pub memberships: u32,
    
    /// Which groups this collider can collide with (bit mask)
    pub filter: u32,
}

impl Default for CollisionGroups {
    fn default() -> Self {
        Self {
            memberships: 1, // Group 1 by default
            filter: u32::MAX, // Collide with all groups by default
        }
    }
}

/// Predefined collision groups
pub mod collision_groups {
    use super::CollisionGroups;
    
    pub const STATIC: CollisionGroups = CollisionGroups { memberships: 1 << 0, filter: u32::MAX };
    pub const DYNAMIC: CollisionGroups = CollisionGroups { memberships: 1 << 1, filter: u32::MAX };
    pub const PLAYER: CollisionGroups = CollisionGroups { memberships: 1 << 2, filter: u32::MAX };
    pub const ENEMY: CollisionGroups = CollisionGroups { memberships: 1 << 3, filter: u32::MAX };
    pub const PROJECTILE: CollisionGroups = CollisionGroups { memberships: 1 << 4, filter: u32::MAX };
    pub const PICKUP: CollisionGroups = CollisionGroups { memberships: 1 << 5, filter: u32::MAX };
    pub const TRIGGER: CollisionGroups = CollisionGroups { memberships: 1 << 6, filter: u32::MAX };
    pub const UI: CollisionGroups = CollisionGroups { memberships: 1 << 7, filter: u32::MAX };
}

/// Collision filter for complex filtering logic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionFilter {
    /// Groups configuration
    pub groups: CollisionGroups,
    
    /// Custom filter predicate (for runtime filtering)
    pub custom_filter: Option<String>, // Could be a script reference or rule name
    
    /// Whether to use precise collision detection
    pub precise: bool,
    
    /// Collision events to generate
    pub events: CollisionEvents,
}

impl Default for CollisionFilter {
    fn default() -> Self {
        Self {
            groups: CollisionGroups::default(),
            custom_filter: None,
            precise: false,
            events: CollisionEvents::default(),
        }
    }
}

/// Configuration for what collision events to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionEvents {
    /// Generate events when collision starts
    pub collision_started: bool,
    
    /// Generate events while collision persists
    pub collision_persisted: bool,
    
    /// Generate events when collision ends
    pub collision_ended: bool,
    
    /// Generate sensor events (for triggers)
    pub sensor_events: bool,
}

impl Default for CollisionEvents {
    fn default() -> Self {
        Self {
            collision_started: true,
            collision_persisted: false,
            collision_ended: true,
            sensor_events: true,
        }
    }
}

impl Collider {
    /// Create a box collider
    pub fn box_collider(size: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box { size },
            ..Default::default()
        }
    }
    
    /// Create a sphere collider
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            ..Default::default()
        }
    }
    
    /// Create a capsule collider
    pub fn capsule(height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule { height, radius },
            ..Default::default()
        }
    }
    
    /// Create a sensor collider (no collision response)
    pub fn sensor(shape: ColliderShape) -> Self {
        Self {
            shape,
            is_sensor: true,
            ..Default::default()
        }
    }
    
    /// Set collision groups
    pub fn with_collision_groups(mut self, groups: CollisionGroups) -> Self {
        self.collision_groups = groups;
        self
    }
    
    /// Set offset from transform
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }
    
    /// Set density
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density.max(0.001);
        self
    }
    
    /// Calculate approximate volume based on shape
    pub fn volume(&self) -> f32 {
        match &self.shape {
            ColliderShape::Box { size } => size.x * size.y * size.z,
            ColliderShape::Sphere { radius } => (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3),
            ColliderShape::Capsule { height, radius } => {
                let cylinder_volume = std::f32::consts::PI * radius.powi(2) * height;
                let sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3);
                cylinder_volume + sphere_volume
            }
            ColliderShape::Cylinder { height, radius } => {
                std::f32::consts::PI * radius.powi(2) * height
            }
            ColliderShape::Cone { height, radius } => {
                (1.0 / 3.0) * std::f32::consts::PI * radius.powi(2) * height
            }
            _ => 1.0, // Default fallback for complex shapes
        }
    }
    
    /// Calculate mass from density and volume
    pub fn calculate_mass(&self) -> f32 {
        self.density * self.volume()
    }
    
    /// Check if this collider can collide with another
    pub fn can_collide_with(&self, other: &CollisionGroups) -> bool {
        (self.collision_groups.filter & other.memberships) != 0 &&
        (other.filter & self.collision_groups.memberships) != 0
    }
}

impl Collider2D {
    /// Create a box collider
    pub fn box_collider(size: Vec2) -> Self {
        Self {
            shape: ColliderShape2D::Box { size },
            ..Default::default()
        }
    }
    
    /// Create a circle collider
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape2D::Circle { radius },
            ..Default::default()
        }
    }
    
    /// Create a capsule collider
    pub fn capsule(height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape2D::Capsule { height, radius },
            ..Default::default()
        }
    }
    
    /// Create a sensor collider
    pub fn sensor(shape: ColliderShape2D) -> Self {
        Self {
            shape,
            is_sensor: true,
            ..Default::default()
        }
    }
    
    /// Calculate approximate area based on shape
    pub fn area(&self) -> f32 {
        match &self.shape {
            ColliderShape2D::Box { size } => size.x * size.y,
            ColliderShape2D::Circle { radius } => std::f32::consts::PI * radius.powi(2),
            ColliderShape2D::Capsule { height, radius } => {
                let rectangle_area = height * (radius * 2.0);
                let circle_area = std::f32::consts::PI * radius.powi(2);
                rectangle_area + circle_area
            }
            _ => 1.0, // Default fallback
        }
    }
    
    /// Calculate mass from density and area
    pub fn calculate_mass(&self) -> f32 {
        self.density * self.area()
    }
}

impl CollisionGroups {
    /// Create groups that belong to and collide with specific groups
    pub fn new(memberships: u32, filter: u32) -> Self {
        Self { memberships, filter }
    }
    
    /// Create groups that only belong to specific groups (collide with all)
    pub fn membership_only(memberships: u32) -> Self {
        Self { memberships, filter: u32::MAX }
    }
    
    /// Create groups that only filter specific groups (belong to group 1)
    pub fn filter_only(filter: u32) -> Self {
        Self { memberships: 1, filter }
    }
    
    /// Check if this group can collide with another
    pub fn can_collide_with(&self, other: &Self) -> bool {
        (self.filter & other.memberships) != 0 && (other.filter & self.memberships) != 0
    }
}