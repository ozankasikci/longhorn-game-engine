// Styling module for Longhorn-style editor theme
//
// This module organizes all styling-related functionality into logical components:
// - colors: Color palette definitions
// - spacing: Layout and spacing constants
// - theme: Main theme application
// - widgets: Widget-specific styling
// - fonts: Font configuration

pub mod colors;
pub mod fonts;
pub mod spacing;
pub mod theme;
pub mod widgets;

// Re-export main styling functions for convenience
pub use fonts::setup_custom_fonts;
pub use theme::{apply_longhorn_style, setup_custom_style};
