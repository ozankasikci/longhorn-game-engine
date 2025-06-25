#![warn(missing_docs)]

//! # Graphics Traits
//! 
//! This crate provides abstract graphics API traits that allow the engine
//! to work with different graphics backends (WGPU, OpenGL, Vulkan, etc.)
//! without tight coupling to any specific implementation.

/// Buffer-related traits and types
pub mod buffer;
/// Device traits for resource creation
pub mod device;
/// Error types for graphics operations
pub mod error;
/// Pipeline and shader traits
pub mod pipeline;
/// Texture-related traits and types
pub mod texture;
/// Common types used across the graphics API
pub mod types;

pub use buffer::*;
pub use device::*;
pub use error::*;
pub use pipeline::*;
pub use texture::*;
pub use types::*;