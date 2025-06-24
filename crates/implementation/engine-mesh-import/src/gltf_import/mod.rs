use crate::types::{Material, MaterialProperty, MeshData, MeshImporter, Vertex};
use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError, ImportResult};
use std::path::Path;

pub struct GltfImporter;

impl Default for GltfImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl GltfImporter {
    pub fn new() -> Self {
        Self
    }
}

impl MeshImporter for GltfImporter {}

#[async_trait]
impl AssetImporter for GltfImporter {
    type Asset = Vec<MeshData>; // GLTF can contain multiple meshes

    fn supported_extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }

    fn can_import(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.supported_extensions().contains(&ext))
            .unwrap_or(false)
    }

    async fn import(&self, path: &Path, _context: &ImportContext) -> ImportResult<Self::Asset> {
        // Load GLTF file
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| ImportError::ParseError(format!("Failed to load GLTF: {}", e)))?;

        let mut meshes = Vec::new();

        // Process each mesh in the GLTF file
        for mesh in document.meshes() {
            let name = mesh.name().unwrap_or("GLTF Mesh").to_string();

            // Process each primitive in the mesh
            for primitive in mesh.primitives() {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();

                // Get accessor for positions
                let positions = primitive
                    .get(&gltf::Semantic::Positions)
                    .ok_or_else(|| ImportError::ParseError("Mesh has no positions".into()))?;

                let position_data = match positions.view() {
                    Some(view) => {
                        &buffers[view.buffer().index()]
                            [view.offset()..view.offset() + view.length()]
                    }
                    None => {
                        return Err(ImportError::ParseError(
                            "No buffer view for positions".into(),
                        ))
                    }
                };

                // Get normals if available
                let normals = primitive.get(&gltf::Semantic::Normals);
                let normal_data = normals.and_then(|n| n.view()).map(|view| {
                    &buffers[view.buffer().index()][view.offset()..view.offset() + view.length()]
                });

                // Get texture coordinates if available
                let tex_coords = primitive.get(&gltf::Semantic::TexCoords(0));
                let tex_coord_data = tex_coords.and_then(|t| t.view()).map(|view| {
                    &buffers[view.buffer().index()][view.offset()..view.offset() + view.length()]
                });

                // Extract vertex data
                let position_count = positions.count();
                for i in 0..position_count {
                    let pos_offset = i * 12; // 3 floats * 4 bytes
                    let position = [
                        f32::from_le_bytes([
                            position_data[pos_offset],
                            position_data[pos_offset + 1],
                            position_data[pos_offset + 2],
                            position_data[pos_offset + 3],
                        ]),
                        f32::from_le_bytes([
                            position_data[pos_offset + 4],
                            position_data[pos_offset + 5],
                            position_data[pos_offset + 6],
                            position_data[pos_offset + 7],
                        ]),
                        f32::from_le_bytes([
                            position_data[pos_offset + 8],
                            position_data[pos_offset + 9],
                            position_data[pos_offset + 10],
                            position_data[pos_offset + 11],
                        ]),
                    ];

                    let normal = if let Some(data) = normal_data {
                        let offset = i * 12;
                        [
                            f32::from_le_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]),
                            f32::from_le_bytes([
                                data[offset + 4],
                                data[offset + 5],
                                data[offset + 6],
                                data[offset + 7],
                            ]),
                            f32::from_le_bytes([
                                data[offset + 8],
                                data[offset + 9],
                                data[offset + 10],
                                data[offset + 11],
                            ]),
                        ]
                    } else {
                        [0.0, 0.0, 1.0]
                    };

                    let tex_coords = if let Some(data) = tex_coord_data {
                        let offset = i * 8; // 2 floats * 4 bytes
                        [
                            f32::from_le_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]),
                            f32::from_le_bytes([
                                data[offset + 4],
                                data[offset + 5],
                                data[offset + 6],
                                data[offset + 7],
                            ]),
                        ]
                    } else {
                        [0.0, 0.0]
                    };

                    vertices.push(Vertex {
                        position,
                        normal,
                        tex_coords,
                        color: [1.0, 1.0, 1.0, 1.0],
                    });
                }

                // Extract indices
                if let Some(accessor) = primitive.indices() {
                    let view = accessor.view().ok_or_else(|| {
                        ImportError::ParseError("No buffer view for indices".into())
                    })?;
                    let data = &buffers[view.buffer().index()]
                        [view.offset()..view.offset() + view.length()];

                    match accessor.data_type() {
                        gltf::accessor::DataType::U16 => {
                            for i in 0..accessor.count() {
                                let offset = i * 2;
                                let index =
                                    u16::from_le_bytes([data[offset], data[offset + 1]]) as u32;
                                indices.push(index);
                            }
                        }
                        gltf::accessor::DataType::U32 => {
                            for i in 0..accessor.count() {
                                let offset = i * 4;
                                let index = u32::from_le_bytes([
                                    data[offset],
                                    data[offset + 1],
                                    data[offset + 2],
                                    data[offset + 3],
                                ]);
                                indices.push(index);
                            }
                        }
                        _ => return Err(ImportError::ParseError("Unsupported index type".into())),
                    }
                } else {
                    // Generate indices for non-indexed geometry
                    for i in 0..vertices.len() as u32 {
                        indices.push(i);
                    }
                }

                // Extract material
                let material = {
                    let mat = primitive.material();
                    let mut material = Material::new(mat.name().unwrap_or("GLTF Material"));

                    // PBR metallic roughness
                    let pbr = mat.pbr_metallic_roughness();
                    material.set_property(MaterialProperty::BaseColor(pbr.base_color_factor()));
                    material.set_property(MaterialProperty::Metallic(pbr.metallic_factor()));
                    material.set_property(MaterialProperty::Roughness(pbr.roughness_factor()));

                    // Emissive
                    material.set_property(MaterialProperty::EmissiveFactor(mat.emissive_factor()));

                    // Alpha mode
                    material.set_property(MaterialProperty::AlphaMode(format!(
                        "{:?}",
                        mat.alpha_mode()
                    )));
                    material.set_property(MaterialProperty::AlphaCutoff(
                        mat.alpha_cutoff().unwrap_or(0.5),
                    ));

                    // Double sided
                    material.set_property(MaterialProperty::DoubleSided(mat.double_sided()));

                    Some(material)
                };

                meshes.push(MeshData {
                    name: name.clone(),
                    vertices,
                    indices,
                    material,
                });
            }
        }

        Ok(meshes)
    }
}
