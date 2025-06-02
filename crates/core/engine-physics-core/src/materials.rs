//! Physics material abstractions for friction and restitution

use serde::{Serialize, Deserialize};

/// Handle for physics material resources
pub type MaterialHandle = u64;

/// Physics material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    pub name: String,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub friction_combine_rule: CombineRule,
    pub restitution_combine_rule: CombineRule,
}

/// Rules for combining material properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombineRule {
    Average,
    Minimum,
    Maximum,
    Multiply,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            friction_combine_rule: CombineRule::Average,
            restitution_combine_rule: CombineRule::Average,
        }
    }
}