//! Vertex data structures and attribute definitions

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// Standard vertex data structure for 3D meshes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
}

/// Vertex attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexAttribute {
    Position,
    Normal,
    Tangent,
    Bitangent,
    TexCoord0,
    TexCoord1,
    Color0,
    Color1,
    Joints0,
    Weights0,
    Custom(u32),
}

/// Vertex data variants for different use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VertexData {
    /// Standard 3D vertex with position, normal, UV, color
    Standard(Vec<Vertex>),
    /// Skinned mesh vertex with bone weights
    Skinned(Vec<SkinnedVertex>),
    /// Simple 2D vertex for UI and sprites
    Simple2D(Vec<SimpleVertex2D>),
    /// Particle vertex for particle systems
    Particle(Vec<ParticleVertex>),
    /// Custom vertex format
    Custom {
        data: Vec<u8>,
        stride: u32,
        attributes: Vec<VertexAttributeDescriptor>,
    },
}

/// Skinned vertex for animated meshes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct SkinnedVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
    pub bone_indices: [u32; 4],
    pub bone_weights: [f32; 4],
}

/// Simple 2D vertex for UI and sprites
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct SimpleVertex2D {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: [f32; 4],
}

/// Particle vertex for particle systems
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
pub struct ParticleVertex {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: [f32; 4],
    pub size: f32,
    pub rotation: f32,
    pub life: f32,
    pub _padding: f32,
}

/// Vertex attribute descriptor for custom formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttributeDescriptor {
    pub attribute: VertexAttribute,
    pub format: VertexAttributeFormat,
    pub offset: u32,
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexAttributeFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
    Unorm8x4,
    Snorm8x4,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            uv: Vec2::ZERO,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Vertex {
    /// Create a new vertex with position
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set vertex normal
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal.normalize();
        self
    }

    /// Set vertex UV coordinates
    pub fn with_uv(mut self, uv: Vec2) -> Self {
        self.uv = uv;
        self
    }

    /// Set vertex color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// Set vertex color from Vec4
    pub fn with_color_vec4(mut self, color: Vec4) -> Self {
        self.color = color.to_array();
        self
    }

    /// Transform the vertex position and normal by a matrix
    pub fn transform(&mut self, transform: &glam::Mat4) {
        let pos = transform.transform_point3(self.position);
        let normal = transform.transform_vector3(self.normal).normalize();
        self.position = pos;
        self.normal = normal;
    }

    /// Create a transformed copy of this vertex
    pub fn transformed(&self, transform: &glam::Mat4) -> Self {
        let mut vertex = *self;
        vertex.transform(transform);
        vertex
    }
}

impl SkinnedVertex {
    /// Create a new skinned vertex
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv: Vec2::ZERO,
            color: [1.0, 1.0, 1.0, 1.0],
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Set bone weights and indices
    pub fn with_bones(mut self, indices: [u32; 4], weights: [f32; 4]) -> Self {
        self.bone_indices = indices;
        self.bone_weights = weights;
        self
    }

    /// Normalize bone weights to sum to 1.0
    pub fn normalize_weights(&mut self) {
        let sum = self.bone_weights.iter().sum::<f32>();
        if sum > 0.0 {
            for weight in &mut self.bone_weights {
                *weight /= sum;
            }
        }
    }
}

impl SimpleVertex2D {
    /// Create a new 2D vertex
    pub fn new(position: Vec2, uv: Vec2) -> Self {
        Self {
            position,
            uv,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set vertex color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
}

impl ParticleVertex {
    /// Create a new particle vertex
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            rotation: 0.0,
            life: 1.0,
            _padding: 0.0,
        }
    }

    /// Set particle velocity
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set particle size
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set particle life
    pub fn with_life(mut self, life: f32) -> Self {
        self.life = life;
        self
    }
}

impl VertexData {
    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        match self {
            VertexData::Standard(vertices) => vertices.len(),
            VertexData::Skinned(vertices) => vertices.len(),
            VertexData::Simple2D(vertices) => vertices.len(),
            VertexData::Particle(vertices) => vertices.len(),
            VertexData::Custom { data, stride, .. } => {
                if *stride > 0 {
                    data.len() / *stride as usize
                } else {
                    0
                }
            }
        }
    }

    /// Get the vertex stride in bytes
    pub fn vertex_stride(&self) -> u32 {
        match self {
            VertexData::Standard(_) => std::mem::size_of::<Vertex>() as u32,
            VertexData::Skinned(_) => std::mem::size_of::<SkinnedVertex>() as u32,
            VertexData::Simple2D(_) => std::mem::size_of::<SimpleVertex2D>() as u32,
            VertexData::Particle(_) => std::mem::size_of::<ParticleVertex>() as u32,
            VertexData::Custom { stride, .. } => *stride,
        }
    }

    /// Get the raw data as bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            VertexData::Standard(vertices) => bytemuck::cast_slice(vertices),
            VertexData::Skinned(vertices) => bytemuck::cast_slice(vertices),
            VertexData::Simple2D(vertices) => bytemuck::cast_slice(vertices),
            VertexData::Particle(vertices) => bytemuck::cast_slice(vertices),
            VertexData::Custom { data, .. } => data,
        }
    }

    /// Transform all vertices by a matrix (where applicable)
    pub fn transform(&mut self, transform: &glam::Mat4) {
        match self {
            VertexData::Standard(vertices) => {
                for vertex in vertices {
                    vertex.transform(transform);
                }
            }
            VertexData::Skinned(vertices) => {
                for vertex in vertices {
                    let pos = transform.transform_point3(vertex.position);
                    let normal = transform.transform_vector3(vertex.normal).normalize();
                    vertex.position = pos;
                    vertex.normal = normal;
                }
            }
            VertexData::Particle(vertices) => {
                for vertex in vertices {
                    vertex.position = transform.transform_point3(vertex.position);
                    vertex.velocity = transform.transform_vector3(vertex.velocity);
                }
            }
            _ => {} // 2D and custom formats don't support 3D transforms
        }
    }
}

impl VertexAttributeFormat {
    /// Get the size in bytes of this attribute format
    pub fn size(&self) -> u32 {
        match self {
            Self::Float32 => 4,
            Self::Float32x2 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
            Self::Uint32 => 4,
            Self::Uint32x2 => 8,
            Self::Uint32x3 => 12,
            Self::Uint32x4 => 16,
            Self::Sint32 => 4,
            Self::Sint32x2 => 8,
            Self::Sint32x3 => 12,
            Self::Sint32x4 => 16,
            Self::Uint16x2 => 4,
            Self::Uint16x4 => 8,
            Self::Sint16x2 => 4,
            Self::Sint16x4 => 8,
            Self::Unorm8x4 => 4,
            Self::Snorm8x4 => 4,
        }
    }

    /// Get the component count
    pub fn component_count(&self) -> u32 {
        match self {
            Self::Float32 | Self::Uint32 | Self::Sint32 => 1,
            Self::Float32x2 | Self::Uint32x2 | Self::Sint32x2 | Self::Uint16x2 | Self::Sint16x2 => {
                2
            }
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 3,
            Self::Float32x4
            | Self::Uint32x4
            | Self::Sint32x4
            | Self::Uint16x4
            | Self::Sint16x4
            | Self::Unorm8x4
            | Self::Snorm8x4 => 4,
        }
    }
}
