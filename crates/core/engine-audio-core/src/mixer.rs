//! Audio mixing abstractions

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Audio mixer group for organizing and controlling audio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMixerGroup {
    /// Unique identifier
    pub name: String,
    
    /// Parent group (for hierarchical mixing)
    pub parent: Option<String>,
    
    /// Volume multiplier (0.0 to 1.0+)
    pub volume: f32,
    
    /// Pitch multiplier (0.1 to 4.0)
    pub pitch: f32,
    
    /// Whether this group is muted
    pub muted: bool,
    
    /// Whether this group bypasses effects
    pub bypass_effects: bool,
    
    /// Low-frequency gain in dB
    pub low_gain: f32,
    
    /// Mid-frequency gain in dB
    pub mid_gain: f32,
    
    /// High-frequency gain in dB
    pub high_gain: f32,
    
    /// Send level to reverb bus
    pub reverb_send: f32,
    
    /// Send level to chorus bus
    pub chorus_send: f32,
}

impl Default for AudioMixerGroup {
    fn default() -> Self {
        Self {
            name: "Master".to_string(),
            parent: None,
            volume: 1.0,
            pitch: 1.0,
            muted: false,
            bypass_effects: false,
            low_gain: 0.0,
            mid_gain: 0.0,
            high_gain: 0.0,
            reverb_send: 0.0,
            chorus_send: 0.0,
        }
    }
}

/// Audio mixer configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMixer {
    /// All mixer groups
    pub groups: HashMap<String, AudioMixerGroup>,
    
    /// Master volume (affects all audio)
    pub master_volume: f32,
    
    /// Master mute state
    pub master_muted: bool,
    
    /// Active audio profiles
    pub active_profile: Option<String>,
    
    /// Available audio profiles
    pub profiles: HashMap<String, AudioProfile>,
}

impl Default for AudioMixer {
    fn default() -> Self {
        let mut groups = HashMap::new();
        
        // Create default mixer groups
        groups.insert("Master".to_string(), AudioMixerGroup {
            name: "Master".to_string(),
            parent: None,
            ..Default::default()
        });
        
        groups.insert("Music".to_string(), AudioMixerGroup {
            name: "Music".to_string(),
            parent: Some("Master".to_string()),
            ..Default::default()
        });
        
        groups.insert("SFX".to_string(), AudioMixerGroup {
            name: "SFX".to_string(),
            parent: Some("Master".to_string()),
            ..Default::default()
        });
        
        groups.insert("Voice".to_string(), AudioMixerGroup {
            name: "Voice".to_string(),
            parent: Some("Master".to_string()),
            ..Default::default()
        });
        
        groups.insert("UI".to_string(), AudioMixerGroup {
            name: "UI".to_string(),
            parent: Some("SFX".to_string()),
            ..Default::default()
        });
        
        Self {
            groups,
            master_volume: 1.0,
            master_muted: false,
            active_profile: None,
            profiles: HashMap::new(),
        }
    }
}

/// Audio profile for different scenarios (e.g., gameplay, menu, cutscene)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioProfile {
    /// Profile name
    pub name: String,
    
    /// Volume settings for each mixer group
    pub group_volumes: HashMap<String, f32>,
    
    /// Effect settings
    pub reverb_enabled: bool,
    pub reverb_amount: f32,
    
    /// Dynamic range compression
    pub compression_enabled: bool,
    pub compression_threshold: f32,
    pub compression_ratio: f32,
    
    /// Ducking settings (lower music when voice plays)
    pub ducking_enabled: bool,
    pub ducking_threshold: f32,
    pub ducking_amount: f32,
}

impl Default for AudioProfile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            group_volumes: HashMap::new(),
            reverb_enabled: false,
            reverb_amount: 0.3,
            compression_enabled: false,
            compression_threshold: -12.0,
            compression_ratio: 3.0,
            ducking_enabled: false,
            ducking_threshold: -30.0,
            ducking_amount: 0.5,
        }
    }
}

/// Audio bus for effects sends
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBus {
    /// Bus name
    pub name: String,
    
    /// Bus volume
    pub volume: f32,
    
    /// Whether bus is muted
    pub muted: bool,
    
    /// Effect chain for this bus
    pub effects: Vec<String>, // Effect names/IDs
}

impl AudioMixer {
    /// Create new mixer with default groups
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a new mixer group
    pub fn add_group(&mut self, group: AudioMixerGroup) {
        self.groups.insert(group.name.clone(), group);
    }
    
    /// Get mixer group by name
    pub fn get_group(&self, name: &str) -> Option<&AudioMixerGroup> {
        self.groups.get(name)
    }
    
    /// Get mutable mixer group by name
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut AudioMixerGroup> {
        self.groups.get_mut(name)
    }
    
    /// Set volume for a mixer group
    pub fn set_group_volume(&mut self, group_name: &str, volume: f32) {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.volume = volume.clamp(0.0, 2.0);
        }
    }
    
    /// Mute/unmute a mixer group
    pub fn set_group_muted(&mut self, group_name: &str, muted: bool) {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.muted = muted;
        }
    }
    
    /// Calculate final volume for a group (including parent groups)
    pub fn calculate_final_volume(&self, group_name: &str) -> f32 {
        let mut volume = self.master_volume;
        let mut current_group = group_name;
        
        // Walk up the hierarchy multiplying volumes
        while let Some(group) = self.groups.get(current_group) {
            if self.master_muted || group.muted {
                return 0.0;
            }
            
            volume *= group.volume;
            
            if let Some(parent) = &group.parent {
                current_group = parent;
            } else {
                break;
            }
        }
        
        volume
    }
    
    /// Apply an audio profile
    pub fn apply_profile(&mut self, profile_name: &str) {
        if let Some(profile) = self.profiles.get(profile_name).cloned() {
            // Apply group volume settings
            for (group_name, volume) in &profile.group_volumes {
                self.set_group_volume(group_name, *volume);
            }
            
            self.active_profile = Some(profile_name.to_string());
        }
    }
    
    /// Create a gameplay audio profile
    pub fn create_gameplay_profile() -> AudioProfile {
        let mut profile = AudioProfile {
            name: "Gameplay".to_string(),
            reverb_enabled: true,
            reverb_amount: 0.2,
            ducking_enabled: true,
            ..Default::default()
        };
        
        profile.group_volumes.insert("Music".to_string(), 0.7);
        profile.group_volumes.insert("SFX".to_string(), 1.0);
        profile.group_volumes.insert("Voice".to_string(), 1.0);
        profile.group_volumes.insert("UI".to_string(), 0.8);
        
        profile
    }
    
    /// Create a menu audio profile
    pub fn create_menu_profile() -> AudioProfile {
        let mut profile = AudioProfile {
            name: "Menu".to_string(),
            reverb_enabled: false,
            ducking_enabled: false,
            ..Default::default()
        };
        
        profile.group_volumes.insert("Music".to_string(), 1.0);
        profile.group_volumes.insert("SFX".to_string(), 0.5);
        profile.group_volumes.insert("Voice".to_string(), 0.8);
        profile.group_volumes.insert("UI".to_string(), 1.0);
        
        profile
    }
}