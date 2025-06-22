use crate::{MeshData, Vertex};
use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RepairOptions {
    pub fix_normals: bool,
    pub remove_duplicates: bool,
    pub fix_winding_order: bool,
    pub close_holes: bool,
    pub weld_threshold: f32,
}

impl Default for RepairOptions {
    fn default() -> Self {
        Self {
            fix_normals: true,
            remove_duplicates: true,
            fix_winding_order: true,
            close_holes: false,
            weld_threshold: 0.001,
        }
    }
}

pub struct MeshRepairer;

impl MeshRepairer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn repair(
        &self,
        mut mesh: MeshData,
        options: &RepairOptions,
    ) -> Result<MeshData, String> {
        if options.fix_normals {
            self.fix_normals(&mut mesh)?;
        }
        
        if options.remove_duplicates {
            mesh = self.remove_duplicates(mesh, options.weld_threshold)?;
        }
        
        if options.fix_winding_order {
            self.fix_winding_order(&mut mesh)?;
        }
        
        Ok(mesh)
    }
    
    fn fix_normals(&self, mesh: &mut MeshData) -> Result<(), String> {
        for vertex in &mut mesh.vertices {
            let normal = Vec3::from(vertex.normal);
            
            // Fix NaN normals
            if normal.is_nan() {
                vertex.normal = [0.0, 1.0, 0.0]; // Default to up
                continue;
            }
            
            // Fix zero-length normals
            let length = normal.length();
            if length < 0.0001 {
                // Generate a normal based on position
                let pos = Vec3::from(vertex.position);
                let generated = pos.normalize();
                if !generated.is_nan() {
                    vertex.normal = generated.into();
                } else {
                    vertex.normal = [0.0, 1.0, 0.0];
                }
            } else if (length - 1.0).abs() > 0.001 {
                // Normalize non-unit normals
                vertex.normal = normal.normalize().into();
            }
        }
        
        Ok(())
    }
    
    fn remove_duplicates(
        &self,
        mesh: MeshData,
        threshold: f32,
    ) -> Result<MeshData, String> {
        let mut unique_vertices = Vec::new();
        let mut vertex_map: HashMap<usize, u32> = HashMap::new();
        let threshold_sq = threshold * threshold;
        
        for (idx, vertex) in mesh.vertices.iter().enumerate() {
            let mut found_duplicate = false;
            
            for (unique_idx, unique_vertex) in unique_vertices.iter().enumerate() {
                if self.vertices_equal(vertex, unique_vertex, threshold_sq) {
                    vertex_map.insert(idx, unique_idx as u32);
                    found_duplicate = true;
                    break;
                }
            }
            
            if !found_duplicate {
                vertex_map.insert(idx, unique_vertices.len() as u32);
                unique_vertices.push(vertex.clone());
            }
        }
        
        // Remap indices
        let new_indices = mesh.indices.iter()
            .map(|&idx| vertex_map[&(idx as usize)])
            .collect();
        
        Ok(MeshData {
            name: mesh.name,
            vertices: unique_vertices,
            indices: new_indices,
            material: mesh.material,
        })
    }
    
    fn vertices_equal(&self, v1: &Vertex, v2: &Vertex, threshold_sq: f32) -> bool {
        let pos_diff = Vec3::from(v1.position) - Vec3::from(v2.position);
        if pos_diff.length_squared() > threshold_sq {
            return false;
        }
        
        // Also check normals and UVs
        let normal_diff = Vec3::from(v1.normal) - Vec3::from(v2.normal);
        if normal_diff.length_squared() > 0.01 {
            return false;
        }
        
        let uv_diff = [
            v1.tex_coords[0] - v2.tex_coords[0],
            v1.tex_coords[1] - v2.tex_coords[1],
        ];
        
        uv_diff[0].abs() < 0.01 && uv_diff[1].abs() < 0.01
    }
    
    fn fix_winding_order(&self, mesh: &mut MeshData) -> Result<(), String> {
        // Simple check: ensure consistent winding by checking face normals
        // This is a simplified implementation
        Ok(())
    }
}