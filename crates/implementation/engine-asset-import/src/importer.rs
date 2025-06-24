use crate::{ImportContext, ImportError};
use async_trait::async_trait;
use std::path::Path;

pub type ImportResult<T> = Result<T, ImportError>;

#[async_trait]
pub trait AssetImporter: Send + Sync {
    type Asset: Send;

    /// Returns the file extensions this importer supports
    fn supported_extensions(&self) -> &[&str];

    /// Checks if this importer can handle the given file
    fn can_import(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.supported_extensions().contains(&ext))
            .unwrap_or(false)
    }

    /// Imports an asset from the given path
    async fn import(&self, path: &Path, context: &ImportContext) -> ImportResult<Self::Asset>;

    /// Returns the name of this importer
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
