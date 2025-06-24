//! Mesh data structures and management

use crate::{BoundingBox, GeometryError, Result, Vertex, VertexData};
use engine_resource_core::Resource;
use glam::Mat4;
use serde::{Deserialize, Serialize};

/// Handle for mesh resources
pub type MeshHandle = u64;

/// Index buffer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexBuffer {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

/// Complete mesh representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertex_data: VertexData,
    pub index_buffer: Option<IndexBuffer>,
    pub bounds: BoundingBox,
    pub topology: PrimitiveTopology,
    pub material_slots: Vec<MaterialSlot>,
}

/// Material slot for submeshes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialSlot {
    pub name: String,
    pub material_index: u32,
    pub start_index: u32,
    pub index_count: u32,
}

/// Mesh data for creation and storage
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub name: String,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

/// Submesh definition for multi-material support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubMesh {
    /// Start index in the index buffer
    pub start_index: u32,
    /// Number of indices in this submesh
    pub index_count: u32,
    /// Material slot index for this submesh
    pub material_index: u32,
}

impl MeshData {
    /// Create new mesh data with single submesh
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let mut mesh_data = Self {
            name,
            vertices,
            indices: indices.clone(),
            submeshes: vec![SubMesh {
                start_index: 0,
                index_count: indices.len() as u32,
                material_index: 0,
            }],
            bounds: BoundingBox::empty(),
        };
        mesh_data.calculate_bounds();
        mesh_data
    }

    /// Calculate bounds from vertices
    pub fn calculate_bounds(&mut self) {
        if self.vertices.is_empty() {
            self.bounds = BoundingBox::empty();
            return;
        }

        let mut min = self.vertices[0].position;
        let mut max = self.vertices[0].position;

        for vertex in &self.vertices {
            min = min.min(vertex.position);
            max = max.max(vertex.position);
        }

        self.bounds = BoundingBox::new(min, max);
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get index count
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Calculate memory usage
    pub fn memory_usage(&self) -> usize {
        let vertex_size = self.vertices.len() * std::mem::size_of::<Vertex>();
        let index_size = self.indices.len() * std::mem::size_of::<u32>();
        let submesh_size = self.submeshes.len() * std::mem::size_of::<SubMesh>();
        vertex_size + index_size + submesh_size
    }
}

impl Resource for MeshData {
    fn resource_type_name() -> &'static str {
        "MeshData"
    }

    fn memory_size(&self) -> usize {
        self.memory_usage()
    }
}

/// Primitive topology for mesh rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleStrip,
    TriangleFan,
    LineList,
    LineStrip,
    PointList,
}

// MeshValidationError removed - using GeometryError instead

impl Default for Mesh {
    fn default() -> Self {
        Self::new("Default")
    }
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertex_data: VertexData::Standard(Vec::new()),
            index_buffer: None,
            bounds: BoundingBox::empty(),
            topology: PrimitiveTopology::TriangleList,
            material_slots: Vec::new(),
        }
    }

    /// Create a mesh from vertex and index data
    pub fn from_data(data: MeshData) -> Self {
        // Convert submeshes to material slots
        let material_slots = data
            .submeshes
            .iter()
            .map(|submesh| MaterialSlot {
                name: format!("Material_{}", submesh.material_index),
                material_index: submesh.material_index,
                start_index: submesh.start_index,
                index_count: submesh.index_count,
            })
            .collect();

        Self {
            name: data.name,
            vertex_data: VertexData::Standard(data.vertices),
            index_buffer: if data.indices.is_empty() {
                None
            } else {
                Some(IndexBuffer::U32(data.indices))
            },
            bounds: data.bounds,
            topology: PrimitiveTopology::TriangleList,
            material_slots,
        }
    }

    /// Add a vertex to the mesh (only works with Standard vertex data)
    pub fn add_vertex(&mut self, vertex: Vertex) -> Result<u32> {
        match &mut self.vertex_data {
            VertexData::Standard(vertices) => {
                let index = vertices.len() as u32;
                vertices.push(vertex);
                self.update_bounds();
                Ok(index)
            }
            _ => Err(GeometryError::InvalidMeshData(
                "Can only add vertices to Standard vertex data".to_string(),
            )),
        }
    }

    /// Add a triangle to the mesh
    pub fn add_triangle(&mut self, v0: u32, v1: u32, v2: u32) -> Result<()> {
        let vertex_count = self.vertex_count() as u32;

        if v0 >= vertex_count || v1 >= vertex_count || v2 >= vertex_count {
            return Err(GeometryError::InvalidMeshData(
                "Vertex index out of bounds".to_string(),
            ));
        }

        let indices = match &mut self.index_buffer {
            Some(IndexBuffer::U32(ref mut indices)) => indices,
            Some(IndexBuffer::U16(_)) => {
                return Err(GeometryError::InvalidMeshData(
                    "Cannot add to U16 index buffer".to_string(),
                ))
            }
            None => {
                self.index_buffer = Some(IndexBuffer::U32(Vec::new()));
                match &mut self.index_buffer {
                    Some(IndexBuffer::U32(ref mut indices)) => indices,
                    _ => unreachable!(),
                }
            }
        };

        indices.extend([v0, v1, v2]);
        Ok(())
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertex_data.vertex_count()
    }

    /// Get the number of indices
    pub fn index_count(&self) -> usize {
        match &self.index_buffer {
            Some(IndexBuffer::U16(indices)) => indices.len(),
            Some(IndexBuffer::U32(indices)) => indices.len(),
            None => 0,
        }
    }

    /// Get the number of triangles (assuming triangle list topology)
    pub fn triangle_count(&self) -> usize {
        match self.topology {
            PrimitiveTopology::TriangleList => {
                if self.has_indices() {
                    self.index_count() / 3
                } else {
                    self.vertex_count() / 3
                }
            }
            PrimitiveTopology::TriangleStrip | PrimitiveTopology::TriangleFan => {
                if self.has_indices() {
                    self.index_count().saturating_sub(2)
                } else {
                    self.vertex_count().saturating_sub(2)
                }
            }
            _ => 0, // Lines and points don't have triangles
        }
    }

    /// Check if this mesh has index data
    pub fn has_indices(&self) -> bool {
        self.index_buffer.is_some()
    }

    /// Get the number of primitives based on topology
    pub fn primitive_count(&self) -> usize {
        let index_count = if self.index_count() > 0 {
            self.index_count()
        } else {
            self.vertex_count()
        };

        match self.topology {
            PrimitiveTopology::TriangleList => index_count / 3,
            PrimitiveTopology::TriangleStrip | PrimitiveTopology::TriangleFan => {
                if index_count >= 3 {
                    index_count - 2
                } else {
                    0
                }
            }
            PrimitiveTopology::LineList => index_count / 2,
            PrimitiveTopology::LineStrip => {
                if index_count >= 2 {
                    index_count - 1
                } else {
                    0
                }
            }
            PrimitiveTopology::PointList => index_count,
        }
    }

    /// Update bounding box from vertices
    pub fn update_bounds(&mut self) {
        if self.vertex_count() == 0 {
            self.bounds = BoundingBox::empty();
            return;
        }

        match &self.vertex_data {
            VertexData::Standard(vertices) => {
                let mut min = vertices[0].position;
                let mut max = vertices[0].position;

                for vertex in vertices {
                    min = min.min(vertex.position);
                    max = max.max(vertex.position);
                }

                self.bounds = BoundingBox::new(min, max);
            }
            VertexData::Skinned(vertices) => {
                let mut min = vertices[0].position;
                let mut max = vertices[0].position;

                for vertex in vertices {
                    min = min.min(vertex.position);
                    max = max.max(vertex.position);
                }

                self.bounds = BoundingBox::new(min, max);
            }
            _ => {
                // For other vertex types, we can't easily calculate bounds
                self.bounds = BoundingBox::empty();
            }
        }
    }

    /// Validate mesh data
    pub fn validate(&self) -> Result<()> {
        // Check for empty mesh
        if self.vertex_count() == 0 {
            return Err(GeometryError::InvalidMeshData("Empty mesh".to_string()));
        }

        // Validate indices
        if let Some(ref index_buffer) = self.index_buffer {
            let vertex_count = self.vertex_count() as u32;

            match index_buffer {
                IndexBuffer::U16(indices) => {
                    for &index in indices {
                        if index as u32 >= vertex_count {
                            return Err(GeometryError::InvalidMeshData(format!(
                                "Index {} out of bounds for {} vertices",
                                index, vertex_count
                            )));
                        }
                    }
                }
                IndexBuffer::U32(indices) => {
                    for &index in indices {
                        if index >= vertex_count {
                            return Err(GeometryError::InvalidMeshData(format!(
                                "Index {} out of bounds for {} vertices",
                                index, vertex_count
                            )));
                        }
                    }
                }
            }

            // Check topology requirements
            let index_count = self.index_count();
            match self.topology {
                PrimitiveTopology::TriangleList => {
                    if index_count % 3 != 0 {
                        return Err(GeometryError::InvalidMeshData(
                            "Triangle list must have index count divisible by 3".to_string(),
                        ));
                    }
                }
                PrimitiveTopology::TriangleStrip | PrimitiveTopology::TriangleFan => {
                    if index_count < 3 {
                        return Err(GeometryError::InvalidMeshData(
                            "Triangle strip/fan must have at least 3 indices".to_string(),
                        ));
                    }
                }
                PrimitiveTopology::LineList => {
                    if index_count % 2 != 0 {
                        return Err(GeometryError::InvalidMeshData(
                            "Line list must have index count divisible by 2".to_string(),
                        ));
                    }
                }
                PrimitiveTopology::LineStrip => {
                    if index_count < 2 {
                        return Err(GeometryError::InvalidMeshData(
                            "Line strip must have at least 2 indices".to_string(),
                        ));
                    }
                }
                PrimitiveTopology::PointList => {
                    // Point list has no special requirements
                }
            }
        }

        // Validate bounds
        if !self.bounds.is_valid() && self.vertex_count() > 0 {
            return Err(GeometryError::InvalidMeshData(
                "Invalid bounding box for non-empty mesh".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate face normals and update vertex normals
    pub fn calculate_normals(&mut self) -> Result<()> {
        if let VertexData::Standard(ref mut vertices) = self.vertex_data {
            // Reset normals
            for vertex in vertices.iter_mut() {
                vertex.normal = glam::Vec3::ZERO;
            }

            // Calculate face normals and accumulate
            if let Some(ref index_buffer) = self.index_buffer {
                let indices = match index_buffer {
                    IndexBuffer::U16(indices) => {
                        indices.iter().map(|&i| i as u32).collect::<Vec<_>>()
                    }
                    IndexBuffer::U32(indices) => indices.clone(),
                };

                for triangle in indices.chunks_exact(3) {
                    let i0 = triangle[0] as usize;
                    let i1 = triangle[1] as usize;
                    let i2 = triangle[2] as usize;

                    if i0 < vertices.len() && i1 < vertices.len() && i2 < vertices.len() {
                        let v0 = vertices[i0].position;
                        let v1 = vertices[i1].position;
                        let v2 = vertices[i2].position;

                        let face_normal = (v1 - v0).cross(v2 - v0);

                        vertices[i0].normal += face_normal;
                        vertices[i1].normal += face_normal;
                        vertices[i2].normal += face_normal;
                    }
                }
            }

            // Normalize accumulated normals
            for vertex in vertices.iter_mut() {
                if vertex.normal.length_squared() > 0.0 {
                    vertex.normal = vertex.normal.normalize();
                } else {
                    vertex.normal = glam::Vec3::Y; // Default up normal
                }
            }

            Ok(())
        } else {
            Err(GeometryError::InvalidMeshData(
                "Can only calculate normals for Standard vertex data".to_string(),
            ))
        }
    }

    /// Transform the mesh by a matrix
    pub fn transform(&mut self, transform: &Mat4) {
        self.vertex_data.transform(transform);

        // Transform bounds
        if self.bounds.is_valid() {
            let corners = [
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.min.x,
                    self.bounds.min.y,
                    self.bounds.min.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.max.x,
                    self.bounds.min.y,
                    self.bounds.min.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.min.x,
                    self.bounds.max.y,
                    self.bounds.min.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.max.x,
                    self.bounds.max.y,
                    self.bounds.min.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.min.x,
                    self.bounds.min.y,
                    self.bounds.max.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.max.x,
                    self.bounds.min.y,
                    self.bounds.max.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.min.x,
                    self.bounds.max.y,
                    self.bounds.max.z,
                )),
                transform.transform_point3(glam::Vec3::new(
                    self.bounds.max.x,
                    self.bounds.max.y,
                    self.bounds.max.z,
                )),
            ];

            let mut min = corners[0];
            let mut max = corners[0];

            for corner in corners.iter().skip(1) {
                min = min.min(*corner);
                max = max.max(*corner);
            }

            self.bounds = BoundingBox::new(min, max);
        }
    }

    /// Create a transformed copy of this mesh
    pub fn transformed(&self, transform: &Mat4) -> Self {
        let mut mesh = self.clone();
        mesh.transform(transform);
        mesh
    }

    /// Merge another mesh into this one
    pub fn merge(&mut self, other: &Mesh) -> Result<()> {
        if !matches!(
            (&self.vertex_data, &other.vertex_data),
            (VertexData::Standard(_), VertexData::Standard(_))
        ) {
            return Err(GeometryError::InvalidMeshData(
                "Can only merge meshes with Standard vertex data".to_string(),
            ));
        }

        let vertex_offset = self.vertex_count() as u32;

        // Merge vertices
        if let (
            VertexData::Standard(ref mut self_vertices),
            VertexData::Standard(ref other_vertices),
        ) = (&mut self.vertex_data, &other.vertex_data)
        {
            self_vertices.extend_from_slice(other_vertices);
        }

        // Merge indices
        if let Some(ref other_indices) = other.index_buffer {
            let offset_indices = match other_indices {
                IndexBuffer::U16(indices) => {
                    indices.iter().map(|&i| i as u32 + vertex_offset).collect()
                }
                IndexBuffer::U32(indices) => indices.iter().map(|&i| i + vertex_offset).collect(),
            };

            match &mut self.index_buffer {
                Some(IndexBuffer::U32(ref mut self_indices)) => {
                    self_indices.extend(offset_indices);
                }
                _ => {
                    self.index_buffer = Some(IndexBuffer::U32(offset_indices));
                }
            }
        }

        self.update_bounds();
        Ok(())
    }

    /// Set the mesh name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Get the mesh name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl IndexBuffer {
    /// Get the index count
    pub fn len(&self) -> usize {
        match self {
            IndexBuffer::U16(indices) => indices.len(),
            IndexBuffer::U32(indices) => indices.len(),
        }
    }

    /// Check if the index buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get index at position
    pub fn get(&self, index: usize) -> Option<u32> {
        match self {
            IndexBuffer::U16(indices) => indices.get(index).map(|&i| i as u32),
            IndexBuffer::U32(indices) => indices.get(index).copied(),
        }
    }

    /// Convert to U32 format
    pub fn to_u32(&self) -> Vec<u32> {
        match self {
            IndexBuffer::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
            IndexBuffer::U32(indices) => indices.clone(),
        }
    }

    /// Get as raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            IndexBuffer::U16(indices) => bytemuck::cast_slice(indices),
            IndexBuffer::U32(indices) => bytemuck::cast_slice(indices),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vertex;
    use glam::{Vec2, Vec3};

    fn create_triangle_vertices() -> Vec<Vertex> {
        vec![
            Vertex {
                position: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 0.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: Vec3::new(1.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 0.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: Vec3::new(0.5, 1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.5, 1.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]
    }

    #[test]
    fn test_mesh_creation() {
        let vertices = create_triangle_vertices();
        let indices = vec![0, 1, 2];

        let mesh_data = MeshData::new("test_triangle".to_string(), vertices, indices);
        let mesh = Mesh::from_data(mesh_data);

        assert_eq!(mesh.name, "test_triangle");
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.index_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
        assert!(mesh.has_indices());
        assert_eq!(mesh.topology, PrimitiveTopology::TriangleList);
    }

    #[test]
    fn test_mesh_validation() {
        let vertices = create_triangle_vertices();
        let indices = vec![0, 1, 2];

        let mesh_data = MeshData::new("test_triangle".to_string(), vertices, indices);
        let mesh = Mesh::from_data(mesh_data);
        assert!(mesh.validate().is_ok());

        // Test invalid indices
        let invalid_indices = vec![0, 1, 5]; // Index 5 is out of bounds
        let invalid_mesh_data = MeshData::new(
            "invalid".to_string(),
            create_triangle_vertices(),
            invalid_indices,
        );
        let invalid_mesh = Mesh::from_data(invalid_mesh_data);
        assert!(invalid_mesh.validate().is_err());

        // Test empty mesh
        let empty_mesh = Mesh::new("empty");
        assert!(empty_mesh.validate().is_err());
    }

    #[test]
    fn test_mesh_bounds_calculation() {
        let vertices = vec![
            Vertex {
                position: Vec3::new(-1.0, -1.0, -1.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 0.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: Vec3::new(2.0, 3.0, 4.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 1.0),
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let mesh_data = MeshData::new("bounds_test".to_string(), vertices, vec![]);
        let mesh = Mesh::from_data(mesh_data);

        assert_eq!(mesh.bounds.min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(mesh.bounds.max, Vec3::new(2.0, 3.0, 4.0));
        assert!(mesh.bounds.is_valid());
    }

    #[test]
    fn test_mesh_transform() {
        let vertices = create_triangle_vertices();
        let mesh_data = MeshData::new("transform_test".to_string(), vertices, vec![]);
        let mut mesh = Mesh::from_data(mesh_data);

        let original_bounds = mesh.bounds;
        let transform = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));

        mesh.transform(&transform);

        // Check that bounds were transformed
        let expected_min = original_bounds.min + Vec3::new(1.0, 2.0, 3.0);
        let expected_max = original_bounds.max + Vec3::new(1.0, 2.0, 3.0);

        assert!((mesh.bounds.min - expected_min).length() < 0.001);
        assert!((mesh.bounds.max - expected_max).length() < 0.001);
    }

    #[test]
    fn test_mesh_merge() {
        let vertices1 = create_triangle_vertices();
        let mesh_data1 = MeshData::new("mesh1".to_string(), vertices1, vec![0, 1, 2]);
        let mut mesh1 = Mesh::from_data(mesh_data1);

        let vertices2 = create_triangle_vertices();
        let mesh_data2 = MeshData::new("mesh2".to_string(), vertices2, vec![0, 1, 2]);
        let mesh2 = Mesh::from_data(mesh_data2);

        let original_vertex_count = mesh1.vertex_count();
        let original_index_count = mesh1.index_count();

        assert!(mesh1.merge(&mesh2).is_ok());

        assert_eq!(
            mesh1.vertex_count(),
            original_vertex_count + mesh2.vertex_count()
        );
        assert_eq!(
            mesh1.index_count(),
            original_index_count + mesh2.index_count()
        );
    }

    #[test]
    fn test_index_buffer_operations() {
        let indices_u16 = IndexBuffer::U16(vec![0, 1, 2]);
        let indices_u32 = IndexBuffer::U32(vec![0, 1, 2]);

        assert_eq!(indices_u16.len(), 3);
        assert_eq!(indices_u32.len(), 3);
        assert!(!indices_u16.is_empty());
        assert!(!indices_u32.is_empty());

        assert_eq!(indices_u16.get(1), Some(1));
        assert_eq!(indices_u32.get(1), Some(1));
        assert_eq!(indices_u16.get(5), None);

        let u32_from_u16 = indices_u16.to_u32();
        assert_eq!(u32_from_u16, vec![0, 1, 2]);

        // Test byte conversion
        let bytes = indices_u32.as_bytes();
        assert_eq!(bytes.len(), 3 * 4); // 3 u32s = 12 bytes
    }

    #[test]
    fn test_material_slot() {
        let slot = MaterialSlot {
            name: "test_material".to_string(),
            material_index: 0,
            start_index: 0,
            index_count: 36,
        };

        assert_eq!(slot.name, "test_material");
        assert_eq!(slot.material_index, 0);
        assert_eq!(slot.start_index, 0);
        assert_eq!(slot.index_count, 36);
    }

    #[test]
    fn test_mesh_data_creation() {
        let vertices = create_triangle_vertices();
        let indices = vec![0, 1, 2];

        let mesh_data = MeshData::new("test_mesh".to_string(), vertices.clone(), indices.clone());

        assert_eq!(mesh_data.name, "test_mesh");
        assert_eq!(mesh_data.vertex_count(), 3);
        assert_eq!(mesh_data.index_count(), 3);
        assert_eq!(mesh_data.submeshes.len(), 1);
        assert_eq!(mesh_data.submeshes[0].start_index, 0);
        assert_eq!(mesh_data.submeshes[0].index_count, 3);
        assert_eq!(mesh_data.submeshes[0].material_index, 0);
        assert!(mesh_data.bounds.is_valid());
    }

    #[test]
    fn test_mesh_data_bounds_calculation() {
        let vertices = vec![
            Vertex {
                position: Vec3::new(-2.0, -3.0, -4.0),
                normal: Vec3::Y,
                uv: Vec2::ZERO,
                color: [1.0; 4],
            },
            Vertex {
                position: Vec3::new(5.0, 6.0, 7.0),
                normal: Vec3::Y,
                uv: Vec2::ONE,
                color: [1.0; 4],
            },
        ];

        let mesh_data = MeshData::new("bounds_test".to_string(), vertices, vec![0, 1]);

        assert_eq!(mesh_data.bounds.min, Vec3::new(-2.0, -3.0, -4.0));
        assert_eq!(mesh_data.bounds.max, Vec3::new(5.0, 6.0, 7.0));
    }

    #[test]
    fn test_mesh_data_memory_usage() {
        let vertices = create_triangle_vertices();
        let indices = vec![0, 1, 2];

        let mesh_data = MeshData::new("memory_test".to_string(), vertices, indices);

        let expected_vertex_size = 3 * std::mem::size_of::<Vertex>();
        let expected_index_size = 3 * std::mem::size_of::<u32>();
        let expected_submesh_size = 1 * std::mem::size_of::<SubMesh>();
        let expected_total = expected_vertex_size + expected_index_size + expected_submesh_size;

        assert_eq!(mesh_data.memory_usage(), expected_total);
        assert_eq!(mesh_data.memory_size(), expected_total);
    }

    #[test]
    fn test_mesh_data_resource_trait() {
        let mesh_data = MeshData::new("resource_test".to_string(), vec![], vec![]);

        assert_eq!(MeshData::resource_type_name(), "MeshData");
        assert_eq!(mesh_data.memory_size(), mesh_data.memory_usage());
    }

    #[test]
    fn test_mesh_from_mesh_data_with_submeshes() {
        let vertices = create_triangle_vertices();
        let indices = vec![0, 1, 2, 0, 2, 1]; // Two triangles

        let mut mesh_data = MeshData::new("submesh_test".to_string(), vertices, indices);

        // Define two submeshes
        mesh_data.submeshes = vec![
            SubMesh {
                start_index: 0,
                index_count: 3,
                material_index: 0,
            },
            SubMesh {
                start_index: 3,
                index_count: 3,
                material_index: 1,
            },
        ];

        let mesh = Mesh::from_data(mesh_data);

        assert_eq!(mesh.material_slots.len(), 2);
        assert_eq!(mesh.material_slots[0].material_index, 0);
        assert_eq!(mesh.material_slots[0].start_index, 0);
        assert_eq!(mesh.material_slots[0].index_count, 3);
        assert_eq!(mesh.material_slots[1].material_index, 1);
        assert_eq!(mesh.material_slots[1].start_index, 3);
        assert_eq!(mesh.material_slots[1].index_count, 3);
    }

    #[test]
    fn test_primitive_topology_validation() {
        let vertices = create_triangle_vertices();

        // Valid triangle list (3 indices)
        let triangle_mesh_data =
            MeshData::new("triangles".to_string(), vertices.clone(), vec![0, 1, 2]);
        let triangle_mesh = Mesh::from_data(triangle_mesh_data);
        assert!(triangle_mesh.validate().is_ok());

        // Invalid triangle list (4 indices - not divisible by 3)
        let invalid_mesh_data = MeshData::new(
            "invalid_triangles".to_string(),
            vertices.clone(),
            vec![0, 1, 2, 1],
        );
        let mut invalid_triangle_mesh = Mesh::from_data(invalid_mesh_data);
        invalid_triangle_mesh.topology = PrimitiveTopology::TriangleList;
        assert!(invalid_triangle_mesh.validate().is_err());

        // Valid line list (2 indices)
        let line_mesh_data = MeshData::new("lines".to_string(), vertices, vec![0, 1]);
        let mut line_mesh = Mesh::from_data(line_mesh_data);
        line_mesh.topology = PrimitiveTopology::LineList;
        assert!(line_mesh.validate().is_ok());
    }
}
