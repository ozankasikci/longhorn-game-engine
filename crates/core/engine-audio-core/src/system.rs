//! Core audio system traits and configuration

use crate::{SoundHandle, AudioError, Result};
use glam::Vec3;
use serde::{Serialize, Deserialize};

/// Core audio system trait that all audio implementations must implement
pub trait AudioSystem: Send + Sync {
    /// Initialize the audio system
    fn initialize(&mut self, config: &AudioConfig) -> Result<()>;
    
    /// Shutdown the audio system
    fn shutdown(&mut self) -> Result<()>;
    
    /// Load an audio buffer from raw data
    fn load_buffer(&mut self, data: &crate::source::AudioBuffer) -> Result<SoundHandle>;
    
    /// Unload an audio buffer
    fn unload_buffer(&mut self, handle: SoundHandle) -> Result<()>;
    
    /// Play a sound and return an instance handle
    fn play_sound(&mut self, source: &crate::source::AudioSource) -> Result<u64>;
    
    /// Stop a playing sound instance
    fn stop_sound(&mut self, instance: u64) -> Result<()>;
    
    /// Pause a playing sound instance
    fn pause_sound(&mut self, instance: u64) -> Result<()>;
    
    /// Resume a paused sound instance
    fn resume_sound(&mut self, instance: u64) -> Result<()>;
    
    /// Set master volume (0.0 to 1.0)
    fn set_master_volume(&mut self, volume: f32) -> Result<()>;
    
    /// Get master volume
    fn master_volume(&self) -> f32;
    
    /// Set listener position and orientation for 3D audio
    fn set_listener(&mut self, position: Vec3, forward: Vec3, up: Vec3) -> Result<()>;
    
    /// Update the audio system (call once per frame)
    fn update(&mut self, delta_time: f32) -> Result<()>;
    
    /// Get number of active sound instances
    fn active_sounds(&self) -> usize;
    
    /// Get audio system capabilities
    fn capabilities(&self) -> AudioCapabilities;
}

/// Audio system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub max_sounds: u32,
    pub output_channels: u32,
    pub enable_3d_audio: bool,
    pub enable_effects: bool,
    pub device_name: Option<String>,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub max_input_channels: u32,
    pub max_output_channels: u32,
    pub supported_sample_rates: Vec<u32>,
    pub default_sample_rate: u32,
}

/// Audio system capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCapabilities {
    pub max_simultaneous_sounds: u32,
    pub supports_3d_audio: bool,
    pub supports_streaming: bool,
    pub supports_effects: bool,
    pub supports_compression: Vec<AudioCompressionFormat>,
    pub min_latency_ms: f32,
    pub max_latency_ms: f32,
}

/// Audio compression formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCompressionFormat {
    None,
    Mp3,
    Ogg,
    Aac,
    Flac,
    Wav,
}

/// Audio output modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioOutputMode {
    Stereo,
    Surround5_1,
    Surround7_1,
    Headphones,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 1024,
            max_sounds: 32,
            output_channels: 2,
            enable_3d_audio: true,
            enable_effects: true,
            device_name: None,
        }
    }
}

impl AudioConfig {
    /// Create a new audio configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }
    
    /// Set buffer size (smaller = lower latency, higher CPU usage)
    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = buffer_size;
        self
    }
    
    /// Set maximum simultaneous sounds
    pub fn with_max_sounds(mut self, max_sounds: u32) -> Self {
        self.max_sounds = max_sounds;
        self
    }
    
    /// Set output channels (2 = stereo, 6 = 5.1 surround, etc.)
    pub fn with_output_channels(mut self, channels: u32) -> Self {
        self.output_channels = channels;
        self
    }
    
    /// Enable or disable 3D audio
    pub fn with_3d_audio(mut self, enable: bool) -> Self {
        self.enable_3d_audio = enable;
        self
    }
    
    /// Enable or disable audio effects
    pub fn with_effects(mut self, enable: bool) -> Self {
        self.enable_effects = enable;
        self
    }
    
    /// Set specific audio device
    pub fn with_device(mut self, device_name: &str) -> Self {
        self.device_name = Some(device_name.to_string());
        self
    }
    
    /// Create a low-latency configuration for real-time audio
    pub fn low_latency() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 256,
            max_sounds: 16,
            output_channels: 2,
            enable_3d_audio: false,
            enable_effects: false,
            device_name: None,
        }
    }
    
    /// Create a high-quality configuration for music playback
    pub fn high_quality() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 2048,
            max_sounds: 64,
            output_channels: 2,
            enable_3d_audio: true,
            enable_effects: true,
            device_name: None,
        }
    }
    
    /// Create a mobile-optimized configuration
    pub fn mobile_optimized() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 1024,
            max_sounds: 16,
            output_channels: 2,
            enable_3d_audio: true,
            enable_effects: false,
            device_name: None,
        }
    }
    
    /// Calculate estimated latency in milliseconds
    pub fn estimated_latency_ms(&self) -> f32 {
        (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }
}

impl Default for AudioCapabilities {
    fn default() -> Self {
        Self {
            max_simultaneous_sounds: 32,
            supports_3d_audio: true,
            supports_streaming: true,
            supports_effects: true,
            supports_compression: vec![
                AudioCompressionFormat::None,
                AudioCompressionFormat::Wav,
                AudioCompressionFormat::Ogg,
            ],
            min_latency_ms: 5.0,
            max_latency_ms: 100.0,
        }
    }
}

impl AudioCapabilities {
    /// Check if a compression format is supported
    pub fn supports_format(&self, format: AudioCompressionFormat) -> bool {
        self.supports_compression.contains(&format)
    }
    
    /// Check if low-latency audio is supported
    pub fn supports_low_latency(&self) -> bool {
        self.min_latency_ms < 20.0
    }
    
    /// Check if real-time audio is supported
    pub fn supports_real_time(&self) -> bool {
        self.min_latency_ms < 10.0
    }
}

/// Audio device manager trait
pub trait AudioDeviceManager: Send + Sync {
    /// Get list of available audio devices
    fn available_devices(&self) -> Result<Vec<AudioDevice>>;
    
    /// Get the default audio device
    fn default_device(&self) -> Result<AudioDevice>;
    
    /// Check if a device is available
    fn is_device_available(&self, device_name: &str) -> bool;
    
    /// Get device capabilities
    fn device_capabilities(&self, device_name: &str) -> Result<AudioCapabilities>;
}

/// Audio format converter trait
pub trait AudioFormatConverter: Send + Sync {
    /// Convert audio buffer to target format
    fn convert(&self, 
               input: &crate::source::AudioBuffer, 
               target_format: &crate::source::AudioFormat) -> Result<crate::source::AudioBuffer>;
    
    /// Resample audio to target sample rate
    fn resample(&self, 
                input: &crate::source::AudioBuffer, 
                target_sample_rate: u32) -> Result<crate::source::AudioBuffer>;
    
    /// Convert mono to stereo
    fn mono_to_stereo(&self, input: &crate::source::AudioBuffer) -> Result<crate::source::AudioBuffer>;
    
    /// Convert stereo to mono
    fn stereo_to_mono(&self, input: &crate::source::AudioBuffer) -> Result<crate::source::AudioBuffer>;
}

/// Audio codec trait for encoding/decoding
pub trait AudioCodec: Send + Sync {
    /// Get the codec format
    fn format(&self) -> AudioCompressionFormat;
    
    /// Encode audio buffer to compressed format
    fn encode(&self, input: &crate::source::AudioBuffer) -> Result<Vec<u8>>;
    
    /// Decode compressed data to audio buffer
    fn decode(&self, input: &[u8]) -> Result<crate::source::AudioBuffer>;
    
    /// Get codec-specific information
    fn info(&self) -> CodecInfo;
}

/// Codec information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    pub name: String,
    pub version: String,
    pub supports_streaming: bool,
    pub compression_ratio: f32,
    pub quality_loss: bool,
}

/// Mock audio system for testing
pub struct MockAudioSystem {
    master_volume: f32,
    active_sounds: usize,
    config: AudioConfig,
}

impl MockAudioSystem {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            active_sounds: 0,
            config: AudioConfig::default(),
        }
    }
}

impl Default for MockAudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioSystem for MockAudioSystem {
    fn initialize(&mut self, config: &AudioConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        self.active_sounds = 0;
        Ok(())
    }
    
    fn load_buffer(&mut self, _data: &crate::source::AudioBuffer) -> Result<SoundHandle> {
        Ok(1) // Mock handle
    }
    
    fn unload_buffer(&mut self, _handle: SoundHandle) -> Result<()> {
        Ok(())
    }
    
    fn play_sound(&mut self, _source: &crate::source::AudioSource) -> Result<u64> {
        self.active_sounds += 1;
        Ok(self.active_sounds as u64)
    }
    
    fn stop_sound(&mut self, _instance: u64) -> Result<()> {
        if self.active_sounds > 0 {
            self.active_sounds -= 1;
        }
        Ok(())
    }
    
    fn pause_sound(&mut self, _instance: u64) -> Result<()> {
        Ok(())
    }
    
    fn resume_sound(&mut self, _instance: u64) -> Result<()> {
        Ok(())
    }
    
    fn set_master_volume(&mut self, volume: f32) -> Result<()> {
        self.master_volume = volume.clamp(0.0, 1.0);
        Ok(())
    }
    
    fn master_volume(&self) -> f32 {
        self.master_volume
    }
    
    fn set_listener(&mut self, _position: Vec3, _forward: Vec3, _up: Vec3) -> Result<()> {
        Ok(())
    }
    
    fn update(&mut self, _delta_time: f32) -> Result<()> {
        Ok(())
    }
    
    fn active_sounds(&self) -> usize {
        self.active_sounds
    }
    
    fn capabilities(&self) -> AudioCapabilities {
        AudioCapabilities::default()
    }
}