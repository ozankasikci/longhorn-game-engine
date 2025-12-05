use std::io;
use std::path::PathBuf;

/// Trait for loading assets from different sources
pub trait AssetSource {
    /// Load bytes from the given path
    fn load_bytes(&self, path: &str) -> io::Result<Vec<u8>>;

    /// Check if an asset exists at the given path
    fn exists(&self, path: &str) -> bool;
}

/// Asset source that loads from the filesystem
pub struct FilesystemSource {
    root: PathBuf,
}

impl FilesystemSource {
    /// Create a new filesystem source with the given root directory
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the full path for a relative asset path
    fn resolve_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }
}

impl AssetSource for FilesystemSource {
    fn load_bytes(&self, path: &str) -> io::Result<Vec<u8>> {
        let full_path = self.resolve_path(path);
        std::fs::read(full_path)
    }

    fn exists(&self, path: &str) -> bool {
        let full_path = self.resolve_path(path);
        full_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_filesystem_source_load_bytes() {
        // Create a temporary directory
        let temp_dir = std::env::temp_dir().join("longhorn_assets_test");
        fs::create_dir_all(&temp_dir).unwrap();

        // Write a test file
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, b"Hello, Longhorn!").unwrap();

        // Create filesystem source
        let source = FilesystemSource::new(&temp_dir);

        // Load the file
        let bytes = source.load_bytes("test.txt").unwrap();
        assert_eq!(bytes, b"Hello, Longhorn!");

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_filesystem_source_exists() {
        let temp_dir = std::env::temp_dir().join("longhorn_assets_test_exists");
        fs::create_dir_all(&temp_dir).unwrap();

        let test_file = temp_dir.join("exists.txt");
        fs::write(&test_file, b"test").unwrap();

        let source = FilesystemSource::new(&temp_dir);

        assert!(source.exists("exists.txt"));
        assert!(!source.exists("does_not_exist.txt"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_filesystem_source_load_nonexistent() {
        let temp_dir = std::env::temp_dir().join("longhorn_assets_test_nonexistent");
        fs::create_dir_all(&temp_dir).unwrap();

        let source = FilesystemSource::new(&temp_dir);

        let result = source.load_bytes("nonexistent.txt");
        assert!(result.is_err());

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
