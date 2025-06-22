use crate::types::{MeshData, Vertex};
use std::collections::HashMap;

pub struct MeshOptimizer;

impl MeshOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn optimize(&self, mut mesh_data: MeshData) -> Result<MeshData, String> {
        // Remove duplicate vertices
        let (unique_vertices, index_map) = self.remove_duplicate_vertices(&mesh_data.vertices);
        
        // Remap indices
        let remapped_indices = mesh_data.indices.iter()
            .map(|&idx| index_map[&idx])
            .collect();
        
        mesh_data.vertices = unique_vertices;
        mesh_data.indices = remapped_indices;
        
        Ok(mesh_data)
    }
    
    fn remove_duplicate_vertices(&self, vertices: &[Vertex]) -> (Vec<Vertex>, HashMap<u32, u32>) {
        let mut unique_vertices = Vec::new();
        let mut vertex_map: HashMap<String, u32> = HashMap::new();
        let mut index_map: HashMap<u32, u32> = HashMap::new();
        
        for (idx, vertex) in vertices.iter().enumerate() {
            let key = self.vertex_key(vertex);
            
            if let Some(&unique_idx) = vertex_map.get(&key) {
                // Duplicate found, map to existing vertex
                index_map.insert(idx as u32, unique_idx);
            } else {
                // New unique vertex
                let unique_idx = unique_vertices.len() as u32;
                unique_vertices.push(vertex.clone());
                vertex_map.insert(key, unique_idx);
                index_map.insert(idx as u32, unique_idx);
            }
        }
        
        (unique_vertices, index_map)
    }
    
    fn vertex_key(&self, vertex: &Vertex) -> String {
        // Create a string key for vertex comparison
        // In a real implementation, you might use a hash or more efficient comparison
        format!(
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            vertex.position[0], vertex.position[1], vertex.position[2],
            vertex.normal[0], vertex.normal[1], vertex.normal[2],
            vertex.tex_coords[0], vertex.tex_coords[1]
        )
    }
}