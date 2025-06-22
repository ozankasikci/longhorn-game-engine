use crate::types::{MeshData, Bounds};

pub fn calculate_bounds(mesh_data: &MeshData) -> Bounds {
    if mesh_data.vertices.is_empty() {
        return Bounds {
            min: [0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0],
        };
    }
    
    let mut min = mesh_data.vertices[0].position;
    let mut max = mesh_data.vertices[0].position;
    
    for vertex in &mesh_data.vertices[1..] {
        for i in 0..3 {
            min[i] = min[i].min(vertex.position[i]);
            max[i] = max[i].max(vertex.position[i]);
        }
    }
    
    Bounds { min, max }
}