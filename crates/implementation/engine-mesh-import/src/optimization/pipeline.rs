use crate::{MeshData, Vertex};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OptimizationError {
    #[error("Optimization failed: {0}")]
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct OptimizationOptions {
    pub merge_vertices: bool,
    pub optimize_vertex_cache: bool,
    pub remove_unused_vertices: bool,
    pub quantize_positions: bool,
    pub target_index_buffer_size: Option<usize>,
}

impl Default for OptimizationOptions {
    fn default() -> Self {
        Self {
            merge_vertices: true,
            optimize_vertex_cache: true,
            remove_unused_vertices: true,
            quantize_positions: false,
            target_index_buffer_size: None,
        }
    }
}

pub struct OptimizationPipeline;

impl OptimizationPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize(
        &mut self,
        mut mesh: MeshData,
        options: OptimizationOptions,
    ) -> Result<MeshData, OptimizationError> {
        if options.remove_unused_vertices {
            mesh = self.remove_unused_vertices(mesh)?;
        }

        if options.merge_vertices {
            mesh = self.merge_duplicate_vertices(mesh)?;
        }

        if options.optimize_vertex_cache {
            // This would use the VertexCacheOptimizer
            // For now, we'll skip this step
        }

        Ok(mesh)
    }

    fn remove_unused_vertices(&self, mesh: MeshData) -> Result<MeshData, OptimizationError> {
        let mut used_vertices = vec![false; mesh.vertices.len()];
        let mut vertex_remap = vec![0u32; mesh.vertices.len()];

        // Mark used vertices
        for &idx in &mesh.indices {
            used_vertices[idx as usize] = true;
        }

        // Build new vertex list and remap table
        let mut new_vertices = Vec::new();
        let mut new_index = 0u32;

        for (old_index, vertex) in mesh.vertices.iter().enumerate() {
            if used_vertices[old_index] {
                new_vertices.push(vertex.clone());
                vertex_remap[old_index] = new_index;
                new_index += 1;
            }
        }

        // Remap indices
        let new_indices = mesh
            .indices
            .iter()
            .map(|&idx| vertex_remap[idx as usize])
            .collect();

        Ok(MeshData {
            name: mesh.name,
            vertices: new_vertices,
            indices: new_indices,
            material: mesh.material,
        })
    }

    fn merge_duplicate_vertices(&self, mesh: MeshData) -> Result<MeshData, OptimizationError> {
        let mut vertex_map: HashMap<String, u32> = HashMap::new();
        let mut unique_vertices = Vec::new();
        let mut vertex_remap = vec![0u32; mesh.vertices.len()];

        for (idx, vertex) in mesh.vertices.iter().enumerate() {
            let key = self.vertex_key(vertex);

            if let Some(&unique_idx) = vertex_map.get(&key) {
                vertex_remap[idx] = unique_idx;
            } else {
                let unique_idx = unique_vertices.len() as u32;
                unique_vertices.push(vertex.clone());
                vertex_map.insert(key, unique_idx);
                vertex_remap[idx] = unique_idx;
            }
        }

        // Remap indices
        let new_indices = mesh
            .indices
            .iter()
            .map(|&idx| vertex_remap[idx as usize])
            .collect();

        Ok(MeshData {
            name: mesh.name,
            vertices: unique_vertices,
            indices: new_indices,
            material: mesh.material,
        })
    }

    fn vertex_key(&self, vertex: &Vertex) -> String {
        format!(
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            vertex.position[0],
            vertex.position[1],
            vertex.position[2],
            vertex.normal[0],
            vertex.normal[1],
            vertex.normal[2],
            vertex.tex_coords[0],
            vertex.tex_coords[1]
        )
    }
}
