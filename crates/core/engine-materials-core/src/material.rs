//! Material system abstractions

use crate::Color;
use crate::texture::TextureHandle;
use serde::{Serialize, Deserialize};

/// Handle for material resources
pub type MaterialHandle = u64;

/// PBR material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub pbr: PbrMaterial,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
    pub unlit: bool,
}

/// Physically-based rendering material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMaterial {
    pub albedo: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Color,
    pub normal_strength: f32,
    pub occlusion_strength: f32,
    pub textures: MaterialTextures,
}

/// Material textures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialTextures {
    pub albedo: Option<TextureHandle>,
    pub metallic_roughness: Option<TextureHandle>,
    pub normal: Option<TextureHandle>,
    pub emission: Option<TextureHandle>,
    pub occlusion: Option<TextureHandle>,
}

/// Alpha blending modes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            pbr: PbrMaterial::default(),
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            unlit: false,
        }
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emission: Color::BLACK,
            normal_strength: 1.0,
            occlusion_strength: 1.0,
            textures: MaterialTextures::default(),
        }
    }
}