use egui::{Margin, Style, Vec2, vec2};

/// Longhorn-inspired spacing configuration
pub struct Spacing;

impl Spacing {
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
