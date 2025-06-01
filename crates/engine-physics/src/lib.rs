//! Physics system for the mobile game engine
//! 
//! This crate provides 2D and 3D physics simulation using Rapier,
//! including rigid bodies, colliders, and joints.

pub mod world;
pub mod bodies;
pub mod colliders;
pub mod joints;
pub mod queries;
pub mod events;

pub use world::{PhysicsWorld, PhysicsWorld2D, PhysicsWorld3D};
pub use bodies::{RigidBody, RigidBodyBuilder};
pub use colliders::{Collider, ColliderBuilder};

/// Physics system errors
#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("Failed to create physics world")]
    WorldCreation,
    #[error("Invalid physics parameters: {0}")]
    InvalidParameters(String),
    #[error("Physics simulation error: {0}")]
    SimulationError(String),
}

/// Physics system result type
pub type PhysicsResult<T> = Result<T, PhysicsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        // Placeholder test
        assert!(true);
    }
}