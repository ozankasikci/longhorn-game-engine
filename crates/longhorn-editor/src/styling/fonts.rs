use egui::Style;

/// Font sizes used throughout the editor
pub struct Fonts;

impl Fonts {
    pub const SMALL: f32 = 10.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 14.0;
    pub const HEADING: f32 = 16.0;
    pub const TITLE: f32 = 18.0;
}

pub fn apply_font_styles(_style: &mut Style) {
    // Font styling can be added here when custom fonts are implemented
}
