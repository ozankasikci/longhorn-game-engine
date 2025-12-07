use crate::handle::AssetHandle;
use crate::loader::{load_json, TextureData};
use crate::source::AssetSource;
use crate::registry::AssetRegistry;
use longhorn_core::AssetId;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Manages loading and caching of assets
pub struct AssetManager<S: AssetSource> {
    source: S,
    texture_cache: HashMap<String, (AssetId, TextureData)>,
    json_cache: HashMap<String, (AssetId, Vec<u8>)>,
    next_id: AtomicU64,
    registry: AssetRegistry,
    project_root: PathBuf,
}

impl<S: AssetSource> AssetManager<S> {
    /// Create a new asset manager with the given source and project root
    pub fn new(source: S, project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let registry_path = project_root.join("assets.json");
        let registry = AssetRegistry::load(&registry_path).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load asset registry: {}. Starting with empty registry.", e);
            AssetRegistry::new()
        });

        // Initialize next_id to match the registry's next_id to avoid ID collisions
        let initial_next_id = registry.next_id();

        Self {
            source,
            texture_cache: HashMap::new(),
            json_cache: HashMap::new(),
            next_id: AtomicU64::new(initial_next_id),
            registry,
            project_root,
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

    /// Load a texture by its AssetId (looks up path in registry)
    pub fn load_texture_by_id(&mut self, asset_id: AssetId) -> io::Result<AssetHandle<TextureData>> {
        // First check if it's already loaded in the cache
        if self.texture_cache.values().any(|(id, _)| *id == asset_id) {
            return Ok(AssetHandle::new(asset_id));
        }

        // Look up the path in the registry
        let path = self.registry.get_path(asset_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Asset ID {:?} not found in registry", asset_id),
            )
        })?;

        // Load the texture from the path
        let bytes = self.source.load_bytes(path)?;
        let texture_data = TextureData::from_bytes(&bytes)?;

        // Cache it with the existing asset ID (not a new one)
        self.texture_cache.insert(path.to_string(), (asset_id, texture_data));

        Ok(AssetHandle::new(asset_id))
    }

    /// Check if a texture is loaded (in cache)
    pub fn is_texture_loaded(&self, asset_id: AssetId) -> bool {
        self.texture_cache.values().any(|(id, _)| *id == asset_id)
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

    /// Import an asset by copying it to the project and registering it
    ///
    /// # Arguments
    /// * `source_path` - Absolute path to the source file
    /// * `dest_relative_path` - Relative path within the project (e.g., "sprites/player.png")
    ///
    /// # Returns
    /// The AssetId assigned to the imported asset
    pub fn import_asset(&mut self, source_path: impl AsRef<Path>, dest_relative_path: &str) -> io::Result<AssetId> {
        let source_path = source_path.as_ref();

        // Verify source file exists
        if !source_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file not found: {}", source_path.display()),
            ));
        }

        // Build destination path
        let dest_path = self.project_root.join(dest_relative_path);

        // Create parent directory if needed
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Copy the file
        std::fs::copy(source_path, &dest_path)?;

        // Register in the registry
        let asset_id = self.registry.register(dest_relative_path);

        // Save the registry
        self.save_registry()?;

        Ok(asset_id)
    }

    /// Save the asset registry to disk
    pub fn save_registry(&self) -> io::Result<()> {
        let registry_path = self.project_root.join("assets.json");
        self.registry.save(registry_path)
    }

    /// Reload the asset registry from disk
    pub fn load_registry(&mut self) -> io::Result<()> {
        let registry_path = self.project_root.join("assets.json");
        self.registry = AssetRegistry::load(registry_path)?;
        Ok(())
    }

    /// Get the asset registry
    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }

    /// Get asset ID by path from the registry
    pub fn get_asset_id(&self, path: &str) -> Option<AssetId> {
        self.registry.get_id(path)
    }

    /// Get asset path by ID from the registry
    pub fn get_asset_path(&self, id: AssetId) -> Option<&str> {
        self.registry.get_path(id)
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

    fn create_test_image(path: &std::path::Path) {
        let img = image::RgbaImage::from_raw(4, 4, vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            255, 255, 0, 255, 255, 0, 255, 255, 0, 255, 255, 255, 128, 128, 128, 255,
            100, 100, 100, 255, 200, 200, 200, 255, 50, 50, 50, 255, 150, 150, 150, 255,
            255, 128, 0, 255, 0, 128, 255, 255, 128, 255, 0, 255, 255, 255, 128, 255,
        ]).unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        img.save(path).unwrap();
    }

    #[test]
    fn test_load_texture() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source, &temp_dir);

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
        let mut manager = AssetManager::new(source, &temp_dir);

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
        let mut manager = AssetManager::new(source, &temp_dir);

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
        let mut manager = AssetManager::new(source, &temp_dir);

        let config: TestConfig = manager.load_json("config.json").unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.count, 42);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_json_caching() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source, &temp_dir);

        let config1: TestConfig = manager.load_json("config.json").unwrap();
        let config2: TestConfig = manager.load_json("config.json").unwrap();

        assert_eq!(config1, config2);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_exists() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let manager = AssetManager::new(source, &temp_dir);

        assert!(manager.exists("test.png"));
        assert!(manager.exists("config.json"));
        assert!(!manager.exists("nonexistent.txt"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_preload() {
        let temp_dir = setup_test_dir();
        let source = FilesystemSource::new(&temp_dir);
        let mut manager = AssetManager::new(source, &temp_dir);

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
        let mut manager = AssetManager::new(source, &temp_dir);

        let result = manager.preload("nonexistent.txt");
        assert!(result.is_err());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_import_asset() {
        let temp_dir = setup_test_dir();

        // Create a source image outside the project directory
        let source_image_path = temp_dir.join("external_image.png");
        create_test_image(&source_image_path);

        // Create project directory
        let project_dir = temp_dir.join("project");
        fs::create_dir_all(&project_dir).unwrap();

        let source = FilesystemSource::new(&project_dir);
        let mut manager = AssetManager::new(source, &project_dir);

        // Import the asset
        let asset_id = manager.import_asset(&source_image_path, "sprites/player.png").unwrap();

        // Verify the file was copied
        assert!(project_dir.join("sprites/player.png").exists());

        // Verify it was registered
        assert_eq!(manager.get_asset_id("sprites/player.png"), Some(asset_id));

        // Verify assets.json was created
        assert!(project_dir.join("assets.json").exists());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_import_multiple_assets() {
        let temp_dir = setup_test_dir();

        // Create source images
        let source1 = temp_dir.join("img1.png");
        let source2 = temp_dir.join("img2.png");
        create_test_image(&source1);
        create_test_image(&source2);

        let project_dir = temp_dir.join("project");
        fs::create_dir_all(&project_dir).unwrap();

        let source = FilesystemSource::new(&project_dir);
        let mut manager = AssetManager::new(source, &project_dir);

        // Import multiple assets
        let id1 = manager.import_asset(&source1, "sprites/player.png").unwrap();
        let id2 = manager.import_asset(&source2, "sprites/enemy.png").unwrap();

        // Verify both were registered with different IDs
        assert_ne!(id1, id2);
        assert_eq!(manager.get_asset_id("sprites/player.png"), Some(id1));
        assert_eq!(manager.get_asset_id("sprites/enemy.png"), Some(id2));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_registry_on_startup() {
        let temp_dir = setup_test_dir();
        let project_dir = temp_dir.join("project");
        fs::create_dir_all(&project_dir).unwrap();

        // Create and populate a manager
        {
            let source = FilesystemSource::new(&project_dir);
            let mut manager = AssetManager::new(source, &project_dir);

            // Create source images
            let source_img = temp_dir.join("test_image.png");
            create_test_image(&source_img);

            // Import an asset
            manager.import_asset(&source_img, "test.png").unwrap();
        }

        // Create a new manager - should load the registry
        let source = FilesystemSource::new(&project_dir);
        let manager = AssetManager::new(source, &project_dir);

        // Verify the asset is still registered
        let asset_id = manager.get_asset_id("test.png");
        assert!(asset_id.is_some());
        assert_eq!(manager.get_asset_path(asset_id.unwrap()), Some("test.png"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_get_asset_id_by_path() {
        let temp_dir = setup_test_dir();
        let project_dir = temp_dir.join("project");
        fs::create_dir_all(&project_dir).unwrap();

        let source = FilesystemSource::new(&project_dir);
        let mut manager = AssetManager::new(source, &project_dir);

        let source_img = temp_dir.join("img.png");
        create_test_image(&source_img);

        // Import asset
        let asset_id = manager.import_asset(&source_img, "assets/sprite.png").unwrap();

        // Get ID by path
        assert_eq!(manager.get_asset_id("assets/sprite.png"), Some(asset_id));
        assert_eq!(manager.get_asset_id("nonexistent.png"), None);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_import_nonexistent_source() {
        let temp_dir = setup_test_dir();
        let project_dir = temp_dir.join("project");
        fs::create_dir_all(&project_dir).unwrap();

        let source = FilesystemSource::new(&project_dir);
        let mut manager = AssetManager::new(source, &project_dir);

        // Try to import a file that doesn't exist
        let result = manager.import_asset("/nonexistent/file.png", "test.png");
        assert!(result.is_err());

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
