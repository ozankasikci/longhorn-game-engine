use crate::types::MeshData as ImportMeshData;
use engine_geometry_core::{
    mesh::PrimitiveTopology, BoundingBox, IndexBuffer, Mesh, Vertex, VertexData,
};
use glam::Vec3;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Empty mesh data")]
    EmptyMesh,

    #[error("Invalid vertex data: {0}")]
    InvalidVertexData(String),

    #[error("Failed to build mesh: {0}")]
    BuildError(String),
}

pub struct MeshConverter;

impl Default for MeshConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(&self, import_mesh: &ImportMeshData) -> Result<Mesh, ConversionError> {
        if import_mesh.vertices.is_empty() {
            return Err(ConversionError::EmptyMesh);
        }

        // Convert vertices to engine format
        let vertices: Vec<Vertex> = import_mesh
            .vertices
            .iter()
            .map(|v| Vertex {
                position: Vec3::from(v.position),
                normal: Vec3::from(v.normal),
                uv: v.tex_coords.into(),
                color: v.color,
            })
            .collect();

        // Create vertex data
        let vertex_data = VertexData::Standard(vertices.clone());

        // Create index buffer
        let index_buffer = if !import_mesh.indices.is_empty() {
            Some(IndexBuffer::U32(import_mesh.indices.clone()))
        } else {
            None
        };

        // Calculate bounds
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for vertex in &vertices {
            min = min.min(vertex.position);
            max = max.max(vertex.position);
        }

        let bounds = BoundingBox { min, max };

        // Create material slots
        let material_slots = vec![];

        Ok(Mesh {
            name: import_mesh.name.clone(),
            vertex_data,
            index_buffer,
            bounds,
            topology: PrimitiveTopology::TriangleList,
            material_slots,
        })
    }

    pub fn convert_multiple(
        &self,
        meshes: &[ImportMeshData],
    ) -> Result<Vec<Mesh>, ConversionError> {
        meshes
            .iter()
            .map(|mesh_data| self.convert(mesh_data))
            .collect()
    }
}
