//! Physics constraint abstractions

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Distance constraint component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistanceConstraint {
    /// Target entity to constrain to
    pub target_entity: Option<u32>,
    /// Target distance to maintain
    pub distance: f32,
    /// Constraint stiffness (0.0 = soft, 1.0 = rigid)
    pub stiffness: f32,
    /// Damping factor
    pub damping: f32,
    /// Whether constraint is enabled
    pub enabled: bool,
}

/// Position constraint component  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionConstraint {
    /// Target position to constrain to
    pub target_position: Vec3,
    /// Constraint stiffness
    pub stiffness: f32,
    /// Damping factor
    pub damping: f32,
    /// Whether constraint is enabled
    pub enabled: bool,
}

/// Look-at constraint component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookAtConstraint {
    /// Target entity to look at
    pub target_entity: Option<u32>,
    /// Target position to look at
    pub target_position: Option<Vec3>,
    /// Up vector for orientation
    pub up_vector: Vec3,
    /// Constraint strength
    pub strength: f32,
    /// Whether constraint is enabled
    pub enabled: bool,
}
