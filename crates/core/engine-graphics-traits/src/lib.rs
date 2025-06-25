#![warn(missing_docs)]

//! # Graphics Traits
//!
//! This crate provides abstract graphics API traits that allow the engine
//! to work with different graphics backends (WGPU, OpenGL, Vulkan, etc.)
//! without tight coupling to any specific implementation.

/// Bind group and layout traits
pub mod bind_group;
/// Buffer-related traits and types
pub mod buffer;
/// Command encoding traits
pub mod command;
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

pub use bind_group::*;
pub use buffer::*;
pub use command::*;
pub use device::*;
pub use error::*;
pub use pipeline::*;
pub use texture::*;
pub use types::*;
