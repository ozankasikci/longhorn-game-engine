// Scene view module - handles 3D scene rendering and interaction

pub mod rendering;
pub mod navigation;
pub mod gizmos;

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use engine_camera::Camera;
use crate::types::{SceneNavigation, SceneTool, GizmoSystem};

/// Scene view panel for 3D scene rendering and manipulation
pub struct SceneView;

impl SceneView {
    /// Main entry point for rendering the scene view
    pub fn show(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        scene_navigation: &mut SceneNavigation,
        gizmo_system: &mut GizmoSystem,
        scene_tool: SceneTool,
    ) {
        // The implementation will be moved here from main.rs
        ui.label("Scene View - To be implemented");
    }
}