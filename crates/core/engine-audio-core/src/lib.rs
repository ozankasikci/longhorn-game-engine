//! Core audio abstractions for the mobile game engine
//! 
//! This crate provides the fundamental audio types and traits that are implemented
//! by specific audio backends (like Rodio, FMOD, etc.) in the implementation tier.
//!
//! Key abstractions:
//! - Audio sources and clips
//! - Audio effects and processing
//! - Spatial audio positioning
//! - Audio mixer concepts
//! - Streaming audio support

pub mod source;
pub mod effects;
pub mod spatial;
pub mod mixer;
pub mod streaming;
pub mod manager;
pub mod system;

// Re-export main types
pub use source::*;
pub use effects::*;
pub use spatial::*;
pub use mixer::*;
pub use streaming::*;
pub use manager::*;
pub use system::*;

use thiserror::Error;

/// Core audio system result type
pub type Result<T> = std::result::Result<T, AudioError>;

/// Core audio system errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Audio initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Audio source not found: {0}")]
    SourceNotFound(String),
    
    #[error("Invalid audio format: {0}")]
    InvalidFormat(String),
    
    #[error("Audio playback error: {0}")]
    PlaybackError(String),
    
    #[error("Audio streaming error: {0}")]
    StreamingError(String),
    
    #[error("Audio device error: {0}")]
    DeviceError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Audio processing error: {0}")]
    ProcessingError(String),
}