use std::path::Path;
use longhorn_core::{World, Result, LonghornError};

/// Placeholder script runtime.
/// In full implementation, this would use deno_core for TypeScript.
pub struct ScriptRuntime {
    game_path: Option<String>,
    initialized: bool,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            game_path: None,
            initialized: false,
        }
    }

    /// Load a game from a directory containing game.json.
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(LonghornError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Game directory not found: {}", path.display())
            )));
        }

        let manifest_path = path.join("game.json");
        if !manifest_path.exists() {
            return Err(LonghornError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "game.json not found"
            )));
        }

        self.game_path = Some(path.display().to_string());
        log::info!("Loaded game from: {}", path.display());
        Ok(())
    }

    /// Initialize the game (call onStart).
    pub fn initialize(&mut self, _world: &mut World) -> Result<()> {
        if self.game_path.is_none() {
            return Err(LonghornError::Scripting("No game loaded".to_string()));
        }
        log::debug!("ScriptRuntime::initialize (stub)");
        self.initialized = true;
        Ok(())
    }

    /// Update the game (call onUpdate).
    pub fn update(&mut self, _world: &mut World, _delta: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        // Stub - would call TypeScript onUpdate
        Ok(())
    }

    /// Handle touch start event.
    pub fn on_touch_start(&mut self, _world: &mut World, _x: f32, _y: f32) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        // Stub - would call TypeScript onTouchStart
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.game_path.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}
