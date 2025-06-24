//! Asset I/O operations

use crate::{AssetError, AssetResult};
use std::path::Path;

/// Asset I/O operations
pub struct AssetIO;

impl AssetIO {
    /// Read file contents as bytes
    pub fn read_bytes<P: AsRef<Path>>(path: P) -> AssetResult<Vec<u8>> {
        std::fs::read(path).map_err(|e| AssetError::LoadError(e.to_string()))
    }

    /// Read file contents as string
    pub fn read_string<P: AsRef<Path>>(path: P) -> AssetResult<String> {
        std::fs::read_to_string(path).map_err(|e| AssetError::LoadError(e.to_string()))
    }

    /// Check if file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}
