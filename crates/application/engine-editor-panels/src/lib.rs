//! Editor panels for the Longhorn Game Engine
//! 
//! This crate contains the standard editor panels:
//! - Inspector: Component property editing
//! - Hierarchy: Scene object tree view
//! - Console: Log output
//! - Project: Asset browser
//! - Game View: Runtime game preview

use eframe::egui;
use engine_ecs_core::{World, Entity};

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
pub mod inspector;
pub mod hierarchy;
pub mod console;
pub mod project;
pub mod game_view;

// Re-export panels
pub use inspector::InspectorPanel;
pub use hierarchy::HierarchyPanel;
pub use console::ConsolePanel;
pub use project::ProjectPanel;
pub use game_view::GameViewPanel;

// Re-export commonly used types
pub use types::{ConsoleMessage, ConsoleMessageType, PlayState, SceneTool, ProjectAsset, HierarchyObject, ObjectType, GizmoSystem};