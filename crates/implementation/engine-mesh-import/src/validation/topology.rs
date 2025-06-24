use crate::MeshData;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TopologyError {
    #[error("Non-manifold edge found between vertices {0} and {1}")]
    NonManifoldEdge(u32, u32),

    #[error("Open edges found: {0} edges are not shared by exactly 2 faces")]
    OpenEdges(usize),

    #[error("Isolated vertices found: {0:?} vertices are not referenced by any face")]
    IsolatedVertices(Vec<u32>),

    #[error("Inconsistent winding order detected")]
    InconsistentWinding,

    #[error("Self-intersecting faces detected")]
    SelfIntersection,
}

pub struct TopologyValidator;

impl TopologyValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, mesh: &MeshData) -> Result<(), TopologyError> {
        // Check for non-manifold edges
        self.check_manifold_edges(mesh)?;

        // Check for open edges (holes)
        self.check_open_edges(mesh)?;

        // Check for isolated vertices
        self.check_isolated_vertices(mesh)?;

        Ok(())
    }

    fn check_manifold_edges(&self, mesh: &MeshData) -> Result<(), TopologyError> {
        // Count faces per edge
        let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();

        for face in mesh.indices.chunks(3) {
            if face.len() != 3 {
                continue;
            }

            let edges = [
                (face[0].min(face[1]), face[0].max(face[1])),
                (face[1].min(face[2]), face[1].max(face[2])),
                (face[2].min(face[0]), face[2].max(face[0])),
            ];

            for edge in &edges {
                *edge_face_count.entry(*edge).or_insert(0) += 1;
            }
        }

        // Check for non-manifold edges (more than 2 faces sharing an edge)
        for ((v1, v2), count) in &edge_face_count {
            if *count > 2 {
                return Err(TopologyError::NonManifoldEdge(*v1, *v2));
            }
        }

        Ok(())
    }

    fn check_open_edges(&self, mesh: &MeshData) -> Result<(), TopologyError> {
        let mut edge_count: HashMap<(u32, u32), i32> = HashMap::new();

        for face in mesh.indices.chunks(3) {
            if face.len() != 3 {
                continue;
            }

            // Add edges with consistent ordering
            edge_count
                .entry((face[0], face[1]))
                .and_modify(|c| *c += 1)
                .or_insert(1);
            edge_count
                .entry((face[1], face[2]))
                .and_modify(|c| *c += 1)
                .or_insert(1);
            edge_count
                .entry((face[2], face[0]))
                .and_modify(|c| *c += 1)
                .or_insert(1);

            // Subtract reverse edges
            edge_count
                .entry((face[1], face[0]))
                .and_modify(|c| *c -= 1)
                .or_insert(-1);
            edge_count
                .entry((face[2], face[1]))
                .and_modify(|c| *c -= 1)
                .or_insert(-1);
            edge_count
                .entry((face[0], face[2]))
                .and_modify(|c| *c -= 1)
                .or_insert(-1);
        }

        // Count open edges (count != 0 means edge is not matched)
        let open_edges = edge_count.values().filter(|&&count| count != 0).count();

        if open_edges > 0 {
            return Err(TopologyError::OpenEdges(open_edges));
        }

        Ok(())
    }

    fn check_isolated_vertices(&self, mesh: &MeshData) -> Result<(), TopologyError> {
        let mut used_vertices = HashSet::new();

        for &idx in &mesh.indices {
            used_vertices.insert(idx);
        }

        let isolated: Vec<u32> = (0..mesh.vertices.len() as u32)
            .filter(|v| !used_vertices.contains(v))
            .collect();

        if !isolated.is_empty() {
            return Err(TopologyError::IsolatedVertices(isolated));
        }

        Ok(())
    }
}
