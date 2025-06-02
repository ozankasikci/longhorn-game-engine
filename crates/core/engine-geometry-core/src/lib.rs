//! Core geometry and mesh data structures for the mobile game engine
//! 
//! This crate provides pure geometric data structures and spatial mathematics
//! with no implementation dependencies. All geometry processing and mesh
//! generation is handled here.

pub mod mesh;
pub mod bounds;
pub mod spatial;
pub mod primitives;
pub mod vertex;

// Re-export core types
pub use mesh::{Mesh, MeshHandle, IndexBuffer, MeshData};
pub use bounds::{BoundingBox, BoundingSphere, Bounds};
pub use spatial::{Ray, Plane, Frustum, SpatialQuery};
pub use primitives::{PrimitiveType, MeshPrimitives};
pub use vertex::{Vertex, VertexData, VertexAttribute};

/// Handle type for mesh resources
pub type Handle = u64;

/// Handle type for buffer resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);