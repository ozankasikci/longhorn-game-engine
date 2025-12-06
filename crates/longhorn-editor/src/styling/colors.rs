use egui::Color32;

/// Longhorn Game Engine color palette with grayscale hierarchy
/// Darker values to match v1 appearance
pub struct Colors;

impl Colors {
    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_gray(200);

    // Background colors (from darkest to lightest) - darker than before
    pub const BG_EXTREME: Color32 = Color32::from_gray(20);
    pub const BG_PANEL: Color32 = Color32::from_gray(30);
    pub const BG_WINDOW: Color32 = Color32::from_gray(35);
    pub const BG_WIDGET_INACTIVE: Color32 = Color32::from_gray(38);
    pub const BG_WIDGET_DEFAULT: Color32 = Color32::from_gray(40);
    pub const BG_WIDGET_HOVERED: Color32 = Color32::from_gray(50);

    // Border/stroke colors
    pub const STROKE_DARK: Color32 = Color32::from_gray(15);
    pub const STROKE_DEFAULT: Color32 = Color32::from_gray(25);
    pub const STROKE_HOVERED: Color32 = Color32::from_gray(70);
    pub const STROKE_ACTIVE: Color32 = Color32::from_gray(100);

    // Accent colors - balanced blue (between muted and vibrant)
    pub const ACCENT: Color32 = Color32::from_rgb(50, 110, 180);
    pub const ACCENT_MUTED: Color32 = Color32::from_rgb(44, 93, 135);

    // Selection uses the balanced accent
    pub const SELECTION_BG: Color32 = Color32::from_rgb(50, 110, 180);

    // Active state colors
    pub const ACTIVE_BG: Color32 = Color32::from_gray(35);
}

impl Colors {
    pub fn stroke_default() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DEFAULT)
    }

    pub fn stroke_dark() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DARK)
    }

    pub fn stroke_hovered() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_HOVERED)
    }

    pub fn stroke_active() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_ACTIVE)
    }
}
