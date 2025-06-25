// Editor settings and preferences

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorSettings {
    pub camera: CameraSettings,
    pub grid: GridSettings,
    pub snap: SnapSettings,
    pub theme: ThemeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    pub movement_speed: f32,
    pub fast_multiplier: f32,
    pub rotation_sensitivity: f32,
    pub smoothing_enabled: bool,
    pub invert_y: bool,
    pub invert_x: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSettings {
    pub enabled: bool,
    pub size: f32,
    pub subdivisions: u32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSettings {
    pub enabled: bool,
    pub position_increment: f32,
    pub rotation_increment: f32,
    pub scale_increment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub ui_scale: f32,
    pub font_size: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            movement_speed: 5.0,
            fast_multiplier: 3.0,
            rotation_sensitivity: 0.005,
            smoothing_enabled: true,
            invert_y: false,
            invert_x: false,
        }
    }
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            subdivisions: 10,
            color: [0.3, 0.3, 0.3, 0.5],
        }
    }
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            position_increment: 0.5,
            rotation_increment: 15.0,
            scale_increment: 0.1,
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            font_size: 12.0,
        }
    }
}

impl EditorSettings {
    /// Load settings from file
    #[allow(dead_code)]
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let settings_path = config_dir.join("longhorn").join("editor_settings.toml");
            if settings_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&settings_path) {
                    if let Ok(settings) = toml::from_str(&contents) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    /// Save settings to file
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let longhorn_dir = config_dir.join("longhorn");
            std::fs::create_dir_all(&longhorn_dir)?;

            let settings_path = longhorn_dir.join("editor_settings.toml");
            let contents = toml::to_string_pretty(self)?;
            std::fs::write(settings_path, contents)?;
        }
        Ok(())
    }
}
