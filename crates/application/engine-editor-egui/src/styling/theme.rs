// Main theme application and configuration for Longhorn editor

use eframe::egui;
use super::colors::LonghornColors;
use super::spacing::apply_longhorn_spacing;
use super::widgets::apply_longhorn_widget_styles;
use super::fonts::apply_longhorn_font_styles;

/// Setup Longhorn visual style (convenience function that calls apply_longhorn_style)
pub fn setup_custom_style(ctx: &egui::Context) {
    apply_longhorn_style(ctx);
}

/// Apply comprehensive Longhorn theme to the egui context
pub fn apply_longhorn_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Enable dark mode
    style.visuals.dark_mode = true;
    
    // Apply color scheme
    apply_longhorn_colors(&mut style);
    
    // Apply spacing configuration
    apply_longhorn_spacing(&mut style);
    
    // Apply widget styles
    apply_longhorn_widget_styles(&mut style);
    
    // Apply font styles
    apply_longhorn_font_styles(&mut style);
    
    // Disable shadows for cleaner look
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    
    ctx.set_style(style);
}

/// Apply Longhorn color scheme to the style
fn apply_longhorn_colors(style: &mut egui::Style) {
    // Text colors
    style.visuals.override_text_color = Some(LonghornColors::TEXT_PRIMARY);
    
    // Background colors
    style.visuals.panel_fill = LonghornColors::BG_PANEL;
    style.visuals.window_fill = LonghornColors::BG_WINDOW;
    style.visuals.extreme_bg_color = LonghornColors::BG_EXTREME;
    
    // Selection colors
    style.visuals.selection.bg_fill = LonghornColors::SELECTION_BG;
}