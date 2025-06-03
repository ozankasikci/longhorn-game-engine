// Unity-style editor theme and styling

use eframe::egui;

/// Setup custom fonts for Unity-like appearance
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    
    // For now, use default fonts with adjusted sizes
    // TODO: Add custom Unity-like fonts later
    
    ctx.set_fonts(fonts);
}

/// Setup Unity-like visual style
pub fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Unity dark theme colors
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_gray(220));
    style.visuals.panel_fill = egui::Color32::from_rgb(56, 56, 56);
    style.visuals.window_fill = egui::Color32::from_rgb(56, 56, 56);
    
    // Set all widget visuals
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(65, 65, 65);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
    
    ctx.set_style(style);
}

/// Apply Unity-style theme
pub fn apply_unity_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Unity dark theme colors
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_gray(220));
    style.visuals.panel_fill = egui::Color32::from_rgb(56, 56, 56);
    style.visuals.window_fill = egui::Color32::from_rgb(56, 56, 56);
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    
    // Fix extreme dark/light colors (like pure black)
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(56, 56, 56);
    
    // Widget styling
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(65, 65, 65);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 48));
    
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 48));
    
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 90, 90));
    
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(55, 55, 55);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
    
    // Button styling
    style.visuals.button_frame = true;
    
    // Text selection
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(62, 107, 150);
    
    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.indent = 20.0;
    
    ctx.set_style(style);
}