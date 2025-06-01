// Unified Component System for Game Engine Editor
// Professional, cohesive UI components with consistent theming

pub mod theme;
pub mod colors;
pub mod typography;
pub mod spacing;
pub mod components;
pub mod widgets;
pub mod utils;
pub mod design_constraints;
pub mod design_loader;

// Re-export core types
pub use theme::{EditorTheme, ThemeManager, Themeable};
pub use colors::{Color, ColorPalette};
pub use typography::{Typography, FontSizes, FontWeights};
pub use spacing::{Spacing, Sizes, Layout};
pub use design_constraints::DesignConstraints;
pub use design_loader::{load_current_design, setup_custom_theme, preview_current_constraints, apply_constraints_to_window};

// Re-export main components
pub use components::{
    button::{EditorButton, ButtonVariant, ButtonSize},
    input::{EditorInput, InputVariant, InputState, InputSize},
    panel::{EditorPanel, PanelStyle},
    toolbar::{EditorToolbar, Tool, ToolGroup},
};

// Re-export widgets
pub use widgets::{
    vector_field::Vector3Field,
    enum_dropdown::EnumDropdown,
    asset_field::AssetField,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = EditorTheme::unity_dark();
        assert_eq!(theme.name, "Unity Dark");
    }
}