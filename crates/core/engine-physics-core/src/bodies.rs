//! Rigid body abstractions

use engine_ecs_core::Component;
use glam::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Handle to a physics body
pub type BodyHandle = u32;

/// Rigid body component for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidBody {
    /// Type of rigid body
    pub body_type: BodyType,

    /// Mass of the body (kg)
    pub mass: f32,

    /// Linear velocity
    pub linear_velocity: Vec3,

    /// Angular velocity (radians per second)
    pub angular_velocity: Vec3,

    /// Linear damping factor (0.0 to 1.0)
    pub linear_damping: f32,

    /// Angular damping factor (0.0 to 1.0)
    pub angular_damping: f32,

    /// Gravity scale (1.0 = normal gravity, 0.0 = no gravity)
    pub gravity_scale: f32,

    /// Whether this body can sleep
    pub can_sleep: bool,

    /// Whether continuous collision detection is enabled
    pub ccd_enabled: bool,

    /// Lock rotation on specific axes
    pub lock_rotation: RotationLock,

    /// Lock translation on specific axes
    pub lock_translation: TranslationLock,

    /// User data for custom identification
    pub user_data: Option<u64>,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.01,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            can_sleep: true,
            ccd_enabled: false,
            lock_rotation: RotationLock::NONE,
            lock_translation: TranslationLock::NONE,
            user_data: None,
        }
    }
}

impl Component for RigidBody {}

/// 2D rigid body component (for 2D physics)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RigidBody2D {
    /// Type of rigid body
    pub body_type: BodyType,

    /// Mass of the body (kg)
    pub mass: f32,

    /// Linear velocity
    pub linear_velocity: Vec2,

    /// Angular velocity (radians per second)
    pub angular_velocity: f32,

    /// Linear damping factor (0.0 to 1.0)
    pub linear_damping: f32,

    /// Angular damping factor (0.0 to 1.0)
    pub angular_damping: f32,

    /// Gravity scale (1.0 = normal gravity, 0.0 = no gravity)
    pub gravity_scale: f32,

    /// Whether this body can sleep
    pub can_sleep: bool,

    /// Whether continuous collision detection is enabled
    pub ccd_enabled: bool,

    /// Whether rotation is locked
    pub lock_rotation: bool,

    /// User data for custom identification
    pub user_data: Option<u64>,
}

impl Default for RigidBody2D {
    fn default() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            mass: 1.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            linear_damping: 0.01,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            can_sleep: true,
            ccd_enabled: false,
            lock_rotation: false,
            user_data: None,
        }
    }
}

impl Component for RigidBody2D {}

/// Types of rigid bodies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyType {
    /// Static body (never moves, infinite mass)
    Static,
    /// Kinematic body (moves but not affected by forces)
    Kinematic,
    /// Dynamic body (affected by forces and collisions)
    Dynamic,
}

/// Rotation lock flags for 3D bodies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationLock {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl RotationLock {
    pub const NONE: Self = Self {
        x: false,
        y: false,
        z: false,
    };
    pub const ALL: Self = Self {
        x: true,
        y: true,
        z: true,
    };
    pub const X: Self = Self {
        x: true,
        y: false,
        z: false,
    };
    pub const Y: Self = Self {
        x: false,
        y: true,
        z: false,
    };
    pub const Z: Self = Self {
        x: false,
        y: false,
        z: true,
    };
}

/// Translation lock flags for 3D bodies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationLock {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl TranslationLock {
    pub const NONE: Self = Self {
        x: false,
        y: false,
        z: false,
    };
    pub const ALL: Self = Self {
        x: true,
        y: true,
        z: true,
    };
    pub const X: Self = Self {
        x: true,
        y: false,
        z: false,
    };
    pub const Y: Self = Self {
        x: false,
        y: true,
        z: false,
    };
    pub const Z: Self = Self {
        x: false,
        y: false,
        z: true,
    };
}

/// Mass properties for a rigid body
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MassProperties {
    /// Mass in kilograms
    pub mass: f32,

    /// Center of mass offset from body origin
    pub center_of_mass: Vec3,

    /// Moment of inertia tensor (3x3 matrix stored as 9 elements)
    pub inertia_tensor: [f32; 9],

    /// Whether mass properties are automatically calculated
    pub auto_calculate: bool,
}

impl Default for MassProperties {
    fn default() -> Self {
        Self {
            mass: 1.0,
            center_of_mass: Vec3::ZERO,
            inertia_tensor: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            auto_calculate: true,
        }
    }
}

impl RigidBody {
    /// Create a new dynamic rigid body
    pub fn dynamic() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            ..Default::default()
        }
    }

    /// Create a new static rigid body
    pub fn static_body() -> Self {
        Self {
            body_type: BodyType::Static,
            mass: f32::INFINITY,
            can_sleep: false,
            ..Default::default()
        }
    }

    /// Create a new kinematic rigid body
    pub fn kinematic() -> Self {
        Self {
            body_type: BodyType::Kinematic,
            ..Default::default()
        }
    }

    /// Set mass
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass.max(0.001); // Prevent zero or negative mass
        self
    }

    /// Set linear velocity
    pub fn with_linear_velocity(mut self, velocity: Vec3) -> Self {
        self.linear_velocity = velocity;
        self
    }

    /// Set angular velocity
    pub fn with_angular_velocity(mut self, velocity: Vec3) -> Self {
        self.angular_velocity = velocity;
        self
    }

    /// Set gravity scale
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Enable continuous collision detection
    pub fn with_ccd(mut self) -> Self {
        self.ccd_enabled = true;
        self
    }

    /// Lock rotation on specific axes
    pub fn with_rotation_lock(mut self, lock: RotationLock) -> Self {
        self.lock_rotation = lock;
        self
    }

    /// Lock translation on specific axes
    pub fn with_translation_lock(mut self, lock: TranslationLock) -> Self {
        self.lock_translation = lock;
        self
    }

    /// Check if this is a dynamic body
    pub fn is_dynamic(&self) -> bool {
        matches!(self.body_type, BodyType::Dynamic)
    }

    /// Check if this is a static body
    pub fn is_static(&self) -> bool {
        matches!(self.body_type, BodyType::Static)
    }

    /// Check if this is a kinematic body
    pub fn is_kinematic(&self) -> bool {
        matches!(self.body_type, BodyType::Kinematic)
    }

    /// Calculate kinetic energy
    pub fn kinetic_energy(&self) -> f32 {
        if self.is_static() {
            return 0.0;
        }

        let linear_energy = 0.5 * self.mass * self.linear_velocity.length_squared();
        let angular_energy = 0.5 * self.angular_velocity.length_squared(); // Simplified

        linear_energy + angular_energy
    }

    /// Apply impulse to the body
    pub fn apply_impulse(&mut self, impulse: Vec3, point: Option<Vec3>) {
        if !self.is_dynamic() {
            return;
        }

        // Apply linear impulse
        self.linear_velocity += impulse / self.mass;

        // Apply angular impulse if point is specified
        if let Some(point) = point {
            let torque = point.cross(impulse);
            self.angular_velocity += torque / self.mass; // Simplified
        }
    }

    /// Apply force (requires delta time for integration)
    pub fn apply_force(&mut self, force: Vec3, delta_time: f32) {
        if !self.is_dynamic() {
            return;
        }

        let acceleration = force / self.mass;
        self.linear_velocity += acceleration * delta_time;
    }
}
