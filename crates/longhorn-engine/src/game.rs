use serde::{Deserialize, Serialize};
use std::path::Path;

/// Game manifest (matches game.json structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManifest {
    /// Game name
    pub name: String,
    /// Game version
    pub version: String,
    /// Entry script path
    pub entry: String,
    /// Viewport configuration
    pub viewport: ViewportConfig,
    /// Assets configuration
    #[serde(default)]
    pub assets: AssetsConfig,
}

impl GameManifest {
    /// Load a game manifest from a directory containing game.json
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let manifest_path = path.join("game.json");

        if !manifest_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("game.json not found in: {}", path.display()),
            ));
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: GameManifest = serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse game.json: {}", e),
            )
        })?;

        Ok(manifest)
    }
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Viewport width in pixels
    pub width: u32,
    /// Viewport height in pixels
    pub height: u32,
}

/// Assets configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetsConfig {
    /// Assets to preload
    #[serde(default)]
    pub preload: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_game() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "longhorn_game_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let manifest = GameManifest {
            name: "Test Game".to_string(),
            version: "1.0.0".to_string(),
            entry: "main.ts".to_string(),
            viewport: ViewportConfig {
                width: 800,
                height: 600,
            },
            assets: AssetsConfig {
                preload: vec!["sprites/player.png".to_string()],
            },
        };

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(temp_dir.join("game.json"), manifest_json).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_manifest() {
        let temp_dir = setup_test_game();

        let manifest = GameManifest::load(&temp_dir).unwrap();
        assert_eq!(manifest.name, "Test Game");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entry, "main.ts");
        assert_eq!(manifest.viewport.width, 800);
        assert_eq!(manifest.viewport.height, 600);
        assert_eq!(manifest.assets.preload.len(), 1);
        assert_eq!(manifest.assets.preload[0], "sprites/player.png");

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_manifest_not_found() {
        let temp_dir = std::env::temp_dir().join("nonexistent_game");
        let result = GameManifest::load(&temp_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_without_assets() {
        let temp_dir = std::env::temp_dir().join(format!(
            "longhorn_game_test_no_assets_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let manifest_json = r#"{
            "name": "Simple Game",
            "version": "0.1.0",
            "entry": "index.ts",
            "viewport": {
                "width": 1280,
                "height": 720
            }
        }"#;

        fs::write(temp_dir.join("game.json"), manifest_json).unwrap();

        let manifest = GameManifest::load(&temp_dir).unwrap();
        assert_eq!(manifest.name, "Simple Game");
        assert_eq!(manifest.assets.preload.len(), 0);

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
