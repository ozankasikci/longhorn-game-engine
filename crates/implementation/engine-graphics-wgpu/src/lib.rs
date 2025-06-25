#![warn(missing_docs)]

//! # WGPU Graphics Implementation
//! 
//! This crate provides a WGPU-based implementation of the graphics traits
//! defined in engine-graphics-traits.

/// Bind group implementation for WGPU
pub mod bind_group;
/// Buffer implementation for WGPU
pub mod buffer;
/// Device implementation for WGPU
pub mod device;
/// Error types and conversions
pub mod error;
/// Texture implementation for WGPU
pub mod texture;

pub use bind_group::*;
pub use buffer::*;
pub use device::*;
pub use error::*;
pub use texture::*;