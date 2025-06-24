//! Camera utilities for mobile-first game engine
//!
//! This crate provides camera matrix calculations, culling utilities,
//! and viewport management for the camera systems.

pub mod culling;
pub mod matrices;
pub mod viewport;

// Core exports
pub use culling::*;
pub use viewport::{Viewport, ViewportTransform};

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
