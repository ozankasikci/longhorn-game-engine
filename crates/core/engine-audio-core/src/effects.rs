//! Audio effects and processing abstractions

use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;

/// Audio effects component for entities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEffects {
    /// List of effects to apply
    pub effects: Vec<AudioEffect>,
    
    /// Whether effects are enabled
    pub enabled: bool,
    
    /// Dry/wet mix (0.0 = no effects, 1.0 = full effects)
    pub mix: f32,
}

impl Default for AudioEffects {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            enabled: true,
            mix: 1.0,
        }
    }
}

impl Component for AudioEffects {}


/// Audio effect types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioEffect {
    /// Low-pass filter
    LowPass {
        /// Cutoff frequency in Hz
        cutoff: f32,
        /// Filter resonance
        resonance: f32,
    },
    
    /// High-pass filter
    HighPass {
        /// Cutoff frequency in Hz
        cutoff: f32,
        /// Filter resonance
        resonance: f32,
    },
    
    /// Band-pass filter
    BandPass {
        /// Center frequency in Hz
        frequency: f32,
        /// Bandwidth in Hz
        bandwidth: f32,
    },
    
    /// Reverb effect
    Reverb {
        /// Room size (0.0 to 1.0)
        room_size: f32,
        /// Damping factor (0.0 to 1.0)
        damping: f32,
        /// Wet level (0.0 to 1.0)
        wet_level: f32,
        /// Dry level (0.0 to 1.0)
        dry_level: f32,
    },
    
    /// Echo/delay effect
    Echo {
        /// Delay time in seconds
        delay: f32,
        /// Feedback amount (0.0 to 1.0)
        feedback: f32,
        /// Wet level (0.0 to 1.0)
        wet_level: f32,
    },
    
    /// Distortion effect
    Distortion {
        /// Drive amount (1.0 to 10.0+)
        drive: f32,
        /// Output level (0.0 to 1.0)
        level: f32,
    },
    
    /// Compressor/limiter
    Compressor {
        /// Compression threshold in dB
        threshold: f32,
        /// Compression ratio (1.0 = no compression)
        ratio: f32,
        /// Attack time in seconds
        attack: f32,
        /// Release time in seconds
        release: f32,
    },
    
    /// Equalizer (3-band)
    Equalizer {
        /// Low frequency gain in dB
        low_gain: f32,
        /// Mid frequency gain in dB
        mid_gain: f32,
        /// High frequency gain in dB
        high_gain: f32,
    },
    
    /// Chorus effect
    Chorus {
        /// Modulation rate in Hz
        rate: f32,
        /// Modulation depth (0.0 to 1.0)
        depth: f32,
        /// Feedback amount (0.0 to 1.0)
        feedback: f32,
        /// Wet level (0.0 to 1.0)
        wet_level: f32,
    },
    
    /// Flanger effect
    Flanger {
        /// Modulation rate in Hz
        rate: f32,
        /// Modulation depth (0.0 to 1.0)
        depth: f32,
        /// Feedback amount (0.0 to 1.0)
        feedback: f32,
    },
}

impl AudioEffect {
    /// Create a simple low-pass filter
    pub fn low_pass(cutoff: f32) -> Self {
        Self::LowPass {
            cutoff,
            resonance: 1.0,
        }
    }
    
    /// Create a simple high-pass filter
    pub fn high_pass(cutoff: f32) -> Self {
        Self::HighPass {
            cutoff,
            resonance: 1.0,
        }
    }
    
    /// Create a room reverb effect
    pub fn reverb_room() -> Self {
        Self::Reverb {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
        }
    }
    
    /// Create a hall reverb effect
    pub fn reverb_hall() -> Self {
        Self::Reverb {
            room_size: 0.8,
            damping: 0.2,
            wet_level: 0.4,
            dry_level: 0.6,
        }
    }
    
    /// Create a simple echo effect
    pub fn echo(delay_seconds: f32) -> Self {
        Self::Echo {
            delay: delay_seconds,
            feedback: 0.3,
            wet_level: 0.4,
        }
    }
    
    /// Create a soft compressor
    pub fn compressor_soft() -> Self {
        Self::Compressor {
            threshold: -12.0,
            ratio: 3.0,
            attack: 0.003,
            release: 0.1,
        }
    }
    
    /// Check if this effect modifies frequency response
    pub fn affects_frequency(&self) -> bool {
        matches!(
            self,
            AudioEffect::LowPass { .. }
                | AudioEffect::HighPass { .. }
                | AudioEffect::BandPass { .. }
                | AudioEffect::Equalizer { .. }
        )
    }
    
    /// Check if this effect adds spatial characteristics
    pub fn affects_space(&self) -> bool {
        matches!(
            self,
            AudioEffect::Reverb { .. }
                | AudioEffect::Echo { .. }
                | AudioEffect::Chorus { .. }
                | AudioEffect::Flanger { .. }
        )
    }
}

impl AudioEffects {
    /// Create new effects component
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an effect
    pub fn add_effect(mut self, effect: AudioEffect) -> Self {
        self.effects.push(effect);
        self
    }
    
    /// Set the dry/wet mix
    pub fn with_mix(mut self, mix: f32) -> Self {
        self.mix = mix.clamp(0.0, 1.0);
        self
    }
    
    /// Enable/disable effects
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
    
    /// Check if any effects are active
    pub fn has_effects(&self) -> bool {
        self.enabled && !self.effects.is_empty()
    }
}