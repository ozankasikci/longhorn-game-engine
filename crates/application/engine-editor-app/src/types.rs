// Common types and enums used throughout the editor

use eframe::egui;

// Re-export types from scene view crate
pub use engine_editor_scene_view::types::{SceneNavigation, PlayState, SceneTool};

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

// GizmoSystem moved to a simpler implementation

#[derive(Debug, Clone)]
pub struct GizmoSystem {
    active_tool: SceneTool,
}

impl GizmoSystem {
    pub fn new() -> Self {
        Self {
            active_tool: SceneTool::Select,
        }
    }
    
    pub fn get_active_tool(&self) -> SceneTool {
        self.active_tool
    }
    
    pub fn set_active_tool(&mut self, tool: SceneTool) {
        self.active_tool = tool;
    }
    
    pub fn enable_move_gizmo(&mut self) {
        self.active_tool = SceneTool::Move;
    }
    
    pub fn disable_move_gizmo(&mut self) {
        self.active_tool = SceneTool::Select;
    }
}

impl engine_editor_scene_view::types::GizmoSystem for GizmoSystem {
    fn get_active_tool(&self) -> SceneTool {
        self.active_tool
    }
    
    fn set_active_tool(&mut self, tool: SceneTool) {
        self.active_tool = tool;
    }
}