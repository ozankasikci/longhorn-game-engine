// Design Constraints System - Precise control over every UI element
// Define exact pixel values, colors, and styling for components

use serde::{Serialize, Deserialize};
// use crate::Color;

/// Complete design constraints for the UI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignConstraints {
    pub colors: ColorConstraints,
    pub typography: TypographyConstraints,
    pub geometry: GeometryConstraints,
    pub spacing: SpacingConstraints,
    pub effects: EffectConstraints,
}

/// Color constraints - exact hex values for every element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConstraints {
    // Background colors
    pub window_background: String,          // e.g. "#1E1E1E"
    pub panel_background: String,           // e.g. "#2D2D2D"
    pub toolbar_background: String,         // e.g. "#383838"
    pub sidebar_background: String,         // e.g. "#252525"
    
    // Button colors
    pub button_primary_bg: String,          // e.g. "#007AFF"
    pub button_primary_hover: String,       // e.g. "#0056CC"
    pub button_primary_active: String,      // e.g. "#003D99"
    pub button_secondary_bg: String,        // e.g. "#404040"
    pub button_secondary_hover: String,     // e.g. "#4A4A4A"
    pub button_danger_bg: String,           // e.g. "#FF3B30"
    
    // Input colors
    pub input_background: String,           // e.g. "#3A3A3A"
    pub input_border: String,               // e.g. "#555555"
    pub input_border_focus: String,         // e.g. "#007AFF"
    pub input_text: String,                 // e.g. "#FFFFFF"
    pub input_placeholder: String,          // e.g. "#888888"
    
    // Text colors
    pub text_primary: String,               // e.g. "#FFFFFF"
    pub text_secondary: String,             // e.g. "#ABABAB"
    pub text_disabled: String,              // e.g. "#666666"
    pub text_accent: String,                // e.g. "#007AFF"
    
    // Border colors
    pub border_primary: String,             // e.g. "#404040"
    pub border_secondary: String,           // e.g. "#2A2A2A"
    pub border_focus: String,               // e.g. "#007AFF"
    
    // Status colors
    pub success: String,                    // e.g. "#34C759"
    pub warning: String,                    // e.g. "#FF9500"
    pub error: String,                      // e.g. "#FF3B30"
    pub info: String,                       // e.g. "#007AFF"
}

/// Typography constraints - exact font specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyConstraints {
    // Font families
    pub primary_font: String,               // e.g. "SF Pro Display"
    pub monospace_font: String,             // e.g. "SF Mono"
    
    // Font sizes (in pixels)
    pub font_size_xs: f32,                  // e.g. 10.0
    pub font_size_sm: f32,                  // e.g. 11.0
    pub font_size_base: f32,                // e.g. 12.0
    pub font_size_md: f32,                  // e.g. 13.0
    pub font_size_lg: f32,                  // e.g. 14.0
    pub font_size_xl: f32,                  // e.g. 16.0
    
    // Font weights
    pub weight_light: u32,                  // e.g. 300
    pub weight_normal: u32,                 // e.g. 400
    pub weight_medium: u32,                 // e.g. 500
    pub weight_semibold: u32,               // e.g. 600
    pub weight_bold: u32,                   // e.g. 700
    
    // Line heights (multipliers)
    pub line_height_tight: f32,             // e.g. 1.2
    pub line_height_normal: f32,            // e.g. 1.4
    pub line_height_relaxed: f32,           // e.g. 1.6
}

/// Geometry constraints - exact pixel dimensions and shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryConstraints {
    // Button dimensions
    pub button_height_sm: f32,              // e.g. 24.0
    pub button_height_md: f32,              // e.g. 28.0
    pub button_height_lg: f32,              // e.g. 32.0
    pub button_min_width: f32,              // e.g. 64.0
    pub button_padding_x: f32,              // e.g. 12.0
    pub button_padding_y: f32,              // e.g. 6.0
    
    // Input dimensions
    pub input_height_sm: f32,               // e.g. 24.0
    pub input_height_md: f32,               // e.g. 28.0
    pub input_height_lg: f32,               // e.g. 32.0
    pub input_padding_x: f32,               // e.g. 8.0
    pub input_padding_y: f32,               // e.g. 4.0
    
    // Panel dimensions
    pub panel_min_width: f32,               // e.g. 200.0
    pub panel_min_height: f32,              // e.g. 150.0
    pub panel_header_height: f32,           // e.g. 32.0
    pub sidebar_width: f32,                 // e.g. 250.0
    pub toolbar_height: f32,                // e.g. 36.0
    
    // Border radii (roundness)
    pub border_radius_none: f32,            // e.g. 0.0
    pub border_radius_sm: f32,              // e.g. 2.0
    pub border_radius_md: f32,              // e.g. 4.0
    pub border_radius_lg: f32,              // e.g. 6.0
    pub border_radius_xl: f32,              // e.g. 8.0
    pub border_radius_full: f32,            // e.g. 999.0
    
    // Border widths
    pub border_width_none: f32,             // e.g. 0.0
    pub border_width_thin: f32,             // e.g. 1.0
    pub border_width_thick: f32,            // e.g. 2.0
    
    // Icon sizes
    pub icon_size_xs: f32,                  // e.g. 12.0
    pub icon_size_sm: f32,                  // e.g. 14.0
    pub icon_size_md: f32,                  // e.g. 16.0
    pub icon_size_lg: f32,                  // e.g. 20.0
    pub icon_size_xl: f32,                  // e.g. 24.0
}

/// Spacing constraints - exact pixel spacing between elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingConstraints {
    // Base spacing units
    pub space_none: f32,                    // e.g. 0.0
    pub space_xs: f32,                      // e.g. 2.0
    pub space_sm: f32,                      // e.g. 4.0
    pub space_md: f32,                      // e.g. 8.0
    pub space_lg: f32,                      // e.g. 12.0
    pub space_xl: f32,                      // e.g. 16.0
    pub space_xxl: f32,                     // e.g. 24.0
    pub space_xxxl: f32,                    // e.g. 32.0
    
    // Component-specific spacing
    pub button_gap: f32,                    // Space between buttons
    pub panel_padding: f32,                 // Padding inside panels
    pub section_spacing: f32,               // Space between sections
    pub field_spacing: f32,                 // Space between form fields
    pub toolbar_item_spacing: f32,          // Space between toolbar items
}

/// Effect constraints - shadows, animations, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectConstraints {
    // Shadow specifications
    pub shadow_sm_blur: f32,                // e.g. 2.0
    pub shadow_sm_offset_x: f32,            // e.g. 0.0
    pub shadow_sm_offset_y: f32,            // e.g. 1.0
    pub shadow_sm_color: String,            // e.g. "#00000020"
    
    pub shadow_md_blur: f32,                // e.g. 4.0
    pub shadow_md_offset_x: f32,            // e.g. 0.0
    pub shadow_md_offset_y: f32,            // e.g. 2.0
    pub shadow_md_color: String,            // e.g. "#00000030"
    
    pub shadow_lg_blur: f32,                // e.g. 8.0
    pub shadow_lg_offset_x: f32,            // e.g. 0.0
    pub shadow_lg_offset_y: f32,            // e.g. 4.0
    pub shadow_lg_color: String,            // e.g. "#00000040"
    
    // Animation durations (milliseconds)
    pub animation_fast: f32,                // e.g. 100.0
    pub animation_normal: f32,              // e.g. 200.0
    pub animation_slow: f32,                // e.g. 300.0
    
    // Transition properties
    pub transition_easing: String,          // e.g. "ease-in-out"
    pub hover_transition: f32,              // e.g. 150.0
    pub focus_transition: f32,              // e.g. 100.0
}

impl DesignConstraints {
    /// Unity-style dark constraints (as starting point)
    pub fn unity_dark() -> Self {
        Self {
            colors: ColorConstraints {
                window_background: "#1E1E1E".to_string(),
                panel_background: "#2D2D2D".to_string(),
                toolbar_background: "#383838".to_string(),
                sidebar_background: "#252525".to_string(),
                
                button_primary_bg: "#007AFF".to_string(),
                button_primary_hover: "#0056CC".to_string(),
                button_primary_active: "#003D99".to_string(),
                button_secondary_bg: "#404040".to_string(),
                button_secondary_hover: "#4A4A4A".to_string(),
                button_danger_bg: "#FF3B30".to_string(),
                
                input_background: "#3A3A3A".to_string(),
                input_border: "#555555".to_string(),
                input_border_focus: "#007AFF".to_string(),
                input_text: "#FFFFFF".to_string(),
                input_placeholder: "#888888".to_string(),
                
                text_primary: "#FFFFFF".to_string(),
                text_secondary: "#ABABAB".to_string(),
                text_disabled: "#666666".to_string(),
                text_accent: "#007AFF".to_string(),
                
                border_primary: "#404040".to_string(),
                border_secondary: "#2A2A2A".to_string(),
                border_focus: "#007AFF".to_string(),
                
                success: "#34C759".to_string(),
                warning: "#FF9500".to_string(),
                error: "#FF3B30".to_string(),
                info: "#007AFF".to_string(),
            },
            typography: TypographyConstraints {
                primary_font: "SF Pro Display".to_string(),
                monospace_font: "SF Mono".to_string(),
                
                font_size_xs: 10.0,
                font_size_sm: 11.0,
                font_size_base: 12.0,
                font_size_md: 13.0,
                font_size_lg: 14.0,
                font_size_xl: 16.0,
                
                weight_light: 300,
                weight_normal: 400,
                weight_medium: 500,
                weight_semibold: 600,
                weight_bold: 700,
                
                line_height_tight: 1.2,
                line_height_normal: 1.4,
                line_height_relaxed: 1.6,
            },
            geometry: GeometryConstraints {
                button_height_sm: 24.0,
                button_height_md: 28.0,
                button_height_lg: 32.0,
                button_min_width: 64.0,
                button_padding_x: 12.0,
                button_padding_y: 6.0,
                
                input_height_sm: 24.0,
                input_height_md: 28.0,
                input_height_lg: 32.0,
                input_padding_x: 8.0,
                input_padding_y: 4.0,
                
                panel_min_width: 200.0,
                panel_min_height: 150.0,
                panel_header_height: 32.0,
                sidebar_width: 250.0,
                toolbar_height: 36.0,
                
                border_radius_none: 0.0,
                border_radius_sm: 2.0,
                border_radius_md: 4.0,
                border_radius_lg: 6.0,
                border_radius_xl: 8.0,
                border_radius_full: 999.0,
                
                border_width_none: 0.0,
                border_width_thin: 1.0,
                border_width_thick: 2.0,
                
                icon_size_xs: 12.0,
                icon_size_sm: 14.0,
                icon_size_md: 16.0,
                icon_size_lg: 20.0,
                icon_size_xl: 24.0,
            },
            spacing: SpacingConstraints {
                space_none: 0.0,
                space_xs: 2.0,
                space_sm: 4.0,
                space_md: 8.0,
                space_lg: 12.0,
                space_xl: 16.0,
                space_xxl: 24.0,
                space_xxxl: 32.0,
                
                button_gap: 8.0,
                panel_padding: 12.0,
                section_spacing: 16.0,
                field_spacing: 8.0,
                toolbar_item_spacing: 4.0,
            },
            effects: EffectConstraints {
                shadow_sm_blur: 2.0,
                shadow_sm_offset_x: 0.0,
                shadow_sm_offset_y: 1.0,
                shadow_sm_color: "#00000020".to_string(),
                
                shadow_md_blur: 4.0,
                shadow_md_offset_x: 0.0,
                shadow_md_offset_y: 2.0,
                shadow_md_color: "#00000030".to_string(),
                
                shadow_lg_blur: 8.0,
                shadow_lg_offset_x: 0.0,
                shadow_lg_offset_y: 4.0,
                shadow_lg_color: "#00000040".to_string(),
                
                animation_fast: 100.0,
                animation_normal: 200.0,
                animation_slow: 300.0,
                
                transition_easing: "ease-in-out".to_string(),
                hover_transition: 150.0,
                focus_transition: 100.0,
            },
        }
    }
    
    /// Flat/minimal constraints (completely flat design)
    pub fn flat_minimal() -> Self {
        let mut constraints = Self::unity_dark();
        
        // Make everything flat
        constraints.geometry.border_radius_sm = 0.0;
        constraints.geometry.border_radius_md = 0.0;
        constraints.geometry.border_radius_lg = 0.0;
        constraints.geometry.border_radius_xl = 0.0;
        
        // Remove shadows
        constraints.effects.shadow_sm_blur = 0.0;
        constraints.effects.shadow_md_blur = 0.0;
        constraints.effects.shadow_lg_blur = 0.0;
        
        constraints
    }
    
    /// Generate CSS from constraints - COMPREHENSIVE version including ALL settings
    pub fn to_css(&self) -> String {
        format!(
            r#"
            :root {{
                /* Background Colors */
                --window-background: {window_background};
                --panel-background: {panel_background};
                --toolbar-background: {toolbar_background};
                --sidebar-background: {sidebar_background};
                
                /* Button Colors */
                --button-primary-bg: {button_primary_bg};
                --button-primary-hover: {button_primary_hover};
                --button-primary-active: {button_primary_active};
                --button-secondary-bg: {button_secondary_bg};
                --button-secondary-hover: {button_secondary_hover};
                --button-danger-bg: {button_danger_bg};
                
                /* Input Colors */
                --input-background: {input_background};
                --input-border: {input_border};
                --input-border-focus: {input_border_focus};
                --input-text: {input_text};
                --input-placeholder: {input_placeholder};
                
                /* Text Colors */
                --text-primary: {text_primary};
                --text-secondary: {text_secondary};
                --text-disabled: {text_disabled};
                --text-accent: {text_accent};
                
                /* Border Colors */
                --border-primary: {border_primary};
                --border-secondary: {border_secondary};
                --border-focus: {border_focus};
                
                /* Status Colors */
                --success: {success};
                --warning: {warning};
                --error: {error};
                --info: {info};
                
                /* Typography */
                --font-primary: "{font_primary}";
                --font-mono: "{font_mono}";
                --font-size-xs: {font_size_xs}px;
                --font-size-sm: {font_size_sm}px;
                --font-size-base: {font_size_base}px;
                --font-size-md: {font_size_md}px;
                --font-size-lg: {font_size_lg}px;
                --font-size-xl: {font_size_xl}px;
                --font-weight-light: {font_weight_light};
                --font-weight-normal: {font_weight_normal};
                --font-weight-medium: {font_weight_medium};
                --font-weight-semibold: {font_weight_semibold};
                --font-weight-bold: {font_weight_bold};
                --line-height-tight: {line_height_tight};
                --line-height-normal: {line_height_normal};
                --line-height-relaxed: {line_height_relaxed};
                
                /* Button Geometry */
                --button-height-sm: {button_height_sm}px;
                --button-height-md: {button_height_md}px;
                --button-height-lg: {button_height_lg}px;
                --button-min-width: {button_min_width}px;
                --button-padding-x: {button_padding_x}px;
                --button-padding-y: {button_padding_y}px;
                
                /* Input Geometry */
                --input-height-sm: {input_height_sm}px;
                --input-height-md: {input_height_md}px;
                --input-height-lg: {input_height_lg}px;
                --input-padding-x: {input_padding_x}px;
                --input-padding-y: {input_padding_y}px;
                
                /* Panel Geometry */
                --panel-min-width: {panel_min_width}px;
                --panel-min-height: {panel_min_height}px;
                --panel-header-height: {panel_header_height}px;
                --sidebar-width: {sidebar_width}px;
                --toolbar-height: {toolbar_height}px;
                
                /* Border Radii */
                --border-radius-none: {border_radius_none}px;
                --border-radius-sm: {border_radius_sm}px;
                --border-radius-md: {border_radius_md}px;
                --border-radius-lg: {border_radius_lg}px;
                --border-radius-xl: {border_radius_xl}px;
                --border-radius-full: {border_radius_full}px;
                
                /* Border Widths */
                --border-width-none: {border_width_none}px;
                --border-width-thin: {border_width_thin}px;
                --border-width-thick: {border_width_thick}px;
                
                /* Icon Sizes */
                --icon-size-xs: {icon_size_xs}px;
                --icon-size-sm: {icon_size_sm}px;
                --icon-size-md: {icon_size_md}px;
                --icon-size-lg: {icon_size_lg}px;
                --icon-size-xl: {icon_size_xl}px;
                
                /* Spacing */
                --space-none: {space_none}px;
                --space-xs: {space_xs}px;
                --space-sm: {space_sm}px;
                --space-md: {space_md}px;
                --space-lg: {space_lg}px;
                --space-xl: {space_xl}px;
                --space-xxl: {space_xxl}px;
                --space-xxxl: {space_xxxl}px;
                --button-gap: {button_gap}px;
                --panel-padding: {panel_padding}px;
                --section-spacing: {section_spacing}px;
                --field-spacing: {field_spacing}px;
                --toolbar-item-spacing: {toolbar_item_spacing}px;
                
                /* Effects */
                --shadow-sm-blur: {shadow_sm_blur}px;
                --shadow-sm-offset-x: {shadow_sm_offset_x}px;
                --shadow-sm-offset-y: {shadow_sm_offset_y}px;
                --shadow-sm-color: {shadow_sm_color};
                --shadow-md-blur: {shadow_md_blur}px;
                --shadow-md-offset-x: {shadow_md_offset_x}px;
                --shadow-md-offset-y: {shadow_md_offset_y}px;
                --shadow-md-color: {shadow_md_color};
                --shadow-lg-blur: {shadow_lg_blur}px;
                --shadow-lg-offset-x: {shadow_lg_offset_x}px;
                --shadow-lg-offset-y: {shadow_lg_offset_y}px;
                --shadow-lg-color: {shadow_lg_color};
                --animation-fast: {animation_fast}ms;
                --animation-normal: {animation_normal}ms;
                --animation-slow: {animation_slow}ms;
                --transition-easing: {transition_easing};
                --hover-transition: {hover_transition}ms;
                --focus-transition: {focus_transition}ms;
                
                /* Legacy CSS Variables for Backward Compatibility */
                --window-bg: {window_background};
                --panel-bg: {panel_background};
                --toolbar-bg: {toolbar_background};
                --button-primary: {button_primary_bg};
                --input-bg: {input_background};
                --border-radius: {border_radius_md}px;
                --border-width: {border_width_thin}px;
                --button-height: {button_height_md}px;
                --input-height: {input_height_md}px;
                --space-sm: {space_sm}px;
                --space-md: {space_md}px;
                --space-lg: {space_lg}px;
                --transition-speed: {hover_transition}ms;
            }}
            "#,
            // Background Colors
            window_background = self.colors.window_background,
            panel_background = self.colors.panel_background,
            toolbar_background = self.colors.toolbar_background,
            sidebar_background = self.colors.sidebar_background,
            
            // Button Colors
            button_primary_bg = self.colors.button_primary_bg,
            button_primary_hover = self.colors.button_primary_hover,
            button_primary_active = self.colors.button_primary_active,
            button_secondary_bg = self.colors.button_secondary_bg,
            button_secondary_hover = self.colors.button_secondary_hover,
            button_danger_bg = self.colors.button_danger_bg,
            
            // Input Colors
            input_background = self.colors.input_background,
            input_border = self.colors.input_border,
            input_border_focus = self.colors.input_border_focus,
            input_text = self.colors.input_text,
            input_placeholder = self.colors.input_placeholder,
            
            // Text Colors
            text_primary = self.colors.text_primary,
            text_secondary = self.colors.text_secondary,
            text_disabled = self.colors.text_disabled,
            text_accent = self.colors.text_accent,
            
            // Border Colors
            border_primary = self.colors.border_primary,
            border_secondary = self.colors.border_secondary,
            border_focus = self.colors.border_focus,
            
            // Status Colors
            success = self.colors.success,
            warning = self.colors.warning,
            error = self.colors.error,
            info = self.colors.info,
            
            // Typography
            font_primary = self.typography.primary_font,
            font_mono = self.typography.monospace_font,
            font_size_xs = self.typography.font_size_xs,
            font_size_sm = self.typography.font_size_sm,
            font_size_base = self.typography.font_size_base,
            font_size_md = self.typography.font_size_md,
            font_size_lg = self.typography.font_size_lg,
            font_size_xl = self.typography.font_size_xl,
            font_weight_light = self.typography.weight_light,
            font_weight_normal = self.typography.weight_normal,
            font_weight_medium = self.typography.weight_medium,
            font_weight_semibold = self.typography.weight_semibold,
            font_weight_bold = self.typography.weight_bold,
            line_height_tight = self.typography.line_height_tight,
            line_height_normal = self.typography.line_height_normal,
            line_height_relaxed = self.typography.line_height_relaxed,
            
            // Button Geometry
            button_height_sm = self.geometry.button_height_sm,
            button_height_md = self.geometry.button_height_md,
            button_height_lg = self.geometry.button_height_lg,
            button_min_width = self.geometry.button_min_width,
            button_padding_x = self.geometry.button_padding_x,
            button_padding_y = self.geometry.button_padding_y,
            
            // Input Geometry
            input_height_sm = self.geometry.input_height_sm,
            input_height_md = self.geometry.input_height_md,
            input_height_lg = self.geometry.input_height_lg,
            input_padding_x = self.geometry.input_padding_x,
            input_padding_y = self.geometry.input_padding_y,
            
            // Panel Geometry
            panel_min_width = self.geometry.panel_min_width,
            panel_min_height = self.geometry.panel_min_height,
            panel_header_height = self.geometry.panel_header_height,
            sidebar_width = self.geometry.sidebar_width,
            toolbar_height = self.geometry.toolbar_height,
            
            // Border Radii
            border_radius_none = self.geometry.border_radius_none,
            border_radius_sm = self.geometry.border_radius_sm,
            border_radius_md = self.geometry.border_radius_md,
            border_radius_lg = self.geometry.border_radius_lg,
            border_radius_xl = self.geometry.border_radius_xl,
            border_radius_full = self.geometry.border_radius_full,
            
            // Border Widths
            border_width_none = self.geometry.border_width_none,
            border_width_thin = self.geometry.border_width_thin,
            border_width_thick = self.geometry.border_width_thick,
            
            // Icon Sizes
            icon_size_xs = self.geometry.icon_size_xs,
            icon_size_sm = self.geometry.icon_size_sm,
            icon_size_md = self.geometry.icon_size_md,
            icon_size_lg = self.geometry.icon_size_lg,
            icon_size_xl = self.geometry.icon_size_xl,
            
            // Spacing
            space_none = self.spacing.space_none,
            space_xs = self.spacing.space_xs,
            space_sm = self.spacing.space_sm,
            space_md = self.spacing.space_md,
            space_lg = self.spacing.space_lg,
            space_xl = self.spacing.space_xl,
            space_xxl = self.spacing.space_xxl,
            space_xxxl = self.spacing.space_xxxl,
            button_gap = self.spacing.button_gap,
            panel_padding = self.spacing.panel_padding,
            section_spacing = self.spacing.section_spacing,
            field_spacing = self.spacing.field_spacing,
            toolbar_item_spacing = self.spacing.toolbar_item_spacing,
            
            // Effects
            shadow_sm_blur = self.effects.shadow_sm_blur,
            shadow_sm_offset_x = self.effects.shadow_sm_offset_x,
            shadow_sm_offset_y = self.effects.shadow_sm_offset_y,
            shadow_sm_color = self.effects.shadow_sm_color,
            shadow_md_blur = self.effects.shadow_md_blur,
            shadow_md_offset_x = self.effects.shadow_md_offset_x,
            shadow_md_offset_y = self.effects.shadow_md_offset_y,
            shadow_md_color = self.effects.shadow_md_color,
            shadow_lg_blur = self.effects.shadow_lg_blur,
            shadow_lg_offset_x = self.effects.shadow_lg_offset_x,
            shadow_lg_offset_y = self.effects.shadow_lg_offset_y,
            shadow_lg_color = self.effects.shadow_lg_color,
            animation_fast = self.effects.animation_fast,
            animation_normal = self.effects.animation_normal,
            animation_slow = self.effects.animation_slow,
            transition_easing = self.effects.transition_easing,
            hover_transition = self.effects.hover_transition,
            focus_transition = self.effects.focus_transition,
        )
    }
}