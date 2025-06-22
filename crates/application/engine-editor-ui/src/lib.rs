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
pub mod toolbar;
pub mod menu_bar;
pub mod settings_dialog;
pub mod tab_viewer;

// Styling
pub mod styling;

// Re-export main components
pub use toolbar::Toolbar;
pub use menu_bar::MenuBar;
pub use settings_dialog::SettingsDialog;
pub use tab_viewer::{EditorTabViewer, EditorApp};

// Re-export types
pub use types::{PanelType, EditorSettings, ConsoleMessage, PlayState, SceneTool, GizmoSystem, SceneNavigation};

// Re-export styling utilities
pub use styling::{setup_custom_fonts, setup_custom_style, apply_longhorn_style};