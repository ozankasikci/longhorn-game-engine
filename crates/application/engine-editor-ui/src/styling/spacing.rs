// Longhorn-style spacing and layout constants

use eframe::egui;

/// Longhorn-inspired spacing configuration
pub struct LonghornSpacing;

impl LonghornSpacing {
    // Item spacing
    pub const ITEM_SPACING: egui::Vec2 = egui::vec2(8.0, 4.0);

    // Button padding
    pub const BUTTON_PADDING: egui::Vec2 = egui::vec2(6.0, 2.0);

    // Indentation
    pub const INDENT: f32 = 20.0;

    // Common margins
    pub const MARGIN_SMALL: f32 = 4.0;
    pub const MARGIN_MEDIUM: f32 = 8.0;
    pub const MARGIN_LARGE: f32 = 16.0;

    // Panel spacing
    pub const PANEL_MARGIN: egui::Margin = egui::Margin::same(4.0);
    pub const WINDOW_MARGIN: egui::Margin = egui::Margin::same(8.0);
}

/// Apply Longhorn spacing to egui style
pub fn apply_longhorn_spacing(style: &mut egui::Style) {
    style.spacing.item_spacing = LonghornSpacing::ITEM_SPACING;
    style.spacing.button_padding = LonghornSpacing::BUTTON_PADDING;
    style.spacing.indent = LonghornSpacing::INDENT;
}
