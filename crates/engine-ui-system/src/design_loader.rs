// Design constraints loader for the engine
// Automatically loads custom design constraints if available

use crate::{DesignConstraints, utils, typography::LineHeights};
use std::fs;
use std::path::Path;

/// Load design constraints from the constraint editor
pub fn load_current_design() -> DesignConstraints {
    let config_path = "target/current_design_constraints.json";
    
    if Path::new(config_path).exists() {
        match fs::read_to_string(config_path) {
            Ok(json) => {
                match serde_json::from_str::<DesignConstraints>(&json) {
                    Ok(constraints) => {
                        println!("✅ Loaded custom design constraints from {}", config_path);
                        
                        // Note: CSS will be applied later in the UI setup
                        println!("🎨 Custom styling ready to apply");
                        
                        return constraints;
                    },
                    Err(e) => {
                        eprintln!("⚠️  Warning: Failed to parse design constraints: {}", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to read design constraints: {}", e);
            }
        }
    } else {
        println!("ℹ️  No custom design constraints found. Using default Unity Dark theme.");
        println!("💡 Run 'cargo run --package engine-ui-system --example constraint_editor' to customize the UI.");
    }
    
    // Fallback to default
    DesignConstraints::unity_dark()
}

/// Apply design constraints and return a configured theme manager
pub fn setup_custom_theme() -> crate::ThemeManager {
    let constraints = load_current_design();
    
    // Convert constraints to theme
    let theme = constraints_to_theme(&constraints);
    
    let mut theme_manager = crate::ThemeManager::new();
    theme_manager.add_theme(theme);
    theme_manager.switch_theme("Custom").unwrap_or_else(|e| {
        eprintln!("⚠️  Warning: Failed to apply custom theme: {}", e);
    });
    
    theme_manager
}

/// Convert design constraints to an EditorTheme
fn constraints_to_theme(constraints: &DesignConstraints) -> crate::EditorTheme {
    // Convert colors
    let colors = crate::ColorPalette {
        primary: parse_color(&constraints.colors.button_primary_bg),
        primary_hover: parse_color(&constraints.colors.button_primary_hover),
        primary_active: parse_color(&constraints.colors.button_primary_active),
        primary_disabled: parse_color(&constraints.colors.button_primary_bg).with_alpha(0.5),
        
        secondary: parse_color(&constraints.colors.button_secondary_bg),
        secondary_hover: parse_color(&constraints.colors.button_secondary_hover),
        secondary_active: parse_color(&constraints.colors.button_secondary_bg).darken(0.1),
        
        success: parse_color(&constraints.colors.success),
        warning: parse_color(&constraints.colors.warning),
        error: parse_color(&constraints.colors.error),
        info: parse_color(&constraints.colors.info),
        
        background: parse_color(&constraints.colors.window_background),
        surface: parse_color(&constraints.colors.panel_background),
        surface_elevated: parse_color(&constraints.colors.panel_background).lighten(0.05),
        surface_hover: parse_color(&constraints.colors.panel_background).lighten(0.1),
        surface_active: parse_color(&constraints.colors.panel_background).lighten(0.15),
        
        border: parse_color(&constraints.colors.border_primary),
        border_hover: parse_color(&constraints.colors.border_primary).lighten(0.1),
        border_focus: parse_color(&constraints.colors.border_focus),
        
        text_primary: parse_color(&constraints.colors.text_primary),
        text_secondary: parse_color(&constraints.colors.text_secondary),
        text_disabled: parse_color(&constraints.colors.text_disabled),
        text_inverse: parse_color(&constraints.colors.text_primary).lighten(0.9),
        
        shadow: crate::Color::rgba(0.0, 0.0, 0.0, 0.3),
        overlay: crate::Color::rgba(0.0, 0.0, 0.0, 0.6),
        selection: parse_color(&constraints.colors.button_primary_bg).with_alpha(0.3),
    };
    
    // Convert typography
    let typography = crate::Typography {
        font_family_primary: constraints.typography.primary_font.clone(),
        font_family_mono: constraints.typography.monospace_font.clone(),
        font_family_display: constraints.typography.primary_font.clone(),
        sizes: crate::FontSizes {
            xs: constraints.typography.font_size_xs,
            sm: constraints.typography.font_size_sm,
            base: constraints.typography.font_size_base,
            lg: constraints.typography.font_size_lg,
            xl: constraints.typography.font_size_xl,
            xxl: constraints.typography.font_size_xl * 1.5,
            xxxl: constraints.typography.font_size_xl * 2.0,
        },
        weights: crate::FontWeights {
            light: constraints.typography.weight_light,
            normal: constraints.typography.weight_normal,
            medium: constraints.typography.weight_medium,
            semibold: constraints.typography.weight_semibold,
            bold: constraints.typography.weight_bold,
        },
        line_heights: LineHeights {
            tight: constraints.typography.line_height_tight,
            normal: constraints.typography.line_height_normal,
            relaxed: constraints.typography.line_height_relaxed,
            loose: constraints.typography.line_height_relaxed * 1.25,
        },
        letter_spacing_tight: -0.025,
        letter_spacing_normal: 0.0,
        letter_spacing_wide: 0.025,
    };
    
    // Convert spacing
    let spacing = crate::Spacing {
        none: constraints.spacing.space_none,
        xs: constraints.spacing.space_xs,
        sm: constraints.spacing.space_sm,
        md: constraints.spacing.space_md,
        lg: constraints.spacing.space_lg,
        xl: constraints.spacing.space_xl,
        xxl: constraints.spacing.space_xxl,
        xxxl: constraints.spacing.space_xxxl,
    };
    
    // Convert sizes
    let sizes = crate::Sizes {
        button_height_sm: constraints.geometry.button_height_sm,
        button_height_md: constraints.geometry.button_height_md,
        button_height_lg: constraints.geometry.button_height_lg,
        button_min_width: constraints.geometry.button_min_width,
        
        input_height_sm: constraints.geometry.input_height_sm,
        input_height_md: constraints.geometry.input_height_md,
        input_height_lg: constraints.geometry.input_height_lg,
        
        icon_xs: constraints.geometry.icon_size_xs,
        icon_sm: constraints.geometry.icon_size_sm,
        icon_md: constraints.geometry.icon_size_md,
        icon_lg: constraints.geometry.icon_size_lg,
        icon_xl: constraints.geometry.icon_size_xl,
        
        panel_min_width: constraints.geometry.panel_min_width,
        panel_min_height: constraints.geometry.panel_min_height,
        sidebar_width: constraints.geometry.sidebar_width,
        toolbar_height: constraints.geometry.toolbar_height,
        
        border_width: constraints.geometry.border_width_thin,
        border_radius_sm: constraints.geometry.border_radius_sm,
        border_radius_md: constraints.geometry.border_radius_md,
        border_radius_lg: constraints.geometry.border_radius_lg,
        border_radius_xl: constraints.geometry.border_radius_xl,
        
        shadow_sm: constraints.effects.shadow_sm_blur,
        shadow_md: constraints.effects.shadow_md_blur,
        shadow_lg: constraints.effects.shadow_lg_blur,
        shadow_xl: constraints.effects.shadow_lg_blur * 2.0,
    };
    
    // Convert layout
    let layout = crate::Layout {
        z_dropdown: 1000,
        z_sticky: 1020,
        z_fixed: 1030,
        z_modal_backdrop: 1040,
        z_modal: 1050,
        z_popover: 1060,
        z_tooltip: 1070,
        
        timing_fast: constraints.effects.animation_fast,
        timing_base: constraints.effects.animation_normal,
        timing_slow: constraints.effects.animation_slow,
        
        breakpoint_sm: 640.0,
        breakpoint_md: 768.0,
        breakpoint_lg: 1024.0,
        breakpoint_xl: 1280.0,
    };
    
    // Determine if it's a dark theme based on background color
    let bg_color = parse_color(&constraints.colors.window_background);
    let is_dark = (bg_color.r + bg_color.g + bg_color.b) / 3.0 < 0.5;
    
    crate::EditorTheme {
        name: "Custom".to_string(),
        colors,
        typography,
        spacing,
        sizes,
        layout,
        is_dark,
    }
}

/// Parse a color string (hex format) to Color
fn parse_color(hex: &str) -> crate::Color {
    crate::Color::from_hex(hex).unwrap_or_else(|_| {
        eprintln!("⚠️  Warning: Invalid color '{}', using fallback", hex);
        crate::Color::WHITE
    })
}

/// Apply design constraints CSS to a GTK window
pub fn apply_constraints_to_window<W: gtk4::prelude::WidgetExt>(window: &W) -> DesignConstraints {
    let constraints = load_current_design();
    
    // Apply the CSS to this window's display
    let css = constraints.to_css();
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);
    
    gtk4::style_context_add_provider_for_display(
        &window.display(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    println!("🎨 Applied custom CSS styling to window");
    constraints
}

/// Get quick preview of current constraints
pub fn preview_current_constraints() {
    let constraints = load_current_design();
    
    println!("🎨 Current Design Constraints Preview:");
    println!("  Colors: {} (window) | {} (button) | {} (text)", 
        constraints.colors.window_background,
        constraints.colors.button_primary_bg,
        constraints.colors.text_primary);
    println!("  Geometry: {}px (button) | {}px (radius) | {}px (spacing)", 
        constraints.geometry.button_height_md,
        constraints.geometry.border_radius_md,
        constraints.spacing.space_md);
    println!("  Typography: {} | {}px base", 
        constraints.typography.primary_font,
        constraints.typography.font_size_base);
}