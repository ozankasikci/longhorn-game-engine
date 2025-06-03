//! 3D rendering components for the mobile game engine
//! 
//! This crate provides 3D-specific components including Transform, Mesh,
//! Material, Light, and Visibility components.

use serde::{Serialize, Deserialize};
use engine_component_traits::Component;

// Transform component - fundamental for all spatial objects
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3], 
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Transform {
    /// Create a new Transform with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a transform matrix from this transform
    pub fn matrix(&self) -> glam::Mat4 {
        let translation = glam::Vec3::from_array(self.position);
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            self.rotation[0],
            self.rotation[1], 
            self.rotation[2]
        );
        let scale = glam::Vec3::from_array(self.scale);
        
        glam::Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
    
    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = [x, y, z];
        self
    }
    
    /// Set rotation (in radians)
    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = [x, y, z];
        self
    }
    
    /// Set scale
    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = [x, y, z];
        self
    }
}

// Component trait implementation
impl Component for Transform {}

// Mesh component - defines what mesh to render
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_type: MeshType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MeshType {
    Cube,
    Sphere,
    Plane,
    Custom(String), // Asset path for custom meshes
}

// Component trait implementation
impl Component for Mesh {}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            mesh_type: MeshType::Cube,
        }
    }
}

// Material component - defines how the mesh should be rendered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub color: [f32; 4], // RGBA
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3], // RGB emissive color
}

// Component trait implementation
impl Component for Material {}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: [0.8, 0.8, 0.8, 1.0], // Light gray
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }
    }
}

// Light component - defines various light types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3], // RGB
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point { range: f32 },
    Spot { range: f32, angle: f32 },
}

// Component trait implementation
impl Component for Light {}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        }
    }
}

// Visibility component - whether the object should be rendered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visibility {
    pub visible: bool,
}

// Component trait implementation
impl Component for Visibility {}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

// ============================================================================
// COMPONENT BUNDLES - Quick solution for multi-component entities
// ============================================================================

use engine_component_traits::Bundle;

/// Bundle for standard 3D game objects
pub struct GameObject3DBundle {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,
    pub visibility: Visibility,
}

impl Bundle for GameObject3DBundle {
    fn component_ids() -> Vec<std::any::TypeId> where Self: Sized {
        vec![
            std::any::TypeId::of::<Transform>(),
            std::any::TypeId::of::<Mesh>(),
            std::any::TypeId::of::<Material>(),
            std::any::TypeId::of::<Visibility>(),
        ]
    }
    
    fn into_components(self) -> Vec<(std::any::TypeId, Box<dyn engine_component_traits::ComponentClone>)> {
        vec![
            (std::any::TypeId::of::<Transform>(), Box::new(self.transform)),
            (std::any::TypeId::of::<Mesh>(), Box::new(self.mesh)),
            (std::any::TypeId::of::<Material>(), Box::new(self.material)),
            (std::any::TypeId::of::<Visibility>(), Box::new(self.visibility)),
        ]
    }
}

impl Default for GameObject3DBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            mesh: Mesh::default(),
            material: Material::default(),
            visibility: Visibility::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }
    
    #[test]
    fn test_material_default() {
        let material = Material::default();
        assert_eq!(material.color, [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
        assert_eq!(material.emissive, [0.0, 0.0, 0.0]);
    }
    
    #[test]
    fn test_light_default() {
        let light = Light::default();
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.intensity, 1.0);
        assert!(matches!(light.light_type, LightType::Directional));
    }
}