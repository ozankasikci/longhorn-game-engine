#![warn(missing_docs)]

//! # WGPU Graphics Implementation
//! 
//! This crate provides a WGPU-based implementation of the graphics traits
//! defined in engine-graphics-traits.

/// Buffer implementation for WGPU
pub mod buffer;
/// Device implementation for WGPU
pub mod device;
/// Error types and conversions
pub mod error;

pub use buffer::*;
pub use device::*;
pub use error::*;