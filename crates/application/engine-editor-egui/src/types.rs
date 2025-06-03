// Common types and enums used throughout the editor

use eframe::egui;
use engine_components_3d::Transform;

/// Play state for editor mode management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    Editing,   // Normal editor mode - full editing capabilities
    Playing,   // Game running - properties locked, runtime active  
    Paused,    // Game paused - can inspect state, limited editing
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Editing
    }
}

/// Scene navigation state for Longhorn/Unreal style camera controls
#[derive(Debug, Clone)]
pub struct SceneNavigation {
    pub enabled: bool,
    pub is_navigating: bool,
    pub movement_speed: f32,
    pub rotation_sensitivity: f32,
    pub fast_movement_multiplier: f32,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub scene_camera_transform: Transform,
}

impl Default for SceneNavigation {
    fn default() -> Self {
        Self {
            enabled: true,
            is_navigating: false,
            movement_speed: 5.0,                    // Units per second
            rotation_sensitivity: 0.002,            // Radians per pixel - reduced for smoother control
            fast_movement_multiplier: 3.0,          // Shift speed boost
            last_mouse_pos: None,
            scene_camera_transform: Transform {
                position: [0.0, 2.0, 5.0],          // Default camera position
                rotation: [0.0, 0.0, 0.0],          // Looking forward
                scale: [1.0, 1.0, 1.0],
            },
        }
    }
}

/// Scene manipulation tool types
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

/// Gizmo axis selection for movement constraints
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoAxis {
    X,   // Red axis - Left/Right
    Y,   // Green axis - Up/Down  
    Z,   // Blue axis - Forward/Backward
}

/// Gizmo plane selection for planar movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoPlane {
    XY,  // Blue square - Z locked
    XZ,  // Green square - Y locked
    YZ,  // Red square - X locked
}

/// Gizmo component that can be interacted with
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoComponent {
    Axis(GizmoAxis),
    Plane(GizmoPlane),
    Center,  // Screen-space movement
}

/// Current gizmo interaction state
#[derive(Debug, Clone)]
pub enum GizmoInteractionState {
    Idle,                                    // No interaction
    Hovering(GizmoComponent),               // Mouse over component
    Dragging {
        component: GizmoComponent,
        start_mouse_pos: egui::Pos2,
        start_object_pos: [f32; 3],
    },
}

impl Default for GizmoInteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Texture asset for displaying in editor
#[derive(Clone)]
pub struct TextureAsset {
    pub id: egui::TextureId,
    pub name: String,
    pub size: egui::Vec2,
    pub path: String,
}

/// Project asset representation
#[derive(Clone)]
pub struct ProjectAsset {
    pub name: String,
    pub children: Option<Vec<ProjectAsset>>,
}

impl ProjectAsset {
    pub fn file(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: None,
        }
    }
    
    pub fn folder(name: &str, children: Vec<ProjectAsset>) -> Self {
        Self {
            name: name.to_string(),
            children: Some(children),
        }
    }
}

/// Different types of dockable panels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Hierarchy,
    Inspector,
    SceneView,
    GameView,
    Console,
    Project,
}

/// Hierarchy object representation
#[derive(Clone)]
pub struct HierarchyObject {
    pub name: String,
    pub object_type: ObjectType,
    pub children: Option<Vec<HierarchyObject>>,
}

impl HierarchyObject {
    pub fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            name: name.to_string(),
            object_type,
            children: None,
        }
    }
    
    pub fn parent(name: &str, children: Vec<HierarchyObject>) -> Self {
        Self {
            name: name.to_string(),
            object_type: ObjectType::GameObject,
            children: Some(children),
        }
    }
}

#[derive(Clone)]
pub enum ObjectType {
    GameObject,
    Camera,
    Light,
}

// Re-export GizmoSystem from the proper module
pub use crate::panels::scene_view::gizmos::GizmoSystem;