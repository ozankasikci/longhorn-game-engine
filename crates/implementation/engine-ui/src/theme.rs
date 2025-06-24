//! UI theming system

use serde::{Deserialize, Serialize};

/// UI theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background_color: (f32, f32, f32, f32),
    pub text_color: (f32, f32, f32, f32),
    pub accent_color: (f32, f32, f32, f32),
}

/// Theme manager for loading and applying themes
pub struct ThemeManager {
    current_theme: Theme,
}

impl Theme {
    /// Create a default theme
    pub fn default() -> Self {
        Self {
            name: "Default".to_string(),
            background_color: (0.2, 0.2, 0.2, 1.0),
            text_color: (1.0, 1.0, 1.0, 1.0),
            accent_color: (0.3, 0.6, 1.0, 1.0),
        }
    }
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }
}
