// Typography system for unified components

use serde::{Serialize, Deserialize};

/// Font sizes following a consistent scale
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FontSizes {
    pub xs: f32,     // 10px - Small labels, captions
    pub sm: f32,     // 12px - Body text small
    pub base: f32,   // 14px - Default body text
    pub lg: f32,     // 16px - Headings, important text
    pub xl: f32,     // 18px - Large headings
    pub xxl: f32,    // 24px - Page titles
    pub xxxl: f32,   // 32px - Display text
}

impl FontSizes {
    /// Standard font scale
    pub fn standard() -> Self {
        Self {
            xs: 10.0,
            sm: 12.0,
            base: 14.0,
            lg: 16.0,
            xl: 18.0,
            xxl: 24.0,
            xxxl: 32.0,
        }
    }
    
    /// Compact font scale (for dense UIs)
    pub fn compact() -> Self {
        Self {
            xs: 9.0,
            sm: 10.0,
            base: 12.0,
            lg: 14.0,
            xl: 16.0,
            xxl: 20.0,
            xxxl: 28.0,
        }
    }
}

impl Default for FontSizes {
    fn default() -> Self {
        Self::standard()
    }
}

/// Font weights
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FontWeights {
    pub light: u32,     // 300
    pub normal: u32,    // 400
    pub medium: u32,    // 500
    pub semibold: u32,  // 600
    pub bold: u32,      // 700
}

impl FontWeights {
    pub fn standard() -> Self {
        Self {
            light: 300,
            normal: 400,
            medium: 500,
            semibold: 600,
            bold: 700,
        }
    }
}

impl Default for FontWeights {
    fn default() -> Self {
        Self::standard()
    }
}

/// Line heights for different text contexts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LineHeights {
    pub tight: f32,    // 1.2 - Headings
    pub normal: f32,   // 1.4 - Body text
    pub relaxed: f32,  // 1.6 - Reading text
    pub loose: f32,    // 2.0 - Large text
}

impl LineHeights {
    pub fn standard() -> Self {
        Self {
            tight: 1.2,
            normal: 1.4,
            relaxed: 1.6,
            loose: 2.0,
        }
    }
}

impl Default for LineHeights {
    fn default() -> Self {
        Self::standard()
    }
}

/// Complete typography configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    // Font families
    pub font_family_primary: String,   // Main UI font
    pub font_family_mono: String,      // Code/monospace font
    pub font_family_display: String,   // Headings/display font
    
    // Font scales
    pub sizes: FontSizes,
    pub weights: FontWeights,
    pub line_heights: LineHeights,
    
    // Letter spacing
    pub letter_spacing_tight: f32,     // -0.025em
    pub letter_spacing_normal: f32,    // 0em
    pub letter_spacing_wide: f32,      // 0.025em
}

impl Typography {
    /// macOS/iOS system typography
    pub fn system_apple() -> Self {
        Self {
            font_family_primary: "SF Pro Display".to_string(),
            font_family_mono: "SF Mono".to_string(),
            font_family_display: "SF Pro Display".to_string(),
            sizes: FontSizes::standard(),
            weights: FontWeights::standard(),
            line_heights: LineHeights::standard(),
            letter_spacing_tight: -0.025,
            letter_spacing_normal: 0.0,
            letter_spacing_wide: 0.025,
        }
    }
    
    /// Windows system typography
    pub fn system_windows() -> Self {
        Self {
            font_family_primary: "Segoe UI".to_string(),
            font_family_mono: "Consolas".to_string(),
            font_family_display: "Segoe UI".to_string(),
            sizes: FontSizes::standard(),
            weights: FontWeights::standard(),
            line_heights: LineHeights::standard(),
            letter_spacing_tight: -0.025,
            letter_spacing_normal: 0.0,
            letter_spacing_wide: 0.025,
        }
    }
    
    /// Linux system typography
    pub fn system_linux() -> Self {
        Self {
            font_family_primary: "Inter".to_string(),
            font_family_mono: "JetBrains Mono".to_string(),
            font_family_display: "Inter".to_string(),
            sizes: FontSizes::standard(),
            weights: FontWeights::standard(),
            line_heights: LineHeights::standard(),
            letter_spacing_tight: -0.025,
            letter_spacing_normal: 0.0,
            letter_spacing_wide: 0.025,
        }
    }
    
    /// Compact typography for dense interfaces
    pub fn compact() -> Self {
        Self {
            font_family_primary: "SF Pro Display".to_string(),
            font_family_mono: "SF Mono".to_string(), 
            font_family_display: "SF Pro Display".to_string(),
            sizes: FontSizes::compact(),
            weights: FontWeights::standard(),
            line_heights: LineHeights::standard(),
            letter_spacing_tight: -0.025,
            letter_spacing_normal: 0.0,
            letter_spacing_wide: 0.025,
        }
    }
    
    /// Auto-detect system typography
    pub fn system_default() -> Self {
        #[cfg(target_os = "macos")]
        return Self::system_apple();
        
        #[cfg(target_os = "windows")]
        return Self::system_windows();
        
        #[cfg(target_os = "linux")]
        return Self::system_linux();
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return Self::system_linux(); // Fallback
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self::system_default()
    }
}

/// Text style presets for common UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextStyle {
    // Headings
    DisplayLarge,       // Page titles
    DisplayMedium,      // Section headings
    DisplaySmall,       // Subsection headings
    
    // Headlines
    HeadlineLarge,      // Card titles
    HeadlineMedium,     // List headers
    HeadlineSmall,      // Group labels
    
    // Titles
    TitleLarge,         // Panel titles
    TitleMedium,        // Dialog titles
    TitleSmall,         // Field labels
    
    // Labels
    LabelLarge,         // Button text
    LabelMedium,        // Tab labels
    LabelSmall,         // Captions
    
    // Body
    BodyLarge,          // Main content
    BodyMedium,         // Secondary content
    BodySmall,          // Helper text
    
    // Special
    Code,               // Code snippets
    Caption,            // Image captions
}

impl TextStyle {
    /// Get font size for this text style
    pub fn font_size(&self, typography: &Typography) -> f32 {
        match self {
            TextStyle::DisplayLarge => typography.sizes.xxxl,
            TextStyle::DisplayMedium => typography.sizes.xxl,
            TextStyle::DisplaySmall => typography.sizes.xl,
            
            TextStyle::HeadlineLarge => typography.sizes.xl,
            TextStyle::HeadlineMedium => typography.sizes.lg,
            TextStyle::HeadlineSmall => typography.sizes.base,
            
            TextStyle::TitleLarge => typography.sizes.lg,
            TextStyle::TitleMedium => typography.sizes.base,
            TextStyle::TitleSmall => typography.sizes.sm,
            
            TextStyle::LabelLarge => typography.sizes.base,
            TextStyle::LabelMedium => typography.sizes.sm,
            TextStyle::LabelSmall => typography.sizes.xs,
            
            TextStyle::BodyLarge => typography.sizes.lg,
            TextStyle::BodyMedium => typography.sizes.base,
            TextStyle::BodySmall => typography.sizes.sm,
            
            TextStyle::Code => typography.sizes.sm,
            TextStyle::Caption => typography.sizes.xs,
        }
    }
    
    /// Get font weight for this text style
    pub fn font_weight(&self, typography: &Typography) -> u32 {
        match self {
            TextStyle::DisplayLarge | TextStyle::DisplayMedium | TextStyle::DisplaySmall => {
                typography.weights.bold
            },
            TextStyle::HeadlineLarge | TextStyle::HeadlineMedium | TextStyle::HeadlineSmall => {
                typography.weights.semibold
            },
            TextStyle::TitleLarge | TextStyle::TitleMedium | TextStyle::TitleSmall => {
                typography.weights.medium
            },
            TextStyle::LabelLarge | TextStyle::LabelMedium => {
                typography.weights.medium
            },
            _ => typography.weights.normal,
        }
    }
    
    /// Get line height for this text style
    pub fn line_height(&self, typography: &Typography) -> f32 {
        match self {
            TextStyle::DisplayLarge | TextStyle::DisplayMedium | TextStyle::DisplaySmall |
            TextStyle::HeadlineLarge | TextStyle::HeadlineMedium | TextStyle::HeadlineSmall => {
                typography.line_heights.tight
            },
            TextStyle::BodyLarge | TextStyle::BodyMedium => {
                typography.line_heights.relaxed
            },
            _ => typography.line_heights.normal,
        }
    }
    
    /// Get font family for this text style
    pub fn font_family<'a>(&self, typography: &'a Typography) -> &'a str {
        match self {
            TextStyle::Code => &typography.font_family_mono,
            TextStyle::DisplayLarge | TextStyle::DisplayMedium | TextStyle::DisplaySmall => {
                &typography.font_family_display
            },
            _ => &typography.font_family_primary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_sizes() {
        let sizes = FontSizes::standard();
        assert!(sizes.xs < sizes.sm);
        assert!(sizes.sm < sizes.base);
        assert!(sizes.base < sizes.lg);
    }

    #[test]
    fn test_typography_system_detection() {
        let typography = Typography::system_default();
        assert!(!typography.font_family_primary.is_empty());
        assert!(!typography.font_family_mono.is_empty());
    }

    #[test]
    fn test_text_style_sizes() {
        let typography = Typography::default();
        
        let display_size = TextStyle::DisplayLarge.font_size(&typography);
        let body_size = TextStyle::BodyMedium.font_size(&typography);
        let caption_size = TextStyle::Caption.font_size(&typography);
        
        assert!(display_size > body_size);
        assert!(body_size > caption_size);
    }

    #[test]
    fn test_text_style_weights() {
        let typography = Typography::default();
        
        let display_weight = TextStyle::DisplayLarge.font_weight(&typography);
        let body_weight = TextStyle::BodyMedium.font_weight(&typography);
        
        assert!(display_weight > body_weight);
    }
}