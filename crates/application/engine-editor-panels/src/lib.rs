//! Editor panels for the Longhorn Game Engine
//!
//! This crate contains the standard editor panels:
//! - Inspector: Component property editing
//! - Hierarchy: Scene object tree view
//! - Console: Log output
//! - Project: Asset browser
//! - Game View: Runtime game preview

use eframe::egui;
use engine_ecs_core::{Entity, World};

/// Trait for all editor panels
pub trait Panel: Send + Sync {
    /// Get the panel's display name
    fn name(&self) -> &str;

    /// Show the panel UI
    fn show(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>);

    /// Called when panel is focused
    fn on_focus(&mut self) {}

    /// Called when panel loses focus
    fn on_blur(&mut self) {}
}

// Shared types
pub mod types;

// Panel modules
pub mod console;
pub mod drag_drop;
pub mod folder_manager;
pub mod game_view;
pub mod hierarchy;
pub mod inspector;
pub mod keyboard_shortcuts;
pub mod multi_selection;
pub mod project;
pub mod search;
pub mod undo_redo;

// Re-export panels
pub use console::ConsolePanel;
pub use game_view::GameViewPanel;
pub use hierarchy::HierarchyPanel;
pub use inspector::{InspectorPanel, ScriptLanguage, ScriptTemplate};
pub use project::ProjectPanel;

// Re-export commonly used types
pub use types::{
    ConsoleMessage, ConsoleMessageType, GizmoSystem, HierarchyObject, ObjectType, PlayState,
    ProjectAsset, SceneTool,
};
