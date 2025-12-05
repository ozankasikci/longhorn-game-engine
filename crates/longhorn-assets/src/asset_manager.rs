use crate::handle::AssetHandle;
use crate::loader::{load_json, TextureData};
use crate::source::AssetSource;
use longhorn_core::AssetId;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

/// Manages loading and caching of assets
pub struct AssetManager<S: AssetSource> {
    source: S,
    texture_cache: HashMap<String, (AssetId, TextureData)>,
    json_cache: HashMap<String, (AssetId, Vec<u8>)>,
    next_id: AtomicU64,
}

impl<S: AssetSource> AssetManager<S> {
    /// Create a new asset manager with the given source
    pub fn new(source: S) -> Self {
        Self {
            source,
            texture_cache: HashMap::new(),
            json_cache: HashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// Generate a new unique asset ID
    fn next_id(&self) -> AssetId {
        AssetId::new(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Load a texture from the given path (cached)
    pub fn load_texture(&mut self, path: &str) -> io::Result<AssetHandle<TextureData>> {
        // Check if already cached
        if let Some((id, _)) = self.texture_cache.get(path) {
            return Ok(AssetHandle::new(*id));
        }

        // Load from source
        let bytes = self.source.load_bytes(path)?;
        let texture_data = TextureData::from_bytes(&bytes)?;

        // Cache it
        let id = self.next_id();
        self.texture_cache.insert(path.to_string(), (id, texture_data));

        Ok(AssetHandle::new(id))
    }

    /// Get a texture by its handle
    pub fn get_texture(&self, handle: AssetHandle<TextureData>) -> Option<&TextureData> {
        self.texture_cache
            .values()
            .find(|(id, _)| *id == handle.id())
            .map(|(_, data)| data)
    }

    /// Get a texture by its path
    pub fn get_texture_by_path(&self, path: &str) -> Option<&TextureData> {
        self.texture_cache.get(path).map(|(_, data)| data)
    }

    /// Load and deserialize JSON data from the given path
    pub fn load_json<T: DeserializeOwned>(&mut self, path: &str) -> io::Result<T> {
        // Load bytes (check cache first)
        let bytes = if let Some((_, cached_bytes)) = self.json_cache.get(path) {
            cached_bytes.clone()
        } else {
            let bytes = self.source.load_bytes(path)?;
            let id = self.next_id();
            self.json_cache.insert(path.to_string(), (id, bytes.clone()));
            bytes
        };

        // Deserialize
        load_json(&bytes)
    }

    /// Check if an asset exists at the given path
    pub fn exists(&self, path: &str) -> bool {
        self.source.exists(path)
    }

    /// Preload an asset without returning it (useful for warming cache)
    pub fn preload(&mut self, path: &str) -> io::Result<()> {
        // Try to determine the asset type from extension
        if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") {
            self.load_texture(path)?;
        } else if path.ends_with(".json") {
            // Just load the bytes into cache
            let bytes = self.source.load_bytes(path)?;
            let id = self.next_id();
            self.json_cache.insert(path.to_string(), (id, bytes));
        } else {
            // Just verify it exists
            if !self.exists(path) {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Asset not found: {}", path),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FilesystemSource;
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        count: u32,
    }

    fn setup_test_dir() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("longhorn_assets_mgr_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a test PNG
        let img = image::RgbaImage::from_raw(2, 2, vec![
            255, 0, 0, 255,
            0, 255, 0, 255,
            0, 0, 255, 255,
            255, 255, 255, 255,
        ]).unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        img.save(temp_dir.join("test.png")).unwrap();

        // Create a test JSON
        let config = TestConfig {
            name: "test".to_string(),
            count: 42,
        };
        fs::write(
            temp_dir.join("config.json"),
            serde_json::to_string(&config).unwrap(),
        ).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_texture() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        let handle = manager.load_texture("test.png").unwrap();
        let texture = manager.get_texture(handle).unwrap();

        assert_eq!(texture.width, 2);
        assert_eq!(texture.height, 2);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_texture_caching() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        let handle1 = manager.load_texture("test.png").unwrap();
        let handle2 = manager.load_texture("test.png").unwrap();

        // Should return the same handle (same ID)
        assert_eq!(handle1.id(), handle2.id());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_get_texture_by_path() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        manager.load_texture("test.png").unwrap();
        let texture = manager.get_texture_by_path("test.png").unwrap();

        assert_eq!(texture.width, 2);
        assert_eq!(texture.height, 2);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_json() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        let config: TestConfig = manager.load_json("config.json").unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.count, 42);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_json_caching() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        let config1: TestConfig = manager.load_json("config.json").unwrap();
        let config2: TestConfig = manager.load_json("config.json").unwrap();

        assert_eq!(config1, config2);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_exists() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let manager = AssetManager::new(source);

        assert!(manager.exists("test.png"));
        assert!(manager.exists("config.json"));
        assert!(!manager.exists("nonexistent.txt"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_preload() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        // Preload texture
        manager.preload("test.png").unwrap();
        assert!(manager.get_texture_by_path("test.png").is_some());

        // Preload JSON
        manager.preload("config.json").unwrap();
        let config: TestConfig = manager.load_json("config.json").unwrap();
        assert_eq!(config.name, "test");

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_preload_nonexistent() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source);

        let result = manager.preload("nonexistent.txt");
        assert!(result.is_err());

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
