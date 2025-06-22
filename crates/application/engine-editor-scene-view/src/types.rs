//! Types shared between scene view and editor

use eframe::egui;
use engine_components_3d::Transform;

/// Scene navigation state for camera controls
#[derive(Debug, Clone)]
pub struct SceneNavigation {
    pub enabled: bool,
    pub is_navigating: bool,
    pub movement_speed: f32,
    pub rotation_sensitivity: f32,
    pub fast_movement_multiplier: f32,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub scene_camera_transform: Transform,
    pub rotation_velocity: [f32; 2],
    pub current_tool: SceneTool,
}

impl Default for SceneNavigation {
    fn default() -> Self {
        Self {
            enabled: true,
            is_navigating: false,
            movement_speed: 5.0,
            rotation_sensitivity: 0.005,
            fast_movement_multiplier: 3.0,
            last_mouse_pos: None,
            scene_camera_transform: Transform {
                position: [5.0, 5.0, 15.0],
                rotation: [-0.2, -0.3, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            rotation_velocity: [0.0, 0.0],
            current_tool: SceneTool::default(),
        }
    }
}

/// Current tool selected in scene view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneTool {
    Select,   // Q - Selection tool (default)
    Move,     // W - Move tool with XYZ gizmo
    Rotate,   // E - Rotation tool (future)
    Scale,    // R - Scale tool (future)
}

impl Default for SceneTool {
    fn default() -> Self {
        Self::Select
    }
}

/// Console message for logging
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub message: String,
    pub severity: MessageSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageSeverity {
    Info,
    Warning,
    Error,
}

/// Play state for editor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    Editing,
    Playing,
    Paused,
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Editing
    }
}

/// Simplified gizmo system interface
pub trait GizmoSystem: Send + Sync {
    fn get_active_tool(&self) -> SceneTool;
    fn set_active_tool(&mut self, tool: SceneTool);
}