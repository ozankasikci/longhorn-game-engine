//! Audio system for the mobile game engine
//! 
//! This crate provides audio playback, sound effects, and music management
//! for the game engine using rodio.

pub mod manager;
pub mod source;
pub mod mixer;
pub mod effects;
pub mod streaming;

pub use manager::AudioManager;
pub use source::{AudioSource, SoundEffect, MusicTrack};
pub use mixer::AudioMixer;

/// Audio system errors
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Failed to initialize audio device")]
    DeviceInitialization,
    #[error("Failed to load audio file: {0}")]
    LoadError(String),
    #[error("Audio playback error: {0}")]
    PlaybackError(String),
}

/// Audio system result type
pub type AudioResult<T> = Result<T, AudioError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_creation() {
        // Placeholder test
        assert!(true);
    }
}