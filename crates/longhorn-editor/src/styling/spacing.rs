use egui::{Margin, Style, Vec2, vec2};

/// Semantic spacing tokens for consistent UI layout
/// Based on a 4px grid for visual consistency
pub struct Spacing;

impl Spacing {
    // ============================================
    // BASE UNIT
    // ============================================

    /// Base spacing unit (4px grid)
    pub const UNIT: f32 = 4.0;

    // ============================================
    // ROW HEIGHTS
    // ============================================

    /// Standard list item row height
    pub const ROW_HEIGHT: f32 = 24.0;
    /// Compact row height for tree nodes
    pub const ROW_HEIGHT_SMALL: f32 = 20.0;

    // ============================================
    // GAPS BETWEEN ELEMENTS
    // ============================================

    /// Gap between icon and text label
    pub const ICON_TEXT_GAP: f32 = 6.0;
    /// Gap between major sections (e.g., FOLDERS and FILES)
    pub const SECTION_GAP: f32 = 12.0;
    /// Gap between list items
    pub const ITEM_GAP: f32 = 2.0;

    // ============================================
    // INDENTATION
    // ============================================

    /// Tree hierarchy indentation per level
    pub const TREE_INDENT: f32 = 16.0;

    // ============================================
    // PADDING
    // ============================================

    /// Horizontal padding inside list items
    pub const LIST_ITEM_PADDING_H: f32 = 8.0;
    /// Vertical padding inside list items
    pub const LIST_ITEM_PADDING_V: f32 = 4.0;
    /// Panel edge padding
    pub const PANEL_PADDING: f32 = 8.0;

    // ============================================
    // SECTION HEADERS
    // ============================================

    /// Space above section header
    pub const SECTION_HEADER_TOP: f32 = 8.0;
    /// Space below section header
    pub const SECTION_HEADER_BOTTOM: f32 = 4.0;

    // ============================================
    // LEGACY (for backwards compatibility)
    // ============================================

    pub const ITEM_SPACING: Vec2 = vec2(8.0, 4.0);
    pub const BUTTON_PADDING: Vec2 = vec2(6.0, 2.0);
    pub const INDENT: f32 = 20.0;
    pub const MARGIN_SMALL: f32 = 4.0;
    pub const MARGIN_MEDIUM: f32 = 8.0;
    pub const MARGIN_LARGE: f32 = 16.0;
    pub const PANEL_MARGIN: Margin = Margin::same(4.0);
    pub const WINDOW_MARGIN: Margin = Margin::same(8.0);
}

pub fn apply_spacing(style: &mut Style) {
    style.spacing.item_spacing = Spacing::ITEM_SPACING;
    style.spacing.button_padding = Spacing::BUTTON_PADDING;
    style.spacing.indent = Spacing::INDENT;
}
