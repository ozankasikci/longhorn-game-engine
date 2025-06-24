use crate::MeshData;
use glam::Vec3;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NormalError {
    #[error("Normal generation failed: {0}")]
    GenerationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmoothingMethod {
    Flat,
    Smooth,
    AngleBased { threshold_degrees: f32 },
}

#[derive(Debug, Clone)]
pub struct NormalOptions {
    pub method: SmoothingMethod,
    pub weight_by_area: bool,
    pub normalize: bool,
}

impl Default for NormalOptions {
    fn default() -> Self {
        Self {
            method: SmoothingMethod::Smooth,
            weight_by_area: true,
            normalize: true,
        }
    }
}

pub struct NormalProcessor;

impl NormalProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_normals(
        &self,
        mesh: &mut MeshData,
        options: &NormalOptions,
    ) -> Result<(), NormalError> {
        // Reset all normals
        for vertex in &mut mesh.vertices {
            vertex.normal = [0.0, 0.0, 0.0];
        }

        match options.method {
            SmoothingMethod::Flat => self.generate_flat_normals(mesh, options),
            SmoothingMethod::Smooth => self.generate_smooth_normals(mesh, options),
            SmoothingMethod::AngleBased { threshold_degrees } => {
                self.generate_angle_based_normals(mesh, threshold_degrees, options)
            }
        }
    }

    fn generate_flat_normals(
        &self,
        mesh: &mut MeshData,
        options: &NormalOptions,
    ) -> Result<(), NormalError> {
        // For flat shading, each vertex of a face gets the face normal
        let mut new_vertices = Vec::new();
        let mut new_indices = Vec::new();

        for face in mesh.indices.chunks(3) {
            if face.len() != 3 {
                continue;
            }

            let v0 = Vec3::from(mesh.vertices[face[0] as usize].position);
            let v1 = Vec3::from(mesh.vertices[face[1] as usize].position);
            let v2 = Vec3::from(mesh.vertices[face[2] as usize].position);

            let normal = (v1 - v0).cross(v2 - v0).normalize();

            for &idx in face {
                let mut vertex = mesh.vertices[idx as usize].clone();
                vertex.normal = normal.into();

                let new_idx = new_vertices.len() as u32;
                new_vertices.push(vertex);
                new_indices.push(new_idx);
            }
        }

        mesh.vertices = new_vertices;
        mesh.indices = new_indices;

        Ok(())
    }

    fn generate_smooth_normals(
        &self,
        mesh: &mut MeshData,
        options: &NormalOptions,
    ) -> Result<(), NormalError> {
        // Calculate face normals and accumulate to vertices
        for face in mesh.indices.chunks(3) {
            if face.len() != 3 {
                continue;
            }

            let v0 = Vec3::from(mesh.vertices[face[0] as usize].position);
            let v1 = Vec3::from(mesh.vertices[face[1] as usize].position);
            let v2 = Vec3::from(mesh.vertices[face[2] as usize].position);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let face_normal = edge1.cross(edge2);

            let weight = if options.weight_by_area {
                face_normal.length() * 0.5 // Triangle area
            } else {
                1.0
            };

            let weighted_normal = face_normal.normalize() * weight;

            // Add to each vertex
            for &idx in face {
                let vertex = &mut mesh.vertices[idx as usize];
                vertex.normal[0] += weighted_normal.x;
                vertex.normal[1] += weighted_normal.y;
                vertex.normal[2] += weighted_normal.z;
            }
        }

        // Normalize all vertex normals
        if options.normalize {
            for vertex in &mut mesh.vertices {
                let normal = Vec3::from(vertex.normal).normalize();
                if !normal.is_nan() {
                    vertex.normal = normal.into();
                }
            }
        }

        Ok(())
    }

    fn generate_angle_based_normals(
        &self,
        mesh: &mut MeshData,
        threshold_degrees: f32,
        options: &NormalOptions,
    ) -> Result<(), NormalError> {
        // First generate smooth normals
        self.generate_smooth_normals(mesh, options)?;

        // Then split vertices where angle exceeds threshold
        // This is a simplified implementation
        // A full implementation would properly handle smoothing groups

        Ok(())
    }
}
