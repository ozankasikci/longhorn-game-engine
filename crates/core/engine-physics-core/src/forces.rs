//! Force application abstractions

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Force application modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceMode {
    /// Apply force over time (F = ma)
    Force,
    /// Apply instantaneous impulse (change in momentum)
    Impulse,
    /// Apply acceleration directly (ignores mass)
    Acceleration,
    /// Apply velocity change directly (ignores mass)
    VelocityChange,
}

/// Force application point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceApplication {
    /// Apply at center of mass
    CenterOfMass,
    /// Apply at specific local point
    LocalPoint(Vec3),
    /// Apply at specific world point
    WorldPoint(Vec3),
}

/// Force descriptor for applying forces to rigid bodies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Force {
    /// Force vector
    pub force: Vec3,
    /// How to apply the force
    pub mode: ForceMode,
    /// Where to apply the force
    pub application: ForceApplication,
    /// Duration (for continuous forces)
    pub duration: Option<f32>,
}

/// Torque descriptor for applying rotational forces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Torque {
    /// Torque vector (axis * magnitude)
    pub torque: Vec3,
    /// How to apply the torque
    pub mode: ForceMode,
    /// Duration (for continuous torques)
    pub duration: Option<f32>,
}

impl Force {
    /// Create a force applied at center of mass
    pub fn at_center_of_mass(force: Vec3) -> Self {
        Self {
            force,
            mode: ForceMode::Force,
            application: ForceApplication::CenterOfMass,
            duration: None,
        }
    }

    /// Create an impulse applied at center of mass
    pub fn impulse_at_center_of_mass(impulse: Vec3) -> Self {
        Self {
            force: impulse,
            mode: ForceMode::Impulse,
            application: ForceApplication::CenterOfMass,
            duration: None,
        }
    }

    /// Create a force applied at a world point
    pub fn at_world_point(force: Vec3, point: Vec3) -> Self {
        Self {
            force,
            mode: ForceMode::Force,
            application: ForceApplication::WorldPoint(point),
            duration: None,
        }
    }

    /// Set duration for continuous force
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set force mode
    pub fn with_mode(mut self, mode: ForceMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Torque {
    /// Create a torque
    pub fn new(torque: Vec3) -> Self {
        Self {
            torque,
            mode: ForceMode::Force,
            duration: None,
        }
    }

    /// Create a torque impulse
    pub fn impulse(impulse: Vec3) -> Self {
        Self {
            torque: impulse,
            mode: ForceMode::Impulse,
            duration: None,
        }
    }
}
