//! Audio manager abstractions

use std::collections::HashMap;
use crate::{AudioHandle, AudioClip, AudioMixer, StreamingConfig, StreamingMetrics, Result};

/// Audio manager trait for implementation by specific audio backends
pub trait AudioManager {
    /// Initialize the audio system
    fn initialize(&mut self, config: AudioSystemConfig) -> Result<()>;
    
    /// Shutdown the audio system
    fn shutdown(&mut self) -> Result<()>;
    
    /// Update the audio system (called every frame)
    fn update(&mut self, delta_time: f32) -> Result<()>;
    
    /// Load an audio clip
    fn load_clip(&mut self, path: &str) -> Result<AudioHandle>;
    
    /// Unload an audio clip
    fn unload_clip(&mut self, handle: AudioHandle) -> Result<()>;
    
    /// Get clip metadata
    fn get_clip_info(&self, handle: AudioHandle) -> Option<&AudioClip>;
    
    /// Play an audio clip
    fn play_clip(&mut self, handle: AudioHandle, settings: PlaybackSettings) -> Result<PlaybackHandle>;
    
    /// Stop playback
    fn stop_playback(&mut self, playback: PlaybackHandle) -> Result<()>;
    
    /// Pause playback
    fn pause_playback(&mut self, playback: PlaybackHandle) -> Result<()>;
    
    /// Resume playback
    fn resume_playback(&mut self, playback: PlaybackHandle) -> Result<()>;
    
    /// Set playback volume
    fn set_playback_volume(&mut self, playback: PlaybackHandle, volume: f32) -> Result<()>;
    
    /// Set playback pitch
    fn set_playback_pitch(&mut self, playback: PlaybackHandle, pitch: f32) -> Result<()>;
    
    /// Get current playback position in seconds
    fn get_playback_position(&self, playback: PlaybackHandle) -> Result<f32>;
    
    /// Set playback position in seconds
    fn set_playback_position(&mut self, playback: PlaybackHandle, position: f32) -> Result<()>;
    
    /// Check if playback is still active
    fn is_playback_active(&self, playback: PlaybackHandle) -> bool;
    
    /// Get mixer for advanced audio control
    fn get_mixer(&mut self) -> &mut AudioMixer;
    
    /// Set global volume
    fn set_master_volume(&mut self, volume: f32) -> Result<()>;
    
    /// Get current master volume
    fn get_master_volume(&self) -> f32;
    
    /// Mute/unmute all audio
    fn set_master_muted(&mut self, muted: bool) -> Result<()>;
    
    /// Check if audio is muted
    fn is_master_muted(&self) -> bool;
    
    /// Get streaming metrics
    fn get_streaming_metrics(&self) -> StreamingMetrics;
    
    /// Set streaming configuration
    fn set_streaming_config(&mut self, config: StreamingConfig) -> Result<()>;
    
    /// Preload audio for faster playback
    fn preload_audio(&mut self, handles: &[AudioHandle]) -> Result<()>;
    
    /// Unload preloaded audio
    fn unload_preloaded(&mut self, handles: &[AudioHandle]) -> Result<()>;
}

/// Audio system configuration
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSystemConfig {
    /// Sample rate (44100, 48000, etc.)
    pub sample_rate: u32,
    
    /// Number of output channels (1 = mono, 2 = stereo, 6 = 5.1, 8 = 7.1)
    pub channels: u32,
    
    /// Buffer size in samples (affects latency)
    pub buffer_size: u32,
    
    /// Audio device name (None = default device)
    pub device: Option<String>,
    
    /// Enable 3D audio processing
    pub enable_3d_audio: bool,
    
    /// Enable hardware acceleration if available
    pub enable_hardware_acceleration: bool,
    
    /// Maximum number of concurrent voices
    pub max_voices: u32,
    
    /// Memory budget for audio in bytes
    pub memory_budget: usize,
    
    /// Streaming configuration
    pub streaming: StreamingConfig,
    
    /// Initial mixer configuration
    pub mixer: AudioMixer,
}

impl Default for AudioSystemConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 1024,
            device: None,
            enable_3d_audio: true,
            enable_hardware_acceleration: true,
            max_voices: 64,
            memory_budget: 64 * 1024 * 1024, // 64MB
            streaming: StreamingConfig::default(),
            mixer: AudioMixer::default(),
        }
    }
}

/// Handle to an active audio playback
pub type PlaybackHandle = u32;

/// Settings for audio playback
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackSettings {
    /// Volume (0.0 to 1.0+)
    pub volume: f32,
    
    /// Pitch multiplier (0.1 to 4.0)
    pub pitch: f32,
    
    /// Whether to loop the audio
    pub looping: bool,
    
    /// Start position in seconds
    pub start_position: f32,
    
    /// Mixer group to use
    pub mixer_group: Option<String>,
    
    /// Fade in duration in seconds
    pub fade_in: Option<f32>,
    
    /// Audio priority (higher = more important)
    pub priority: u32,
    
    /// 3D position (None = 2D audio)
    pub position: Option<glam::Vec3>,
    
    /// 3D velocity for doppler effect
    pub velocity: Option<glam::Vec3>,
    
    /// Minimum distance for 3D audio
    pub min_distance: Option<f32>,
    
    /// Maximum distance for 3D audio
    pub max_distance: Option<f32>,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            start_position: 0.0,
            mixer_group: None,
            fade_in: None,
            priority: 128,
            position: None,
            velocity: None,
            min_distance: None,
            max_distance: None,
        }
    }
}

/// Audio resource registry for managing loaded audio assets
#[derive(Debug, Default)]
pub struct AudioRegistry {
    /// Loaded audio clips
    clips: HashMap<AudioHandle, AudioClip>,
    
    /// Path to handle mapping for faster lookups
    path_to_handle: HashMap<String, AudioHandle>,
    
    /// Next available handle
    next_handle: AudioHandle,
    
    /// Reference counting for shared clips
    reference_counts: HashMap<AudioHandle, u32>,
}

impl AudioRegistry {
    /// Create new audio registry
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new audio clip
    pub fn register_clip(&mut self, clip: AudioClip, path: String) -> AudioHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        
        self.clips.insert(handle, clip);
        self.path_to_handle.insert(path, handle);
        self.reference_counts.insert(handle, 1);
        
        handle
    }
    
    /// Get clip by handle
    pub fn get_clip(&self, handle: AudioHandle) -> Option<&AudioClip> {
        self.clips.get(&handle)
    }
    
    /// Get handle by path
    pub fn get_handle(&self, path: &str) -> Option<AudioHandle> {
        self.path_to_handle.get(path).copied()
    }
    
    /// Add reference to a clip
    pub fn add_reference(&mut self, handle: AudioHandle) {
        if let Some(count) = self.reference_counts.get_mut(&handle) {
            *count += 1;
        }
    }
    
    /// Remove reference from a clip
    pub fn remove_reference(&mut self, handle: AudioHandle) -> bool {
        if let Some(count) = self.reference_counts.get_mut(&handle) {
            *count -= 1;
            if *count == 0 {
                // Clean up when no more references
                self.clips.remove(&handle);
                self.reference_counts.remove(&handle);
                
                // Remove from path mapping
                self.path_to_handle.retain(|_, &mut v| v != handle);
                
                return true; // Clip was removed
            }
        }
        false
    }
    
    /// Get all loaded clip handles
    pub fn get_all_handles(&self) -> Vec<AudioHandle> {
        self.clips.keys().copied().collect()
    }
    
    /// Get memory usage estimate
    pub fn memory_usage(&self) -> usize {
        self.clips.values()
            .map(|clip| {
                // Estimate memory usage based on clip data
                let bytes_per_sample = match clip.format {
                    crate::AudioFormat::Pcm => 2, // 16-bit
                    _ => 1, // Compressed formats use less
                };
                (clip.duration * clip.sample_rate as f32) as usize 
                    * clip.channels as usize 
                    * bytes_per_sample
            })
            .sum()
    }
    
    /// Get total number of loaded clips
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }
}