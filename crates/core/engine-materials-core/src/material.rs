//! Material system abstractions

use crate::texture::TextureHandle;
use crate::Color;
use serde::{Deserialize, Serialize};

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

impl Material {
    /// Create a new material with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a metallic material
    pub fn metallic(name: impl Into<String>, albedo: Color, metallic: f32, roughness: f32) -> Self {
        Self {
            name: name.into(),
            pbr: PbrMaterial {
                albedo,
                metallic,
                roughness,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create an unlit material
    pub fn unlit(name: impl Into<String>, albedo: Color) -> Self {
        Self {
            name: name.into(),
            pbr: PbrMaterial {
                albedo,
                ..Default::default()
            },
            unlit: true,
            ..Default::default()
        }
    }
}

impl PbrMaterial {
    /// Create a metallic PBR material
    pub fn metallic(metallic: f32, roughness: f32) -> Self {
        Self {
            metallic,
            roughness,
            ..Default::default()
        }
    }

    /// Create a dielectric (non-metallic) PBR material
    pub fn dielectric(roughness: f32) -> Self {
        Self {
            metallic: 0.0,
            roughness,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_default() {
        let mat = Material::default();
        assert_eq!(mat.name, "Default");
        assert_eq!(mat.alpha_mode, AlphaMode::Opaque);
        assert!(!mat.double_sided);
        assert!(!mat.unlit);
        assert_eq!(mat.pbr.albedo, Color::WHITE);
        assert_eq!(mat.pbr.metallic, 0.0);
        assert_eq!(mat.pbr.roughness, 0.5);
    }

    #[test]
    fn test_material_creation() {
        let mat = Material::new("TestMaterial");
        assert_eq!(mat.name, "TestMaterial");
        assert_eq!(mat.pbr.albedo, Color::WHITE);
    }

    #[test]
    fn test_metallic_material() {
        let mat = Material::metallic("Metal", Color::rgb(0.8, 0.8, 0.8), 1.0, 0.2);
        assert_eq!(mat.name, "Metal");
        assert_eq!(mat.pbr.albedo, Color::rgb(0.8, 0.8, 0.8));
        assert_eq!(mat.pbr.metallic, 1.0);
        assert_eq!(mat.pbr.roughness, 0.2);
        assert!(!mat.unlit);
    }

    #[test]
    fn test_unlit_material() {
        let mat = Material::unlit("Unlit", Color::RED);
        assert_eq!(mat.name, "Unlit");
        assert_eq!(mat.pbr.albedo, Color::RED);
        assert!(mat.unlit);
    }

    #[test]
    fn test_pbr_material_default() {
        let pbr = PbrMaterial::default();
        assert_eq!(pbr.albedo, Color::WHITE);
        assert_eq!(pbr.metallic, 0.0);
        assert_eq!(pbr.roughness, 0.5);
        assert_eq!(pbr.emission, Color::BLACK);
        assert_eq!(pbr.normal_strength, 1.0);
        assert_eq!(pbr.occlusion_strength, 1.0);
        assert!(pbr.textures.albedo.is_none());
        assert!(pbr.textures.metallic_roughness.is_none());
        assert!(pbr.textures.normal.is_none());
        assert!(pbr.textures.emission.is_none());
        assert!(pbr.textures.occlusion.is_none());
    }

    #[test]
    fn test_pbr_material_metallic() {
        let pbr = PbrMaterial::metallic(0.9, 0.1);
        assert_eq!(pbr.metallic, 0.9);
        assert_eq!(pbr.roughness, 0.1);
        assert_eq!(pbr.albedo, Color::WHITE);
    }

    #[test]
    fn test_pbr_material_dielectric() {
        let pbr = PbrMaterial::dielectric(0.7);
        assert_eq!(pbr.metallic, 0.0);
        assert_eq!(pbr.roughness, 0.7);
    }

    #[test]
    fn test_alpha_mode() {
        assert_eq!(AlphaMode::Opaque, AlphaMode::Opaque);
        assert_ne!(AlphaMode::Opaque, AlphaMode::Blend);
        assert_ne!(
            AlphaMode::Mask { cutoff: 0.5 },
            AlphaMode::Mask { cutoff: 0.6 }
        );
        assert_eq!(
            AlphaMode::Mask { cutoff: 0.5 },
            AlphaMode::Mask { cutoff: 0.5 }
        );
    }

    #[test]
    fn test_material_textures() {
        let mut textures = MaterialTextures::default();
        assert!(textures.albedo.is_none());

        textures.albedo = Some(TextureHandle(1));
        textures.normal = Some(TextureHandle(2));

        assert_eq!(textures.albedo, Some(TextureHandle(1)));
        assert_eq!(textures.normal, Some(TextureHandle(2)));
        assert!(textures.metallic_roughness.is_none());
    }

    #[test]
    fn test_material_handle() {
        let handle1: MaterialHandle = 42;
        let handle2: MaterialHandle = 43;
        assert_ne!(handle1, handle2);
        assert_eq!(handle1, 42);
    }

    #[test]
    fn test_material_with_textures() {
        let mut mat = Material::new("Textured");
        mat.pbr.textures.albedo = Some(TextureHandle(1));
        mat.pbr.textures.normal = Some(TextureHandle(2));
        mat.alpha_mode = AlphaMode::Mask { cutoff: 0.5 };
        mat.double_sided = true;

        assert_eq!(mat.name, "Textured");
        assert!(mat.pbr.textures.albedo.is_some());
        assert!(mat.pbr.textures.normal.is_some());
        assert_eq!(mat.alpha_mode, AlphaMode::Mask { cutoff: 0.5 });
        assert!(mat.double_sided);
    }
}
