// Color system for unified components

use serde::{Serialize, Deserialize};
use std::fmt;

/// RGB Color with alpha channel
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32, // 0.0 - 1.0
    pub g: f32, // 0.0 - 1.0  
    pub b: f32, // 0.0 - 1.0
    pub a: f32, // 0.0 - 1.0
}

impl Color {
    /// Create RGB color (alpha = 1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    
    /// Create RGBA color
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    /// Create from hex string (#RRGGBB or #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        csscolorparser::parse(hex)
            .map(|color| Self::rgba(
                color.r as f32,
                color.g as f32, 
                color.b as f32,
                color.a as f32
            ))
            .map_err(|e| format!("Invalid hex color: {}", e))
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.a < 1.0 {
            format!("#{:02x}{:02x}{:02x}{:02x}", 
                (self.r * 255.0) as u8,
                (self.g * 255.0) as u8,
                (self.b * 255.0) as u8,
                (self.a * 255.0) as u8
            )
        } else {
            format!("#{:02x}{:02x}{:02x}",
                (self.r * 255.0) as u8,
                (self.g * 255.0) as u8,
                (self.b * 255.0) as u8
            )
        }
    }
    
    /// Convert to GTK RGBA
    pub fn to_gdk_rgba(&self) -> gdk4::RGBA {
        gdk4::RGBA::new(self.r, self.g, self.b, self.a)
    }
    
    /// Create from GTK RGBA
    pub fn from_gdk_rgba(rgba: &gdk4::RGBA) -> Self {
        Self::rgba(rgba.red(), rgba.green(), rgba.blue(), rgba.alpha())
    }
    
    /// Lighten color by percentage (0.0 - 1.0)
    pub fn lighten(&self, amount: f32) -> Self {
        Self::rgba(
            (self.r + amount).clamp(0.0, 1.0),
            (self.g + amount).clamp(0.0, 1.0),
            (self.b + amount).clamp(0.0, 1.0),
            self.a
        )
    }
    
    /// Darken color by percentage (0.0 - 1.0)
    pub fn darken(&self, amount: f32) -> Self {
        Self::rgba(
            (self.r - amount).clamp(0.0, 1.0),
            (self.g - amount).clamp(0.0, 1.0),
            (self.b - amount).clamp(0.0, 1.0),
            self.a
        )
    }
    
    /// Set alpha channel
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self::rgba(self.r, self.g, self.b, alpha.clamp(0.0, 1.0))
    }

    /// Common colors
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Complete color palette for the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Primary brand colors
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub primary_disabled: Color,
    
    // Secondary colors
    pub secondary: Color,
    pub secondary_hover: Color,
    pub secondary_active: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Neutral colors
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    
    // Border colors
    pub border: Color,
    pub border_hover: Color,
    pub border_focus: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_inverse: Color,
    
    // Special colors
    pub shadow: Color,
    pub overlay: Color,
    pub selection: Color,
}

impl ColorPalette {
    /// Unity-style dark theme colors
    pub fn unity_dark() -> Self {
        Self {
            // Primary (Unity blue)
            primary: Color::from_hex("#007AFF").unwrap(),
            primary_hover: Color::from_hex("#0056CC").unwrap(),
            primary_active: Color::from_hex("#003D99").unwrap(),
            primary_disabled: Color::from_hex("#4D7DFF").unwrap(),
            
            // Secondary (green)
            secondary: Color::from_hex("#34C759").unwrap(),
            secondary_hover: Color::from_hex("#28A745").unwrap(),
            secondary_active: Color::from_hex("#1E7E34").unwrap(),
            
            // Semantic
            success: Color::from_hex("#34C759").unwrap(),
            warning: Color::from_hex("#FF9500").unwrap(),
            error: Color::from_hex("#FF3B30").unwrap(),
            info: Color::from_hex("#007AFF").unwrap(),
            
            // Neutral backgrounds
            background: Color::from_hex("#1E1E1E").unwrap(),      // Main background
            surface: Color::from_hex("#2D2D2D").unwrap(),         // Panel background
            surface_elevated: Color::from_hex("#3A3A3A").unwrap(), // Elevated panels
            surface_hover: Color::from_hex("#404040").unwrap(),    // Hover state
            surface_active: Color::from_hex("#4A4A4A").unwrap(),   // Active state
            
            // Borders
            border: Color::from_hex("#404040").unwrap(),
            border_hover: Color::from_hex("#5A5A5A").unwrap(),
            border_focus: Color::from_hex("#007AFF").unwrap(),
            
            // Text
            text_primary: Color::from_hex("#FFFFFF").unwrap(),
            text_secondary: Color::from_hex("#ABABAB").unwrap(),
            text_disabled: Color::from_hex("#666666").unwrap(),
            text_inverse: Color::from_hex("#000000").unwrap(),
            
            // Special
            shadow: Color::rgba(0.0, 0.0, 0.0, 0.3),
            overlay: Color::rgba(0.0, 0.0, 0.0, 0.6),
            selection: Color::from_hex("#007AFF").unwrap().with_alpha(0.3),
        }
    }
    
    /// Unity-style light theme colors
    pub fn unity_light() -> Self {
        Self {
            // Primary (Unity blue)
            primary: Color::from_hex("#007AFF").unwrap(),
            primary_hover: Color::from_hex("#0056CC").unwrap(),
            primary_active: Color::from_hex("#003D99").unwrap(),
            primary_disabled: Color::from_hex("#B3D4FF").unwrap(),
            
            // Secondary (green)
            secondary: Color::from_hex("#34C759").unwrap(),
            secondary_hover: Color::from_hex("#28A745").unwrap(),
            secondary_active: Color::from_hex("#1E7E34").unwrap(),
            
            // Semantic
            success: Color::from_hex("#34C759").unwrap(),
            warning: Color::from_hex("#FF9500").unwrap(),
            error: Color::from_hex("#FF3B30").unwrap(),
            info: Color::from_hex("#007AFF").unwrap(),
            
            // Neutral backgrounds
            background: Color::from_hex("#FFFFFF").unwrap(),      // Main background
            surface: Color::from_hex("#F5F5F5").unwrap(),         // Panel background
            surface_elevated: Color::from_hex("#FFFFFF").unwrap(), // Elevated panels
            surface_hover: Color::from_hex("#F0F0F0").unwrap(),    // Hover state
            surface_active: Color::from_hex("#E5E5E5").unwrap(),   // Active state
            
            // Borders
            border: Color::from_hex("#D1D1D1").unwrap(),
            border_hover: Color::from_hex("#B0B0B0").unwrap(),
            border_focus: Color::from_hex("#007AFF").unwrap(),
            
            // Text
            text_primary: Color::from_hex("#000000").unwrap(),
            text_secondary: Color::from_hex("#666666").unwrap(),
            text_disabled: Color::from_hex("#ABABAB").unwrap(),
            text_inverse: Color::from_hex("#FFFFFF").unwrap(),
            
            // Special
            shadow: Color::rgba(0.0, 0.0, 0.0, 0.1),
            overlay: Color::rgba(0.0, 0.0, 0.0, 0.4),
            selection: Color::from_hex("#007AFF").unwrap().with_alpha(0.2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert_eq!(color.r, 1.0);
        assert!((color.g - 0.5).abs() < 0.01); // Allow for floating point precision
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.to_hex(), "#ff8000");
    }

    #[test]
    fn test_color_manipulation() {
        let color = Color::rgb(0.5, 0.5, 0.5);
        let lighter = color.lighten(0.2);
        let darker = color.darken(0.2);
        
        assert!(lighter.r > color.r);
        assert!(darker.r < color.r);
    }

    #[test]
    fn test_color_palettes() {
        let dark = ColorPalette::unity_dark();
        let light = ColorPalette::unity_light();
        
        // Dark theme should have dark background
        assert!(dark.background.r < 0.5);
        // Light theme should have light background  
        assert!(light.background.r > 0.5);
        
        // Both should have same primary color
        assert_eq!(dark.primary, light.primary);
    }
}