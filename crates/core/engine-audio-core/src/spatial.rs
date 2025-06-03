//! Spatial audio abstractions

use glam::Vec3;
use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;

/// Spatial audio listener component (typically on camera)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioListener {
    /// Whether this listener is active
    pub active: bool,
    
    /// Listener velocity for doppler effect
    pub velocity: Vec3,
    
    /// Global volume multiplier
    pub volume: f32,
    
    /// Doppler scale factor
    pub doppler_scale: f32,
    
    /// Maximum distance for audio (beyond this, audio is silent)
    pub max_distance: f32,
    
    /// Reference distance for volume calculations
    pub reference_distance: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            active: true,
            velocity: Vec3::ZERO,
            volume: 1.0,
            doppler_scale: 1.0,
            max_distance: 1000.0,
            reference_distance: 1.0,
        }
    }
}

impl Component for AudioListener {}


/// Spatial audio source component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialAudioSource {
    /// Whether spatial audio is enabled
    pub enabled: bool,
    
    /// Source velocity for doppler effect
    pub velocity: Vec3,
    
    /// Minimum distance (audio doesn't get louder when closer than this)
    pub min_distance: f32,
    
    /// Maximum distance (audio is silent beyond this)
    pub max_distance: f32,
    
    /// Audio rolloff model
    pub rolloff: AudioRolloff,
    
    /// Directional audio settings
    pub directional: Option<DirectionalAudio>,
    
    /// Spatial blending (0.0 = 2D, 1.0 = full 3D)
    pub spatial_blend: f32,
    
    /// Spread angle for stereo positioning (0-360 degrees)
    pub spread: f32,
}

impl Default for SpatialAudioSource {
    fn default() -> Self {
        Self {
            enabled: true,
            velocity: Vec3::ZERO,
            min_distance: 1.0,
            max_distance: 500.0,
            rolloff: AudioRolloff::Logarithmic,
            directional: None,
            spatial_blend: 1.0,
            spread: 0.0,
        }
    }
}

impl Component for SpatialAudioSource {}


/// Audio rolloff models for distance attenuation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AudioRolloff {
    /// Linear falloff (volume decreases linearly with distance)
    Linear,
    /// Logarithmic falloff (more realistic, volume decreases logarithmically)
    Logarithmic,
    /// Custom curve (implementation-defined)
    Custom(f32),
    /// No distance attenuation
    None,
}

/// Directional audio configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectionalAudio {
    /// Forward direction (in local space)
    pub direction: Vec3,
    
    /// Inner cone angle in degrees (full volume)
    pub inner_angle: f32,
    
    /// Outer cone angle in degrees (reduced volume)
    pub outer_angle: f32,
    
    /// Volume multiplier outside the outer cone
    pub outer_gain: f32,
}

impl Default for DirectionalAudio {
    fn default() -> Self {
        Self {
            direction: Vec3::NEG_Z, // Forward
            inner_angle: 45.0,
            outer_angle: 90.0,
            outer_gain: 0.1,
        }
    }
}

/// Audio occlusion/obstruction settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioOcclusion {
    /// Whether occlusion is enabled
    pub enabled: bool,
    
    /// Low-frequency occlusion factor (0.0 = no occlusion, 1.0 = full occlusion)
    pub low_frequency_occlusion: f32,
    
    /// High-frequency occlusion factor
    pub high_frequency_occlusion: f32,
    
    /// Room effect level when occluded
    pub room_effect: f32,
    
    /// Direct path blocked factor
    pub direct_occlusion: f32,
}

impl Default for AudioOcclusion {
    fn default() -> Self {
        Self {
            enabled: false,
            low_frequency_occlusion: 0.0,
            high_frequency_occlusion: 0.0,
            room_effect: 0.0,
            direct_occlusion: 0.0,
        }
    }
}

impl SpatialAudioSource {
    /// Create new spatial audio source
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set distance range
    pub fn with_distance_range(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min.max(0.1);
        self.max_distance = max.max(self.min_distance);
        self
    }
    
    /// Set rolloff model
    pub fn with_rolloff(mut self, rolloff: AudioRolloff) -> Self {
        self.rolloff = rolloff;
        self
    }
    
    /// Make this a directional audio source
    pub fn with_directional(mut self, inner_angle: f32, outer_angle: f32) -> Self {
        self.directional = Some(DirectionalAudio {
            inner_angle: inner_angle.clamp(0.0, 360.0),
            outer_angle: outer_angle.clamp(inner_angle, 360.0),
            ..Default::default()
        });
        self
    }
    
    /// Set spatial blend (0.0 = 2D, 1.0 = 3D)
    pub fn with_spatial_blend(mut self, blend: f32) -> Self {
        self.spatial_blend = blend.clamp(0.0, 1.0);
        self
    }
    
    /// Calculate distance attenuation based on rolloff model
    pub fn calculate_distance_attenuation(&self, distance: f32) -> f32 {
        if distance <= self.min_distance {
            return 1.0;
        }
        
        if distance >= self.max_distance {
            return 0.0;
        }
        
        match self.rolloff {
            AudioRolloff::Linear => {
                let normalized = (distance - self.min_distance) / (self.max_distance - self.min_distance);
                1.0 - normalized
            }
            AudioRolloff::Logarithmic => {
                self.min_distance / distance
            }
            AudioRolloff::Custom(factor) => {
                let normalized = (distance - self.min_distance) / (self.max_distance - self.min_distance);
                (1.0 - normalized).powf(factor)
            }
            AudioRolloff::None => 1.0,
        }
    }
    
    /// Calculate directional attenuation
    pub fn calculate_directional_attenuation(&self, direction_to_listener: Vec3) -> f32 {
        let Some(ref directional) = self.directional else {
            return 1.0;
        };
        
        let angle = direction_to_listener.dot(directional.direction).acos().to_degrees();
        
        if angle <= directional.inner_angle / 2.0 {
            1.0
        } else if angle <= directional.outer_angle / 2.0 {
            let t = (angle - directional.inner_angle / 2.0) / 
                   (directional.outer_angle / 2.0 - directional.inner_angle / 2.0);
            1.0 + t * (directional.outer_gain - 1.0)
        } else {
            directional.outer_gain
        }
    }
}