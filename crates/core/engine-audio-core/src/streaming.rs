//! Audio streaming abstractions

use serde::{Serialize, Deserialize};
use engine_ecs_core::{Component, ComponentV2};
use crate::AudioHandle;

/// Streaming audio source component for large audio files
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingAudioSource {
    /// Handle to the streaming audio asset
    pub stream: AudioHandle,
    
    /// Volume (0.0 to 1.0+)
    pub volume: f32,
    
    /// Whether the stream should loop
    pub looping: bool,
    
    /// Current streaming state
    pub state: StreamingState,
    
    /// Buffer size in samples
    pub buffer_size: usize,
    
    /// Number of buffers to use
    pub buffer_count: u32,
    
    /// Preload amount in seconds
    pub preload_time: f32,
    
    /// Whether to start playing immediately when loaded
    pub auto_play: bool,
    
    /// Fade in/out settings
    pub fade: Option<StreamingFade>,
}

impl Default for StreamingAudioSource {
    fn default() -> Self {
        Self {
            stream: 0,
            volume: 1.0,
            looping: false,
            state: StreamingState::Unloaded,
            buffer_size: 4096,
            buffer_count: 3,
            preload_time: 1.0,
            auto_play: true,
            fade: None,
        }
    }
}

impl Component for StreamingAudioSource {}
impl ComponentV2 for StreamingAudioSource {}

/// Streaming audio states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingState {
    /// Stream not loaded
    Unloaded,
    /// Stream is loading
    Loading,
    /// Stream loaded and ready
    Ready,
    /// Stream is buffering data
    Buffering,
    /// Stream is playing
    Playing,
    /// Stream is paused
    Paused,
    /// Stream finished (non-looping)
    Finished,
    /// Stream encountered an error
    Error,
}

/// Fade configuration for streaming audio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingFade {
    /// Fade in duration in seconds
    pub fade_in_time: f32,
    
    /// Fade out duration in seconds
    pub fade_out_time: f32,
    
    /// Fade curve type
    pub curve: FadeCurve,
    
    /// Current fade state
    pub current_state: FadeState,
}

/// Fade curve types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeCurve {
    /// Linear fade
    Linear,
    /// Exponential fade (sounds more natural)
    Exponential,
    /// S-curve fade (smooth start and end)
    SCurve,
    /// Logarithmic fade
    Logarithmic,
}

/// Current fade state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeState {
    /// Not fading
    None,
    /// Fading in
    FadingIn,
    /// Fading out
    FadingOut,
    /// Crossfading to another stream
    Crossfading,
}

/// Streaming audio configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum number of concurrent streams
    pub max_concurrent_streams: u32,
    
    /// Memory budget for streaming buffers in bytes
    pub memory_budget: usize,
    
    /// Default buffer size in samples
    pub default_buffer_size: usize,
    
    /// Default number of buffers per stream
    pub default_buffer_count: u32,
    
    /// Minimum buffer size to maintain
    pub min_buffer_threshold: f32,
    
    /// Thread priority for streaming thread
    pub thread_priority: StreamingPriority,
    
    /// Whether to use compression for streaming
    pub use_compression: bool,
    
    /// Compression quality (if enabled)
    pub compression_quality: f32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 8,
            memory_budget: 16 * 1024 * 1024, // 16MB
            default_buffer_size: 4096,
            default_buffer_count: 3,
            min_buffer_threshold: 0.5,
            thread_priority: StreamingPriority::Normal,
            use_compression: false,
            compression_quality: 0.7,
        }
    }
}

/// Streaming thread priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingPriority {
    /// Low priority (background streaming)
    Low,
    /// Normal priority
    Normal,
    /// High priority (critical audio)
    High,
    /// Real-time priority (use with caution)
    RealTime,
}

/// Streaming audio metrics for monitoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamingMetrics {
    /// Number of currently active streams
    pub active_streams: u32,
    
    /// Total memory used by streaming buffers
    pub memory_used: usize,
    
    /// Number of buffer underruns (audio glitches)
    pub underrun_count: u32,
    
    /// Average buffer fill percentage
    pub average_buffer_fill: f32,
    
    /// Network bandwidth used (if streaming from network)
    pub network_bandwidth: f32,
    
    /// Disk I/O rate in bytes per second
    pub disk_io_rate: f32,
}

impl StreamingAudioSource {
    /// Create new streaming audio source
    pub fn new(stream: AudioHandle) -> Self {
        Self {
            stream,
            ..Default::default()
        }
    }
    
    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }
    
    /// Enable looping
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
    
    /// Set buffer configuration
    pub fn with_buffering(mut self, buffer_size: usize, buffer_count: u32) -> Self {
        self.buffer_size = buffer_size.max(1024);
        self.buffer_count = buffer_count.max(2);
        self
    }
    
    /// Set fade configuration
    pub fn with_fade(mut self, fade_in: f32, fade_out: f32) -> Self {
        self.fade = Some(StreamingFade {
            fade_in_time: fade_in.max(0.0),
            fade_out_time: fade_out.max(0.0),
            curve: FadeCurve::Exponential,
            current_state: FadeState::None,
        });
        self
    }
    
    /// Check if stream is ready to play
    pub fn is_ready(&self) -> bool {
        matches!(self.state, StreamingState::Ready | StreamingState::Playing | StreamingState::Paused)
    }
    
    /// Check if stream is currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.state, StreamingState::Playing)
    }
    
    /// Check if stream needs buffering
    pub fn needs_buffering(&self) -> bool {
        matches!(self.state, StreamingState::Buffering)
    }
    
    /// Check if stream has an error
    pub fn has_error(&self) -> bool {
        matches!(self.state, StreamingState::Error)
    }
    
    /// Calculate memory usage for this stream
    pub fn memory_usage(&self) -> usize {
        // Estimate based on buffer configuration
        // Assuming 16-bit stereo audio (4 bytes per sample)
        self.buffer_size * self.buffer_count as usize * 4
    }
}