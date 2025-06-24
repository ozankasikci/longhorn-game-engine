//! User interface system for the mobile game engine
//!
//! This crate provides immediate mode GUI capabilities using egui,
//! supporting both game UI and editor interfaces.

pub mod context;
pub mod events;
pub mod layout;
pub mod renderer;
pub mod theme;
pub mod widgets;

pub use context::UiContext;
pub use renderer::UiRenderer;
pub use theme::{Theme, ThemeManager};
pub use widgets::{Button, Image, Panel, Text, Widget};

/// UI system errors
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("Failed to initialize UI system")]
    Initialization,
    #[error("Widget creation failed: {0}")]
    WidgetError(String),
    #[error("UI rendering error: {0}")]
    RenderError(String),
    #[error("Theme loading error: {0}")]
    ThemeError(String),
}

/// UI system result type
pub type UiResult<T> = Result<T, UiError>;

#[cfg(test)]
mod tests {

    #[test]
    fn test_ui_context_creation() {
        // Placeholder test
        // Placeholder test
    }
}
