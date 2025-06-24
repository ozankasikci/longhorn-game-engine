//! Collider abstractions

use engine_ecs_core::Component;
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

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
    Box { size: Vec3 },

    /// Sphere collider
    Sphere { radius: f32 },

    /// Capsule collider
    Capsule { height: f32, radius: f32 },

    /// Cylinder collider
    Cylinder { height: f32, radius: f32 },

    /// Cone collider
    Cone { height: f32, radius: f32 },

    /// Convex hull from points
    ConvexHull { points: Vec<Vec3> },

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
    Box { size: Vec2 },

    /// Circle collider
    Circle { radius: f32 },

    /// Capsule collider
    Capsule { height: f32, radius: f32 },

    /// Convex polygon
    ConvexPolygon { points: Vec<Vec2> },

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
            memberships: 1,   // Group 1 by default
            filter: u32::MAX, // Collide with all groups by default
        }
    }
}

/// Predefined collision groups
pub mod collision_groups {
    use super::CollisionGroups;

    pub const STATIC: CollisionGroups = CollisionGroups {
        memberships: 1 << 0,
        filter: u32::MAX,
    };
    pub const DYNAMIC: CollisionGroups = CollisionGroups {
        memberships: 1 << 1,
        filter: u32::MAX,
    };
    pub const PLAYER: CollisionGroups = CollisionGroups {
        memberships: 1 << 2,
        filter: u32::MAX,
    };
    pub const ENEMY: CollisionGroups = CollisionGroups {
        memberships: 1 << 3,
        filter: u32::MAX,
    };
    pub const PROJECTILE: CollisionGroups = CollisionGroups {
        memberships: 1 << 4,
        filter: u32::MAX,
    };
    pub const PICKUP: CollisionGroups = CollisionGroups {
        memberships: 1 << 5,
        filter: u32::MAX,
    };
    pub const TRIGGER: CollisionGroups = CollisionGroups {
        memberships: 1 << 6,
        filter: u32::MAX,
    };
    pub const UI: CollisionGroups = CollisionGroups {
        memberships: 1 << 7,
        filter: u32::MAX,
    };
}

/// Collision filter for complex filtering logic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CollisionFilter {
    /// Groups configuration
    #[serde(default)]
    pub groups: CollisionGroups,

    /// Custom filter predicate (for runtime filtering)
    pub custom_filter: Option<String>, // Could be a script reference or rule name

    /// Whether to use precise collision detection
    pub precise: bool,

    /// Collision events to generate
    #[serde(default)]
    pub events: CollisionEvents,
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
        (self.collision_groups.filter & other.memberships) != 0
            && (other.filter & self.collision_groups.memberships) != 0
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
        Self {
            memberships,
            filter,
        }
    }

    /// Create groups that only belong to specific groups (collide with all)
    pub fn membership_only(memberships: u32) -> Self {
        Self {
            memberships,
            filter: u32::MAX,
        }
    }

    /// Create groups that only filter specific groups (belong to group 1)
    pub fn filter_only(filter: u32) -> Self {
        Self {
            memberships: 1,
            filter,
        }
    }

    /// Check if this group can collide with another
    pub fn can_collide_with(&self, other: &Self) -> bool {
        (self.filter & other.memberships) != 0 && (other.filter & self.memberships) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collider_default() {
        let collider = Collider::default();
        assert_eq!(collider.shape, ColliderShape::Box { size: Vec3::ONE });
        assert_eq!(collider.offset, Vec3::ZERO);
        assert!(!collider.is_sensor);
        assert_eq!(collider.collision_groups, CollisionGroups::default());
        assert!(collider.material.is_none());
        assert_eq!(collider.density, 1000.0);
        assert!(collider.user_data.is_none());
        assert!(collider.enabled);
    }

    #[test]
    fn test_collider2d_default() {
        let collider = Collider2D::default();
        assert_eq!(collider.shape, ColliderShape2D::Box { size: Vec2::ONE });
        assert_eq!(collider.offset, Vec2::ZERO);
        assert!(!collider.is_sensor);
        assert_eq!(collider.collision_groups, CollisionGroups::default());
        assert!(collider.material.is_none());
        assert_eq!(collider.density, 1000.0);
        assert!(collider.user_data.is_none());
        assert!(collider.enabled);
    }

    #[test]
    fn test_collider_constructors() {
        let box_collider = Collider::box_collider(Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(
            box_collider.shape,
            ColliderShape::Box {
                size: Vec3::new(2.0, 3.0, 4.0)
            }
        );
        assert!(!box_collider.is_sensor);

        let sphere_collider = Collider::sphere(5.0);
        assert_eq!(sphere_collider.shape, ColliderShape::Sphere { radius: 5.0 });

        let capsule_collider = Collider::capsule(10.0, 2.5);
        assert_eq!(
            capsule_collider.shape,
            ColliderShape::Capsule {
                height: 10.0,
                radius: 2.5
            }
        );
    }

    #[test]
    fn test_collider2d_constructors() {
        let box_collider = Collider2D::box_collider(Vec2::new(2.0, 3.0));
        assert_eq!(
            box_collider.shape,
            ColliderShape2D::Box {
                size: Vec2::new(2.0, 3.0)
            }
        );
        assert!(!box_collider.is_sensor);

        let circle_collider = Collider2D::circle(5.0);
        assert_eq!(
            circle_collider.shape,
            ColliderShape2D::Circle { radius: 5.0 }
        );

        let capsule_collider = Collider2D::capsule(10.0, 2.5);
        assert_eq!(
            capsule_collider.shape,
            ColliderShape2D::Capsule {
                height: 10.0,
                radius: 2.5
            }
        );
    }

    #[test]
    fn test_sensor_colliders() {
        let sensor_shape = ColliderShape::Box { size: Vec3::ONE };
        let sensor_collider = Collider::sensor(sensor_shape.clone());
        assert_eq!(sensor_collider.shape, sensor_shape);
        assert!(sensor_collider.is_sensor);

        let sensor_shape_2d = ColliderShape2D::Circle { radius: 1.0 };
        let sensor_collider_2d = Collider2D::sensor(sensor_shape_2d.clone());
        assert_eq!(sensor_collider_2d.shape, sensor_shape_2d);
        assert!(sensor_collider_2d.is_sensor);
    }

    #[test]
    fn test_collider_builder_pattern() {
        let groups = CollisionGroups::new(0b0010, 0b1100);
        let collider = Collider::box_collider(Vec3::ONE)
            .with_collision_groups(groups)
            .with_offset(Vec3::new(1.0, 2.0, 3.0))
            .with_density(2500.0);

        assert_eq!(collider.collision_groups, groups);
        assert_eq!(collider.offset, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(collider.density, 2500.0);
    }

    #[test]
    fn test_density_clamping() {
        let collider = Collider::box_collider(Vec3::ONE).with_density(-100.0);
        assert_eq!(collider.density, 0.001); // Should be clamped to minimum

        let collider2 = Collider::box_collider(Vec3::ONE).with_density(0.0);
        assert_eq!(collider2.density, 0.001); // Should be clamped to minimum

        let collider3 = Collider::box_collider(Vec3::ONE).with_density(1500.0);
        assert_eq!(collider3.density, 1500.0); // Should remain unchanged
    }

    #[test]
    fn test_volume_calculations() {
        // Box volume
        let box_collider = Collider::box_collider(Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(box_collider.volume(), 24.0);

        // Sphere volume: (4/3) * π * r³
        let sphere_collider = Collider::sphere(2.0);
        let expected_sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * 8.0;
        assert!((sphere_collider.volume() - expected_sphere_volume).abs() < 0.001);

        // Cylinder volume: π * r² * h
        let cylinder_collider = Collider {
            shape: ColliderShape::Cylinder {
                height: 5.0,
                radius: 2.0,
            },
            ..Default::default()
        };
        let expected_cylinder_volume = std::f32::consts::PI * 4.0 * 5.0;
        assert!((cylinder_collider.volume() - expected_cylinder_volume).abs() < 0.001);

        // Cone volume: (1/3) * π * r² * h
        let cone_collider = Collider {
            shape: ColliderShape::Cone {
                height: 6.0,
                radius: 3.0,
            },
            ..Default::default()
        };
        let expected_cone_volume = (1.0 / 3.0) * std::f32::consts::PI * 9.0 * 6.0;
        assert!((cone_collider.volume() - expected_cone_volume).abs() < 0.001);

        // Capsule volume: cylinder + sphere
        let capsule_collider = Collider::capsule(4.0, 1.0);
        let cylinder_vol = std::f32::consts::PI * 1.0 * 4.0;
        let sphere_vol = (4.0 / 3.0) * std::f32::consts::PI * 1.0;
        let expected_capsule_volume = cylinder_vol + sphere_vol;
        assert!((capsule_collider.volume() - expected_capsule_volume).abs() < 0.001);
    }

    #[test]
    fn test_area_calculations_2d() {
        // Box area
        let box_collider = Collider2D::box_collider(Vec2::new(3.0, 4.0));
        assert_eq!(box_collider.area(), 12.0);

        // Circle area: π * r²
        let circle_collider = Collider2D::circle(2.0);
        let expected_circle_area = std::f32::consts::PI * 4.0;
        assert!((circle_collider.area() - expected_circle_area).abs() < 0.001);

        // Capsule area: rectangle + circle
        let capsule_collider = Collider2D::capsule(5.0, 1.5);
        let rectangle_area = 5.0 * 3.0; // height * diameter
        let circle_area = std::f32::consts::PI * 2.25; // π * r²
        let expected_capsule_area = rectangle_area + circle_area;
        assert!((capsule_collider.area() - expected_capsule_area).abs() < 0.001);
    }

    #[test]
    fn test_mass_calculations() {
        let density = 2000.0;
        let collider = Collider::box_collider(Vec3::new(2.0, 2.0, 2.0)).with_density(density);
        let expected_mass = density * 8.0; // volume = 8, density = 2000
        assert_eq!(collider.calculate_mass(), expected_mass);

        let collider_2d = Collider2D::box_collider(Vec2::new(3.0, 4.0));
        // Use default density of 1000
        let expected_mass_2d = 1000.0 * 12.0; // area = 12, density = 1000
        assert_eq!(collider_2d.calculate_mass(), expected_mass_2d);
    }

    #[test]
    fn test_collision_groups_default() {
        let groups = CollisionGroups::default();
        assert_eq!(groups.memberships, 1);
        assert_eq!(groups.filter, u32::MAX);
    }

    #[test]
    fn test_collision_groups_constructors() {
        let groups = CollisionGroups::new(0b0110, 0b1010);
        assert_eq!(groups.memberships, 0b0110);
        assert_eq!(groups.filter, 0b1010);

        let membership_groups = CollisionGroups::membership_only(0b1100);
        assert_eq!(membership_groups.memberships, 0b1100);
        assert_eq!(membership_groups.filter, u32::MAX);

        let filter_groups = CollisionGroups::filter_only(0b0011);
        assert_eq!(filter_groups.memberships, 1);
        assert_eq!(filter_groups.filter, 0b0011);
    }

    #[test]
    fn test_collision_groups_filtering() {
        let group_a = CollisionGroups::new(0b0001, 0b0110); // Member of group 1, collides with groups 2,3
        let group_b = CollisionGroups::new(0b0010, 0b0101); // Member of group 2, collides with groups 1,3
        let group_c = CollisionGroups::new(0b0100, 0b0001); // Member of group 3, collides with group 1

        // A and B should collide (A filter includes B membership, B filter includes A membership)
        assert!(group_a.can_collide_with(&group_b));
        assert!(group_b.can_collide_with(&group_a));

        // A and C should collide
        assert!(group_a.can_collide_with(&group_c));
        assert!(group_c.can_collide_with(&group_a));

        // B and C should not collide (B filter doesn't include C membership, C filter doesn't include B membership)
        assert!(!group_b.can_collide_with(&group_c));
        assert!(!group_c.can_collide_with(&group_b));
    }

    #[test]
    fn test_collider_can_collide_with() {
        let collider_a = Collider::box_collider(Vec3::ONE)
            .with_collision_groups(CollisionGroups::new(0b0001, 0b0010));
        let groups_b = CollisionGroups::new(0b0010, 0b0001);
        let groups_c = CollisionGroups::new(0b0100, 0b1000);

        assert!(collider_a.can_collide_with(&groups_b));
        assert!(!collider_a.can_collide_with(&groups_c));
    }

    #[test]
    fn test_predefined_collision_groups() {
        use collision_groups::*;

        // Test that predefined groups have correct bit patterns
        assert_eq!(STATIC.memberships, 1 << 0); // Bit 0
        assert_eq!(DYNAMIC.memberships, 1 << 1); // Bit 1
        assert_eq!(PLAYER.memberships, 1 << 2); // Bit 2
        assert_eq!(ENEMY.memberships, 1 << 3); // Bit 3
        assert_eq!(PROJECTILE.memberships, 1 << 4); // Bit 4
        assert_eq!(PICKUP.memberships, 1 << 5); // Bit 5
        assert_eq!(TRIGGER.memberships, 1 << 6); // Bit 6
        assert_eq!(UI.memberships, 1 << 7); // Bit 7

        // All predefined groups should collide with everything by default
        for group in &[
            STATIC, DYNAMIC, PLAYER, ENEMY, PROJECTILE, PICKUP, TRIGGER, UI,
        ] {
            assert_eq!(group.filter, u32::MAX);
        }

        // Test collision between predefined groups
        assert!(PLAYER.can_collide_with(&ENEMY));
        assert!(PROJECTILE.can_collide_with(&STATIC));
    }

    #[test]
    fn test_collision_events_default() {
        let events = CollisionEvents::default();
        assert!(events.collision_started);
        assert!(!events.collision_persisted);
        assert!(events.collision_ended);
        assert!(events.sensor_events);
    }

    #[test]
    fn test_collision_filter_default() {
        let filter = CollisionFilter::default();
        assert_eq!(filter.groups, CollisionGroups::default());
        assert!(filter.custom_filter.is_none());
        assert!(!filter.precise);
        assert_eq!(filter.events, CollisionEvents::default());
    }

    #[test]
    fn test_shape_enum_variants() {
        // Test 3D shapes
        let box_shape = ColliderShape::Box { size: Vec3::ONE };
        let sphere_shape = ColliderShape::Sphere { radius: 1.0 };
        let _capsule_shape = ColliderShape::Capsule {
            height: 2.0,
            radius: 0.5,
        };
        let _cylinder_shape = ColliderShape::Cylinder {
            height: 3.0,
            radius: 1.0,
        };
        let _cone_shape = ColliderShape::Cone {
            height: 4.0,
            radius: 2.0,
        };

        match box_shape {
            ColliderShape::Box { size } => assert_eq!(size, Vec3::ONE),
            _ => panic!("Should be box shape"),
        }

        match sphere_shape {
            ColliderShape::Sphere { radius } => assert_eq!(radius, 1.0),
            _ => panic!("Should be sphere shape"),
        }

        // Test 2D shapes
        let box_shape_2d = ColliderShape2D::Box { size: Vec2::ONE };
        let circle_shape_2d = ColliderShape2D::Circle { radius: 1.0 };

        match box_shape_2d {
            ColliderShape2D::Box { size } => assert_eq!(size, Vec2::ONE),
            _ => panic!("Should be box shape"),
        }

        match circle_shape_2d {
            ColliderShape2D::Circle { radius } => assert_eq!(radius, 1.0),
            _ => panic!("Should be circle shape"),
        }
    }

    #[test]
    fn test_complex_shapes() {
        // Test convex hull
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let convex_hull = ColliderShape::ConvexHull {
            points: points.clone(),
        };

        match convex_hull {
            ColliderShape::ConvexHull {
                points: shape_points,
            } => {
                assert_eq!(shape_points.len(), 4);
                assert_eq!(shape_points[0], Vec3::new(0.0, 0.0, 0.0));
            }
            _ => panic!("Should be convex hull shape"),
        }

        // Test triangle mesh
        let vertices = vec![Vec3::ZERO, Vec3::X, Vec3::Y];
        let indices = vec![[0, 1, 2]];
        let triangle_mesh = ColliderShape::TriangleMesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
        };

        match triangle_mesh {
            ColliderShape::TriangleMesh {
                vertices: mesh_verts,
                indices: mesh_indices,
            } => {
                assert_eq!(mesh_verts.len(), 3);
                assert_eq!(mesh_indices.len(), 1);
                assert_eq!(mesh_indices[0], [0, 1, 2]);
            }
            _ => panic!("Should be triangle mesh shape"),
        }

        // Test compound shape
        let compound_shapes = vec![
            (Vec3::ZERO, ColliderShape::Box { size: Vec3::ONE }),
            (
                Vec3::new(2.0, 0.0, 0.0),
                ColliderShape::Sphere { radius: 0.5 },
            ),
        ];
        let compound = ColliderShape::Compound {
            shapes: compound_shapes.clone(),
        };

        match compound {
            ColliderShape::Compound { shapes } => {
                assert_eq!(shapes.len(), 2);
                assert_eq!(shapes[0].0, Vec3::ZERO);
                assert_eq!(shapes[1].0, Vec3::new(2.0, 0.0, 0.0));
            }
            _ => panic!("Should be compound shape"),
        }
    }

    #[test]
    fn test_heightfield_shape() {
        let heightfield = ColliderShape::Heightfield {
            width: 10,
            height: 10,
            heights: vec![0.0; 100], // 10x10 grid
            scale: Vec3::new(1.0, 1.0, 1.0),
        };

        match heightfield {
            ColliderShape::Heightfield {
                width,
                height,
                heights,
                scale,
            } => {
                assert_eq!(width, 10);
                assert_eq!(height, 10);
                assert_eq!(heights.len(), 100);
                assert_eq!(scale, Vec3::ONE);
            }
            _ => panic!("Should be heightfield shape"),
        }
    }
}
