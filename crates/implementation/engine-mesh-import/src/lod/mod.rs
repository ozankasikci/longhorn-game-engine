use crate::MeshData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LODError {
    #[error("LOD generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid LOD options")]
    InvalidOptions,
}

#[derive(Debug, Clone)]
pub struct LODLevel {
    pub distance: f32,
    pub quality: f32,
}

#[derive(Debug, Clone)]
pub struct LODOptions {
    pub levels: Vec<LODLevel>,
    pub preserve_boundaries: bool,
    pub preserve_seams: bool,
    pub preserve_uv_boundaries: bool,
}

pub struct LODGenerator;

impl Default for LODGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl LODGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_lods(
        &self,
        base_mesh: &MeshData,
        options: &LODOptions,
    ) -> Result<Vec<MeshData>, LODError> {
        if options.levels.is_empty() {
            return Err(LODError::InvalidOptions);
        }

        let mut lod_meshes = Vec::new();

        for level in &options.levels {
            let simplified = self.simplify_mesh(base_mesh, level.quality)?;
            lod_meshes.push(simplified);
        }

        Ok(lod_meshes)
    }

    fn simplify_mesh(&self, mesh: &MeshData, quality: f32) -> Result<MeshData, LODError> {
        if quality >= 1.0 {
            // Full quality, return copy
            return Ok(mesh.clone());
        }

        // Simple decimation algorithm
        // In a real implementation, you would use quadric error metrics
        let target_triangle_count = ((mesh.indices.len() / 3) as f32 * quality) as usize;

        // For now, just subsample triangles
        let mut new_indices = Vec::new();
        let step = ((mesh.indices.len() / 3) as f32 / target_triangle_count as f32).ceil() as usize;

        for (i, face) in mesh.indices.chunks(3).enumerate() {
            if face.len() == 3 && i % step == 0 {
                new_indices.extend_from_slice(face);
            }
        }

        // Remove unused vertices
        let mut used_vertices = vec![false; mesh.vertices.len()];
        for &idx in &new_indices {
            used_vertices[idx as usize] = true;
        }

        let mut vertex_remap = vec![0u32; mesh.vertices.len()];
        let mut new_vertices = Vec::new();
        let mut new_idx = 0u32;

        for (old_idx, vertex) in mesh.vertices.iter().enumerate() {
            if used_vertices[old_idx] {
                new_vertices.push(vertex.clone());
                vertex_remap[old_idx] = new_idx;
                new_idx += 1;
            }
        }

        // Remap indices
        let remapped_indices = new_indices
            .iter()
            .map(|&idx| vertex_remap[idx as usize])
            .collect();

        Ok(MeshData {
            name: format!("{}_LOD", mesh.name),
            vertices: new_vertices,
            indices: remapped_indices,
            material: mesh.material.clone(),
        })
    }
}
