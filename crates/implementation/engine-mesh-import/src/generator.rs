use crate::types::MeshData;
use glam::Vec3;

pub struct NormalGenerator;

impl NormalGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_normals(&self, mesh_data: &mut MeshData) {
        // First, reset all normals to zero
        for vertex in &mut mesh_data.vertices {
            vertex.normal = [0.0, 0.0, 0.0];
        }
        
        // Calculate face normals and accumulate to vertices
        for face in mesh_data.indices.chunks(3) {
            if face.len() != 3 {
                continue;
            }
            
            let idx0 = face[0] as usize;
            let idx1 = face[1] as usize;
            let idx2 = face[2] as usize;
            
            if idx0 >= mesh_data.vertices.len() || 
               idx1 >= mesh_data.vertices.len() || 
               idx2 >= mesh_data.vertices.len() {
                continue;
            }
            
            let v0 = Vec3::from(mesh_data.vertices[idx0].position);
            let v1 = Vec3::from(mesh_data.vertices[idx1].position);
            let v2 = Vec3::from(mesh_data.vertices[idx2].position);
            
            // Calculate face normal
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let face_normal = edge1.cross(edge2).normalize();
            
            // Add face normal to each vertex
            if !face_normal.is_nan() {
                for i in 0..3 {
                    let idx = face[i] as usize;
                    let vertex = &mut mesh_data.vertices[idx];
                    vertex.normal[0] += face_normal.x;
                    vertex.normal[1] += face_normal.y;
                    vertex.normal[2] += face_normal.z;
                }
            }
        }
        
        // Normalize all vertex normals
        for vertex in &mut mesh_data.vertices {
            let normal = Vec3::from(vertex.normal).normalize();
            if !normal.is_nan() {
                vertex.normal = normal.into();
            } else {
                // Default to up vector if normalization fails
                vertex.normal = [0.0, 1.0, 0.0];
            }
        }
    }
}