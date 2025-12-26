use egui::RichText;
use super::Colors;

/// Typography tokens for consistent text styling
pub struct Typography;

impl Typography {
    // ============================================
    // FONT SIZES
    // ============================================

    /// Extra small - file sizes, timestamps, badges
    pub const SIZE_XS: f32 = 10.0;
    /// Small - section headers (FOLDERS, FILES)
    pub const SIZE_SM: f32 = 11.0;
    /// Base - body text, list items
    pub const SIZE_BASE: f32 = 13.0;
    /// Large - panel headings
    pub const SIZE_LG: f32 = 14.0;
    /// Extra large - dialog titles
    pub const SIZE_XL: f32 = 16.0;

    // ============================================
    // TEXT STYLE HELPERS
    // ============================================

    /// Section header style (e.g., "FOLDERS", "FILES")
    pub fn section_header(text: impl Into<String>) -> RichText {
        RichText::new(text)
            .size(Self::SIZE_SM)
            .color(Colors::TEXT_MUTED)
            .strong()
    }

    /// Body text style
    pub fn body(text: impl Into<String>) -> RichText {
        RichText::new(text).size(Self::SIZE_BASE)
    }

    /// Muted/secondary text style
    pub fn muted(text: impl Into<String>) -> RichText {
        RichText::new(text)
            .size(Self::SIZE_XS)
            .color(Colors::TEXT_MUTED)
    }

    /// Label text style (slightly smaller than body)
    pub fn label(text: impl Into<String>) -> RichText {
        RichText::new(text)
            .size(Self::SIZE_SM)
            .color(Colors::TEXT_SECONDARY)
    }

    /// Heading style for panel titles
    pub fn heading(text: impl Into<String>) -> RichText {
        RichText::new(text)
            .size(Self::SIZE_LG)
            .strong()
    }

    /// Empty state text (e.g., "Empty folder", "No entities")
    pub fn empty_state(text: impl Into<String>) -> RichText {
        RichText::new(text)
            .size(Self::SIZE_BASE)
            .color(Colors::TEXT_MUTED)
    }
}
