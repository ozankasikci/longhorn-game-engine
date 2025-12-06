use egui::{Margin, Style, Vec2, vec2};
use super::colors::Colors;

pub fn apply_widget_styles(style: &mut Style) {
    // Widget styling with gray hierarchy
    style.visuals.widgets.noninteractive.bg_fill = Colors::BG_WIDGET_DEFAULT;
    style.visuals.widgets.noninteractive.bg_stroke = Colors::stroke_default();

    style.visuals.widgets.inactive.bg_fill = Colors::BG_WIDGET_INACTIVE;
    style.visuals.widgets.inactive.bg_stroke = Colors::stroke_default();

    style.visuals.widgets.hovered.bg_fill = Colors::BG_WIDGET_HOVERED;
    style.visuals.widgets.hovered.bg_stroke = Colors::stroke_hovered();

    style.visuals.widgets.active.bg_fill = Colors::ACTIVE_BG;
    style.visuals.widgets.active.bg_stroke = Colors::stroke_active();

    // Button styling
    style.visuals.button_frame = true;

    // Separators and lines
    style.visuals.widgets.noninteractive.fg_stroke = Colors::stroke_dark();
}

/// Button size presets
pub struct ButtonSize;

impl ButtonSize {
    pub fn small() -> Vec2 {
        vec2(60.0, 20.0)
    }

    pub fn medium() -> Vec2 {
        vec2(80.0, 24.0)
    }

    pub fn large() -> Vec2 {
        vec2(120.0, 32.0)
    }
}

/// Panel configuration
pub struct PanelStyle;

impl PanelStyle {
    pub fn default_margin() -> Margin {
        Margin::same(4.0)
    }

    pub fn large_margin() -> Margin {
        Margin::same(8.0)
    }
}
