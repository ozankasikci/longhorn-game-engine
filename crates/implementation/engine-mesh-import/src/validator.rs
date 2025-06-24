use crate::types::MeshData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Mesh has no vertices")]
    NoVertices,

    #[error("Mesh has no indices")]
    NoIndices,

    #[error("Invalid index: {0} exceeds vertex count {1}")]
    InvalidIndex(u32, usize),

    #[error("Degenerate triangle detected")]
    DegenerateTriangle,

    #[error("Invalid normal: zero length or NaN")]
    InvalidNormal,

    #[error("Invalid UV coordinates: outside 0-1 range")]
    InvalidUV,
}

pub struct MeshValidator;

impl Default for MeshValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, mesh_data: &MeshData) -> Result<(), ValidationError> {
        // Check for empty mesh
        if mesh_data.vertices.is_empty() {
            return Err(ValidationError::NoVertices);
        }

        // Check indices
        if !mesh_data.indices.is_empty() {
            let vertex_count = mesh_data.vertices.len();

            for &index in &mesh_data.indices {
                if index as usize >= vertex_count {
                    return Err(ValidationError::InvalidIndex(index, vertex_count));
                }
            }

            // Check for degenerate triangles
            for chunk in mesh_data.indices.chunks(3) {
                if chunk.len() == 3
                    && (chunk[0] == chunk[1] || chunk[1] == chunk[2] || chunk[0] == chunk[2])
                {
                    return Err(ValidationError::DegenerateTriangle);
                }
            }
        }

        // Validate vertex data
        for vertex in &mesh_data.vertices {
            // Check normals
            let normal_length_sq = vertex.normal[0] * vertex.normal[0]
                + vertex.normal[1] * vertex.normal[1]
                + vertex.normal[2] * vertex.normal[2];

            if normal_length_sq > 0.0 && !(0.9..=1.1).contains(&normal_length_sq) {
                // Allow some tolerance for normalized normals
                return Err(ValidationError::InvalidNormal);
            }

            // Check for NaN
            if vertex.normal.iter().any(|&n| n.is_nan()) {
                return Err(ValidationError::InvalidNormal);
            }
        }

        Ok(())
    }
}
