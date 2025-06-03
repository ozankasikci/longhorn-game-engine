// UI components module

pub mod toolbar;
pub mod menu_bar;
pub mod style;
pub mod tab_viewer;

// Re-export commonly used items
pub use toolbar::Toolbar;
pub use menu_bar::MenuBar;
pub use tab_viewer::EditorTabViewer;