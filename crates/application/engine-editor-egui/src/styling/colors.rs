// Unity-style color palette definitions

use eframe::egui;

/// Unity-inspired color palette with multiple shades of gray
pub struct UnityColors;

impl UnityColors {
    // Text colors
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_gray(200);
    
    // Background colors (from darkest to lightest)
    pub const BG_EXTREME: egui::Color32 = egui::Color32::from_gray(30);    // Headers and extreme contrasts
    pub const BG_PANEL: egui::Color32 = egui::Color32::from_gray(40);      // Main panel background
    pub const BG_WINDOW: egui::Color32 = egui::Color32::from_gray(45);     // Window backgrounds
    pub const BG_WIDGET_INACTIVE: egui::Color32 = egui::Color32::from_gray(48);  // Inactive elements
    pub const BG_WIDGET_DEFAULT: egui::Color32 = egui::Color32::from_gray(50);   // Input fields, buttons
    pub const BG_WIDGET_HOVERED: egui::Color32 = egui::Color32::from_gray(60);   // Hover state
    
    // Border/stroke colors
    pub const STROKE_DARK: egui::Color32 = egui::Color32::from_gray(25);   // Darker lines, separators
    pub const STROKE_DEFAULT: egui::Color32 = egui::Color32::from_gray(35); // Default borders
    pub const STROKE_HOVERED: egui::Color32 = egui::Color32::from_gray(80); // Hovered borders
    pub const STROKE_ACTIVE: egui::Color32 = egui::Color32::from_gray(120); // Active/pressed borders
    
    // Selection colors - Unity-style blue
    pub const SELECTION_BG: egui::Color32 = egui::Color32::from_rgb(44, 93, 135);
    
    // Active state colors
    pub const ACTIVE_BG: egui::Color32 = egui::Color32::from_gray(45);
}

/// Create stroke with Unity colors
impl UnityColors {
    pub fn stroke_default() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DEFAULT)
    }
    
    pub fn stroke_dark() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_DARK)
    }
    
    pub fn stroke_hovered() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_HOVERED)
    }
    
    pub fn stroke_active() -> egui::Stroke {
        egui::Stroke::new(1.0, Self::STROKE_ACTIVE)
    }
}