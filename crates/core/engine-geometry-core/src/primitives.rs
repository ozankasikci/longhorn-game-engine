//! Primitive shape abstractions and interfaces
//! 
//! This module defines the core abstractions for primitive shapes.
//! Concrete implementations are provided by implementation crates.

use crate::{Mesh, MeshData};
use serde::{Serialize, Deserialize};

/// Primitive shape types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Cylinder,
    Cone,
    Plane,
    Quad,
    Triangle,
    Capsule,
    Torus,
}

/// Parameters for cube generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubeParams {
    pub size: f32,
}

impl Default for CubeParams {
    fn default() -> Self {
        Self { size: 1.0 }
    }
}

/// Parameters for sphere generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereParams {
    pub radius: f32,
    pub rings: u32,
    pub sectors: u32,
}

impl Default for SphereParams {
    fn default() -> Self {
        Self {
            radius: 0.5,
            rings: 16,
            sectors: 32,
        }
    }
}

/// Parameters for cylinder generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CylinderParams {
    pub top_radius: f32,
    pub bottom_radius: f32,
    pub height: f32,
    pub sectors: u32,
}

impl Default for CylinderParams {
    fn default() -> Self {
        Self {
            top_radius: 0.5,
            bottom_radius: 0.5,
            height: 1.0,
            sectors: 16,
        }
    }
}

/// Parameters for cone generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConeParams {
    pub radius: f32,
    pub height: f32,
    pub sectors: u32,
}

impl Default for ConeParams {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            sectors: 16,
        }
    }
}

/// Parameters for plane generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaneParams {
    pub width: f32,
    pub height: f32,
    pub subdivisions_x: u32,
    pub subdivisions_y: u32,
}

impl Default for PlaneParams {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            subdivisions_x: 1,
            subdivisions_y: 1,
        }
    }
}

/// Parameters for primitive generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveParams {
    Cube(CubeParams),
    Sphere(SphereParams),
    Cylinder(CylinderParams),
    Cone(ConeParams),
    Plane(PlaneParams),
    Quad,
    Triangle,
    Capsule(CylinderParams), // Reuse cylinder params for now
    Torus(SphereParams),     // Reuse sphere params for now
}

impl PrimitiveParams {
    /// Create default parameters for a primitive type
    pub fn for_type(primitive_type: PrimitiveType) -> Self {
        match primitive_type {
            PrimitiveType::Cube => Self::Cube(CubeParams::default()),
            PrimitiveType::Sphere => Self::Sphere(SphereParams::default()),
            PrimitiveType::Cylinder => Self::Cylinder(CylinderParams::default()),
            PrimitiveType::Cone => Self::Cone(ConeParams::default()),
            PrimitiveType::Plane => Self::Plane(PlaneParams::default()),
            PrimitiveType::Quad => Self::Quad,
            PrimitiveType::Triangle => Self::Triangle,
            PrimitiveType::Capsule => Self::Capsule(CylinderParams::default()),
            PrimitiveType::Torus => Self::Torus(SphereParams::default()),
        }
    }
    
    /// Get the primitive type
    pub fn primitive_type(&self) -> PrimitiveType {
        match self {
            Self::Cube(_) => PrimitiveType::Cube,
            Self::Sphere(_) => PrimitiveType::Sphere,
            Self::Cylinder(_) => PrimitiveType::Cylinder,
            Self::Cone(_) => PrimitiveType::Cone,
            Self::Plane(_) => PrimitiveType::Plane,
            Self::Quad => PrimitiveType::Quad,
            Self::Triangle => PrimitiveType::Triangle,
            Self::Capsule(_) => PrimitiveType::Capsule,
            Self::Torus(_) => PrimitiveType::Torus,
        }
    }
}

/// Trait for primitive mesh generation
/// 
/// This trait should be implemented by geometry implementation crates
/// to provide actual mesh generation algorithms.
pub trait PrimitiveGenerator: Send + Sync {
    /// Generate a mesh for the given primitive parameters
    fn generate(&self, params: &PrimitiveParams) -> crate::Result<Mesh>;
    
    /// Generate a mesh for a primitive type with default parameters
    fn generate_default(&self, primitive_type: PrimitiveType) -> crate::Result<Mesh> {
        let params = PrimitiveParams::for_type(primitive_type);
        self.generate(&params)
    }
    
    /// Check if this generator supports a specific primitive type
    fn supports(&self, primitive_type: PrimitiveType) -> bool;
    
    /// Get a list of supported primitive types
    fn supported_types(&self) -> Vec<PrimitiveType>;
}

/// Simple primitive mesh factory trait
/// 
/// Provides a simplified interface for common primitive generation.
pub trait PrimitiveMeshFactory {
    /// Create a cube mesh
    fn cube(size: f32) -> crate::Result<Mesh>;
    
    /// Create a sphere mesh
    fn sphere(radius: f32, rings: u32, sectors: u32) -> crate::Result<Mesh>;
    
    /// Create a cylinder mesh
    fn cylinder(top_radius: f32, bottom_radius: f32, height: f32, sectors: u32) -> crate::Result<Mesh>;
    
    /// Create a cone mesh
    fn cone(radius: f32, height: f32, sectors: u32) -> crate::Result<Mesh>;
    
    /// Create a plane mesh
    fn plane(width: f32, height: f32, subdivisions_x: u32, subdivisions_y: u32) -> crate::Result<Mesh>;
    
    /// Create a quad mesh
    fn quad() -> crate::Result<Mesh>;
    
    /// Create a triangle mesh
    fn triangle() -> crate::Result<Mesh>;
}

/// Mesh builder for creating custom primitives
pub struct PrimitiveMeshBuilder {
    params: PrimitiveParams,
    name: Option<String>,
}

impl PrimitiveMeshBuilder {
    /// Create a new builder for the given primitive type
    pub fn new(primitive_type: PrimitiveType) -> Self {
        Self {
            params: PrimitiveParams::for_type(primitive_type),
            name: None,
        }
    }
    
    /// Set custom parameters
    pub fn with_params(mut self, params: PrimitiveParams) -> Self {
        self.params = params;
        self
    }
    
    /// Set mesh name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    /// Build the mesh using the provided generator
    pub fn build(self, generator: &dyn PrimitiveGenerator) -> crate::Result<Mesh> {
        let mut mesh = generator.generate(&self.params)?;
        
        if let Some(name) = self.name {
            mesh.set_name(name);
        }
        
        Ok(mesh)
    }
    
    /// Get the parameters
    pub fn params(&self) -> &PrimitiveParams {
        &self.params
    }
}

impl Default for PrimitiveMeshBuilder {
    fn default() -> Self {
        Self::new(PrimitiveType::Cube)
    }
}