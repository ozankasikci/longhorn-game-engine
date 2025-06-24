//! Core geometry and mesh data structures for the mobile game engine
//!
//! This crate provides pure geometric data structures and spatial mathematics
//! with no implementation dependencies. All geometry processing and mesh
//! generation is handled here.

pub mod bounds;
pub mod mesh;
pub mod primitives;
pub mod spatial;
pub mod vertex;

// Re-export core types
pub use bounds::{BoundingBox, BoundingSphere, Bounds};
pub use mesh::{IndexBuffer, Mesh, MeshData, MeshHandle, SubMesh};
pub use primitives::*;
pub use spatial::{Frustum, Plane, Ray, SpatialQuery};
pub use vertex::{Vertex, VertexAttribute, VertexData};

/// Common error type for geometry operations
pub type Result<T> = std::result::Result<T, GeometryError>;

/// Geometry-related errors
#[derive(Debug, thiserror::Error)]
pub enum GeometryError {
    #[error("Invalid mesh data: {0}")]
    InvalidMeshData(String),

    #[error("Primitive generation failed: {0}")]
    PrimitiveGenerationFailed(String),

    #[error("Unsupported primitive type: {0:?}")]
    UnsupportedPrimitiveType(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Handle type for mesh resources
pub type Handle = u64;

/// Handle type for buffer resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);
