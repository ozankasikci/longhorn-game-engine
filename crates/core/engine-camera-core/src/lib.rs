//! Advanced camera system for mobile-first game engine
//! 
//! This crate provides sophisticated camera management with viewport control,
//! efficient culling, and mobile-optimized rendering capabilities.

pub mod camera;
pub mod viewport;
pub mod projection;
pub mod culling;
pub mod components;

// Core exports
pub use camera::{Camera as AdvancedCamera, CameraType, CameraComponent, CameraUniform};
pub use viewport::{Viewport, ViewportTransform};
pub use projection::{ProjectionMatrix, OrthographicProjection, PerspectiveProjection};
pub use culling::{Frustum, CullingResult, CullingStats};

// ECS Component exports
pub use components::{Camera, Camera2D};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Invalid viewport dimensions: width={0}, height={1}")]
    InvalidViewport(u32, u32),
    
    #[error("Invalid projection parameters: {0}")]
    InvalidProjection(String),
    
    #[error("Matrix calculation failed: {0}")]
    MatrixCalculationFailed(String),
    
    #[error("Culling operation failed: {0}")]
    CullingFailed(String),
}

pub type Result<T> = std::result::Result<T, CameraError>;