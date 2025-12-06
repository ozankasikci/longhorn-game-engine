use longhorn_core::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Asset registry that maps file paths to stable IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRegistry {
    /// Maps asset path to asset ID
    path_to_id: HashMap<String, u64>,
    /// Maps asset ID to asset path (reverse lookup)
    id_to_path: HashMap<u64, String>,
    /// Next available ID for new assets
    next_id: u64,
}

impl AssetRegistry {
    /// Create a new empty asset registry
    pub fn new() -> Self {
        Self {
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new asset and return its ID
    /// If the asset is already registered, returns its existing ID
    pub fn register(&mut self, path: impl Into<String>) -> AssetId {
        let path = path.into();

        // If already registered, return existing ID
        if let Some(&id) = self.path_to_id.get(&path) {
            return AssetId::new(id);
        }

        // Generate new ID
        let id = self.next_id;
        self.next_id += 1;

        // Store both mappings
        self.path_to_id.insert(path.clone(), id);
        self.id_to_path.insert(id, path);

        AssetId::new(id)
    }

    /// Get the ID for a given path
    pub fn get_id(&self, path: &str) -> Option<AssetId> {
        self.path_to_id.get(path).map(|&id| AssetId::new(id))
    }

    /// Get the path for a given ID
    pub fn get_path(&self, id: AssetId) -> Option<&str> {
        self.id_to_path.get(&id.0).map(|s| s.as_str())
    }

    /// Iterate over all registered assets
    pub fn iter(&self) -> impl Iterator<Item = (&str, AssetId)> {
        self.path_to_id.iter().map(|(path, &id)| (path.as_str(), AssetId::new(id)))
    }

    /// Get the next available ID (useful for synchronizing ID generation)
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// Load the registry from a JSON file
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        // If file doesn't exist, return empty registry
        if !path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(path)?;

        // Parse the JSON file
        let path_to_id: HashMap<String, u64> = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Build the reverse mapping and find the max ID
        let mut id_to_path = HashMap::new();
        let mut max_id = 0;

        for (path, id) in &path_to_id {
            id_to_path.insert(*id, path.clone());
            if *id > max_id {
                max_id = *id;
            }
        }

        Ok(Self {
            path_to_id,
            id_to_path,
            next_id: max_id + 1,
        })
    }

    /// Save the registry to a JSON file
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&self.path_to_id)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to file
        fs::write(path, json)?;

        Ok(())
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "longhorn_registry_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn test_create_registry() {
        let registry = AssetRegistry::new();
        assert_eq!(registry.path_to_id.len(), 0);
        assert_eq!(registry.id_to_path.len(), 0);
        assert_eq!(registry.next_id, 1);
    }

    #[test]
    fn test_register_asset() {
        let mut registry = AssetRegistry::new();

        let id1 = registry.register("sprites/player.png");
        assert_eq!(id1.0, 1);

        let id2 = registry.register("sprites/enemy.png");
        assert_eq!(id2.0, 2);

        // Registering the same path should return the same ID
        let id1_again = registry.register("sprites/player.png");
        assert_eq!(id1_again.0, 1);
    }

    #[test]
    fn test_get_id_by_path() {
        let mut registry = AssetRegistry::new();

        let id = registry.register("test.png");

        assert_eq!(registry.get_id("test.png"), Some(id));
        assert_eq!(registry.get_id("nonexistent.png"), None);
    }

    #[test]
    fn test_get_path_by_id() {
        let mut registry = AssetRegistry::new();

        let id = registry.register("test.png");

        assert_eq!(registry.get_path(id), Some("test.png"));
        assert_eq!(registry.get_path(AssetId::new(999)), None);
    }

    #[test]
    fn test_save_and_load() {
        let path = temp_path("save_load");

        // Create and populate registry
        let mut registry = AssetRegistry::new();
        registry.register("sprites/player.png");
        registry.register("sprites/enemy.png");
        registry.register("audio/theme.ogg");

        // Save to file
        registry.save(&path).unwrap();

        // Load from file
        let loaded = AssetRegistry::load(&path).unwrap();

        // Verify all entries are present
        assert_eq!(loaded.get_id("sprites/player.png"), Some(AssetId::new(1)));
        assert_eq!(loaded.get_id("sprites/enemy.png"), Some(AssetId::new(2)));
        assert_eq!(loaded.get_id("audio/theme.ogg"), Some(AssetId::new(3)));

        // Verify reverse lookups work
        assert_eq!(loaded.get_path(AssetId::new(1)), Some("sprites/player.png"));
        assert_eq!(loaded.get_path(AssetId::new(2)), Some("sprites/enemy.png"));
        assert_eq!(loaded.get_path(AssetId::new(3)), Some("audio/theme.ogg"));

        // Verify next_id is correct
        assert_eq!(loaded.next_id, 4);

        // Clean up
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = temp_path("nonexistent");

        // Loading a nonexistent file should return an empty registry
        let registry = AssetRegistry::load(&path).unwrap();
        assert_eq!(registry.path_to_id.len(), 0);
        assert_eq!(registry.next_id, 1);
    }

    #[test]
    fn test_roundtrip_with_new_registrations() {
        let path = temp_path("roundtrip");

        // Create, populate, and save registry
        let mut registry = AssetRegistry::new();
        registry.register("file1.png");
        registry.register("file2.png");
        registry.save(&path).unwrap();

        // Load and add more entries
        let mut loaded = AssetRegistry::load(&path).unwrap();
        let new_id = loaded.register("file3.png");

        // New ID should be 3 (after 1 and 2)
        assert_eq!(new_id.0, 3);

        // Save again
        loaded.save(&path).unwrap();

        // Load one more time and verify
        let final_registry = AssetRegistry::load(&path).unwrap();
        assert_eq!(final_registry.get_id("file1.png"), Some(AssetId::new(1)));
        assert_eq!(final_registry.get_id("file2.png"), Some(AssetId::new(2)));
        assert_eq!(final_registry.get_id("file3.png"), Some(AssetId::new(3)));
        assert_eq!(final_registry.next_id, 4);

        // Clean up
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_id_stability() {
        let path = temp_path("id_stability");

        // Create registry with specific entries
        let mut registry1 = AssetRegistry::new();
        let id1 = registry1.register("asset1.png");
        let id2 = registry1.register("asset2.png");
        registry1.save(&path).unwrap();

        // Load and verify IDs are the same
        let registry2 = AssetRegistry::load(&path).unwrap();
        assert_eq!(registry2.get_id("asset1.png"), Some(id1));
        assert_eq!(registry2.get_id("asset2.png"), Some(id2));

        // Clean up
        fs::remove_file(&path).unwrap();
    }
}
