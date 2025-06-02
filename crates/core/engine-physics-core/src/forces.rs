//! Force and impulse abstractions for physics simulation

use glam::Vec3;
use serde::{Serialize, Deserialize};

/// Force application modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForceMode {
    Force,        // Add continuous force
    Impulse,      // Add instantaneous impulse
    Acceleration, // Add acceleration (ignores mass)
    VelocityChange, // Add velocity change (ignores mass)
}

/// Force representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Force {
    pub force: Vec3,
    pub point: Option<Vec3>,
    pub mode: ForceMode,
}

/// Impulse representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Impulse {
    pub impulse: Vec3,
    pub point: Option<Vec3>,
}

impl Force {
    /// Create a force at center of mass
    pub fn at_center(force: Vec3, mode: ForceMode) -> Self {
        Self {
            force,
            point: None,
            mode,
        }
    }
    
    /// Create a force at specific point
    pub fn at_point(force: Vec3, point: Vec3, mode: ForceMode) -> Self {
        Self {
            force,
            point: Some(point),
            mode,
        }
    }
}

impl Impulse {
    /// Create an impulse at center of mass
    pub fn at_center(impulse: Vec3) -> Self {
        Self {
            impulse,
            point: None,
        }
    }
    
    /// Create an impulse at specific point
    pub fn at_point(impulse: Vec3, point: Vec3) -> Self {
        Self {
            impulse,
            point: Some(point),
        }
    }
}