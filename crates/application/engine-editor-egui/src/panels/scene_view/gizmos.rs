// Gizmo system for object manipulation in the scene view

use eframe::egui;
use crate::types::{SceneTool, GizmoComponent, GizmoInteractionState, GizmoAxis, GizmoPlane};

/// Move gizmo for 3D object manipulation
#[derive(Debug, Clone)]
pub struct MoveGizmo {
    /// World position of the gizmo (object center)
    position: [f32; 3],
    /// Scale factor based on camera distance for consistent screen size
    scale: f32,
    /// Current interaction state
    interaction_state: GizmoInteractionState,
    /// Whether gizmo is visible and active
    enabled: bool,
}

impl MoveGizmo {
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            scale: 1.0,
            interaction_state: GizmoInteractionState::default(),
            enabled: true,
        }
    }
    
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }
    
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
    
    pub fn is_interacting(&self) -> bool {
        matches!(self.interaction_state, GizmoInteractionState::Dragging { .. })
    }
    
    pub fn get_hovered_component(&self) -> Option<GizmoComponent> {
        match &self.interaction_state {
            GizmoInteractionState::Hovering(component) => Some(*component),
            GizmoInteractionState::Dragging { component, .. } => Some(*component),
            _ => None,
        }
    }
    
    pub fn get_position(&self) -> [f32; 3] {
        self.position
    }
    
    pub fn get_scale(&self) -> f32 {
        self.scale
    }
    
    pub fn get_interaction_state(&self) -> &GizmoInteractionState {
        &self.interaction_state
    }
    
    pub fn set_interaction_state(&mut self, state: GizmoInteractionState) {
        self.interaction_state = state;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Gizmo system for managing scene manipulation tools
#[derive(Debug, Clone)]
pub struct GizmoSystem {
    /// Currently active scene tool
    active_tool: SceneTool,
    /// Move gizmo instance
    move_gizmo: Option<MoveGizmo>,
    /// Whether gizmos should be rendered
    enabled: bool,
    /// Grid snapping settings
    snap_enabled: bool,
    snap_increment: f32,
}

impl Default for GizmoSystem {
    fn default() -> Self {
        Self {
            active_tool: SceneTool::default(),
            move_gizmo: None,
            enabled: true,
            snap_enabled: false,
            snap_increment: 1.0,
        }
    }
}

impl GizmoSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_active_tool(&mut self, tool: SceneTool) {
        self.active_tool = tool;
    }
    
    pub fn get_active_tool(&self) -> SceneTool {
        self.active_tool
    }
    
    pub fn enable_move_gizmo(&mut self, position: [f32; 3]) {
        if self.active_tool == SceneTool::Move {
            self.move_gizmo = Some(MoveGizmo::new(position));
        }
    }
    
    pub fn disable_move_gizmo(&mut self) {
        self.move_gizmo = None;
    }
    
    pub fn get_move_gizmo_mut(&mut self) -> Option<&mut MoveGizmo> {
        self.move_gizmo.as_mut()
    }
    
    pub fn get_move_gizmo(&self) -> Option<&MoveGizmo> {
        self.move_gizmo.as_ref()
    }
    
    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
    }
    
    pub fn set_snap_increment(&mut self, increment: f32) {
        self.snap_increment = increment;
    }
    
    pub fn apply_snap(&self, value: f32) -> f32 {
        if self.snap_enabled {
            (value / self.snap_increment).round() * self.snap_increment
        } else {
            value
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_snap_enabled(&self) -> bool {
        self.snap_enabled
    }
    
    pub fn get_snap_increment(&self) -> f32 {
        self.snap_increment
    }
}

/// Ray for 3D intersection testing
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self { origin, direction }
    }
    
    /// Get point along ray at distance t
    pub fn at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
    }
}