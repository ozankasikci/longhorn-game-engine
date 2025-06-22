// Widget-specific styling for Longhorn-style editor

use eframe::egui;
use super::colors::LonghornColors;

/// Apply Longhorn-style widget styling
pub fn apply_longhorn_widget_styles(style: &mut egui::Style) {
    // Widget styling with better gray hierarchy
    style.visuals.widgets.noninteractive.bg_fill = LonghornColors::BG_WIDGET_DEFAULT;
    style.visuals.widgets.noninteractive.bg_stroke = LonghornColors::stroke_default();
    
    style.visuals.widgets.inactive.bg_fill = LonghornColors::BG_WIDGET_INACTIVE;
    style.visuals.widgets.inactive.bg_stroke = LonghornColors::stroke_default();
    
    style.visuals.widgets.hovered.bg_fill = LonghornColors::BG_WIDGET_HOVERED;
    style.visuals.widgets.hovered.bg_stroke = LonghornColors::stroke_hovered();
    
    style.visuals.widgets.active.bg_fill = LonghornColors::ACTIVE_BG;
    style.visuals.widgets.active.bg_stroke = LonghornColors::stroke_active();
    
    // Button styling
    style.visuals.button_frame = true;
    
    // Separators and lines
    style.visuals.widgets.noninteractive.fg_stroke = LonghornColors::stroke_dark();
}

/// Longhorn-style button configuration
pub struct LonghornButton;

impl LonghornButton {
    pub fn small() -> egui::Vec2 {
        egui::vec2(60.0, 20.0)
    }
    
    pub fn medium() -> egui::Vec2 {
        egui::vec2(80.0, 24.0)
    }
    
    pub fn large() -> egui::Vec2 {
        egui::vec2(120.0, 32.0)
    }
}

/// Longhorn-style panel configuration
pub struct LonghornPanel;

impl LonghornPanel {
    pub fn default_margin() -> egui::Margin {
        egui::Margin::same(4.0)
    }
    
    pub fn large_margin() -> egui::Margin {
        egui::Margin::same(8.0)
    }
}