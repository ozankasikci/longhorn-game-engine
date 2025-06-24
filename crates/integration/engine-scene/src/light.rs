//! Lighting system for scene illumination

use engine_materials_core::Color;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Light component for scene illumination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: Color,
    pub intensity: f32,
    pub enabled: bool,
    pub cast_shadows: bool,
    pub shadow_settings: ShadowSettings,
}

/// Types of lights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Directional(DirectionalLight),
    Point(PointLight),
    Spot(SpotLight),
    Area(AreaLight),
}

/// Directional light (like sun)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Vec3,
}

/// Point light (omnidirectional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub range: f32,
    pub attenuation: Attenuation,
}

/// Spot light (cone of light)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub direction: Vec3,
    pub range: f32,
    pub inner_cone_angle: f32, // In radians
    pub outer_cone_angle: f32, // In radians
    pub attenuation: Attenuation,
}

/// Area light (rectangular light source)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaLight {
    pub width: f32,
    pub height: f32,
    pub two_sided: bool,
}

/// Light attenuation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

/// Shadow settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSettings {
    pub resolution: u32,
    pub bias: f32,
    pub normal_bias: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub cascade_count: u32, // For directional lights
}

impl Default for Light {
    fn default() -> Self {
        Self::directional(Vec3::new(-1.0, -1.0, -1.0), Color::WHITE, 1.0)
    }
}

impl Default for Attenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            resolution: 1024,
            bias: 0.005,
            normal_bias: 0.02,
            near_plane: 0.1,
            far_plane: 100.0,
            cascade_count: 4,
        }
    }
}

impl Light {
    /// Create a directional light
    pub fn directional(direction: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional(DirectionalLight {
                direction: direction.normalize(),
            }),
            color,
            intensity,
            enabled: true,
            cast_shadows: true,
            shadow_settings: ShadowSettings::default(),
        }
    }

    /// Create a point light
    pub fn point(color: Color, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point(PointLight {
                range,
                attenuation: Attenuation::default(),
            }),
            color,
            intensity,
            enabled: true,
            cast_shadows: false, // Expensive for point lights
            shadow_settings: ShadowSettings::default(),
        }
    }

    /// Create a spot light
    pub fn spot(
        direction: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            light_type: LightType::Spot(SpotLight {
                direction: direction.normalize(),
                range,
                inner_cone_angle: inner_angle,
                outer_cone_angle: outer_angle,
                attenuation: Attenuation::default(),
            }),
            color,
            intensity,
            enabled: true,
            cast_shadows: false,
            shadow_settings: ShadowSettings::default(),
        }
    }

    /// Create an area light
    pub fn area(color: Color, intensity: f32, width: f32, height: f32) -> Self {
        Self {
            light_type: LightType::Area(AreaLight {
                width,
                height,
                two_sided: false,
            }),
            color,
            intensity,
            enabled: true,
            cast_shadows: false,
            shadow_settings: ShadowSettings::default(),
        }
    }

    /// Enable or disable the light
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Enable or disable shadow casting
    pub fn with_shadows(mut self, cast_shadows: bool) -> Self {
        self.cast_shadows = cast_shadows;
        self
    }

    /// Set shadow settings
    pub fn with_shadow_settings(mut self, settings: ShadowSettings) -> Self {
        self.shadow_settings = settings;
        self
    }

    /// Get light direction (for directional and spot lights)
    pub fn direction(&self) -> Option<Vec3> {
        match &self.light_type {
            LightType::Directional(light) => Some(light.direction),
            LightType::Spot(light) => Some(light.direction),
            _ => None,
        }
    }

    /// Get light range (for point and spot lights)
    pub fn range(&self) -> Option<f32> {
        match &self.light_type {
            LightType::Point(light) => Some(light.range),
            LightType::Spot(light) => Some(light.range),
            _ => None,
        }
    }

    /// Check if light affects a point in space
    pub fn affects_point(&self, light_position: Vec3, point: Vec3) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.light_type {
            LightType::Directional(_) => true, // Affects everything
            LightType::Point(light) => {
                let distance = (point - light_position).length();
                distance <= light.range
            }
            LightType::Spot(light) => {
                let distance = (point - light_position).length();
                if distance > light.range {
                    return false;
                }

                let to_point = (point - light_position).normalize();
                let angle = light.direction.dot(to_point).acos();
                angle <= light.outer_cone_angle
            }
            LightType::Area(_) => {
                // Area lights have complex falloff, simplified here
                true
            }
        }
    }

    /// Calculate light contribution at a point
    pub fn calculate_contribution(&self, light_position: Vec3, point: Vec3, normal: Vec3) -> f32 {
        if !self.enabled || !self.affects_point(light_position, point) {
            return 0.0;
        }

        match &self.light_type {
            LightType::Directional(light) => {
                let dot = normal.dot(-light.direction).max(0.0);
                self.intensity * dot
            }
            LightType::Point(light) => {
                let to_light = light_position - point;
                let distance = to_light.length();
                let light_dir = to_light / distance;

                let dot = normal.dot(light_dir).max(0.0);
                let attenuation = 1.0
                    / (light.attenuation.constant
                        + light.attenuation.linear * distance
                        + light.attenuation.quadratic * distance * distance);

                self.intensity * dot * attenuation
            }
            LightType::Spot(light) => {
                let to_light = light_position - point;
                let distance = to_light.length();
                let light_dir = to_light / distance;

                let dot = normal.dot(light_dir).max(0.0);
                let attenuation = 1.0
                    / (light.attenuation.constant
                        + light.attenuation.linear * distance
                        + light.attenuation.quadratic * distance * distance);

                // Spot light cone falloff
                let angle = light.direction.dot(-light_dir).acos();
                let spot_factor = if angle <= light.inner_cone_angle {
                    1.0
                } else if angle <= light.outer_cone_angle {
                    let falloff = (light.outer_cone_angle - angle)
                        / (light.outer_cone_angle - light.inner_cone_angle);
                    falloff.powf(2.0) // Smooth falloff
                } else {
                    0.0
                };

                self.intensity * dot * attenuation * spot_factor
            }
            LightType::Area(_) => {
                // Simplified area light calculation
                let to_light = (light_position - point).normalize();
                let dot = normal.dot(to_light).max(0.0);
                self.intensity * dot
            }
        }
    }
}

impl Attenuation {
    /// Create linear attenuation
    pub fn linear(range: f32) -> Self {
        Self {
            constant: 1.0,
            linear: 2.0 / range,
            quadratic: 1.0 / (range * range),
        }
    }

    /// Create constant attenuation (no falloff)
    pub fn constant() -> Self {
        Self {
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }

    /// Create realistic attenuation for given range
    pub fn realistic(range: f32) -> Self {
        Self {
            constant: 1.0,
            linear: 4.5 / range,
            quadratic: 75.0 / (range * range),
        }
    }
}

impl ShadowSettings {
    /// Create shadow settings for directional light
    pub fn directional(resolution: u32, cascade_count: u32) -> Self {
        Self {
            resolution,
            cascade_count,
            bias: 0.001,
            normal_bias: 0.02,
            near_plane: 0.1,
            far_plane: 200.0,
        }
    }

    /// Create shadow settings for point light
    pub fn point(resolution: u32) -> Self {
        Self {
            resolution,
            cascade_count: 1,
            bias: 0.005,
            normal_bias: 0.02,
            near_plane: 0.1,
            far_plane: 100.0,
        }
    }

    /// Create shadow settings for spot light
    pub fn spot(resolution: u32) -> Self {
        Self {
            resolution,
            cascade_count: 1,
            bias: 0.002,
            normal_bias: 0.02,
            near_plane: 0.1,
            far_plane: 100.0,
        }
    }
}
