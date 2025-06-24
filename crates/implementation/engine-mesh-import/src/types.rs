use engine_asset_import::AssetImporter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Option<Material>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    name: String,
    properties: HashMap<String, MaterialProperty>,
}

impl Material {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_property(&mut self, property: MaterialProperty) {
        let key = match &property {
            MaterialProperty::BaseColor(_) => "base_color",
            MaterialProperty::Metallic(_) => "metallic",
            MaterialProperty::Roughness(_) => "roughness",
            MaterialProperty::EmissiveFactor(_) => "emissive_factor",
            MaterialProperty::AlphaMode(_) => "alpha_mode",
            MaterialProperty::AlphaCutoff(_) => "alpha_cutoff",
            MaterialProperty::DoubleSided(_) => "double_sided",
        };
        self.properties.insert(key.to_string(), property);
    }

    pub fn get_property(&self, key: &str) -> Option<&MaterialProperty> {
        self.properties.get(key)
    }
}

impl Default for Material {
    fn default() -> Self {
        let mut material = Self::new("Default");
        material.set_property(MaterialProperty::BaseColor([1.0, 1.0, 1.0, 1.0]));
        material.set_property(MaterialProperty::Metallic(0.0));
        material.set_property(MaterialProperty::Roughness(0.5));
        material
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialProperty {
    BaseColor([f32; 4]),
    Metallic(f32),
    Roughness(f32),
    EmissiveFactor([f32; 3]),
    AlphaMode(String),
    AlphaCutoff(f32),
    DoubleSided(bool),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Common trait for mesh importers
pub trait MeshImporter: AssetImporter {
    // Remove methods that conflict with AssetImporter
}
