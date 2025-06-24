//! Spatial audio abstractions

use engine_ecs_core::Component;
use glam::Vec3;
use serde::{Deserialize, Serialize};

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
                let normalized =
                    (distance - self.min_distance) / (self.max_distance - self.min_distance);
                1.0 - normalized
            }
            AudioRolloff::Logarithmic => self.min_distance / distance,
            AudioRolloff::Custom(factor) => {
                let normalized =
                    (distance - self.min_distance) / (self.max_distance - self.min_distance);
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

        let angle = direction_to_listener
            .dot(directional.direction)
            .acos()
            .to_degrees();

        if angle <= directional.inner_angle / 2.0 {
            1.0
        } else if angle <= directional.outer_angle / 2.0 {
            let t = (angle - directional.inner_angle / 2.0)
                / (directional.outer_angle / 2.0 - directional.inner_angle / 2.0);
            1.0 + t * (directional.outer_gain - 1.0)
        } else {
            directional.outer_gain
        }
    }
}

impl AudioListener {
    /// Create new audio listener
    pub fn new() -> Self {
        Self::default()
    }

    /// Set global volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.max(0.0);
        self
    }

    /// Set doppler scale
    pub fn with_doppler_scale(mut self, scale: f32) -> Self {
        self.doppler_scale = scale.max(0.0);
        self
    }

    /// Set distance range
    pub fn with_distance_range(mut self, reference: f32, max: f32) -> Self {
        self.reference_distance = reference.max(0.1);
        self.max_distance = max.max(self.reference_distance);
        self
    }

    /// Calculate doppler effect
    pub fn calculate_doppler_factor(
        &self,
        source_velocity: Vec3,
        direction_to_source: Vec3,
    ) -> f32 {
        if self.doppler_scale <= 0.0 {
            return 1.0;
        }

        let sound_speed = 343.0; // Speed of sound in m/s
        let listener_velocity_component = self.velocity.dot(direction_to_source);
        let source_velocity_component = source_velocity.dot(direction_to_source);

        let factor =
            (sound_speed + listener_velocity_component) / (sound_speed + source_velocity_component);

        (factor * self.doppler_scale).clamp(0.1, 2.0) // Clamp for stability
    }
}

impl DirectionalAudio {
    /// Create new directional audio configuration
    pub fn new(inner_angle: f32, outer_angle: f32) -> Self {
        Self {
            inner_angle: inner_angle.clamp(0.0, 360.0),
            outer_angle: outer_angle.clamp(inner_angle, 360.0),
            ..Default::default()
        }
    }

    /// Set direction
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = direction.normalize();
        self
    }

    /// Set outer gain
    pub fn with_outer_gain(mut self, gain: f32) -> Self {
        self.outer_gain = gain.clamp(0.0, 1.0);
        self
    }
}

impl AudioOcclusion {
    /// Create new occlusion configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable occlusion with settings
    pub fn enabled(mut self, low_freq: f32, high_freq: f32) -> Self {
        self.enabled = true;
        self.low_frequency_occlusion = low_freq.clamp(0.0, 1.0);
        self.high_frequency_occlusion = high_freq.clamp(0.0, 1.0);
        self
    }

    /// Set room effect
    pub fn with_room_effect(mut self, effect: f32) -> Self {
        self.room_effect = effect.clamp(0.0, 1.0);
        self
    }

    /// Apply occlusion to a volume value
    pub fn apply_occlusion(&self, base_volume: f32, frequency: f32) -> f32 {
        if !self.enabled {
            return base_volume;
        }

        // Simple frequency-based occlusion (higher frequencies are more occluded)
        let occlusion_factor = if frequency > 1000.0 {
            self.high_frequency_occlusion
        } else {
            self.low_frequency_occlusion
        };

        base_volume * (1.0 - occlusion_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_listener_default() {
        let listener = AudioListener::default();
        assert!(listener.active);
        assert_eq!(listener.velocity, Vec3::ZERO);
        assert_eq!(listener.volume, 1.0);
        assert_eq!(listener.doppler_scale, 1.0);
        assert_eq!(listener.max_distance, 1000.0);
        assert_eq!(listener.reference_distance, 1.0);
    }

    #[test]
    fn test_audio_listener_builder() {
        let listener = AudioListener::new()
            .with_volume(0.8)
            .with_doppler_scale(1.5)
            .with_distance_range(2.0, 800.0);

        assert_eq!(listener.volume, 0.8);
        assert_eq!(listener.doppler_scale, 1.5);
        assert_eq!(listener.reference_distance, 2.0);
        assert_eq!(listener.max_distance, 800.0);
    }

    #[test]
    fn test_audio_listener_volume_clamping() {
        let listener = AudioListener::new().with_volume(-0.5);
        assert_eq!(listener.volume, 0.0); // Should be clamped to 0

        let listener2 = AudioListener::new().with_volume(2.0);
        assert_eq!(listener2.volume, 2.0); // No upper limit
    }

    #[test]
    fn test_audio_listener_doppler_calculation() {
        let listener = AudioListener::new().with_doppler_scale(1.0);

        // Stationary source and listener
        let doppler = listener.calculate_doppler_factor(Vec3::ZERO, Vec3::X);
        assert!((doppler - 1.0).abs() < 0.001);

        // Source moving away from listener
        let doppler_away = listener.calculate_doppler_factor(Vec3::new(10.0, 0.0, 0.0), Vec3::X);
        assert!(doppler_away < 1.0); // Should be lower frequency

        // Source moving towards listener
        let doppler_towards =
            listener.calculate_doppler_factor(Vec3::new(-10.0, 0.0, 0.0), Vec3::X);
        assert!(doppler_towards > 1.0); // Should be higher frequency
    }

    #[test]
    fn test_spatial_audio_source_default() {
        let source = SpatialAudioSource::default();
        assert!(source.enabled);
        assert_eq!(source.velocity, Vec3::ZERO);
        assert_eq!(source.min_distance, 1.0);
        assert_eq!(source.max_distance, 500.0);
        assert_eq!(source.rolloff, AudioRolloff::Logarithmic);
        assert!(source.directional.is_none());
        assert_eq!(source.spatial_blend, 1.0);
        assert_eq!(source.spread, 0.0);
    }

    #[test]
    fn test_spatial_audio_source_builder() {
        let source = SpatialAudioSource::new()
            .with_distance_range(2.0, 100.0)
            .with_rolloff(AudioRolloff::Linear)
            .with_directional(30.0, 60.0)
            .with_spatial_blend(0.5);

        assert_eq!(source.min_distance, 2.0);
        assert_eq!(source.max_distance, 100.0);
        assert_eq!(source.rolloff, AudioRolloff::Linear);
        assert!(source.directional.is_some());
        assert_eq!(source.spatial_blend, 0.5);

        let directional = source.directional.unwrap();
        assert_eq!(directional.inner_angle, 30.0);
        assert_eq!(directional.outer_angle, 60.0);
    }

    #[test]
    fn test_distance_range_clamping() {
        let source = SpatialAudioSource::new().with_distance_range(-1.0, 5.0);
        assert_eq!(source.min_distance, 0.1); // Clamped to minimum
        assert_eq!(source.max_distance, 5.0);

        let source2 = SpatialAudioSource::new().with_distance_range(10.0, 5.0);
        assert_eq!(source2.min_distance, 10.0);
        assert_eq!(source2.max_distance, 10.0); // Max clamped to min
    }

    #[test]
    fn test_spatial_blend_clamping() {
        let source = SpatialAudioSource::new().with_spatial_blend(-0.5);
        assert_eq!(source.spatial_blend, 0.0);

        let source2 = SpatialAudioSource::new().with_spatial_blend(1.5);
        assert_eq!(source2.spatial_blend, 1.0);

        let source3 = SpatialAudioSource::new().with_spatial_blend(0.75);
        assert_eq!(source3.spatial_blend, 0.75);
    }

    #[test]
    fn test_linear_distance_attenuation() {
        let source = SpatialAudioSource::new()
            .with_distance_range(1.0, 10.0)
            .with_rolloff(AudioRolloff::Linear);

        // Within min distance
        assert_eq!(source.calculate_distance_attenuation(0.5), 1.0);
        assert_eq!(source.calculate_distance_attenuation(1.0), 1.0);

        // Beyond max distance
        assert_eq!(source.calculate_distance_attenuation(10.0), 0.0);
        assert_eq!(source.calculate_distance_attenuation(15.0), 0.0);

        // In between
        let mid_distance = 5.5; // Halfway between 1 and 10
        let expected = 1.0 - (mid_distance - 1.0) / (10.0 - 1.0);
        assert!((source.calculate_distance_attenuation(mid_distance) - expected).abs() < 0.001);
    }

    #[test]
    fn test_logarithmic_distance_attenuation() {
        let source = SpatialAudioSource::new()
            .with_distance_range(1.0, 100.0)
            .with_rolloff(AudioRolloff::Logarithmic);

        // Within min distance
        assert_eq!(source.calculate_distance_attenuation(1.0), 1.0);

        // Beyond max distance
        assert_eq!(source.calculate_distance_attenuation(100.0), 0.0);

        // Logarithmic falloff
        let attenuation_2 = source.calculate_distance_attenuation(2.0);
        let attenuation_4 = source.calculate_distance_attenuation(4.0);
        assert!((attenuation_2 - 0.5).abs() < 0.001); // 1/2
        assert!((attenuation_4 - 0.25).abs() < 0.001); // 1/4
    }

    #[test]
    fn test_custom_distance_attenuation() {
        let source = SpatialAudioSource::new()
            .with_distance_range(1.0, 5.0)
            .with_rolloff(AudioRolloff::Custom(2.0));

        let mid_distance = 3.0;
        let normalized = (mid_distance - 1.0) / (5.0 - 1.0); // 0.5
        let expected = (1.0f32 - normalized).powf(2.0); // 0.25
        assert!((source.calculate_distance_attenuation(mid_distance) - expected).abs() < 0.001);
    }

    #[test]
    fn test_no_distance_attenuation() {
        let source = SpatialAudioSource::new()
            .with_distance_range(1.0, 100.0)
            .with_rolloff(AudioRolloff::None);

        // Within range should return 1.0 with no rolloff
        assert_eq!(source.calculate_distance_attenuation(0.5), 1.0); // Below min
        assert_eq!(source.calculate_distance_attenuation(1.0), 1.0); // At min
        assert_eq!(source.calculate_distance_attenuation(50.0), 1.0); // In range

        // Beyond max distance should still return 0.0 (silence)
        assert_eq!(source.calculate_distance_attenuation(100.0), 0.0); // At max
        assert_eq!(source.calculate_distance_attenuation(200.0), 0.0); // Beyond max
    }

    #[test]
    fn test_directional_audio_default() {
        let directional = DirectionalAudio::default();
        assert_eq!(directional.direction, Vec3::NEG_Z);
        assert_eq!(directional.inner_angle, 45.0);
        assert_eq!(directional.outer_angle, 90.0);
        assert_eq!(directional.outer_gain, 0.1);
    }

    #[test]
    fn test_directional_audio_builder() {
        let directional = DirectionalAudio::new(30.0, 80.0)
            .with_direction(Vec3::Y)
            .with_outer_gain(0.2);

        assert_eq!(directional.direction, Vec3::Y);
        assert_eq!(directional.inner_angle, 30.0);
        assert_eq!(directional.outer_angle, 80.0);
        assert_eq!(directional.outer_gain, 0.2);
    }

    #[test]
    fn test_directional_angle_clamping() {
        let directional = DirectionalAudio::new(-10.0, 400.0);
        assert_eq!(directional.inner_angle, 0.0); // Clamped to 0
        assert_eq!(directional.outer_angle, 360.0); // Clamped to 360

        let directional2 = DirectionalAudio::new(100.0, 50.0);
        assert_eq!(directional2.inner_angle, 100.0);
        assert_eq!(directional2.outer_angle, 100.0); // Outer clamped to inner
    }

    #[test]
    fn test_directional_attenuation_non_directional() {
        let source = SpatialAudioSource::new(); // No directional audio
        let attenuation = source.calculate_directional_attenuation(Vec3::X);
        assert_eq!(attenuation, 1.0);
    }

    #[test]
    fn test_directional_attenuation_within_inner_cone() {
        let source = SpatialAudioSource::new().with_directional(60.0, 120.0);
        let direction = Vec3::NEG_Z; // Directly forward (0 degrees from NEG_Z)
        let attenuation = source.calculate_directional_attenuation(direction);
        assert_eq!(attenuation, 1.0);
    }

    #[test]
    fn test_directional_attenuation_outside_outer_cone() {
        let source = SpatialAudioSource::new().with_directional(30.0, 60.0);
        let direction = Vec3::X; // 90 degrees from NEG_Z
        let attenuation = source.calculate_directional_attenuation(direction);

        let directional = source.directional.unwrap();
        assert_eq!(attenuation, directional.outer_gain);
    }

    #[test]
    fn test_audio_rolloff_enum() {
        let rolloffs = [
            AudioRolloff::Linear,
            AudioRolloff::Logarithmic,
            AudioRolloff::Custom(2.0),
            AudioRolloff::None,
        ];

        for rolloff in &rolloffs {
            match rolloff {
                AudioRolloff::Linear => {}
                AudioRolloff::Logarithmic => {}
                AudioRolloff::Custom(factor) => assert_eq!(*factor, 2.0),
                AudioRolloff::None => {}
            }
        }

        assert_eq!(AudioRolloff::Linear, AudioRolloff::Linear);
        assert_ne!(AudioRolloff::Linear, AudioRolloff::Logarithmic);
    }

    #[test]
    fn test_audio_occlusion_default() {
        let occlusion = AudioOcclusion::default();
        assert!(!occlusion.enabled);
        assert_eq!(occlusion.low_frequency_occlusion, 0.0);
        assert_eq!(occlusion.high_frequency_occlusion, 0.0);
        assert_eq!(occlusion.room_effect, 0.0);
        assert_eq!(occlusion.direct_occlusion, 0.0);
    }

    #[test]
    fn test_audio_occlusion_builder() {
        let occlusion = AudioOcclusion::new()
            .enabled(0.3, 0.7)
            .with_room_effect(0.5);

        assert!(occlusion.enabled);
        assert_eq!(occlusion.low_frequency_occlusion, 0.3);
        assert_eq!(occlusion.high_frequency_occlusion, 0.7);
        assert_eq!(occlusion.room_effect, 0.5);
    }

    #[test]
    fn test_occlusion_value_clamping() {
        let occlusion = AudioOcclusion::new()
            .enabled(-0.5, 1.5)
            .with_room_effect(2.0);

        assert_eq!(occlusion.low_frequency_occlusion, 0.0); // Clamped to 0
        assert_eq!(occlusion.high_frequency_occlusion, 1.0); // Clamped to 1
        assert_eq!(occlusion.room_effect, 1.0); // Clamped to 1
    }

    #[test]
    fn test_occlusion_application_disabled() {
        let occlusion = AudioOcclusion::default(); // Disabled by default
        let volume = occlusion.apply_occlusion(0.8, 440.0);
        assert_eq!(volume, 0.8); // Should be unchanged
    }

    #[test]
    fn test_occlusion_application_low_frequency() {
        let occlusion = AudioOcclusion::new().enabled(0.5, 0.8);
        let volume = occlusion.apply_occlusion(1.0, 200.0); // Low frequency
        assert_eq!(volume, 0.5); // 1.0 * (1.0 - 0.5)
    }

    #[test]
    fn test_occlusion_application_high_frequency() {
        let occlusion = AudioOcclusion::new().enabled(0.3, 0.7);
        let volume = occlusion.apply_occlusion(1.0, 2000.0); // High frequency
        assert_eq!(volume, 0.3); // 1.0 * (1.0 - 0.7)
    }

    #[test]
    fn test_outer_gain_clamping() {
        let directional = DirectionalAudio::new(30.0, 60.0).with_outer_gain(-0.5);
        assert_eq!(directional.outer_gain, 0.0); // Clamped to 0

        let directional2 = DirectionalAudio::new(30.0, 60.0).with_outer_gain(1.5);
        assert_eq!(directional2.outer_gain, 1.0); // Clamped to 1

        let directional3 = DirectionalAudio::new(30.0, 60.0).with_outer_gain(0.3);
        assert_eq!(directional3.outer_gain, 0.3); // Within range
    }

    #[test]
    fn test_direction_normalization() {
        let directional =
            DirectionalAudio::new(30.0, 60.0).with_direction(Vec3::new(3.0, 4.0, 0.0)); // Length = 5.0

        assert!((directional.direction.length() - 1.0).abs() < 0.001);
        assert!((directional.direction - Vec3::new(0.6, 0.8, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_doppler_factor_clamping() {
        let listener = AudioListener::new().with_doppler_scale(1.0);

        // Test extreme velocities that might cause instability
        let extreme_doppler = listener.calculate_doppler_factor(
            Vec3::new(1000.0, 0.0, 0.0), // Very fast source
            Vec3::X,
        );

        // Should be clamped between 0.1 and 2.0
        assert!(extreme_doppler >= 0.1);
        assert!(extreme_doppler <= 2.0);
    }

    #[test]
    fn test_doppler_disabled() {
        let listener = AudioListener::new().with_doppler_scale(0.0);
        let doppler = listener.calculate_doppler_factor(Vec3::new(100.0, 0.0, 0.0), Vec3::X);
        assert_eq!(doppler, 1.0); // Should be 1.0 when doppler is disabled
    }

    #[test]
    fn test_complex_spatial_audio_scenario() {
        // Test a realistic scenario with multiple spatial audio features
        let source = SpatialAudioSource::new()
            .with_distance_range(1.0, 50.0)
            .with_rolloff(AudioRolloff::Logarithmic)
            .with_directional(45.0, 90.0)
            .with_spatial_blend(0.8);

        let listener = AudioListener::new()
            .with_volume(0.9)
            .with_doppler_scale(1.2);

        // Calculate various effects
        let distance_attenuation = source.calculate_distance_attenuation(10.0);
        let directional_attenuation = source.calculate_directional_attenuation(Vec3::NEG_Z);
        let doppler_factor = listener.calculate_doppler_factor(Vec3::ZERO, Vec3::X);

        // Verify reasonable values
        assert!(distance_attenuation > 0.0 && distance_attenuation <= 1.0);
        assert!(directional_attenuation > 0.0 && directional_attenuation <= 1.0);
        assert!(doppler_factor > 0.0);
    }
}
