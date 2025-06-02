// Editor panels module

pub mod scene_view;
pub mod hierarchy;
pub mod inspector;
pub mod console;
pub mod project;
pub mod game_view;

// Re-export commonly used items
pub use scene_view::SceneViewPanel;
pub use hierarchy::HierarchyPanel;
pub use inspector::InspectorPanel;
pub use console::ConsolePanel;
pub use project::ProjectPanel;
pub use game_view::GameView;