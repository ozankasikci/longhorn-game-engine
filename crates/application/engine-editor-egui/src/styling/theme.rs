// Main theme application and configuration for Unity-style editor

use eframe::egui;
use super::colors::UnityColors;
use super::spacing::apply_unity_spacing;
use super::widgets::apply_unity_widget_styles;
use super::fonts::apply_unity_font_styles;

/// Setup Unity-like visual style (convenience function that calls apply_unity_style)
pub fn setup_custom_style(ctx: &egui::Context) {
    apply_unity_style(ctx);
}

/// Apply comprehensive Unity-style theme to the egui context
pub fn apply_unity_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Enable dark mode
    style.visuals.dark_mode = true;
    
    // Apply color scheme
    apply_unity_colors(&mut style);
    
    // Apply spacing configuration
    apply_unity_spacing(&mut style);
    
    // Apply widget styles
    apply_unity_widget_styles(&mut style);
    
    // Apply font styles
    apply_unity_font_styles(&mut style);
    
    // Disable shadows for cleaner look
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    
    ctx.set_style(style);
}

/// Apply Unity color scheme to the style
fn apply_unity_colors(style: &mut egui::Style) {
    // Text colors
    style.visuals.override_text_color = Some(UnityColors::TEXT_PRIMARY);
    
    // Background colors
    style.visuals.panel_fill = UnityColors::BG_PANEL;
    style.visuals.window_fill = UnityColors::BG_WINDOW;
    style.visuals.extreme_bg_color = UnityColors::BG_EXTREME;
    
    // Selection colors
    style.visuals.selection.bg_fill = UnityColors::SELECTION_BG;
}