// Theme engine for unified component system

use serde::{Serialize, Deserialize};
use crate::{ColorPalette, Typography, Spacing, Sizes, Layout};

/// Complete editor theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTheme {
    pub name: String,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: Spacing,
    pub sizes: Sizes,
    pub layout: Layout,
    pub is_dark: bool,
}

impl EditorTheme {
    /// Unity-style dark theme
    pub fn unity_dark() -> Self {
        Self {
            name: "Unity Dark".to_string(),
            colors: ColorPalette::unity_dark(),
            typography: Typography::compact(),
            spacing: Spacing::standard(),
            sizes: Sizes::compact(),
            layout: Layout::standard(),
            is_dark: true,
        }
    }
    
    /// Unity-style light theme
    pub fn unity_light() -> Self {
        Self {
            name: "Unity Light".to_string(),
            colors: ColorPalette::unity_light(),
            typography: Typography::compact(),
            spacing: Spacing::standard(),
            sizes: Sizes::compact(),
            layout: Layout::standard(),
            is_dark: false,
        }
    }
    
    /// System default theme (auto-detect light/dark)
    pub fn system_default() -> Self {
        // For now, default to dark theme
        // TODO: Detect system dark/light mode preference
        Self::unity_dark()
    }
    
    /// Create custom theme
    pub fn custom(
        name: String,
        colors: ColorPalette,
        typography: Typography,
        spacing: Spacing,
        sizes: Sizes,
        layout: Layout,
        is_dark: bool,
    ) -> Self {
        Self {
            name,
            colors,
            typography,
            spacing,
            sizes,
            layout,
            is_dark,
        }
    }
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self::system_default()
    }
}

/// Theme manager for switching and applying themes
#[derive(Debug, Clone)]
pub struct ThemeManager {
    current_theme: EditorTheme,
    available_themes: Vec<EditorTheme>,
}

impl ThemeManager {
    /// Create new theme manager with default themes
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: EditorTheme::default(),
            available_themes: Vec::new(),
        };
        
        // Add built-in themes
        manager.add_theme(EditorTheme::unity_dark());
        manager.add_theme(EditorTheme::unity_light());
        
        manager
    }
    
    /// Add a theme to available themes
    pub fn add_theme(&mut self, theme: EditorTheme) {
        self.available_themes.push(theme);
    }
    
    /// Switch to theme by name
    pub fn switch_theme(&mut self, name: &str) -> Result<(), String> {
        if let Some(theme) = self.available_themes.iter().find(|t| t.name == name) {
            self.current_theme = theme.clone();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }
    
    /// Get current theme
    pub fn current_theme(&self) -> &EditorTheme {
        &self.current_theme
    }
    
    /// Get all available theme names
    pub fn theme_names(&self) -> Vec<&str> {
        self.available_themes.iter().map(|t| t.name.as_str()).collect()
    }
    
    /// Toggle between dark and light themes
    pub fn toggle_dark_mode(&mut self) -> Result<(), String> {
        let target_name = if self.current_theme.is_dark {
            "Unity Light"
        } else {
            "Unity Dark"
        };
        self.switch_theme(target_name)
    }
    
    /// Get theme by name
    pub fn get_theme(&self, name: &str) -> Option<&EditorTheme> {
        self.available_themes.iter().find(|t| t.name == name)
    }
    
    /// Save current theme configuration
    pub fn save_current_theme(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.current_theme)
    }
    
    /// Load theme from configuration
    pub fn load_theme(&mut self, config: &str) -> Result<(), Box<dyn std::error::Error>> {
        let theme: EditorTheme = serde_json::from_str(config)?;
        self.current_theme = theme;
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for components that can be themed
pub trait Themeable {
    /// Apply theme to this component
    fn apply_theme(&mut self, theme: &EditorTheme);
    
    /// Get CSS styles for this component based on theme
    fn get_css(&self, theme: &EditorTheme) -> String;
}

/// Helper functions for theme application
impl EditorTheme {
    /// Generate CSS for buttons
    pub fn button_css(&self, variant: &str, size: &str) -> String {
        let (bg_color, text_color, border_color) = match variant {
            "primary" => (
                &self.colors.primary,
                &self.colors.text_inverse,
                &self.colors.primary,
            ),
            "secondary" => (
                &self.colors.surface,
                &self.colors.text_primary,
                &self.colors.border,
            ),
            "danger" => (
                &self.colors.error,
                &self.colors.text_inverse,
                &self.colors.error,
            ),
            _ => (
                &self.colors.surface,
                &self.colors.text_primary,
                &self.colors.border,
            ),
        };
        
        let height = match size {
            "small" => self.sizes.button_height_sm,
            "large" => self.sizes.button_height_lg,
            _ => self.sizes.button_height_md,
        };
        
        format!(
            r#"
            .editor-button.{variant}.{size} {{
                background-color: {bg_color};
                color: {text_color};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                height: {height}px;
                min-width: {min_width}px;
                padding: 0 {padding}px;
                font-family: "{font_family}";
                font-size: {font_size}px;
                font-weight: {font_weight};
            }}
            
            .editor-button.{variant}.{size}:hover {{
                background-color: {hover_color};
            }}
            "#,
            variant = variant,
            size = size,
            bg_color = bg_color.to_hex(),
            text_color = text_color.to_hex(),
            border_color = border_color.to_hex(),
            border_width = self.sizes.border_width,
            border_radius = self.sizes.border_radius_md,
            height = height,
            min_width = self.sizes.button_min_width,
            padding = self.spacing.md,
            font_family = self.typography.font_family_primary,
            font_size = self.typography.sizes.base,
            font_weight = self.typography.weights.medium,
            hover_color = match variant {
                "primary" => self.colors.primary_hover.to_hex(),
                "secondary" => self.colors.surface_hover.to_hex(),
                "danger" => self.colors.error.darken(0.1).to_hex(),
                _ => self.colors.surface_hover.to_hex(),
            }
        )
    }
    
    /// Generate CSS for input fields
    pub fn input_css(&self, variant: &str, size: &str) -> String {
        let height = match size {
            "small" => self.sizes.input_height_sm,
            "large" => self.sizes.input_height_lg,
            _ => self.sizes.input_height_md,
        };
        
        format!(
            r#"
            .editor-input.{variant}.{size} {{
                background-color: {bg_color};
                color: {text_color};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                height: {height}px;
                padding: 0 {padding}px;
                font-family: "{font_family}";
                font-size: {font_size}px;
            }}
            
            .editor-input.{variant}.{size}:focus {{
                border-color: {focus_color};
                outline: none;
                box-shadow: 0 0 0 2px {focus_color}33;
            }}
            
            .editor-input.{variant}.{size}:disabled {{
                background-color: {disabled_bg};
                color: {disabled_text};
                border-color: {disabled_border};
            }}
            "#,
            variant = variant,
            size = size,
            bg_color = self.colors.surface.to_hex(),
            text_color = self.colors.text_primary.to_hex(),
            border_color = self.colors.border.to_hex(),
            border_width = self.sizes.border_width,
            border_radius = self.sizes.border_radius_sm,
            height = height,
            padding = self.spacing.sm,
            font_family = self.typography.font_family_primary,
            font_size = self.typography.sizes.base,
            focus_color = self.colors.border_focus.to_hex(),
            disabled_bg = self.colors.surface.darken(0.1).to_hex(),
            disabled_text = self.colors.text_disabled.to_hex(),
            disabled_border = self.colors.border.darken(0.1).to_hex(),
        )
    }
    
    /// Generate CSS for panels
    pub fn panel_css(&self) -> String {
        format!(
            r#"
            .editor-panel {{
                background-color: {bg_color};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                min-width: {min_width}px;
                min-height: {min_height}px;
                box-shadow: 0 {shadow}px {shadow_blur}px {shadow_color};
            }}
            
            .editor-panel.elevated {{
                background-color: {elevated_bg};
                box-shadow: 0 {shadow_lg}px {shadow_lg_blur}px {shadow_color};
            }}
            
            .editor-panel-header {{
                background-color: {header_bg};
                border-bottom: {border_width}px solid {border_color};
                padding: {padding}px;
                font-family: "{font_family}";
                font-size: {font_size}px;
                font-weight: {font_weight};
                color: {text_color};
            }}
            "#,
            bg_color = self.colors.surface.to_hex(),
            border_color = self.colors.border.to_hex(),
            border_width = self.sizes.border_width,
            border_radius = self.sizes.border_radius_lg,
            min_width = self.sizes.panel_min_width,
            min_height = self.sizes.panel_min_height,
            shadow = self.sizes.shadow_sm,
            shadow_blur = self.sizes.shadow_md,
            shadow_color = self.colors.shadow.to_hex(),
            elevated_bg = self.colors.surface_elevated.to_hex(),
            shadow_lg = self.sizes.shadow_lg,
            shadow_lg_blur = self.sizes.shadow_xl,
            header_bg = self.colors.surface.darken(0.05).to_hex(),
            padding = self.spacing.sm,
            font_family = self.typography.font_family_primary,
            font_size = self.typography.sizes.sm,
            font_weight = self.typography.weights.semibold,
            text_color = self.colors.text_primary.to_hex(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = EditorTheme::unity_dark();
        assert_eq!(theme.name, "Unity Dark");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.theme_names().len(), 2);
        
        assert!(manager.switch_theme("Unity Light").is_ok());
        assert_eq!(manager.current_theme().name, "Unity Light");
        assert!(!manager.current_theme().is_dark);
    }

    #[test]
    fn test_theme_toggle() {
        let mut manager = ThemeManager::new();
        let initial_dark = manager.current_theme().is_dark;
        
        assert!(manager.toggle_dark_mode().is_ok());
        assert_ne!(manager.current_theme().is_dark, initial_dark);
    }

    #[test]
    fn test_css_generation() {
        let theme = EditorTheme::unity_dark();
        let css = theme.button_css("primary", "medium");
        assert!(css.contains("editor-button"));
        assert!(css.contains("background-color"));
    }

    #[test]
    fn test_theme_serialization() {
        let theme = EditorTheme::unity_dark();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: EditorTheme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }
}