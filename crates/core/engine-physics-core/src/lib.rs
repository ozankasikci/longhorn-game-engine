//! Core physics abstractions for the mobile game engine
//!
//! This crate provides the fundamental physics types and traits that are implemented
//! by specific physics backends (like Rapier, Box2D, etc.) in the implementation tier.
//!
//! Key abstractions:
//! - Rigid bodies and colliders
//! - Physics materials and properties
//! - Force and constraint systems
//! - Query and raycasting interfaces
//! - Physics world management
//! - Joint and constraint definitions

pub mod bodies;
pub mod colliders;
pub mod constraints;
pub mod events;
pub mod forces;
pub mod joints;
pub mod materials;
pub mod queries;
pub mod world;

// Re-export main types
pub use bodies::*;
pub use colliders::*;
pub use constraints::*;
pub use events::*;
pub use forces::*;
pub use joints::*;
pub use materials::*;
pub use queries::*;
pub use world::*;

use thiserror::Error;

/// Core physics system result type
pub type Result<T> = std::result::Result<T, PhysicsError>;

/// Core physics system errors
#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("Physics world initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Rigid body not found: {0}")]
    BodyNotFound(String),

    #[error("Collider not found: {0}")]
    ColliderNotFound(String),

    #[error("Joint not found: {0}")]
    JointNotFound(String),

    #[error("Invalid physics configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Physics constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Simulation step failed: {0}")]
    SimulationFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Physics feature not supported: {0}")]
    NotSupported(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_error_display() {
        let errors = [
            PhysicsError::InitializationFailed("test".to_string()),
            PhysicsError::BodyNotFound("body123".to_string()),
            PhysicsError::ColliderNotFound("collider456".to_string()),
            PhysicsError::JointNotFound("joint789".to_string()),
            PhysicsError::InvalidConfiguration("invalid config".to_string()),
            PhysicsError::ConstraintViolation("constraint failed".to_string()),
            PhysicsError::SimulationFailed("simulation error".to_string()),
            PhysicsError::QueryFailed("query error".to_string()),
            PhysicsError::NotSupported("feature xyz".to_string()),
        ];

        for error in &errors {
            // Ensure all errors implement Display properly
            let display_string = format!("{}", error);
            assert!(!display_string.is_empty());

            // Ensure all errors implement Debug properly
            let debug_string = format!("{:?}", error);
            assert!(!debug_string.is_empty());
        }
    }

    #[test]
    fn test_physics_result_type() {
        // Test that our Result type alias works
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(PhysicsError::NotSupported("test".to_string()));

        match success {
            Ok(value) => assert_eq!(value, 42),
            Err(_) => panic!("Should be success"),
        }

        match failure {
            Ok(_) => panic!("Should be error"),
            Err(error) => match error {
                PhysicsError::NotSupported(msg) => assert_eq!(msg, "test"),
                _ => panic!("Wrong error type"),
            },
        }
    }

    #[test]
    fn test_physics_error_variants() {
        // Test specific error message formatting
        let init_error = PhysicsError::InitializationFailed("physics backend failed".to_string());
        assert!(init_error
            .to_string()
            .contains("Physics world initialization failed"));
        assert!(init_error.to_string().contains("physics backend failed"));

        let body_error = PhysicsError::BodyNotFound("entity_42".to_string());
        assert!(body_error.to_string().contains("Rigid body not found"));
        assert!(body_error.to_string().contains("entity_42"));

        let collider_error = PhysicsError::ColliderNotFound("collider_123".to_string());
        assert!(collider_error.to_string().contains("Collider not found"));
        assert!(collider_error.to_string().contains("collider_123"));
    }
}
