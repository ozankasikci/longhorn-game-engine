use glam::Vec2;
use longhorn_core::Transform;

/// Which gizmo tool is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    None,      // No gizmo shown
    Move,      // Translation arrows
    Rotate,    // Rotation circle (future)
    Scale,     // Scale handles (future)
}

/// Individual gizmo components that can be interacted with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandle {
    // Move gizmo
    MoveX,          // X-axis arrow (horizontal)
    MoveY,          // Y-axis arrow (vertical)
    MoveXY,         // Center square (free movement)

    // Rotate gizmo (future)
    RotateCircle,

    // Scale gizmo (future)
    ScaleX,
    ScaleY,
    ScaleXY,
}

/// Current gizmo interaction state
#[derive(Debug, Default)]
pub struct GizmoState {
    pub mode: GizmoMode,
    pub active_handle: Option<GizmoHandle>,
    pub hover_handle: Option<GizmoHandle>,
    pub drag_start_pos: Option<Vec2>,
    pub drag_start_transform: Option<Transform>,
}

impl GizmoState {
    pub fn new(mode: GizmoMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn begin_drag(&mut self, handle: GizmoHandle, mouse_pos: Vec2, transform: Transform) {
        self.active_handle = Some(handle);
        self.drag_start_pos = Some(mouse_pos);
        self.drag_start_transform = Some(transform);
    }

    pub fn end_drag(&mut self) {
        self.active_handle = None;
        self.drag_start_pos = None;
        self.drag_start_transform = None;
    }

    pub fn is_dragging(&self) -> bool {
        self.active_handle.is_some()
    }
}
