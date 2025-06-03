// Font configuration for Longhorn-style editor

use eframe::egui;

/// Font sizes used throughout the editor
pub struct LonghornFonts;

impl LonghornFonts {
    pub const SMALL: f32 = 10.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 14.0;
    pub const HEADING: f32 = 16.0;
    pub const TITLE: f32 = 18.0;
}

/// Setup custom fonts for Longhorn-like appearance
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    
    // For now, use default fonts with adjusted sizes
    // TODO: Add custom Longhorn-like fonts later
    // Could add fonts like:
    // - Inter for UI text
    // - JetBrains Mono for code
    // - Roboto for general text
    
    ctx.set_fonts(fonts);
}

/// Apply font styling to egui style
pub fn apply_longhorn_font_styles(style: &mut egui::Style) {
    // Font styling can be added here when custom fonts are implemented
    // For now, rely on default font configuration
}