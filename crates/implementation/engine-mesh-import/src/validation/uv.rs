use crate::MeshData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UVError {
    #[error("UV coordinates out of range at vertex {0}: ({1}, {2})")]
    OutOfRange(usize, f32, f32),

    #[error("Overlapping faces in UV space: face {0} and face {1}")]
    OverlappingFaces(usize, usize),

    #[error("Degenerate UV mapping at face {0}")]
    DegenerateMapping(usize),

    #[error("UV seam mismatch")]
    SeamMismatch,
}

pub struct UVValidator;

impl Default for UVValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl UVValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, _mesh: &MeshData) -> Result<(), UVError> {
        // Basic validation always passes, warnings are separate
        Ok(())
    }

    pub fn get_warnings(&self, mesh: &MeshData) -> Vec<UVError> {
        let mut warnings = Vec::new();

        // Check UV range
        for (i, vertex) in mesh.vertices.iter().enumerate() {
            if vertex.tex_coords[0] < 0.0
                || vertex.tex_coords[0] > 1.0
                || vertex.tex_coords[1] < 0.0
                || vertex.tex_coords[1] > 1.0
            {
                warnings.push(UVError::OutOfRange(
                    i,
                    vertex.tex_coords[0],
                    vertex.tex_coords[1],
                ));
            }
        }

        // Check for overlapping triangles in UV space
        let faces: Vec<_> = mesh.indices.chunks(3).collect();

        for (i, face1) in faces.iter().enumerate() {
            if face1.len() != 3 {
                continue;
            }

            let uv1 = [
                mesh.vertices[face1[0] as usize].tex_coords,
                mesh.vertices[face1[1] as usize].tex_coords,
                mesh.vertices[face1[2] as usize].tex_coords,
            ];

            // Check if triangle is degenerate in UV space
            if self.is_degenerate_uv_triangle(&uv1) {
                warnings.push(UVError::DegenerateMapping(i));
                continue;
            }

            for (j, face2) in faces.iter().enumerate().skip(i + 1) {
                if face2.len() != 3 {
                    continue;
                }

                let uv2 = [
                    mesh.vertices[face2[0] as usize].tex_coords,
                    mesh.vertices[face2[1] as usize].tex_coords,
                    mesh.vertices[face2[2] as usize].tex_coords,
                ];

                if self.triangles_overlap_uv(&uv1, &uv2) {
                    warnings.push(UVError::OverlappingFaces(i, j));
                }
            }
        }

        warnings
    }

    fn is_degenerate_uv_triangle(&self, uvs: &[[f32; 2]; 3]) -> bool {
        // Calculate area using cross product
        let v1 = [uvs[1][0] - uvs[0][0], uvs[1][1] - uvs[0][1]];
        let v2 = [uvs[2][0] - uvs[0][0], uvs[2][1] - uvs[0][1]];

        let area = (v1[0] * v2[1] - v1[1] * v2[0]).abs();

        area < 0.0001 // Very small area threshold
    }

    fn triangles_overlap_uv(&self, tri1: &[[f32; 2]; 3], tri2: &[[f32; 2]; 3]) -> bool {
        // Simple AABB check first
        let (min1, max1) = self.triangle_bounds_uv(tri1);
        let (min2, max2) = self.triangle_bounds_uv(tri2);

        if max1[0] < min2[0] || max2[0] < min1[0] || max1[1] < min2[1] || max2[1] < min1[1] {
            return false; // No overlap possible
        }

        // More precise check would go here
        // For now, we'll check if any point of one triangle is inside the other
        for &point in tri1 {
            if self.point_in_triangle_uv(point, tri2) {
                return true;
            }
        }

        for &point in tri2 {
            if self.point_in_triangle_uv(point, tri1) {
                return true;
            }
        }

        false
    }

    fn triangle_bounds_uv(&self, tri: &[[f32; 2]; 3]) -> ([f32; 2], [f32; 2]) {
        let mut min = tri[0];
        let mut max = tri[0];

        for &point in &tri[1..] {
            min[0] = min[0].min(point[0]);
            min[1] = min[1].min(point[1]);
            max[0] = max[0].max(point[0]);
            max[1] = max[1].max(point[1]);
        }

        (min, max)
    }

    fn point_in_triangle_uv(&self, p: [f32; 2], tri: &[[f32; 2]; 3]) -> bool {
        let v0 = [tri[2][0] - tri[0][0], tri[2][1] - tri[0][1]];
        let v1 = [tri[1][0] - tri[0][0], tri[1][1] - tri[0][1]];
        let v2 = [p[0] - tri[0][0], p[1] - tri[0][1]];

        let dot00 = v0[0] * v0[0] + v0[1] * v0[1];
        let dot01 = v0[0] * v1[0] + v0[1] * v1[1];
        let dot02 = v0[0] * v2[0] + v0[1] * v2[1];
        let dot11 = v1[0] * v1[0] + v1[1] * v1[1];
        let dot12 = v1[0] * v2[0] + v1[1] * v2[1];

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0)
    }
}
