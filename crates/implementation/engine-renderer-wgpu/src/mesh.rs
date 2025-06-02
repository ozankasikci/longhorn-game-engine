//! Mesh data structures and management

use glam::{Vec3, Vec2};

/// Vertex data structure
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: (f32, f32, f32, f32),
}

/// Mesh representation
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv: Vec2::ZERO,
            color: (1.0, 1.0, 1.0, 1.0),
        }
    }
    
    /// Set vertex normal
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }
    
    /// Set vertex UV coordinates
    pub fn with_uv(mut self, uv: Vec2) -> Self {
        self.uv = uv;
        self
    }
    
    /// Set vertex color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = (r, g, b, a);
        self
    }
}

impl Mesh {
    /// Create a new mesh
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
    
    /// Add a vertex to the mesh
    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vertex);
        index
    }
    
    /// Add a triangle to the mesh
    pub fn add_triangle(&mut self, v0: u32, v1: u32, v2: u32) {
        self.indices.extend([v0, v1, v2]);
    }
    
    /// Create a quad mesh
    pub fn create_quad() -> Self {
        let mut mesh = Self::new("Quad");
        
        // Add vertices
        mesh.add_vertex(Vertex::new(Vec3::new(-0.5, -0.5, 0.0)).with_uv(Vec2::new(0.0, 0.0)));
        mesh.add_vertex(Vertex::new(Vec3::new(0.5, -0.5, 0.0)).with_uv(Vec2::new(1.0, 0.0)));
        mesh.add_vertex(Vertex::new(Vec3::new(0.5, 0.5, 0.0)).with_uv(Vec2::new(1.0, 1.0)));
        mesh.add_vertex(Vertex::new(Vec3::new(-0.5, 0.5, 0.0)).with_uv(Vec2::new(0.0, 1.0)));
        
        // Add triangles
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        
        mesh
    }
}