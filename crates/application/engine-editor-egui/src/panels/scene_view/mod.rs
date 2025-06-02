// Scene view module - handles 3D scene rendering and interaction

pub mod rendering;
pub mod navigation;
pub mod gizmos;

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Mesh, MeshType, Light, Visibility};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::Camera;
use crate::types::{SceneNavigation, SceneTool, GizmoSystem, GizmoComponent, GizmoInteractionState, GizmoAxis};
use crate::editor_state::ConsoleMessage;
use std::collections::HashMap;

/// Scene view panel for 3D scene rendering and manipulation
pub struct SceneViewPanel {
    pub scene_view_active: bool,
    console_messages: Vec<ConsoleMessage>,
}

impl SceneViewPanel {
    pub fn new() -> Self {
        Self {
            scene_view_active: true,
            console_messages: Vec::new(),
        }
    }

    /// Main entry point for rendering the scene view
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        scene_navigation: &mut SceneNavigation,
        gizmo_system: &mut GizmoSystem,
        scene_tool: SceneTool,
        texture_assets: &HashMap<u64, crate::types::TextureAsset>,
        entity_count: &mut Option<usize>,
    ) {
        // For now, delegate to the main implementation
        // This will be fully moved here in the next step
        ui.label("Scene View - Implementation in progress");
    }
}