//! Physics material abstractions

use serde::{Serialize, Deserialize};

/// Physics material properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    /// Coefficient of restitution (bounciness) 0.0 = no bounce, 1.0 = perfect bounce
    pub restitution: f32,
    
    /// Coefficient of friction (0.0 = frictionless, 1.0+ = high friction)
    pub friction: f32,
    
    /// Rolling friction coefficient (for rounded objects)
    pub rolling_friction: f32,
    
    /// Density in kg/m³ (for mass calculation)
    pub density: f32,
    
    /// Combine mode for restitution
    pub restitution_combine: CombineMode,
    
    /// Combine mode for friction
    pub friction_combine: CombineMode,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            restitution: 0.0,
            friction: 0.5,
            rolling_friction: 0.0,
            density: 1000.0,
            restitution_combine: CombineMode::Average,
            friction_combine: CombineMode::Average,
        }
    }
}

/// How to combine material properties between two colliders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombineMode {
    /// Take the average of both values
    Average,
    /// Take the minimum value
    Minimum,
    /// Take the maximum value
    Maximum,
    /// Multiply the values
    Multiply,
}

/// Predefined physics materials
impl PhysicsMaterial {
    /// Ice material (very slippery, some bounce)
    pub fn ice() -> Self {
        Self {
            restitution: 0.1,
            friction: 0.02,
            rolling_friction: 0.001,
            density: 917.0,
            ..Default::default()
        }
    }
    
    /// Rubber material (high bounce, good friction)
    pub fn rubber() -> Self {
        Self {
            restitution: 0.8,
            friction: 1.0,
            rolling_friction: 0.1,
            density: 1522.0,
            ..Default::default()
        }
    }
    
    /// Metal material (no bounce, medium friction)
    pub fn metal() -> Self {
        Self {
            restitution: 0.05,
            friction: 0.4,
            rolling_friction: 0.01,
            density: 7850.0,
            ..Default::default()
        }
    }
    
    /// Wood material (little bounce, good friction)
    pub fn wood() -> Self {
        Self {
            restitution: 0.2,
            friction: 0.7,
            rolling_friction: 0.05,
            density: 600.0,
            ..Default::default()
        }
    }
    
    /// Stone material (no bounce, high friction)
    pub fn stone() -> Self {
        Self {
            restitution: 0.0,
            friction: 0.9,
            rolling_friction: 0.1,
            density: 2500.0,
            ..Default::default()
        }
    }
}