//! Physics joint abstractions

use glam::Vec3;
use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;

/// Physics joint component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    /// Type of joint
    pub joint_type: JointType,
    /// Connected entity
    pub connected_entity: Option<u32>,
    /// Joint anchor point in local space
    pub anchor: Vec3,
    /// Connected anchor point in connected entity's local space
    pub connected_anchor: Vec3,
    /// Whether the joint should break at high forces
    pub breakable: bool,
    /// Force threshold for breaking
    pub break_force: f32,
    /// Torque threshold for breaking
    pub break_torque: f32,
    /// Whether the joint is enabled
    pub enabled: bool,
}

impl Component for Joint {}


/// Types of physics joints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JointType {
    /// Fixed joint (no degrees of freedom)
    Fixed,
    /// Hinge joint (1 rotational DOF)
    Hinge {
        axis: Vec3,
        limits: Option<AngularLimits>,
        motor: Option<AngularMotor>,
    },
    /// Ball joint (3 rotational DOF)
    Ball {
        limits: Option<Vec3>, // Angular limits per axis
    },
    /// Slider joint (1 translational DOF)
    Slider {
        axis: Vec3,
        limits: Option<LinearLimits>,
        motor: Option<LinearMotor>,
    },
    /// Spring joint
    Spring {
        stiffness: f32,
        damping: f32,
        rest_length: f32,
    },
    /// Generic 6DOF joint
    Generic6DOF {
        linear_limits: [Option<LinearLimits>; 3],
        angular_limits: [Option<AngularLimits>; 3],
        linear_motors: [Option<LinearMotor>; 3],
        angular_motors: [Option<AngularMotor>; 3],
    },
}

/// Angular limits for rotational joints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularLimits {
    /// Lower limit in radians
    pub lower: f32,
    /// Upper limit in radians
    pub upper: f32,
    /// Stiffness of the limit
    pub stiffness: f32,
    /// Damping when hitting limits
    pub damping: f32,
}

/// Linear limits for translational joints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearLimits {
    /// Lower limit in meters
    pub lower: f32,
    /// Upper limit in meters
    pub upper: f32,
    /// Stiffness of the limit
    pub stiffness: f32,
    /// Damping when hitting limits
    pub damping: f32,
}

/// Angular motor for rotational joints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AngularMotor {
    /// Target velocity in rad/s
    pub target_velocity: f32,
    /// Maximum force the motor can apply
    pub max_force: f32,
    /// Whether motor is enabled
    pub enabled: bool,
}

/// Linear motor for translational joints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearMotor {
    /// Target velocity in m/s
    pub target_velocity: f32,
    /// Maximum force the motor can apply
    pub max_force: f32,
    /// Whether motor is enabled
    pub enabled: bool,
}

impl Joint {
    /// Create a fixed joint
    pub fn fixed() -> Self {
        Self {
            joint_type: JointType::Fixed,
            connected_entity: None,
            anchor: Vec3::ZERO,
            connected_anchor: Vec3::ZERO,
            breakable: false,
            break_force: f32::INFINITY,
            break_torque: f32::INFINITY,
            enabled: true,
        }
    }
    
    /// Create a hinge joint
    pub fn hinge(axis: Vec3) -> Self {
        Self {
            joint_type: JointType::Hinge { axis, limits: None, motor: None },
            ..Self::fixed()
        }
    }
    
    /// Create a spring joint
    pub fn spring(stiffness: f32, damping: f32) -> Self {
        Self {
            joint_type: JointType::Spring { 
                stiffness, 
                damping, 
                rest_length: 1.0 
            },
            ..Self::fixed()
        }
    }
    
    /// Set connected entity
    pub fn with_connected_entity(mut self, entity: u32) -> Self {
        self.connected_entity = Some(entity);
        self
    }
    
    /// Set anchor points
    pub fn with_anchors(mut self, anchor: Vec3, connected_anchor: Vec3) -> Self {
        self.anchor = anchor;
        self.connected_anchor = connected_anchor;
        self
    }
    
    /// Make joint breakable
    pub fn breakable(mut self, force: f32, torque: f32) -> Self {
        self.breakable = true;
        self.break_force = force;
        self.break_torque = torque;
        self
    }
}