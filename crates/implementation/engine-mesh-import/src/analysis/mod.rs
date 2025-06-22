use crate::MeshData;
use glam::Vec3;

#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub average_edge_length: f32,
    pub min_angle_degrees: f32,
    pub max_angle_degrees: f32,
    pub degenerate_triangles: Vec<usize>,
    pub thin_triangles: Vec<usize>,
}

pub struct MeshAnalyzer;

impl MeshAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(&self, mesh: &MeshData) -> Result<QualityMetrics, String> {
        let mut metrics = QualityMetrics {
            vertex_count: mesh.vertices.len(),
            triangle_count: mesh.indices.len() / 3,
            average_edge_length: 0.0,
            min_angle_degrees: 180.0,
            max_angle_degrees: 0.0,
            degenerate_triangles: Vec::new(),
            thin_triangles: Vec::new(),
        };
        
        let mut total_edge_length = 0.0;
        let mut edge_count = 0;
        
        for (face_idx, face) in mesh.indices.chunks(3).enumerate() {
            if face.len() != 3 {
                continue;
            }
            
            let v0 = Vec3::from(mesh.vertices[face[0] as usize].position);
            let v1 = Vec3::from(mesh.vertices[face[1] as usize].position);
            let v2 = Vec3::from(mesh.vertices[face[2] as usize].position);
            
            // Calculate edge lengths
            let edge_lengths = [
                (v1 - v0).length(),
                (v2 - v1).length(),
                (v0 - v2).length(),
            ];
            
            for &len in &edge_lengths {
                total_edge_length += len;
                edge_count += 1;
            }
            
            // Check for degenerate triangles
            let area = (v1 - v0).cross(v2 - v0).length() * 0.5;
            if area < 0.0001 {
                metrics.degenerate_triangles.push(face_idx);
                continue;
            }
            
            // Calculate angles
            let angles = [
                self.angle_between(v1 - v0, v2 - v0),
                self.angle_between(v0 - v1, v2 - v1),
                self.angle_between(v0 - v2, v1 - v2),
            ];
            
            for &angle in &angles {
                metrics.min_angle_degrees = metrics.min_angle_degrees.min(angle);
                metrics.max_angle_degrees = metrics.max_angle_degrees.max(angle);
            }
            
            // Check for thin triangles (any angle < 5 degrees)
            if angles.iter().any(|&a| a < 5.0) {
                metrics.thin_triangles.push(face_idx);
            }
        }
        
        if edge_count > 0 {
            metrics.average_edge_length = total_edge_length / edge_count as f32;
        }
        
        Ok(metrics)
    }
    
    fn angle_between(&self, v1: Vec3, v2: Vec3) -> f32 {
        let dot = v1.normalize().dot(v2.normalize());
        let angle_rad = dot.clamp(-1.0, 1.0).acos();
        angle_rad.to_degrees()
    }
}