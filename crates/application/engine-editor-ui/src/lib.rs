//! UI components and styling for the Longhorn Game Engine editor
//!
//! This crate contains reusable UI components:
//! - Toolbar
//! - Menu bar  
//! - Settings dialog
//! - Tab viewer
//! - Styling and theming

// Types
pub mod types;

// UI components
pub mod menu_bar;
pub mod settings_dialog;
pub mod tab_viewer;
pub mod toolbar;

// Styling
pub mod styling;

// Re-export main components
pub use menu_bar::MenuBar;
pub use settings_dialog::SettingsDialog;
pub use tab_viewer::{EditorApp, EditorTabViewer};
pub use toolbar::Toolbar;

// Re-export types
pub use types::{
    ConsoleMessage, EditorSettings, GizmoSystem, PanelType, PlayState, SceneNavigation, SceneTool,
};

// Re-export styling utilities
pub use styling::{apply_longhorn_style, setup_custom_fonts, setup_custom_style};
