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
