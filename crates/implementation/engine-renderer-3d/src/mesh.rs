//! Mesh data structures and management

use bytemuck::{Pod, Zeroable};

/// Re-export from renderer for convenience
pub use crate::renderer::Vertex;

/// Mesh data for rendering
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub name: String,
}

impl Mesh {
    /// Create a new mesh
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices,
            indices,
            name,
        }
    }

    /// Create a simple triangle mesh for testing
    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            }, // Top - Red
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            }, // Bottom left - Green
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            }, // Bottom right - Blue
        ];

        let indices = vec![0, 1, 2];

        Self::new("Triangle".to_string(), vertices, indices)
    }

    /// Create a cube mesh
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex {
                position: [-0.5, -0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            // Back face
            Vertex {
                position: [-0.5, -0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
        ];

        let indices = vec![
            // Front face
            0, 1, 2, 2, 3, 0, // Back face
            4, 5, 6, 6, 7, 4, // Left face
            4, 0, 3, 3, 5, 4, // Right face
            1, 7, 6, 6, 2, 1, // Top face
            3, 2, 6, 6, 5, 3, // Bottom face
            4, 7, 1, 1, 0, 4,
        ];

        Self::new("Cube".to_string(), vertices, indices)
    }
}
