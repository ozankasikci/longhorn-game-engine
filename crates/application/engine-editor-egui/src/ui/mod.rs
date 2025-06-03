// UI components module

pub mod toolbar;
pub mod menu_bar;
pub mod tab_viewer;
pub mod settings_dialog;

// Re-export commonly used items
pub use toolbar::Toolbar;
pub use menu_bar::MenuBar;
pub use tab_viewer::EditorTabViewer;
pub use settings_dialog::SettingsDialog;