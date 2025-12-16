// crates/longhorn-editor/src/project.rs
use longhorn_engine::GameManifest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a loaded project
#[derive(Debug, Clone)]
pub struct Project {
    /// Path to the project root directory
    pub path: PathBuf,
    /// Parsed game manifest
    pub manifest: GameManifest,
}

impl Project {
    /// Load a project from a directory
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let manifest = GameManifest::load(&path)?;
        Ok(Self { path, manifest })
    }

    /// Create a new project at the given path
    pub fn create(path: impl AsRef<Path>, name: &str) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create directories
        std::fs::create_dir_all(&path)?;
        std::fs::create_dir_all(path.join("assets"))?;
        std::fs::create_dir_all(path.join("src"))?;

        // Create manifest
        let manifest = GameManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            entry: "src/main.ts".to_string(),
            viewport: longhorn_engine::ViewportConfig {
                width: 1280,
                height: 720,
            },
            assets: longhorn_engine::AssetsConfig::default(),
        };

        // Write game.json
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path.join("game.json"), manifest_json)?;

        Ok(Self { path, manifest })
    }

    /// Save the manifest back to disk
    pub fn save_manifest(&self) -> std::io::Result<()> {
        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(self.path.join("game.json"), manifest_json)
    }
}

/// Tracks which files have unsaved changes
#[derive(Debug, Default)]
pub struct DirtyState {
    /// Scene has unsaved changes
    pub scene: bool,
    /// Scripts with unsaved changes (path -> dirty)
    pub scripts: HashMap<PathBuf, bool>,
    /// Project settings (game.json) has unsaved changes
    pub project_settings: bool,
}

impl DirtyState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any file has unsaved changes
    pub fn any_dirty(&self) -> bool {
        self.scene || self.project_settings || self.scripts.values().any(|&dirty| dirty)
    }

    /// Get list of dirty file names for display
    pub fn dirty_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        if self.scene {
            files.push("scene".to_string());
        }
        if self.project_settings {
            files.push("game.json".to_string());
        }
        for (path, dirty) in &self.scripts {
            if *dirty {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(name.to_string());
                }
            }
        }
        files
    }

    /// Clear all dirty flags
    pub fn clear(&mut self) {
        self.scene = false;
        self.project_settings = false;
        self.scripts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_project() {
        let temp = tempdir().unwrap();
        let project_path = temp.path().join("test-game");

        let project = Project::create(&project_path, "Test Game").unwrap();

        assert_eq!(project.manifest.name, "Test Game");
        assert!(project_path.join("game.json").exists());
        assert!(project_path.join("assets").is_dir());
        assert!(project_path.join("src").is_dir());
    }

    #[test]
    fn test_load_project() {
        let temp = tempdir().unwrap();
        let project_path = temp.path().join("test-game");

        // Create first
        Project::create(&project_path, "Test Game").unwrap();

        // Load it
        let project = Project::load(&project_path).unwrap();
        assert_eq!(project.manifest.name, "Test Game");
    }

    #[test]
    fn test_load_invalid_project() {
        let temp = tempdir().unwrap();
        let result = Project::load(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_dirty_state() {
        let mut dirty = DirtyState::new();
        assert!(!dirty.any_dirty());

        dirty.scene = true;
        assert!(dirty.any_dirty());
        assert_eq!(dirty.dirty_files(), vec!["scene"]);

        dirty.clear();
        assert!(!dirty.any_dirty());
    }
}
