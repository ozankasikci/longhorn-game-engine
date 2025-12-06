use egui::Context;
use super::colors::Colors;
use super::fonts::apply_font_styles;
use super::spacing::apply_spacing;
use super::widgets::apply_widget_styles;

/// Apply the Longhorn theme to an egui context
pub fn apply_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    // Enable dark mode
    style.visuals.dark_mode = true;

    // Apply color scheme
    apply_colors(&mut style);

    // Apply spacing configuration
    apply_spacing(&mut style);

    // Apply widget styles
    apply_widget_styles(&mut style);

    // Apply font styles
    apply_font_styles(&mut style);

    // Disable shadows for cleaner look
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;

    ctx.set_style(style);
}

fn apply_colors(style: &mut egui::Style) {
    // Text colors
    style.visuals.override_text_color = Some(Colors::TEXT_PRIMARY);

    // Background colors
    style.visuals.panel_fill = Colors::BG_PANEL;
    style.visuals.window_fill = Colors::BG_WINDOW;
    style.visuals.extreme_bg_color = Colors::BG_EXTREME;

    // Selection colors
    style.visuals.selection.bg_fill = Colors::SELECTION_BG;
}
