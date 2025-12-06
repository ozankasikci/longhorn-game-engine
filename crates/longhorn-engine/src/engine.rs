use crate::{EngineConfig, GameManifest};
use longhorn_assets::{AssetManager, FilesystemSource};
use longhorn_core::{Time, World};
use longhorn_events::EventBus;
use longhorn_input::{InputState, TouchEvent};
use longhorn_renderer::{Camera, Renderer};
use longhorn_scripting::ScriptRuntime;
use std::path::Path;

/// Main game engine
pub struct Engine {
    /// ECS world
    world: World,
    /// Renderer (optional for headless mode)
    renderer: Option<Renderer>,
    /// Camera
    camera: Camera,
    /// Input state
    input: InputState,
    /// Asset manager
    assets: AssetManager<FilesystemSource>,
    /// Script runtime
    scripting: ScriptRuntime,
    /// Time tracking
    time: Time,
    /// Engine configuration
    config: EngineConfig,
    /// Game manifest
    game_manifest: Option<GameManifest>,
    /// Game path
    game_path: Option<String>,
    /// Event bus
    event_bus: EventBus,
}

impl Engine {
    /// Create a new headless engine (for testing/editor)
    pub fn new_headless() -> Self {
        let config = EngineConfig::default();
        let camera = Camera::new(
            config.viewport_width as f32,
            config.viewport_height as f32,
        );

        // Use a temporary directory for assets
        let temp_source = FilesystemSource::new(std::env::temp_dir());

        Self {
            world: World::new(),
            renderer: None,
            camera,
            input: InputState::new(),
            assets: AssetManager::new(temp_source),
            scripting: ScriptRuntime::new(),
            time: Time::new(),
            config,
            game_manifest: None,
            game_path: None,
            event_bus: EventBus::new(),
        }
    }

    /// Create a new engine with a renderer
    pub async fn new_with_renderer(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        config: EngineConfig,
    ) -> Result<Self, EngineError> {
        let renderer = Renderer::new(
            window,
            config.viewport_width,
            config.viewport_height,
        )
        .await?;

        let camera = Camera::new(
            config.viewport_width as f32,
            config.viewport_height as f32,
        );

        // Use a temporary directory for assets until a game is loaded
        let temp_source = FilesystemSource::new(std::env::temp_dir());

        Ok(Self {
            world: World::new(),
            renderer: Some(renderer),
            camera,
            input: InputState::new(),
            assets: AssetManager::new(temp_source),
            scripting: ScriptRuntime::new(),
            time: Time::new(),
            config,
            game_manifest: None,
            game_path: None,
            event_bus: EventBus::new(),
        })
    }

    /// Load a game from a directory
    pub fn load_game(&mut self, path: impl AsRef<Path>) -> Result<(), EngineError> {
        let path = path.as_ref();

        // Load manifest
        let manifest = GameManifest::load(path)?;

        // Update viewport if needed
        if manifest.viewport.width != self.config.viewport_width
            || manifest.viewport.height != self.config.viewport_height
        {
            self.config.viewport_width = manifest.viewport.width;
            self.config.viewport_height = manifest.viewport.height;
            self.camera.viewport_size = glam::Vec2::new(
                manifest.viewport.width as f32,
                manifest.viewport.height as f32,
            );

            if let Some(renderer) = &mut self.renderer {
                renderer.resize(manifest.viewport.width, manifest.viewport.height);
            }
        }

        // Set up asset manager with game directory
        let game_source = FilesystemSource::new(path);
        self.assets = AssetManager::new(game_source);

        // Preload assets
        for asset_path in &manifest.assets.preload {
            if let Err(e) = self.assets.preload(asset_path) {
                log::warn!("Failed to preload asset '{}': {}", asset_path, e);
            }
        }

        // Load game in script runtime
        self.scripting.load_game(path)?;

        self.game_manifest = Some(manifest);
        self.game_path = Some(path.display().to_string());

        log::info!("Loaded game from: {}", path.display());

        Ok(())
    }

    /// Start the game (call onStart in scripting)
    pub fn start(&mut self) -> Result<(), EngineError> {
        if self.game_path.is_none() {
            return Err(EngineError::NoGameLoaded);
        }

        // Reset time so first frame has delta = 0
        self.time.reset();

        self.scripting.initialize(&mut self.world)?;
        log::info!("Game started");

        Ok(())
    }

    /// Reset the scripting runtime (for editor Stop)
    pub fn reset_scripting(&mut self) {
        self.scripting.reset();
        log::debug!("Script runtime reset");
    }

    /// Handle a touch event
    pub fn handle_touch(&mut self, event: TouchEvent) {
        self.input.handle_event(event);

        // Emit to event bus
        let (event_type, data) = match event {
            TouchEvent::Start { x, y } => (
                longhorn_events::EventType::TouchStarted,
                serde_json::json!({"x": x, "y": y}),
            ),
            TouchEvent::Move { x, y } => (
                longhorn_events::EventType::TouchMoved,
                serde_json::json!({"x": x, "y": y}),
            ),
            TouchEvent::End { x, y } => (
                longhorn_events::EventType::TouchEnded,
                serde_json::json!({"x": x, "y": y}),
            ),
        };
        self.event_bus.emit(event_type, data);

        // Forward to script runtime if it's a touch start
        if event.is_start() {
            let pos = event.position();
            let _ = self.scripting.on_touch_start(&mut self.world, pos.x, pos.y);
        }
    }

    /// Update the engine (main frame update)
    pub fn update(&mut self) -> Result<(), EngineError> {
        // Update time
        self.time.update();

        // Emit frame begin
        self.event_bus.emit(
            longhorn_events::EventType::FrameBegin,
            serde_json::json!({"delta": self.time.delta()}),
        );

        // Process pending events
        let _events = self.event_bus.process();

        // Update scripting
        if self.scripting.is_initialized() {
            self.scripting.update(&mut self.world, self.time.delta())?;
        }

        // Collect events emitted by scripts and forward to EventBus
        let script_events = longhorn_scripting::take_pending_events();
        for (name, data) in script_events {
            self.event_bus.emit(
                longhorn_events::EventType::Custom(name),
                data,
            );
        }

        // Collect targeted events emitted by scripts and forward to EventBus
        let targeted_events = longhorn_scripting::take_pending_targeted_events();
        for (entity_id, name, data) in targeted_events {
            self.event_bus.emit_targeted(
                longhorn_events::EventType::Custom(name),
                longhorn_events::EventTarget::Entity(entity_id),
                data,
            );
        }

        // Render if renderer is available
        if let Some(renderer) = &mut self.renderer {
            // Set clear color from config
            renderer.set_clear_color(self.config.clear_color());

            // Render the world
            renderer.render(&self.world, &self.assets, &self.camera)?;
        }

        // Reset per-frame input state
        self.input.begin_frame();

        // Emit frame end
        self.event_bus.emit(longhorn_events::EventType::FrameEnd, serde_json::json!({}));

        Ok(())
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.viewport_width = width;
        self.config.viewport_height = height;

        self.camera.viewport_size = glam::Vec2::new(width as f32, height as f32);

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    /// Get a reference to the game manifest
    pub fn manifest(&self) -> Option<&GameManifest> {
        self.game_manifest.as_ref()
    }

    /// Get a reference to the world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a mutable reference to the world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Get a reference to the camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get a reference to the input state
    pub fn input(&self) -> &InputState {
        &self.input
    }

    /// Get a reference to the time
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// Get a reference to the asset manager
    pub fn assets(&self) -> &AssetManager<FilesystemSource> {
        &self.assets
    }

    /// Get a mutable reference to the asset manager
    pub fn assets_mut(&mut self) -> &mut AssetManager<FilesystemSource> {
        &mut self.assets
    }

    /// Get the engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get the game path if a game is loaded
    pub fn game_path(&self) -> Option<&Path> {
        self.game_path.as_ref().map(|p| Path::new(p))
    }

    /// Get mutable access to the event bus.
    pub fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    /// Get read-only access to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Spawn an entity with a name and emit EntitySpawned event.
    pub fn spawn_entity(&mut self, name: &str) -> longhorn_core::EntityHandle {
        use longhorn_core::Name;

        let handle = self.world.spawn().with(Name::new(name)).build();
        let id = handle.id().to_bits().get();

        self.event_bus.emit(
            longhorn_events::EventType::EntitySpawned,
            serde_json::json!({
                "entity": id,
                "name": name,
            }),
        );

        handle
    }

    /// Despawn an entity and emit EntityDespawned event.
    pub fn despawn_entity(&mut self, handle: longhorn_core::EntityHandle) -> Result<(), EngineError> {
        let id = handle.id().to_bits().get();

        self.event_bus.emit(
            longhorn_events::EventType::EntityDespawned,
            serde_json::json!({
                "entity": id,
            }),
        );

        self.world.despawn(handle)?;
        Ok(())
    }
}

/// Engine errors
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Renderer error: {0}")]
    Renderer(#[from] longhorn_renderer::RendererError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] longhorn_core::LonghornError),

    #[error("No game loaded")]
    NoGameLoaded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use longhorn_core::Name;
    use std::fs;

    fn setup_test_game() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(format!(
            "longhorn_engine_test_{}",
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
            viewport: crate::game::ViewportConfig {
                width: 800,
                height: 600,
            },
            assets: crate::game::AssetsConfig {
                preload: vec![],
            },
        };

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(temp_dir.join("game.json"), manifest_json).unwrap();

        temp_dir
    }

    #[test]
    fn test_new_headless() {
        let engine = Engine::new_headless();
        assert!(engine.renderer.is_none());
        assert_eq!(engine.world.len(), 0);
        assert!(engine.game_manifest.is_none());
    }

    #[test]
    fn test_load_game() {
        let temp_dir = setup_test_game();
        let mut engine = Engine::new_headless();

        engine.load_game(&temp_dir).unwrap();
        assert!(engine.game_manifest.is_some());
        assert_eq!(engine.config.viewport_width, 800);
        assert_eq!(engine.config.viewport_height, 600);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_start_without_game() {
        let mut engine = Engine::new_headless();
        let result = engine.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_start_with_game() {
        let temp_dir = setup_test_game();
        let mut engine = Engine::new_headless();

        engine.load_game(&temp_dir).unwrap();
        let result = engine.start();
        assert!(result.is_ok());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_handle_touch() {
        let mut engine = Engine::new_headless();

        engine.handle_touch(TouchEvent::Start { x: 100.0, y: 200.0 });
        assert!(engine.input.just_pressed());
        assert_eq!(engine.input.position(), glam::Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_update() {
        let temp_dir = setup_test_game();
        let mut engine = Engine::new_headless();

        engine.load_game(&temp_dir).unwrap();
        engine.start().unwrap();

        // Should not error
        let result = engine.update();
        assert!(result.is_ok());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_resize() {
        let mut engine = Engine::new_headless();

        engine.resize(1920, 1080);
        assert_eq!(engine.config.viewport_width, 1920);
        assert_eq!(engine.config.viewport_height, 1080);
        assert_eq!(engine.camera.viewport_size, glam::Vec2::new(1920.0, 1080.0));
    }

    #[test]
    fn test_world_access() {
        let mut engine = Engine::new_headless();

        // Add an entity
        let entity = engine.world_mut().spawn().with(Name::new("Test")).build();

        // Read it back
        assert_eq!(engine.world().len(), 1);
        let name = engine.world().get::<Name>(entity).unwrap();
        assert_eq!(name.as_str(), "Test");
    }

    #[test]
    fn test_camera_access() {
        let mut engine = Engine::new_headless();

        engine.camera_mut().position = glam::Vec2::new(100.0, 200.0);
        assert_eq!(engine.camera().position, glam::Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_time_tracking() {
        let mut engine = Engine::new_headless();

        // Initial delta should be 0
        assert_eq!(engine.time().delta(), 0.0);

        // After update, time should progress
        std::thread::sleep(std::time::Duration::from_millis(10));
        engine.update().ok();
        assert!(engine.time().delta() > 0.0);
    }

    #[test]
    fn test_manifest_getter() {
        let temp_dir = setup_test_game();
        let mut engine = Engine::new_headless();

        assert!(engine.manifest().is_none());

        engine.load_game(&temp_dir).unwrap();
        let manifest = engine.manifest().unwrap();
        assert_eq!(manifest.name, "Test Game");

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
