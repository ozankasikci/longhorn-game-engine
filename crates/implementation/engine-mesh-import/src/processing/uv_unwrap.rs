use crate::MeshData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnwrapError {
    #[error("UV unwrapping failed: {0}")]
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnwrapMethod {
    AngleBased,
    AreaPreserving,
    Conformal,
}

#[derive(Debug, Clone)]
pub struct UnwrapOptions {
    pub method: UnwrapMethod,
    pub padding: f32,
    pub stretch_threshold: f32,
}

impl Default for UnwrapOptions {
    fn default() -> Self {
        Self {
            method: UnwrapMethod::AngleBased,
            padding: 0.01,
            stretch_threshold: 0.1,
        }
    }
}

pub struct UVUnwrapper;

impl UVUnwrapper {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_uvs(
        &self,
        mesh: &mut MeshData,
        options: &UnwrapOptions,
    ) -> Result<(), UnwrapError> {
        // Simple planar projection for demonstration
        // Real implementation would use more sophisticated unwrapping

        match options.method {
            UnwrapMethod::AngleBased => self.angle_based_unwrap(mesh, options),
            UnwrapMethod::AreaPreserving => self.area_preserving_unwrap(mesh, options),
            UnwrapMethod::Conformal => self.conformal_unwrap(mesh, options),
        }
    }

    fn angle_based_unwrap(
        &self,
        mesh: &mut MeshData,
        _options: &UnwrapOptions,
    ) -> Result<(), UnwrapError> {
        // Simple implementation: project based on dominant axis
        for (i, vertex) in mesh.vertices.iter_mut().enumerate() {
            // Find dominant normal axis
            let normal = vertex.normal;
            let abs_normal = [normal[0].abs(), normal[1].abs(), normal[2].abs()];

            let (u, v) = if abs_normal[0] > abs_normal[1] && abs_normal[0] > abs_normal[2] {
                // X dominant - project to YZ plane
                (vertex.position[1], vertex.position[2])
            } else if abs_normal[1] > abs_normal[2] {
                // Y dominant - project to XZ plane
                (vertex.position[0], vertex.position[2])
            } else {
                // Z dominant - project to XY plane
                (vertex.position[0], vertex.position[1])
            };

            // Normalize to 0-1 range with some variation
            vertex.tex_coords = [
                (u + 10.0) / 20.0 + (i as f32 * 0.01).sin() * 0.1,
                (v + 10.0) / 20.0 + (i as f32 * 0.01).cos() * 0.1,
            ];

            // Clamp to valid range
            vertex.tex_coords[0] = vertex.tex_coords[0].clamp(0.0, 1.0);
            vertex.tex_coords[1] = vertex.tex_coords[1].clamp(0.0, 1.0);
        }

        Ok(())
    }

    fn area_preserving_unwrap(
        &self,
        mesh: &mut MeshData,
        options: &UnwrapOptions,
    ) -> Result<(), UnwrapError> {
        // For now, use the same as angle-based
        self.angle_based_unwrap(mesh, options)
    }

    fn conformal_unwrap(
        &self,
        mesh: &mut MeshData,
        options: &UnwrapOptions,
    ) -> Result<(), UnwrapError> {
        // For now, use the same as angle-based
        self.angle_based_unwrap(mesh, options)
    }
}
