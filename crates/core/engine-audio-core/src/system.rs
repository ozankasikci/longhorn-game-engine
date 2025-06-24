//! Audio ECS system abstractions

use crate::{
    AudioEffects, AudioListener, AudioManager, AudioSource, PlaybackSettings, Result,
    SpatialAudioSource, StreamingAudioSource,
};
use engine_components_3d::Transform;
use engine_ecs_core::{Entity, World};
use glam::Vec3;

/// Audio system for processing ECS audio components
pub trait AudioSystem {
    /// Update all audio sources in the world
    fn update_audio_sources(&mut self, world: &World, delta_time: f32) -> Result<()>;

    /// Update spatial audio calculations
    fn update_spatial_audio(&mut self, world: &World) -> Result<()>;

    /// Update streaming audio sources
    fn update_streaming_sources(&mut self, world: &World, delta_time: f32) -> Result<()>;

    /// Process audio effects
    fn process_audio_effects(&mut self, world: &World) -> Result<()>;

    /// Play audio for an entity
    fn play_entity_audio(&mut self, world: &World, entity: Entity) -> Result<()>;

    /// Stop audio for an entity
    fn stop_entity_audio(&mut self, entity: Entity) -> Result<()>;

    /// Set listener entity (typically the camera)
    fn set_listener_entity(&mut self, entity: Entity);

    /// Get current listener entity
    fn get_listener_entity(&self) -> Option<Entity>;
}

/// Audio system implementation
pub struct AudioSystemImpl<T: AudioManager> {
    /// Audio manager implementation
    audio_manager: T,

    /// Current listener entity
    listener_entity: Option<Entity>,

    /// Active playback handles for entities
    entity_playbacks: std::collections::HashMap<Entity, Vec<crate::PlaybackHandle>>,

    /// Last known listener position for doppler calculations
    last_listener_position: Vec3,

    /// Last known listener velocity
    last_listener_velocity: Vec3,
}

impl<T: AudioManager> AudioSystemImpl<T> {
    /// Create new audio system
    pub fn new(audio_manager: T) -> Self {
        Self {
            audio_manager,
            listener_entity: None,
            entity_playbacks: std::collections::HashMap::new(),
            last_listener_position: Vec3::ZERO,
            last_listener_velocity: Vec3::ZERO,
        }
    }

    /// Get reference to audio manager
    pub fn audio_manager(&self) -> &T {
        &self.audio_manager
    }

    /// Get mutable reference to audio manager
    pub fn audio_manager_mut(&mut self) -> &mut T {
        &mut self.audio_manager
    }
}

impl<T: AudioManager> AudioSystem for AudioSystemImpl<T> {
    fn update_audio_sources(&mut self, world: &World, delta_time: f32) -> Result<()> {
        // Update the audio manager
        self.audio_manager.update(delta_time)?;

        // Process all entities with AudioSource components
        for (entity, audio_source) in world.query_legacy::<AudioSource>() {
            // Check if we need to start playing
            if audio_source.play_on_awake && audio_source.state == crate::PlaybackState::Stopped {
                self.play_entity_audio(world, entity)?;
            }

            // Update volume and pitch for active playbacks
            if let Some(playbacks) = self.entity_playbacks.get(&entity) {
                for &playback_handle in playbacks {
                    if self.audio_manager.is_playback_active(playback_handle) {
                        self.audio_manager
                            .set_playback_volume(playback_handle, audio_source.volume)?;
                        self.audio_manager
                            .set_playback_pitch(playback_handle, audio_source.pitch)?;
                    }
                }
            }
        }

        // Clean up finished playbacks
        self.entity_playbacks.retain(|_, playbacks| {
            playbacks.retain(|&handle| self.audio_manager.is_playback_active(handle));
            !playbacks.is_empty()
        });

        Ok(())
    }

    fn update_spatial_audio(&mut self, world: &World) -> Result<()> {
        // Get listener position and velocity
        let (listener_pos, listener_vel) = if let Some(listener_entity) = self.listener_entity {
            if let (Some(transform), Some(_listener)) = (
                world.get_component::<Transform>(listener_entity),
                world.get_component::<AudioListener>(listener_entity),
            ) {
                let pos = Vec3::from_array(transform.position);
                let vel = (pos - self.last_listener_position) / 0.016; // Assume 60 FPS
                (pos, vel)
            } else {
                (Vec3::ZERO, Vec3::ZERO)
            }
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };

        // Update spatial audio sources
        for (entity, spatial_source) in world.query_legacy::<SpatialAudioSource>() {
            if !spatial_source.enabled {
                continue;
            }

            if let Some(transform) = world.get_component::<Transform>(entity) {
                let source_pos = Vec3::from_array(transform.position);
                let distance = source_pos.distance(listener_pos);

                // Calculate distance attenuation
                let distance_attenuation = spatial_source.calculate_distance_attenuation(distance);

                // Calculate directional attenuation if configured
                let direction_to_listener = (listener_pos - source_pos).normalize_or_zero();
                let directional_attenuation =
                    spatial_source.calculate_directional_attenuation(direction_to_listener);

                // Calculate final volume
                let final_volume = distance_attenuation * directional_attenuation;

                // Update playback volumes for this entity
                if let Some(playbacks) = self.entity_playbacks.get(&entity) {
                    for &playback_handle in playbacks {
                        if self.audio_manager.is_playback_active(playback_handle) {
                            self.audio_manager
                                .set_playback_volume(playback_handle, final_volume)?;
                        }
                    }
                }
            }
        }

        // Update listener state
        self.last_listener_position = listener_pos;
        self.last_listener_velocity = listener_vel;

        Ok(())
    }

    fn update_streaming_sources(&mut self, world: &World, _delta_time: f32) -> Result<()> {
        // Process streaming audio sources
        for (_entity, streaming_source) in world.query_legacy::<StreamingAudioSource>() {
            // Check if streaming source needs attention
            if streaming_source.auto_play
                && streaming_source.is_ready()
                && !streaming_source.is_playing()
            {
                // Could trigger playback here
            }

            if streaming_source.needs_buffering() {
                // Could trigger buffer refill here
            }
        }

        Ok(())
    }

    fn process_audio_effects(&mut self, world: &World) -> Result<()> {
        // Process entities with audio effects
        for (_entity, effects) in world.query_legacy::<AudioEffects>() {
            if effects.has_effects() {
                // Effects processing would be handled by the audio manager implementation
                // This is where we'd apply real-time audio effects
            }
        }

        Ok(())
    }

    fn play_entity_audio(&mut self, world: &World, entity: Entity) -> Result<()> {
        if let Some(audio_source) = world.get_component::<AudioSource>(entity) {
            let mut settings = PlaybackSettings {
                volume: audio_source.volume,
                pitch: audio_source.pitch,
                looping: audio_source.looping,
                mixer_group: audio_source.mixer_group.clone(),
                priority: audio_source.priority,
                ..Default::default()
            };

            // Add 3D positioning if the entity has spatial audio
            if let Some(spatial_source) = world.get_component::<SpatialAudioSource>(entity) {
                if spatial_source.enabled {
                    if let Some(transform) = world.get_component::<Transform>(entity) {
                        settings.position = Some(Vec3::from_array(transform.position));
                        settings.velocity = Some(spatial_source.velocity);
                        settings.min_distance = Some(spatial_source.min_distance);
                        settings.max_distance = Some(spatial_source.max_distance);
                    }
                }
            }

            // Start playback
            let playback_handle = self.audio_manager.play_clip(audio_source.clip, settings)?;

            // Track the playback
            self.entity_playbacks
                .entry(entity)
                .or_default()
                .push(playback_handle);
        }

        Ok(())
    }

    fn stop_entity_audio(&mut self, entity: Entity) -> Result<()> {
        if let Some(playbacks) = self.entity_playbacks.remove(&entity) {
            for playback_handle in playbacks {
                self.audio_manager.stop_playback(playback_handle)?;
            }
        }
        Ok(())
    }

    fn set_listener_entity(&mut self, entity: Entity) {
        self.listener_entity = Some(entity);
    }

    fn get_listener_entity(&self) -> Option<Entity> {
        self.listener_entity
    }
}

/// Audio events that can be fired from gameplay systems
#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {
    /// Play a sound effect at a position
    PlaySfx {
        clip: crate::AudioHandle,
        position: Option<Vec3>,
        volume: f32,
        pitch: f32,
    },

    /// Play background music
    PlayMusic {
        clip: crate::AudioHandle,
        fade_in: Option<f32>,
        volume: f32,
    },

    /// Stop currently playing music
    StopMusic { fade_out: Option<f32> },

    /// Play UI sound
    PlayUiSound {
        clip: crate::AudioHandle,
        volume: f32,
    },

    /// Change mixer group volume
    SetMixerVolume {
        group: String,
        volume: f32,
        fade_time: Option<f32>,
    },

    /// Apply audio profile
    ApplyProfile {
        profile_name: String,
        fade_time: Option<f32>,
    },

    /// Trigger audio snapshot for environmental changes
    TriggerSnapshot { snapshot_name: String },
}

/// Audio event processor for handling game events
pub trait AudioEventProcessor {
    /// Process an audio event
    fn process_event(&mut self, event: AudioEvent) -> Result<()>;

    /// Process multiple events in batch
    fn process_events(&mut self, events: &[AudioEvent]) -> Result<()> {
        for event in events {
            self.process_event(event.clone())?;
        }
        Ok(())
    }
}
