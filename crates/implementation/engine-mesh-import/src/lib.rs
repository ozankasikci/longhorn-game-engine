pub mod analysis;
pub mod converter;
pub mod fbx;
pub mod generator;
pub mod gltf_import;
pub mod lod;
pub mod obj;
pub mod optimization;
pub mod optimizer;
pub mod processing;
pub mod registry;
pub mod repair;
pub mod types;
pub mod utils;
pub mod validation;
pub mod validator;

// Re-export main types
pub use converter::MeshConverter;
pub use fbx::FbxImporter;
pub use generator::NormalGenerator;
pub use gltf_import::GltfImporter;
pub use obj::ObjImporter;
pub use optimizer::MeshOptimizer;
pub use registry::MeshImportRegistry;
pub use types::{Bounds, Material, MaterialProperty, MeshData, Vertex};
pub use utils::calculate_bounds;
pub use validator::{MeshValidator, ValidationError};

// Common trait for all mesh importers
pub use crate::types::MeshImporter;

use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportPipeline, ImportResult};
use std::path::Path;

// Wrapper to convert mesh importers to byte importers for pipeline compatibility
struct MeshImporterWrapper<T: AssetImporter> {
    inner: T,
}

impl<T: AssetImporter> MeshImporterWrapper<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T> AssetImporter for MeshImporterWrapper<T>
where
    T: AssetImporter + Send + Sync,
    T::Asset: serde::Serialize + Send,
{
    type Asset = Vec<u8>;

    fn supported_extensions(&self) -> &[&str] {
        self.inner.supported_extensions()
    }

    fn can_import(&self, path: &Path) -> bool {
        self.inner.can_import(path)
    }

    async fn import(&self, path: &Path, context: &ImportContext) -> ImportResult<Self::Asset> {
        let asset = self.inner.import(path, context).await?;
        let bytes = bincode::serialize(&asset)
            .map_err(|e| engine_asset_import::ImportError::ProcessingError(e.to_string()))?;
        Ok(bytes)
    }
}

/// Creates a mesh import pipeline with all supported importers registered
pub fn create_mesh_import_pipeline() -> ImportPipeline {
    let mut pipeline = ImportPipeline::new();

    // Register mesh importers wrapped to return bytes
    pipeline.register_importer(Box::new(MeshImporterWrapper::new(ObjImporter::new())));
    pipeline.register_importer(Box::new(MeshImporterWrapper::new(GltfImporter::new())));
    pipeline.register_importer(Box::new(MeshImporterWrapper::new(FbxImporter::new())));

    pipeline
}
