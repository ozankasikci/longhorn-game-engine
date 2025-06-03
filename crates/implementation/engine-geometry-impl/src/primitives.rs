//! Primitive mesh generators and geometric shapes

use engine_geometry_core::{
    Mesh, MeshData, Vertex, 
    PrimitiveGenerator, PrimitiveParams, PrimitiveMeshFactory, PrimitiveType,
    CubeParams, SphereParams, CylinderParams, ConeParams, PlaneParams
};
use glam::{Vec3, Vec2};
use serde::{Serialize, Deserialize};

/// Default primitive generator implementation
#[derive(Debug, Clone, Default)]
pub struct DefaultPrimitiveGenerator;

/// Legacy mesh primitive generators for backwards compatibility
pub struct MeshPrimitives;

impl MeshPrimitives {
    /// Create a cube mesh
    pub fn cube(size: f32) -> Mesh {
        let half_size = size * 0.5;
        let vertices = vec![
            // Front face
            Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 1.0)),
            
            // Back face
            Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(0.0, 1.0)),
            
            // Left face
            Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(0.0, 1.0)),
            
            // Right face
            Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::X).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::X).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::X).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::X).with_uv(Vec2::new(0.0, 1.0)),
            
            // Top face
            Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(0.0, 1.0)),
            
            // Bottom face
            Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(0.0, 1.0)),
        ];
        
        let indices = vec![
            // Front face
            0, 1, 2, 0, 2, 3,
            // Back face
            4, 5, 6, 4, 6, 7,
            // Left face
            8, 9, 10, 8, 10, 11,
            // Right face
            12, 13, 14, 12, 14, 15,
            // Top face
            16, 17, 18, 16, 18, 19,
            // Bottom face
            20, 21, 22, 20, 22, 23,
        ];
        
        Mesh::from_data(MeshData::new(
            "Cube".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a quad mesh (single-sided plane)
    pub fn quad() -> Mesh {
        let vertices = vec![
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(0.5, -0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(0.5, 0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 1.0)),
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        Mesh::from_data(MeshData::new(
            "Quad".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a sphere mesh
    pub fn sphere(radius: f32, rings: u32, sectors: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let ring_step = std::f32::consts::PI / rings as f32;
        let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
        
        // Generate vertices
        for i in 0..=rings {
            let ring_angle = i as f32 * ring_step;
            let y = radius * ring_angle.cos();
            let ring_radius = radius * ring_angle.sin();
            
            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;
                let x = ring_radius * sector_angle.cos();
                let z = ring_radius * sector_angle.sin();
                
                let position = Vec3::new(x, y, z);
                let normal = position.normalize();
                let uv = Vec2::new(j as f32 / sectors as f32, i as f32 / rings as f32);
                
                vertices.push(Vertex::new(position).with_normal(normal).with_uv(uv));
            }
        }
        
        // Generate indices
        for i in 0..rings {
            for j in 0..sectors {
                let current = i * (sectors + 1) + j;
                let next = current + sectors + 1;
                
                // Two triangles per quad
                indices.extend([current, next, current + 1]);
                indices.extend([current + 1, next, next + 1]);
            }
        }
        
        Mesh::from_data(MeshData::new(
            "Sphere".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a cylinder mesh
    pub fn cylinder(top_radius: f32, bottom_radius: f32, height: f32, sectors: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let half_height = height * 0.5;
        let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
        
        // Generate side vertices
        for i in 0..=sectors {
            let angle = i as f32 * sector_step;
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();
            
            // Top vertex
            let top_pos = Vec3::new(top_radius * cos_angle, half_height, top_radius * sin_angle);
            let top_normal = Vec3::new(cos_angle, 0.0, sin_angle);
            let top_uv = Vec2::new(i as f32 / sectors as f32, 1.0);
            vertices.push(Vertex::new(top_pos).with_normal(top_normal).with_uv(top_uv));
            
            // Bottom vertex
            let bottom_pos = Vec3::new(bottom_radius * cos_angle, -half_height, bottom_radius * sin_angle);
            let bottom_normal = Vec3::new(cos_angle, 0.0, sin_angle);
            let bottom_uv = Vec2::new(i as f32 / sectors as f32, 0.0);
            vertices.push(Vertex::new(bottom_pos).with_normal(bottom_normal).with_uv(bottom_uv));
        }
        
        // Generate side indices
        for i in 0..sectors {
            let top_current = i * 2;
            let bottom_current = i * 2 + 1;
            let top_next = (i + 1) * 2;
            let bottom_next = (i + 1) * 2 + 1;
            
            // Two triangles per side segment
            indices.extend([top_current, bottom_current, top_next]);
            indices.extend([top_next, bottom_current, bottom_next]);
        }
        
        Mesh::from_data(MeshData::new(
            "Cylinder".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a plane mesh with subdivisions
    pub fn plane(width: f32, height: f32, subdivisions_x: u32, subdivisions_y: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices
        for y in 0..=subdivisions_y {
            for x in 0..=subdivisions_x {
                let u = x as f32 / subdivisions_x as f32;
                let v = y as f32 / subdivisions_y as f32;
                
                let position = Vec3::new(
                    (u - 0.5) * width,
                    0.0,
                    (v - 0.5) * height,
                );
                
                let normal = Vec3::Y;
                let uv = Vec2::new(u, v);
                
                vertices.push(Vertex::new(position).with_normal(normal).with_uv(uv));
            }
        }
        
        // Generate indices
        for y in 0..subdivisions_y {
            for x in 0..subdivisions_x {
                let i = y * (subdivisions_x + 1) + x;
                
                // Two triangles per quad
                indices.extend([i, i + 1, i + subdivisions_x + 1]);
                indices.extend([i + 1, i + subdivisions_x + 2, i + subdivisions_x + 1]);
            }
        }
        
        Mesh::from_data(MeshData::new(
            "Plane".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a triangle mesh
    pub fn triangle() -> Mesh {
        let vertices = vec![
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(0.5, -0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(0.0, 0.5, 0.0)).with_normal(Vec3::Z).with_uv(Vec2::new(0.5, 1.0)),
        ];
        
        let indices = vec![0, 1, 2];
        
        Mesh::from_data(MeshData::new(
            "Triangle".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Create a cone mesh
    pub fn cone(radius: f32, height: f32, sectors: u32) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let half_height = height * 0.5;
        let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
        
        // Tip vertex
        vertices.push(Vertex::new(Vec3::new(0.0, half_height, 0.0)).with_normal(Vec3::Y).with_uv(Vec2::new(0.5, 1.0)));
        
        // Base vertices
        for i in 0..=sectors {
            let angle = i as f32 * sector_step;
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            
            let position = Vec3::new(x, -half_height, z);
            let normal = Vec3::new(x, 0.0, z).normalize();
            let uv = Vec2::new(
                0.5 + 0.5 * angle.cos(),
                0.5 + 0.5 * angle.sin(),
            );
            
            vertices.push(Vertex::new(position).with_normal(normal).with_uv(uv));
        }
        
        // Generate side triangles
        for i in 0..sectors {
            indices.extend([0, i + 1, i + 2]);
        }
        
        Mesh::from_data(MeshData::new(
            "Cone".to_string(),
            vertices,
            indices,
        ))
    }
    
    /// Generate mesh for the given primitive type
    pub fn generate(primitive: PrimitiveType) -> Mesh {
        match primitive {
            PrimitiveType::Cube => Self::cube(1.0),
            PrimitiveType::Sphere => Self::sphere(0.5, 16, 32),
            PrimitiveType::Cylinder => Self::cylinder(0.5, 0.5, 1.0, 16),
            PrimitiveType::Cone => Self::cone(0.5, 1.0, 16),
            PrimitiveType::Plane => Self::plane(1.0, 1.0, 1, 1),
            PrimitiveType::Quad => Self::quad(),
            PrimitiveType::Triangle => Self::triangle(),
            PrimitiveType::Capsule => Self::cylinder(0.5, 0.5, 1.0, 16), // Simplified as cylinder for now
            PrimitiveType::Torus => Self::sphere(0.5, 8, 16), // Simplified as sphere for now
        }
    }
}

impl PrimitiveGenerator for DefaultPrimitiveGenerator {
    fn generate(&self, params: &PrimitiveParams) -> engine_geometry_core::Result<Mesh> {
        let mesh = match params {
            PrimitiveParams::Cube(params) => MeshPrimitives::cube(params.size),
            PrimitiveParams::Sphere(params) => MeshPrimitives::sphere(params.radius, params.rings, params.sectors),
            PrimitiveParams::Cylinder(params) => MeshPrimitives::cylinder(params.top_radius, params.bottom_radius, params.height, params.sectors),
            PrimitiveParams::Cone(params) => MeshPrimitives::cone(params.radius, params.height, params.sectors),
            PrimitiveParams::Plane(params) => MeshPrimitives::plane(params.width, params.height, params.subdivisions_x, params.subdivisions_y),
            PrimitiveParams::Quad => MeshPrimitives::quad(),
            PrimitiveParams::Triangle => MeshPrimitives::triangle(),
            PrimitiveParams::Capsule(params) => MeshPrimitives::cylinder(params.top_radius, params.bottom_radius, params.height, params.sectors),
            PrimitiveParams::Torus(params) => MeshPrimitives::sphere(params.radius, params.rings, params.sectors),
        };
        Ok(mesh)
    }
    
    fn supports(&self, _primitive_type: PrimitiveType) -> bool {
        true // Support all primitive types
    }
    
    fn supported_types(&self) -> Vec<PrimitiveType> {
        vec![
            PrimitiveType::Cube,
            PrimitiveType::Sphere,
            PrimitiveType::Cylinder,
            PrimitiveType::Cone,
            PrimitiveType::Plane,
            PrimitiveType::Quad,
            PrimitiveType::Triangle,
            PrimitiveType::Capsule,
            PrimitiveType::Torus,
        ]
    }
}

impl PrimitiveMeshFactory for DefaultPrimitiveGenerator {
    fn cube(size: f32) -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::cube(size))
    }
    
    fn sphere(radius: f32, rings: u32, sectors: u32) -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::sphere(radius, rings, sectors))
    }
    
    fn cylinder(top_radius: f32, bottom_radius: f32, height: f32, sectors: u32) -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::cylinder(top_radius, bottom_radius, height, sectors))
    }
    
    fn cone(radius: f32, height: f32, sectors: u32) -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::cone(radius, height, sectors))
    }
    
    fn plane(width: f32, height: f32, subdivisions_x: u32, subdivisions_y: u32) -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::plane(width, height, subdivisions_x, subdivisions_y))
    }
    
    fn quad() -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::quad())
    }
    
    fn triangle() -> engine_geometry_core::Result<Mesh> {
        Ok(MeshPrimitives::triangle())
    }
}