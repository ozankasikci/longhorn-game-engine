//! Geometry implementation layer
//!
//! This crate provides concrete implementations for geometry operations,
//! including complex mesh generation algorithms and primitive builders.

pub mod primitives;

// Re-export main types
pub use primitives::*;
