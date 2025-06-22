//! Types for UI components

// Re-export common types from panels
pub use engine_editor_panels::{ConsoleMessage, PlayState, SceneTool, GizmoSystem};
pub use engine_editor_scene_view::types::SceneNavigation;

/// Different types of dockable panels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Hierarchy,
    Inspector,
    SceneView,
    GameView,
    Console,
    Project,
}

/// Camera settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraSettings {
    pub movement_speed: f32,
    pub rotation_sensitivity: f32,
    pub zoom_speed: f32,
    pub fast_multiplier: f32,
    pub smoothing_enabled: bool,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            movement_speed: 5.0,
            rotation_sensitivity: 0.005,
            zoom_speed: 0.1,
            fast_multiplier: 3.0,
            smoothing_enabled: true,
            invert_x: false,
            invert_y: false,
        }
    }
}

/// Render settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderSettings {
    pub show_grid: bool,
    pub grid_size: f32,
    pub show_fps: bool,
    pub show_stats: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            grid_size: 1.0,
            show_fps: true,
            show_stats: false,
        }
    }
}

/// Grid settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridSettings {
    pub enabled: bool,
    pub size: f32,
    pub subdivisions: i32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            subdivisions: 10,
        }
    }
}

/// Snap settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapSettings {
    pub enabled: bool,
    pub position_increment: f32,
    pub rotation_increment: f32,
    pub scale_increment: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            position_increment: 0.25,
            rotation_increment: 15.0,
            scale_increment: 0.1,
        }
    }
}

/// Editor settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorSettings {
    pub theme: String,
    pub font_size: f32,
    pub auto_save: bool,
    pub vsync: bool,
    pub camera: CameraSettings,
    pub render: RenderSettings,
    pub grid: GridSettings,
    pub snap: SnapSettings,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 14.0,
            auto_save: true,
            vsync: true,
            camera: CameraSettings::default(),
            render: RenderSettings::default(),
            grid: GridSettings::default(),
            snap: SnapSettings::default(),
        }
    }
}

impl EditorSettings {
    pub fn load() -> Self {
        if let Some(data_dir) = dirs::data_dir() {
            let config_path = data_dir.join("longhorn-editor").join("settings.toml");
            if config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(config_path) {
                    if let Ok(settings) = toml::from_str(&contents) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(data_dir) = dirs::data_dir() {
            let config_dir = data_dir.join("longhorn-editor");
            std::fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("settings.toml");
            let contents = toml::to_string_pretty(self).unwrap();
            std::fs::write(config_path, contents)?;
        }
        Ok(())
    }
}