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

pub mod effects;
pub mod manager;
pub mod mixer;
pub mod source;
pub mod spatial;
pub mod streaming;
pub mod system;

// Re-export main types
pub use effects::*;
pub use manager::*;
pub use mixer::*;
pub use source::*;
pub use spatial::*;
pub use streaming::*;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_error_display() {
        let errors = [
            AudioError::InitializationFailed("backend failed".to_string()),
            AudioError::SourceNotFound("source123".to_string()),
            AudioError::InvalidFormat("invalid wav".to_string()),
            AudioError::PlaybackError("device busy".to_string()),
            AudioError::StreamingError("buffer underrun".to_string()),
            AudioError::DeviceError("no audio device".to_string()),
            AudioError::ProcessingError("effect failed".to_string()),
        ];

        for error in &errors {
            // Ensure all errors implement Display properly
            let display_string = format!("{}", error);
            assert!(!display_string.is_empty());

            // Ensure all errors implement Debug properly
            let debug_string = format!("{:?}", error);
            assert!(!debug_string.is_empty());
        }
    }

    #[test]
    fn test_audio_result_type() {
        // Test that our Result type alias works
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(AudioError::SourceNotFound("test".to_string()));

        match success {
            Ok(value) => assert_eq!(value, 42),
            Err(_) => panic!("Should be success"),
        }

        match failure {
            Ok(_) => panic!("Should be error"),
            Err(error) => match error {
                AudioError::SourceNotFound(msg) => assert_eq!(msg, "test"),
                _ => panic!("Wrong error type"),
            },
        }
    }

    #[test]
    fn test_audio_error_variants() {
        // Test specific error message formatting
        let init_error = AudioError::InitializationFailed("audio backend failed".to_string());
        assert!(init_error
            .to_string()
            .contains("Audio initialization failed"));
        assert!(init_error.to_string().contains("audio backend failed"));

        let source_error = AudioError::SourceNotFound("clip_42".to_string());
        assert!(source_error.to_string().contains("Audio source not found"));
        assert!(source_error.to_string().contains("clip_42"));

        let format_error = AudioError::InvalidFormat("corrupt mp3".to_string());
        assert!(format_error.to_string().contains("Invalid audio format"));
        assert!(format_error.to_string().contains("corrupt mp3"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let audio_error: AudioError = io_error.into();

        match audio_error {
            AudioError::IoError(_) => {} // Success
            _ => panic!("Should convert to IoError variant"),
        }
    }
}
