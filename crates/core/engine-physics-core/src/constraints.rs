//! Joint and constraint abstractions

use glam::Vec3;
use serde::{Serialize, Deserialize};

/// Handle for joint resources
pub type JointHandle = u64;

/// Joint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JointType {
    Fixed,
    Ball,
    Revolute { axis: Vec3, limits: Option<(f32, f32)> },
    Prismatic { axis: Vec3, limits: Option<(f32, f32)> },
}

/// Joint representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub joint_type: JointType,
    pub body1: crate::bodies::BodyHandle,
    pub body2: crate::bodies::BodyHandle,
    pub anchor1: Vec3,
    pub anchor2: Vec3,
    pub break_force: Option<f32>,
    pub break_torque: Option<f32>,
}

/// Builder for creating joints
pub struct JointBuilder {
    joint: Joint,
}

impl JointBuilder {
    pub fn new(joint_type: JointType, body1: crate::bodies::BodyHandle, body2: crate::bodies::BodyHandle) -> Self {
        Self {
            joint: Joint {
                joint_type,
                body1,
                body2,
                anchor1: Vec3::ZERO,
                anchor2: Vec3::ZERO,
                break_force: None,
                break_torque: None,
            },
        }
    }
    
    pub fn build(self) -> Joint {
        self.joint
    }
}