// Widget-specific styling for Unity-style editor

use eframe::egui;
use super::colors::UnityColors;

/// Apply Unity-style widget styling
pub fn apply_unity_widget_styles(style: &mut egui::Style) {
    // Widget styling with better gray hierarchy
    style.visuals.widgets.noninteractive.bg_fill = UnityColors::BG_WIDGET_DEFAULT;
    style.visuals.widgets.noninteractive.bg_stroke = UnityColors::stroke_default();
    
    style.visuals.widgets.inactive.bg_fill = UnityColors::BG_WIDGET_INACTIVE;
    style.visuals.widgets.inactive.bg_stroke = UnityColors::stroke_default();
    
    style.visuals.widgets.hovered.bg_fill = UnityColors::BG_WIDGET_HOVERED;
    style.visuals.widgets.hovered.bg_stroke = UnityColors::stroke_hovered();
    
    style.visuals.widgets.active.bg_fill = UnityColors::ACTIVE_BG;
    style.visuals.widgets.active.bg_stroke = UnityColors::stroke_active();
    
    // Button styling
    style.visuals.button_frame = true;
    
    // Separators and lines
    style.visuals.widgets.noninteractive.fg_stroke = UnityColors::stroke_dark();
}

/// Unity-style button configuration
pub struct UnityButton;

impl UnityButton {
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

/// Unity-style panel configuration
pub struct UnityPanel;

impl UnityPanel {
    pub fn default_margin() -> egui::Margin {
        egui::Margin::same(4.0)
    }
    
    pub fn large_margin() -> egui::Margin {
        egui::Margin::same(8.0)
    }
}