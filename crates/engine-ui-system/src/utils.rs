// Utility functions and helpers for the UI system

use gtk4::prelude::*;
use gtk4::{CssProvider, StyleContext};
use gdk4;
use crate::{EditorTheme, Color};

/// CSS utility functions
pub mod css {
    use super::*;
    
    /// Apply CSS to a style context
    pub fn apply_css_to_context(context: &StyleContext, css: &str) -> Result<(), String> {
        let provider = CssProvider::new();
        provider.load_from_data(css);
        
        context.add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        Ok(())
    }
    
    /// Apply global CSS to the application
    pub fn apply_global_css(css: &str) -> Result<(), String> {
        let provider = CssProvider::new();
        provider.load_from_data(css);
        
        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().expect("Could not connect to a display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        Ok(())
    }
    
    /// Generate complete theme CSS for the application
    pub fn generate_theme_css(theme: &EditorTheme) -> String {
        format!(
            r#"
            /* Base theme variables */
            :root {{
                --primary-color: {primary};
                --background-color: {background};
                --surface-color: {surface};
                --text-primary: {text_primary};
                --text-secondary: {text_secondary};
                --border-color: {border};
                --border-focus: {border_focus};
                --spacing-xs: {spacing_xs}px;
                --spacing-sm: {spacing_sm}px;
                --spacing-md: {spacing_md}px;
                --spacing-lg: {spacing_lg}px;
                --border-radius: {border_radius}px;
                --font-family: "{font_family}";
                --font-size-base: {font_size}px;
            }}
            
            /* Global styles */
            * {{
                font-family: var(--font-family);
                color: var(--text-primary);
            }}
            
            window {{
                background-color: var(--background-color);
            }}
            
            /* Button styles */
            {button_css}
            
            /* Input styles */
            {input_css}
            
            /* Panel styles */
            {panel_css}
            
            /* Toolbar styles */
            {toolbar_css}
            
            /* Widget-specific styles */
            {widget_css}
            "#,
            primary = theme.colors.primary.to_hex(),
            background = theme.colors.background.to_hex(),
            surface = theme.colors.surface.to_hex(),
            text_primary = theme.colors.text_primary.to_hex(),
            text_secondary = theme.colors.text_secondary.to_hex(),
            border = theme.colors.border.to_hex(),
            border_focus = theme.colors.border_focus.to_hex(),
            spacing_xs = theme.spacing.xs,
            spacing_sm = theme.spacing.sm,
            spacing_md = theme.spacing.md,
            spacing_lg = theme.spacing.lg,
            border_radius = theme.sizes.border_radius_md,
            font_family = theme.typography.font_family_primary,
            font_size = theme.typography.sizes.base,
            button_css = generate_button_css(theme),
            input_css = generate_input_css(theme),
            panel_css = theme.panel_css(),
            toolbar_css = generate_toolbar_css(theme),
            widget_css = generate_widget_css(theme),
        )
    }
    
    /// Generate button CSS for all variants and sizes
    fn generate_button_css(theme: &EditorTheme) -> String {
        let variants = ["primary", "secondary", "outline", "ghost", "danger"];
        let sizes = ["small", "medium", "large"];
        
        let mut css = String::new();
        
        for variant in &variants {
            for size in &sizes {
                css.push_str(&theme.button_css(variant, size));
            }
        }
        
        css
    }
    
    /// Generate input CSS for all variants and sizes
    fn generate_input_css(theme: &EditorTheme) -> String {
        let variants = ["default", "search", "number", "password"];
        let sizes = ["small", "medium", "large"];
        
        let mut css = String::new();
        
        for variant in &variants {
            for size in &sizes {
                css.push_str(&theme.input_css(variant, size));
            }
        }
        
        css
    }
    
    /// Generate toolbar CSS
    fn generate_toolbar_css(theme: &EditorTheme) -> String {
        format!(
            r#"
            .editor-toolbar {{
                background-color: {bg_color};
                border-bottom: {border_width}px solid {border_color};
                min-height: {height}px;
                padding: {padding}px {padding_h}px;
            }}
            "#,
            bg_color = theme.colors.surface.to_hex(),
            border_color = theme.colors.border.to_hex(),
            border_width = theme.sizes.border_width,
            height = theme.sizes.toolbar_height,
            padding = theme.spacing.xs,
            padding_h = theme.spacing.sm,
        )
    }
    
    /// Generate widget-specific CSS
    fn generate_widget_css(theme: &EditorTheme) -> String {
        format!(
            r#"
            /* Vector3Field styles */
            .vector3-field {{
                margin: {margin}px;
            }}
            
            .vector3-field .component-label {{
                color: {label_color};
                font-size: {small_font}px;
                font-weight: {medium_weight};
                text-align: center;
                min-height: {label_height}px;
            }}
            
            /* EnumDropdown styles */
            .enum-dropdown-widget {{
                background-color: {surface};
                color: {text_primary};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                padding: 0 {padding}px;
                min-height: {input_height}px;
            }}
            
            .enum-dropdown-widget:focus {{
                border-color: {focus_color};
                outline: none;
                box-shadow: 0 0 0 2px {focus_color}33;
            }}
            
            /* AssetField styles */
            .asset-container {{
                background-color: {surface};
                border: {border_width}px solid {border_color};
                border-radius: {border_radius}px;
                padding: {padding}px;
                min-height: {input_height}px;
            }}
            
            .asset-container:focus-within {{
                border-color: {focus_color};
                box-shadow: 0 0 0 2px {focus_color}33;
            }}
            
            .asset-name {{
                background-color: transparent;
                border: none;
                color: {text_primary};
            }}
            
            /* Field labels */
            .field-label {{
                color: {text_primary};
                font-size: {small_font}px;
                font-weight: {medium_weight};
                margin-bottom: {xs_margin}px;
            }}
            "#,
            margin = theme.spacing.xs,
            label_color = theme.colors.text_secondary.to_hex(),
            small_font = theme.typography.sizes.sm,
            medium_weight = theme.typography.weights.medium,
            label_height = theme.sizes.input_height_sm / 2.0,
            surface = theme.colors.surface.to_hex(),
            text_primary = theme.colors.text_primary.to_hex(),
            border_width = theme.sizes.border_width,
            border_color = theme.colors.border.to_hex(),
            border_radius = theme.sizes.border_radius_sm,
            padding = theme.spacing.sm,
            input_height = theme.sizes.input_height_md,
            focus_color = theme.colors.border_focus.to_hex(),
            xs_margin = theme.spacing.xs,
        )
    }
}

/// Color utility functions
pub mod color {
    use super::*;
    
    /// Convert Color to GDK RGBA
    pub fn to_gdk_rgba(color: &Color) -> gdk4::RGBA {
        color.to_gdk_rgba()
    }
    
    /// Convert GDK RGBA to Color
    pub fn from_gdk_rgba(rgba: &gdk4::RGBA) -> Color {
        Color::from_gdk_rgba(rgba)
    }
    
    /// Interpolate between two colors
    pub fn lerp(color1: &Color, color2: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgba(
            color1.r + (color2.r - color1.r) * t,
            color1.g + (color2.g - color1.g) * t,
            color1.b + (color2.b - color1.b) * t,
            color1.a + (color2.a - color1.a) * t,
        )
    }
    
    /// Get contrast ratio between two colors
    pub fn contrast_ratio(color1: &Color, color2: &Color) -> f32 {
        let l1 = relative_luminance(color1);
        let l2 = relative_luminance(color2);
        
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        
        (lighter + 0.05) / (darker + 0.05)
    }
    
    /// Calculate relative luminance for contrast calculations
    fn relative_luminance(color: &Color) -> f32 {
        fn linearize(component: f32) -> f32 {
            if component <= 0.03928 {
                component / 12.92
            } else {
                ((component + 0.055) / 1.055).powf(2.4)
            }
        }
        
        let r = linearize(color.r);
        let g = linearize(color.g);
        let b = linearize(color.b);
        
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
    
    /// Check if color combination meets WCAG AA accessibility standards
    pub fn is_accessible(foreground: &Color, background: &Color) -> bool {
        contrast_ratio(foreground, background) >= 4.5
    }
    
    /// Generate accessible text color for background
    pub fn accessible_text_color(background: &Color) -> Color {
        let white = Color::WHITE;
        let black = Color::BLACK;
        
        let white_ratio = contrast_ratio(&white, background);
        let black_ratio = contrast_ratio(&black, background);
        
        if white_ratio > black_ratio {
            white
        } else {
            black
        }
    }
}

/// Layout utility functions
pub mod layout {
    use super::*;
    use gtk4::{Box, Orientation, Widget};
    
    /// Create horizontal box with spacing
    pub fn hbox(spacing: i32) -> Box {
        Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(spacing)
            .build()
    }
    
    /// Create vertical box with spacing
    pub fn vbox(spacing: i32) -> Box {
        Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(spacing)
            .build()
    }
    
    /// Create horizontal box with theme spacing
    pub fn themed_hbox(theme: &EditorTheme) -> Box {
        hbox(theme.spacing.sm as i32)
    }
    
    /// Create vertical box with theme spacing
    pub fn themed_vbox(theme: &EditorTheme) -> Box {
        vbox(theme.spacing.sm as i32)
    }
    
    /// Add widget with theme margins
    pub fn add_with_margins(container: &Box, widget: &impl IsA<Widget>, theme: &EditorTheme) {
        let margin = theme.spacing.sm as i32;
        widget.set_margin_top(margin);
        widget.set_margin_bottom(margin);
        widget.set_margin_start(margin);
        widget.set_margin_end(margin);
        container.append(widget);
    }
    
    /// Create separator with theme styling
    pub fn themed_separator(_theme: &EditorTheme, orientation: Orientation) -> gtk4::Separator {
        let separator = gtk4::Separator::builder()
            .orientation(orientation)
            .build();
        
        let context = separator.style_context();
        context.add_class("themed-separator");
        
        separator
    }
}

/// Icon and asset utilities
pub mod icons {
    /// Standard icon names for Unity-style editor
    pub struct EditorIcons;
    
    impl EditorIcons {
        // File operations
        pub const NEW: &'static str = "document-new";
        pub const OPEN: &'static str = "document-open";
        pub const SAVE: &'static str = "document-save";
        pub const SAVE_AS: &'static str = "document-save-as";
        
        // Edit operations
        pub const UNDO: &'static str = "edit-undo";
        pub const REDO: &'static str = "edit-redo";
        pub const CUT: &'static str = "edit-cut";
        pub const COPY: &'static str = "edit-copy";
        pub const PASTE: &'static str = "edit-paste";
        pub const DELETE: &'static str = "edit-delete";
        pub const CLEAR: &'static str = "edit-clear";
        
        // Transform tools
        pub const SELECT: &'static str = "edit-select";
        pub const MOVE: &'static str = "transform-move";
        pub const ROTATE: &'static str = "transform-rotate";
        pub const SCALE: &'static str = "transform-scale";
        pub const HAND: &'static str = "edit-hand";
        
        // Playback controls
        pub const PLAY: &'static str = "media-playback-start";
        pub const PAUSE: &'static str = "media-playback-pause";
        pub const STOP: &'static str = "media-playback-stop";
        pub const STEP: &'static str = "media-skip-forward";
        
        // View controls
        pub const ZOOM_IN: &'static str = "zoom-in";
        pub const ZOOM_OUT: &'static str = "zoom-out";
        pub const ZOOM_FIT: &'static str = "zoom-fit-best";
        pub const FULLSCREEN: &'static str = "view-fullscreen";
        
        // Asset types
        pub const TEXTURE: &'static str = "image-x-generic";
        pub const MATERIAL: &'static str = "applications-graphics";
        pub const MESH: &'static str = "applications-engineering";
        pub const AUDIO: &'static str = "audio-x-generic";
        pub const SCRIPT: &'static str = "text-x-script";
        pub const PREFAB: &'static str = "package-x-generic";
        pub const SCENE: &'static str = "folder-documents";
        pub const SHADER: &'static str = "text-x-generic";
        
        // UI elements
        pub const DROPDOWN: &'static str = "go-down";
        pub const EXPAND: &'static str = "go-next";
        pub const COLLAPSE: &'static str = "go-down";
        pub const CLOSE: &'static str = "window-close";
        pub const MINIMIZE: &'static str = "window-minimize";
        pub const MAXIMIZE: &'static str = "window-maximize";
        
        // Status indicators
        pub const SUCCESS: &'static str = "emblem-default";
        pub const WARNING: &'static str = "dialog-warning";
        pub const ERROR: &'static str = "dialog-error";
        pub const INFO: &'static str = "dialog-information";
    }
}

/// Animation and transition utilities
pub mod animation {
    use std::time::Duration;
    
    /// Easing functions for smooth animations
    pub enum EasingFunction {
        Linear,
        EaseIn,
        EaseOut,
        EaseInOut,
    }
    
    impl EasingFunction {
        /// Apply easing function to normalized time (0.0 - 1.0)
        pub fn apply(&self, t: f32) -> f32 {
            let t = t.clamp(0.0, 1.0);
            match self {
                EasingFunction::Linear => t,
                EasingFunction::EaseIn => t * t,
                EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
                EasingFunction::EaseInOut => {
                    if t < 0.5 {
                        2.0 * t * t
                    } else {
                        1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                    }
                }
            }
        }
    }
    
    /// Animation duration constants from theme
    pub struct AnimationDurations;
    
    impl AnimationDurations {
        pub const FAST: Duration = Duration::from_millis(150);
        pub const NORMAL: Duration = Duration::from_millis(250);
        pub const SLOW: Duration = Duration::from_millis(350);
    }
}

/// Validation utilities for form inputs
pub mod validation {
    /// Validate numeric input
    pub fn is_valid_number(text: &str) -> bool {
        text.parse::<f64>().is_ok()
    }
    
    /// Validate integer input
    pub fn is_valid_integer(text: &str) -> bool {
        text.parse::<i64>().is_ok()
    }
    
    /// Validate number in range
    pub fn is_number_in_range(text: &str, min: f64, max: f64) -> bool {
        if let Ok(value) = text.parse::<f64>() {
            value >= min && value <= max
        } else {
            false
        }
    }
    
    /// Validate hex color
    pub fn is_valid_hex_color(text: &str) -> bool {
        crate::Color::from_hex(text).is_ok()
    }
    
    /// Validate file path exists
    pub fn path_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    
    /// Validate file has expected extension
    pub fn has_extension(path: &str, extensions: &[&str]) -> bool {
        if let Some(ext) = std::path::Path::new(path).extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            extensions.iter().any(|&e| e.to_lowercase() == ext_str)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_interpolation() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        
        let mid = color::lerp(&black, &white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_contrast_ratio() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        
        let ratio = color::contrast_ratio(&black, &white);
        assert!(ratio > 20.0); // Should be 21:1 for perfect black/white
    }

    #[test]
    fn test_accessibility() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        
        assert!(color::is_accessible(&black, &white));
        assert!(color::is_accessible(&white, &black));
    }

    #[test]
    fn test_easing_functions() {
        let linear = animation::EasingFunction::Linear;
        assert_eq!(linear.apply(0.5), 0.5);
        
        let ease_in = animation::EasingFunction::EaseIn;
        assert!(ease_in.apply(0.5) < 0.5);
        
        let ease_out = animation::EasingFunction::EaseOut;
        assert!(ease_out.apply(0.5) > 0.5);
    }

    #[test]
    fn test_validation() {
        assert!(validation::is_valid_number("123.45"));
        assert!(!validation::is_valid_number("abc"));
        
        assert!(validation::is_valid_integer("123"));
        assert!(!validation::is_valid_integer("123.45"));
        
        assert!(validation::is_number_in_range("5", 0.0, 10.0));
        assert!(!validation::is_number_in_range("15", 0.0, 10.0));
        
        assert!(validation::is_valid_hex_color("#FF0000"));
        assert!(!validation::is_valid_hex_color("red"));
    }
}