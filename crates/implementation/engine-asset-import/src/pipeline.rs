use crate::{AssetImporter, ImportContext, ImportError, ImportResult};
use std::path::Path;

pub struct ImportPipeline {
    importers: Vec<Box<dyn AssetImporter<Asset = Vec<u8>>>>,
}

impl ImportPipeline {
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
        }
    }

    pub fn register_importer(&mut self, importer: Box<dyn AssetImporter<Asset = Vec<u8>>>) {
        self.importers.push(importer);
    }

    pub fn find_importer(&self, path: &Path) -> Option<&dyn AssetImporter<Asset = Vec<u8>>> {
        self.importers
            .iter()
            .find(|importer| importer.can_import(path))
            .map(|importer| importer.as_ref())
    }

    pub async fn import_asset(&self, path: &Path, context: ImportContext) -> ImportResult<Vec<u8>> {
        let importer = self.find_importer(path).ok_or_else(|| {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            ImportError::UnsupportedFormat(ext.to_string())
        })?;

        importer.import(path, &context).await
    }

    pub fn supported_extensions(&self) -> Vec<&str> {
        self.importers
            .iter()
            .flat_map(|importer| importer.supported_extensions())
            .copied()
            .collect()
    }

    pub fn importer_count(&self) -> usize {
        self.importers.len()
    }
}
