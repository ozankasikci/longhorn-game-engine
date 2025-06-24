//! Scene view panel for the Longhorn Game Engine editor
//!
//! This crate was extracted from engine-editor-egui to reduce its size by ~2,319 lines (31.6%).
//! It provides the 3D viewport functionality for the editor.

pub mod types;

// Re-export main types
pub use types::{ConsoleMessage, GizmoSystem, PlayState, SceneNavigation, SceneTool};

use eframe::egui;
use engine_ecs_core::{Entity, World};

/// Main trait for scene view panels
pub trait SceneViewPanel: Send + Sync {
    /// Show the scene view panel
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: Option<Entity>,
        scene_navigation: &mut SceneNavigation,
        gizmo_system: &mut dyn GizmoSystem,
        play_state: PlayState,
    ) -> Vec<ConsoleMessage>;

    /// Check if scene view is active
    fn is_active(&self) -> bool;

    /// Set scene view active state
    fn set_active(&mut self, active: bool);
}

// Scene view modules
pub mod scene_view;

// Re-export the main implementation
pub use scene_view::SceneViewPanel as SceneView;
