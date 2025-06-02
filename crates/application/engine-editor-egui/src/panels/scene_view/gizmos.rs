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
    
    /// Test if mouse position hits any gizmo component
    pub fn test_gizmo_hit(&self, mouse_pos: egui::Pos2, gizmo_center: egui::Pos2) -> Option<GizmoComponent> {
        if !self.move_gizmo.as_ref().map_or(false, |g| g.enabled) {
            return None;
        }
        
        let hit_radius = 15.0; // Increased hit radius for easier selection
        let axis_length = 80.0;
        let plane_size = 20.0;
        
        // Test center sphere first (screen-space movement)
        let center_dist = mouse_pos.distance(gizmo_center);
        if center_dist < hit_radius {
            return Some(GizmoComponent::Center);
        }
        
        // Test axis lines
        let x_end = gizmo_center + egui::vec2(axis_length, 0.0);
        let y_end = gizmo_center + egui::vec2(0.0, -axis_length);
        let z_end = gizmo_center + egui::vec2(axis_length * 0.5, -axis_length * 0.5);
        
        // Use line distance calculation
        if self.point_to_line_distance(mouse_pos, gizmo_center, x_end) < hit_radius {
            return Some(GizmoComponent::Axis(GizmoAxis::X));
        }
        if self.point_to_line_distance(mouse_pos, gizmo_center, y_end) < hit_radius {
            return Some(GizmoComponent::Axis(GizmoAxis::Y));
        }
        if self.point_to_line_distance(mouse_pos, gizmo_center, z_end) < hit_radius {
            return Some(GizmoComponent::Axis(GizmoAxis::Z));
        }
        
        // Test plane handles (positioned at 1/3 of axis length)
        let plane_offset = axis_length / 3.0;
        let xy_plane_pos = gizmo_center + egui::vec2(plane_offset, -plane_offset);
        let xz_plane_pos = gizmo_center + egui::vec2(plane_offset * 1.5, -plane_offset * 0.5);
        let yz_plane_pos = gizmo_center + egui::vec2(plane_offset * 0.5, -plane_offset * 1.5);
        
        if mouse_pos.distance(xy_plane_pos) < plane_size {
            return Some(GizmoComponent::Plane(GizmoPlane::XY));
        }
        if mouse_pos.distance(xz_plane_pos) < plane_size {
            return Some(GizmoComponent::Plane(GizmoPlane::XZ));
        }
        if mouse_pos.distance(yz_plane_pos) < plane_size {
            return Some(GizmoComponent::Plane(GizmoPlane::YZ));
        }
        
        None
    }
    
    /// Calculate distance from point to line segment
    fn point_to_line_distance(&self, point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        let line_len_sq = line_vec.length_sq();
        
        if line_len_sq < 0.0001 {
            return point.distance(line_start);
        }
        
        let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
        let projection = line_start + line_vec * t;
        point.distance(projection)
    }
    
    /// Calculate new position based on gizmo interaction
    pub fn calculate_new_position(&self, start_pos: [f32; 3], mouse_delta: egui::Vec2, component: GizmoComponent, scale: f32) -> [f32; 3] {
        let mut new_pos = start_pos;
        let movement_scale = 1.0 / scale; // Inverse scale for world space movement
        
        match component {
            GizmoComponent::Axis(axis) => {
                match axis {
                    GizmoAxis::X => new_pos[0] = start_pos[0] + mouse_delta.x * movement_scale,
                    GizmoAxis::Y => new_pos[1] = start_pos[1] - mouse_delta.y * movement_scale,
                    GizmoAxis::Z => new_pos[2] = start_pos[2] - mouse_delta.y * movement_scale,
                }
            }
            GizmoComponent::Plane(plane) => {
                match plane {
                    GizmoPlane::XY => {
                        new_pos[0] = start_pos[0] + mouse_delta.x * movement_scale;
                        new_pos[1] = start_pos[1] - mouse_delta.y * movement_scale;
                    }
                    GizmoPlane::XZ => {
                        new_pos[0] = start_pos[0] + mouse_delta.x * movement_scale;
                        new_pos[2] = start_pos[2] - mouse_delta.y * movement_scale;
                    }
                    GizmoPlane::YZ => {
                        new_pos[1] = start_pos[1] - mouse_delta.y * movement_scale;
                        new_pos[2] = start_pos[2] - mouse_delta.x * movement_scale * 0.5;
                    }
                }
            }
            GizmoComponent::Center => {
                // Screen-space movement
                new_pos[0] = start_pos[0] + mouse_delta.x * movement_scale;
                new_pos[2] = start_pos[2] - mouse_delta.y * movement_scale;
            }
        }
        
        // Apply snapping if enabled
        if self.snap_enabled {
            new_pos[0] = (new_pos[0] / self.snap_increment).round() * self.snap_increment;
            new_pos[1] = (new_pos[1] / self.snap_increment).round() * self.snap_increment;
            new_pos[2] = (new_pos[2] / self.snap_increment).round() * self.snap_increment;
        }
        
        new_pos
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