//! 3D spatial audio and positioning

use glam::Vec3;
use serde::{Serialize, Deserialize};

/// 3D spatial audio trait
pub trait SpatialAudio: Send + Sync {
    /// Set listener position and orientation
    fn set_listener(&mut self, listener: AudioListener);
    
    /// Add audio emitter
    fn add_emitter(&mut self, emitter: AudioEmitter) -> u32;
    
    /// Update emitter position
    fn update_emitter(&mut self, id: u32, emitter: AudioEmitter);
    
    /// Remove audio emitter
    fn remove_emitter(&mut self, id: u32);
    
    /// Calculate 3D audio parameters for a sound
    fn calculate_3d_params(&self, emitter_id: u32) -> SpatialParams;
}

/// Audio listener (camera/player)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub velocity: Vec3,
}

/// Audio emitter (sound source in 3D space)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEmitter {
    pub position: Vec3,
    pub velocity: Vec3,
    pub attenuation: AttenuationModel,
    pub directional: Option<DirectionalAudio>,
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff_factor: f32,
}

/// Spatial audio parameters calculated for rendering
#[derive(Debug, Clone, Copy)]
pub struct SpatialParams {
    pub volume: f32,
    pub pan: f32,
    pub pitch: f32,
    pub distance: f32,
    pub angle: f32,
}

/// Audio attenuation models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttenuationModel {
    None,
    Linear,
    Logarithmic,
    Exponential,
    Custom,
}

/// Directional audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalAudio {
    pub direction: Vec3,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub outer_gain: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            velocity: Vec3::ZERO,
        }
    }
}

impl Default for AudioEmitter {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            attenuation: AttenuationModel::Linear,
            directional: None,
            min_distance: 1.0,
            max_distance: 100.0,
            rolloff_factor: 1.0,
        }
    }
}

impl AudioEmitter {
    /// Create omnidirectional emitter
    pub fn omnidirectional(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    
    /// Create directional emitter
    pub fn directional(position: Vec3, direction: Vec3, inner_angle: f32, outer_angle: f32) -> Self {
        Self {
            position,
            directional: Some(DirectionalAudio {
                direction: direction.normalize(),
                inner_angle,
                outer_angle,
                outer_gain: 0.5,
            }),
            ..Default::default()
        }
    }
    
    /// Calculate distance-based attenuation
    pub fn calculate_attenuation(&self, distance: f32) -> f32 {
        if distance <= self.min_distance {
            return 1.0;
        }
        
        if distance >= self.max_distance {
            return 0.0;
        }
        
        let normalized_distance = (distance - self.min_distance) / (self.max_distance - self.min_distance);
        
        match self.attenuation {
            AttenuationModel::None => 1.0,
            AttenuationModel::Linear => 1.0 - normalized_distance,
            AttenuationModel::Logarithmic => {
                1.0 / (1.0 + self.rolloff_factor * distance)
            }
            AttenuationModel::Exponential => {
                (-self.rolloff_factor * normalized_distance).exp()
            }
            AttenuationModel::Custom => {
                // Custom curve - could be configurable
                (1.0 - normalized_distance * normalized_distance).max(0.0)
            }
        }
    }
    
    /// Calculate directional gain
    pub fn calculate_directional_gain(&self, listener_position: Vec3) -> f32 {
        if let Some(ref directional) = self.directional {
            let to_listener = (listener_position - self.position).normalize();
            let dot = directional.direction.dot(to_listener);
            let angle = dot.acos();
            
            if angle <= directional.inner_angle {
                1.0
            } else if angle >= directional.outer_angle {
                directional.outer_gain
            } else {
                let t = (angle - directional.inner_angle) / (directional.outer_angle - directional.inner_angle);
                1.0 + (directional.outer_gain - 1.0) * t
            }
        } else {
            1.0
        }
    }
}

impl SpatialParams {
    /// Create new spatial parameters
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            distance: 0.0,
            angle: 0.0,
        }
    }
    
    /// Calculate parameters from listener and emitter
    pub fn from_positions(listener: &AudioListener, emitter: &AudioEmitter) -> Self {
        let to_emitter = emitter.position - listener.position;
        let distance = to_emitter.length();
        
        // Calculate volume from distance attenuation
        let volume = emitter.calculate_attenuation(distance) * emitter.calculate_directional_gain(listener.position);
        
        // Calculate pan from left-right position
        let right = listener.forward.cross(listener.up).normalize();
        let pan = to_emitter.normalize().dot(right).clamp(-1.0, 1.0);
        
        // Calculate doppler effect
        let relative_velocity = emitter.velocity - listener.velocity;
        let doppler_factor = 1.0 + relative_velocity.dot(to_emitter.normalize()) / 343.0; // Speed of sound
        let pitch = 1.0 / doppler_factor.clamp(0.5, 2.0);
        
        // Calculate angle for HRTF
        let angle = listener.forward.dot(to_emitter.normalize()).acos();
        
        Self {
            volume,
            pan,
            pitch,
            distance,
            angle,
        }
    }
}

impl Default for SpatialParams {
    fn default() -> Self {
        Self::new()
    }
}