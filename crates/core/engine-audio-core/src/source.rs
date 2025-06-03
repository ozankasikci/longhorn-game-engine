//! Audio source abstractions

use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;

/// Handle to an audio asset/resource
pub type AudioHandle = u64;

/// Audio source component for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSource {
    /// Handle to the audio clip/asset
    pub clip: AudioHandle,
    
    /// Volume (0.0 to 1.0+)
    pub volume: f32,
    
    /// Pitch multiplier (1.0 = normal pitch)
    pub pitch: f32,
    
    /// Whether the audio should loop
    pub looping: bool,
    
    /// Whether to play on start
    pub play_on_awake: bool,
    
    /// Current playback state
    pub state: PlaybackState,
    
    /// Audio priority (higher = more important)
    pub priority: u32,
    
    /// Audio output group for mixing
    pub mixer_group: Option<String>,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip: 0,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            play_on_awake: true,
            state: PlaybackState::Stopped,
            priority: 128,
            mixer_group: None,
        }
    }
}

impl Component for AudioSource {}


/// Audio playback states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// Audio is not playing
    Stopped,
    /// Audio is currently playing
    Playing,
    /// Audio is paused (can be resumed)
    Paused,
    /// Audio is fading in
    FadingIn,
    /// Audio is fading out
    FadingOut,
    /// Audio finished playing
    Finished,
}

/// Audio clip metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClip {
    /// Unique identifier
    pub id: AudioHandle,
    
    /// Human-readable name
    pub name: String,
    
    /// Duration in seconds
    pub duration: f32,
    
    /// Number of audio channels
    pub channels: u32,
    
    /// Sample rate in Hz
    pub sample_rate: u32,
    
    /// Audio format information
    pub format: AudioFormat,
    
    /// Whether this clip can be streamed (for large files)
    pub streamable: bool,
    
    /// Compression quality/bitrate info
    pub quality: AudioQuality,
}

/// Audio format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    /// Uncompressed PCM
    Pcm,
    /// MP3 compressed
    Mp3,
    /// OGG Vorbis compressed
    OggVorbis,
    /// WAV container
    Wav,
    /// FLAC lossless
    Flac,
    /// AAC compressed
    Aac,
}

/// Audio quality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioQuality {
    /// Low quality (suitable for effects)
    Low,
    /// Medium quality (suitable for most audio)
    Medium,
    /// High quality (suitable for music)
    High,
    /// Lossless quality
    Lossless,
}

impl AudioSource {
    /// Create new audio source with clip
    pub fn new(clip: AudioHandle) -> Self {
        Self {
            clip,
            ..Default::default()
        }
    }
    
    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }
    
    /// Set pitch
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 4.0);
        self
    }
    
    /// Set looping
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
    
    /// Set mixer group
    pub fn with_mixer_group(mut self, group: impl Into<String>) -> Self {
        self.mixer_group = Some(group.into());
        self
    }
    
    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing | PlaybackState::FadingIn)
    }
    
    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self.state, PlaybackState::Stopped | PlaybackState::Finished)
    }
    
    /// Check if paused
    pub fn is_paused(&self) -> bool {
        matches!(self.state, PlaybackState::Paused)
    }
}