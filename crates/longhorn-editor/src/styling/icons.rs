use egui::{RichText, Color32, FontFamily};
pub use egui_phosphor::regular;

/// Icon size tokens
pub struct IconSize;

impl IconSize {
    /// Extra-small icons for compact UI (arrows, indicators)
    pub const XS: f32 = 10.0;
    /// Small icons inline with text
    pub const SM: f32 = 14.0;
    /// Standard icons for list items
    pub const MD: f32 = 16.0;
    /// Large icons for emphasis
    pub const LG: f32 = 20.0;
}

/// Icon constants using Phosphor icons
pub struct Icons;

impl Icons {
    // ============================================
    // FILE TYPE ICONS
    // ============================================

    pub const FOLDER: &'static str = regular::FOLDER;
    pub const FOLDER_OPEN: &'static str = regular::FOLDER_OPEN;
    pub const FILE: &'static str = regular::FILE;
    pub const FILE_CODE: &'static str = regular::FILE_CODE;
    pub const FILE_IMAGE: &'static str = regular::IMAGE;
    pub const FILE_AUDIO: &'static str = regular::MUSIC_NOTE;
    pub const SCENE: &'static str = regular::CUBE;
    pub const HOME: &'static str = regular::HOUSE;

    // ============================================
    // TREE/NAVIGATION ICONS
    // ============================================

    pub const CARET_RIGHT: &'static str = regular::CARET_RIGHT;
    pub const CARET_DOWN: &'static str = regular::CARET_DOWN;
    pub const CHEVRON_RIGHT: &'static str = regular::CARET_RIGHT;

    // ============================================
    // ACTION ICONS
    // ============================================

    pub const PLUS: &'static str = regular::PLUS;
    pub const TRASH: &'static str = regular::TRASH;
    pub const PENCIL: &'static str = regular::PENCIL;
    pub const REFRESH: &'static str = regular::ARROWS_CLOCKWISE;

    // ============================================
    // ENTITY ICONS
    // ============================================

    pub const ENTITY: &'static str = regular::CUBE;

    // ============================================
    // HELPER METHODS
    // ============================================

    /// Get the phosphor font family
    pub fn font_family() -> FontFamily {
        FontFamily::Name("phosphor".into())
    }

    /// Create an icon with standard size
    pub fn icon(icon: &str) -> RichText {
        RichText::new(icon)
            .family(Self::font_family())
            .size(IconSize::MD)
    }

    /// Create an icon with custom size
    pub fn icon_sized(icon: &str, size: f32) -> RichText {
        RichText::new(icon)
            .family(Self::font_family())
            .size(size)
    }

    /// Create a colored icon
    pub fn icon_colored(icon: &str, color: Color32) -> RichText {
        RichText::new(icon)
            .family(Self::font_family())
            .size(IconSize::MD)
            .color(color)
    }

    /// Create a colored icon with custom size
    pub fn icon_sized_colored(icon: &str, size: f32, color: Color32) -> RichText {
        RichText::new(icon)
            .family(Self::font_family())
            .size(size)
            .color(color)
    }
}

/// Sets up the Phosphor icon font for egui
pub fn setup_icon_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Add Phosphor font data
    fonts.font_data.insert(
        "phosphor".into(),
        egui_phosphor::Variant::Regular.font_data(),
    );

    // Add phosphor as a fallback to the proportional font family
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("phosphor".into());

    // Also register as its own family for explicit use
    fonts.families.insert(
        egui::FontFamily::Name("phosphor".into()),
        vec!["phosphor".into()],
    );

    ctx.set_fonts(fonts);
}
