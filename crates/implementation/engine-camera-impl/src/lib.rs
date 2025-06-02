//! Camera implementation layer
//! 
//! This crate provides concrete implementations for camera operations,
//! including frustum culling algorithms and optimization techniques.

pub mod culling;

// Re-export main types
pub use culling::*;