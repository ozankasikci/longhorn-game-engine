// Spacing and layout system for unified components

use serde::{Serialize, Deserialize};

/// Consistent spacing scale
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Spacing {
    pub none: f32,   // 0px
    pub xs: f32,     // 4px - Minimal spacing
    pub sm: f32,     // 8px - Small spacing
    pub md: f32,     // 16px - Default spacing
    pub lg: f32,     // 24px - Large spacing
    pub xl: f32,     // 32px - Extra large spacing
    pub xxl: f32,    // 48px - Section spacing
    pub xxxl: f32,   // 64px - Page spacing
}

impl Spacing {
    /// Standard spacing scale (4px base unit)
    pub fn standard() -> Self {
        Self {
            none: 0.0,
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            xxxl: 64.0,
        }
    }
    
    /// Compact spacing scale (for dense interfaces)
    pub fn compact() -> Self {
        Self {
            none: 0.0,
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 24.0,
            xxxl: 32.0,
        }
    }
    
    /// Relaxed spacing scale (for comfortable interfaces)
    pub fn relaxed() -> Self {
        Self {
            none: 0.0,
            xs: 6.0,
            sm: 12.0,
            md: 20.0,
            lg: 28.0,
            xl: 40.0,
            xxl: 56.0,
            xxxl: 72.0,
        }
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Self::standard()
    }
}

/// Component sizes for consistent UI elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sizes {
    // Button sizes
    pub button_height_sm: f32,      // 24px
    pub button_height_md: f32,      // 32px
    pub button_height_lg: f32,      // 40px
    pub button_min_width: f32,      // 64px
    
    // Input sizes
    pub input_height_sm: f32,       // 24px
    pub input_height_md: f32,       // 32px
    pub input_height_lg: f32,       // 40px
    
    // Icon sizes
    pub icon_xs: f32,               // 12px
    pub icon_sm: f32,               // 16px
    pub icon_md: f32,               // 20px
    pub icon_lg: f32,               // 24px
    pub icon_xl: f32,               // 32px
    
    // Panel and container sizes
    pub panel_min_width: f32,       // 200px
    pub panel_min_height: f32,      // 150px
    pub sidebar_width: f32,         // 250px
    pub toolbar_height: f32,        // 40px
    
    // Border and corner radii
    pub border_width: f32,          // 1px
    pub border_radius_sm: f32,      // 4px
    pub border_radius_md: f32,      // 6px
    pub border_radius_lg: f32,      // 8px
    pub border_radius_xl: f32,      // 12px
    
    // Shadow and elevation
    pub shadow_sm: f32,             // 2px blur
    pub shadow_md: f32,             // 4px blur
    pub shadow_lg: f32,             // 8px blur
    pub shadow_xl: f32,             // 16px blur
}

impl Sizes {
    /// Standard component sizes
    pub fn standard() -> Self {
        Self {
            // Buttons
            button_height_sm: 24.0,
            button_height_md: 32.0,
            button_height_lg: 40.0,
            button_min_width: 64.0,
            
            // Inputs
            input_height_sm: 24.0,
            input_height_md: 32.0,
            input_height_lg: 40.0,
            
            // Icons
            icon_xs: 12.0,
            icon_sm: 16.0,
            icon_md: 20.0,
            icon_lg: 24.0,
            icon_xl: 32.0,
            
            // Panels
            panel_min_width: 200.0,
            panel_min_height: 150.0,
            sidebar_width: 250.0,
            toolbar_height: 40.0,
            
            // Borders
            border_width: 1.0,
            border_radius_sm: 4.0,
            border_radius_md: 6.0,
            border_radius_lg: 8.0,
            border_radius_xl: 12.0,
            
            // Shadows
            shadow_sm: 2.0,
            shadow_md: 4.0,
            shadow_lg: 8.0,
            shadow_xl: 16.0,
        }
    }
    
    /// Compact sizes (for dense interfaces like Unity)
    pub fn compact() -> Self {
        Self {
            // Buttons
            button_height_sm: 20.0,
            button_height_md: 28.0,
            button_height_lg: 36.0,
            button_min_width: 48.0,
            
            // Inputs
            input_height_sm: 20.0,
            input_height_md: 28.0,
            input_height_lg: 36.0,
            
            // Icons
            icon_xs: 10.0,
            icon_sm: 14.0,
            icon_md: 18.0,
            icon_lg: 22.0,
            icon_xl: 28.0,
            
            // Panels
            panel_min_width: 180.0,
            panel_min_height: 120.0,
            sidebar_width: 220.0,
            toolbar_height: 32.0,
            
            // Borders
            border_width: 1.0,
            border_radius_sm: 3.0,
            border_radius_md: 4.0,
            border_radius_lg: 6.0,
            border_radius_xl: 8.0,
            
            // Shadows
            shadow_sm: 1.0,
            shadow_md: 2.0,
            shadow_lg: 4.0,
            shadow_xl: 8.0,
        }
    }
}

impl Default for Sizes {
    fn default() -> Self {
        Self::compact() // Use compact by default for Unity-style editor
    }
}

/// Layout configuration for consistent positioning
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Layout {
    // Z-index layers
    pub z_dropdown: i32,            // 1000
    pub z_sticky: i32,              // 1020
    pub z_fixed: i32,               // 1030
    pub z_modal_backdrop: i32,      // 1040
    pub z_modal: i32,               // 1050
    pub z_popover: i32,             // 1060
    pub z_tooltip: i32,             // 1070
    
    // Timing (for animations)
    pub timing_fast: f32,           // 150ms
    pub timing_base: f32,           // 250ms
    pub timing_slow: f32,           // 350ms
    
    // Breakpoints (for responsive design)
    pub breakpoint_sm: f32,         // 640px
    pub breakpoint_md: f32,         // 768px
    pub breakpoint_lg: f32,         // 1024px
    pub breakpoint_xl: f32,         // 1280px
}

impl Layout {
    pub fn standard() -> Self {
        Self {
            // Z-index
            z_dropdown: 1000,
            z_sticky: 1020,
            z_fixed: 1030,
            z_modal_backdrop: 1040,
            z_modal: 1050,
            z_popover: 1060,
            z_tooltip: 1070,
            
            // Timing
            timing_fast: 150.0,
            timing_base: 250.0,
            timing_slow: 350.0,
            
            // Breakpoints
            breakpoint_sm: 640.0,
            breakpoint_md: 768.0,
            breakpoint_lg: 1024.0,
            breakpoint_xl: 1280.0,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::standard()
    }
}

/// Padding helper struct
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    /// Uniform padding on all sides
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    /// Horizontal and vertical padding
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
    
    /// Individual side padding
    pub fn custom(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
    
    /// No padding
    pub fn none() -> Self {
        Self::all(0.0)
    }
}

/// Margin helper struct
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    /// Uniform margin on all sides
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    /// Horizontal and vertical margin
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
    
    /// Individual side margin
    pub fn custom(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
    
    /// No margin
    pub fn none() -> Self {
        Self::all(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_scale() {
        let spacing = Spacing::standard();
        assert_eq!(spacing.none, 0.0);
        assert!(spacing.xs < spacing.sm);
        assert!(spacing.sm < spacing.md);
        assert!(spacing.md < spacing.lg);
    }

    #[test]
    fn test_sizes() {
        let sizes = Sizes::standard();
        assert!(sizes.button_height_sm < sizes.button_height_md);
        assert!(sizes.button_height_md < sizes.button_height_lg);
        assert!(sizes.icon_xs < sizes.icon_xl);
    }

    #[test]
    fn test_padding() {
        let padding = Padding::all(16.0);
        assert_eq!(padding.top, 16.0);
        assert_eq!(padding.right, 16.0);
        assert_eq!(padding.bottom, 16.0);
        assert_eq!(padding.left, 16.0);
        
        let symmetric = Padding::symmetric(8.0, 16.0);
        assert_eq!(symmetric.top, 16.0);
        assert_eq!(symmetric.right, 8.0);
        assert_eq!(symmetric.bottom, 16.0);
        assert_eq!(symmetric.left, 8.0);
    }

    #[test]
    fn test_compact_vs_standard() {
        let standard = Sizes::standard();
        let compact = Sizes::compact();
        
        assert!(compact.button_height_md < standard.button_height_md);
        assert!(compact.toolbar_height < standard.toolbar_height);
    }
}