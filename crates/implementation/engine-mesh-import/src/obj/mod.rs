use crate::types::{Material, MeshData, MeshImporter, Vertex};
use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError, ImportResult};
use std::path::Path;

pub struct ObjImporter;

impl Default for ObjImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_obj_content(&self, content: &str) -> Result<MeshData, ImportError> {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Vertex position
                    if parts.len() >= 4 {
                        let x = parts[1]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid vertex x".into()))?;
                        let y = parts[2]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid vertex y".into()))?;
                        let z = parts[3]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid vertex z".into()))?;
                        positions.push([x, y, z]);
                    }
                }
                "vn" => {
                    // Vertex normal
                    if parts.len() >= 4 {
                        let x = parts[1]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid normal x".into()))?;
                        let y = parts[2]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid normal y".into()))?;
                        let z = parts[3]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid normal z".into()))?;
                        normals.push([x, y, z]);
                    }
                }
                "vt" => {
                    // Texture coordinate
                    if parts.len() >= 3 {
                        let u = parts[1]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid tex coord u".into()))?;
                        let v = parts[2]
                            .parse::<f32>()
                            .map_err(|_| ImportError::ParseError("Invalid tex coord v".into()))?;
                        tex_coords.push([u, v]);
                    }
                }
                "f" => {
                    // Face
                    if parts.len() >= 4 {
                        // Parse face vertices (we only support triangles for now)
                        for item in parts.iter().take(3 + 1).skip(1) {
                            let vertex_data = item;
                            let components: Vec<&str> = vertex_data.split('/').collect();

                            let pos_idx = components[0].parse::<usize>().map_err(|_| {
                                ImportError::ParseError("Invalid face index".into())
                            })?;
                            let tex_idx = if components.len() > 1 && !components[1].is_empty() {
                                Some(components[1].parse::<usize>().map_err(|_| {
                                    ImportError::ParseError("Invalid tex index".into())
                                })?)
                            } else {
                                None
                            };
                            let norm_idx = if components.len() > 2 && !components[2].is_empty() {
                                Some(components[2].parse::<usize>().map_err(|_| {
                                    ImportError::ParseError("Invalid normal index".into())
                                })?)
                            } else {
                                None
                            };

                            // OBJ indices are 1-based, convert to 0-based
                            let pos = positions.get(pos_idx - 1).ok_or_else(|| {
                                ImportError::ParseError(format!(
                                    "Invalid position index: {}",
                                    pos_idx
                                ))
                            })?;

                            let normal = norm_idx
                                .and_then(|idx| normals.get(idx - 1))
                                .copied()
                                .unwrap_or([0.0, 0.0, 1.0]);

                            let tex_coord = tex_idx
                                .and_then(|idx| tex_coords.get(idx - 1))
                                .copied()
                                .unwrap_or([0.0, 0.0]);

                            vertices.push(Vertex {
                                position: *pos,
                                normal,
                                tex_coords: tex_coord,
                                color: [1.0, 1.0, 1.0, 1.0],
                            });

                            indices.push((vertices.len() - 1) as u32);
                        }
                    }
                }
                _ => {} // Ignore other commands
            }
        }

        Ok(MeshData {
            name: "OBJ Mesh".to_string(),
            vertices,
            indices,
            material: Some(Material::default()),
        })
    }
}

impl MeshImporter for ObjImporter {}

#[async_trait]
impl AssetImporter for ObjImporter {
    type Asset = MeshData;

    fn supported_extensions(&self) -> &[&str] {
        &["obj"]
    }

    fn can_import(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.supported_extensions().contains(&ext))
            .unwrap_or(false)
    }

    async fn import(&self, path: &Path, _context: &ImportContext) -> ImportResult<Self::Asset> {
        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ImportError::IoError(e.to_string()))?;

        self.parse_obj_content(&content)
    }
}
