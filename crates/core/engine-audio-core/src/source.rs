//! Audio source abstractions

use engine_ecs_core::Component;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_source_default() {
        let source = AudioSource::default();
        assert_eq!(source.clip, 0);
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 1.0);
        assert!(!source.looping);
        assert!(source.play_on_awake);
        assert_eq!(source.state, PlaybackState::Stopped);
        assert_eq!(source.priority, 128);
        assert!(source.mixer_group.is_none());
    }

    #[test]
    fn test_audio_source_creation() {
        let source = AudioSource::new(42);
        assert_eq!(source.clip, 42);
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_audio_source_builder() {
        let source = AudioSource::new(123)
            .with_volume(0.5)
            .with_pitch(1.5)
            .with_looping(true)
            .with_mixer_group("music");

        assert_eq!(source.clip, 123);
        assert_eq!(source.volume, 0.5);
        assert_eq!(source.pitch, 1.5);
        assert!(source.looping);
        assert_eq!(source.mixer_group, Some("music".to_string()));
    }

    #[test]
    fn test_volume_clamping() {
        let source1 = AudioSource::new(1).with_volume(-0.5);
        assert_eq!(source1.volume, 0.0);

        let source2 = AudioSource::new(1).with_volume(3.0);
        assert_eq!(source2.volume, 2.0);

        let source3 = AudioSource::new(1).with_volume(0.75);
        assert_eq!(source3.volume, 0.75);
    }

    #[test]
    fn test_pitch_clamping() {
        let source1 = AudioSource::new(1).with_pitch(0.05);
        assert_eq!(source1.pitch, 0.1);

        let source2 = AudioSource::new(1).with_pitch(5.0);
        assert_eq!(source2.pitch, 4.0);

        let source3 = AudioSource::new(1).with_pitch(2.0);
        assert_eq!(source3.pitch, 2.0);
    }

    #[test]
    fn test_playback_state_checks() {
        let mut source = AudioSource::default();

        // Test stopped state
        source.state = PlaybackState::Stopped;
        assert!(!source.is_playing());
        assert!(source.is_stopped());
        assert!(!source.is_paused());

        // Test playing state
        source.state = PlaybackState::Playing;
        assert!(source.is_playing());
        assert!(!source.is_stopped());
        assert!(!source.is_paused());

        // Test paused state
        source.state = PlaybackState::Paused;
        assert!(!source.is_playing());
        assert!(!source.is_stopped());
        assert!(source.is_paused());

        // Test fading in state
        source.state = PlaybackState::FadingIn;
        assert!(source.is_playing());
        assert!(!source.is_stopped());
        assert!(!source.is_paused());

        // Test finished state
        source.state = PlaybackState::Finished;
        assert!(!source.is_playing());
        assert!(source.is_stopped());
        assert!(!source.is_paused());
    }

    #[test]
    fn test_playback_state_enum() {
        assert_eq!(PlaybackState::Stopped, PlaybackState::Stopped);
        assert_ne!(PlaybackState::Playing, PlaybackState::Paused);
        assert_ne!(PlaybackState::FadingIn, PlaybackState::FadingOut);
    }

    #[test]
    fn test_audio_format_enum() {
        let formats = [
            AudioFormat::Pcm,
            AudioFormat::Mp3,
            AudioFormat::OggVorbis,
            AudioFormat::Wav,
            AudioFormat::Flac,
            AudioFormat::Aac,
        ];

        for format in &formats {
            // Ensure all variants can be matched
            match format {
                AudioFormat::Pcm => {}
                AudioFormat::Mp3 => {}
                AudioFormat::OggVorbis => {}
                AudioFormat::Wav => {}
                AudioFormat::Flac => {}
                AudioFormat::Aac => {}
            }
        }

        assert_eq!(AudioFormat::Mp3, AudioFormat::Mp3);
        assert_ne!(AudioFormat::Mp3, AudioFormat::Wav);
    }

    #[test]
    fn test_audio_quality_enum() {
        assert_eq!(AudioQuality::Low, AudioQuality::Low);
        assert_ne!(AudioQuality::Low, AudioQuality::High);
        assert_ne!(AudioQuality::Medium, AudioQuality::Lossless);
    }

    #[test]
    fn test_audio_clip_creation() {
        let clip = AudioClip {
            id: 456,
            name: "test_clip".to_string(),
            duration: 10.5,
            channels: 2,
            sample_rate: 44100,
            format: AudioFormat::Mp3,
            streamable: true,
            quality: AudioQuality::High,
        };

        assert_eq!(clip.id, 456);
        assert_eq!(clip.name, "test_clip");
        assert_eq!(clip.duration, 10.5);
        assert_eq!(clip.channels, 2);
        assert_eq!(clip.sample_rate, 44100);
        assert_eq!(clip.format, AudioFormat::Mp3);
        assert!(clip.streamable);
        assert_eq!(clip.quality, AudioQuality::High);
    }

    #[test]
    fn test_audio_handle() {
        let handle1: AudioHandle = 42;
        let handle2: AudioHandle = 43;
        assert_ne!(handle1, handle2);
        assert_eq!(handle1, 42);
    }
}
