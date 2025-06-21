//! 3D Gizmo input handling - converts 2D mouse input to 3D object transformations

use glam::{Vec3, Vec4, Mat4};
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use eframe::egui;

#[derive(Debug, Clone)]
pub struct Gizmo3DInput {
    // Current interaction state
    dragging: bool,
    active_axis: Option<Axis>,
    drag_start_mouse: Option<egui::Pos2>,
    drag_start_world_pos: Option<Vec3>,
    drag_plane_normal: Option<Vec3>,
    drag_plane_point: Option<Vec3>,
    
    // For hit testing
    last_gizmo_position: Option<Vec3>,
    axis_length: f32,
    hit_threshold: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Gizmo3DInput {
    pub fn new() -> Self {
        Self {
            dragging: false,
            active_axis: None,
            drag_start_mouse: None,
            drag_start_world_pos: None,
            drag_plane_normal: None,
            drag_plane_point: None,
            last_gizmo_position: None,
            axis_length: 1.0, // World units
            hit_threshold: 20.0, // Pixels
        }
    }
    
    /// Handle input for 3D gizmos
    pub fn handle_input(
        &mut self,
        world: &mut World,
        selected_entity: Option<Entity>,
        response: &egui::Response,
        rect: egui::Rect,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> bool {
        let Some(entity) = selected_entity else { 
            self.reset();
            return false;
        };
        
        let Some(transform) = world.get_component::<Transform>(entity).cloned() else {
            self.reset();
            return false;
        };
        
        let world_pos = Vec3::from_array(transform.position);
        self.last_gizmo_position = Some(world_pos);
        
        // Get mouse position
        let Some(mouse_pos) = response.hover_pos() else {
            return false;
        };
        
        let mut handled = false;
        
        // Only handle primary button (left click) for gizmo dragging
        if response.drag_started_by(egui::PointerButton::Primary) && !self.dragging {
            eprintln!("3D GIZMO INPUT: Drag started at mouse pos {:?}", mouse_pos);
            // Start drag - perform hit test
            if let Some(axis) = self.hit_test_gizmo(mouse_pos, world_pos, rect, view_matrix, projection_matrix) {
                eprintln!("3D GIZMO INPUT: Hit test successful - axis {:?}", axis);
                self.start_drag(axis, mouse_pos, world_pos, view_matrix);
                handled = true;
            } else {
                eprintln!("3D GIZMO INPUT: Hit test failed - no axis hit");
            }
        } else if response.dragged_by(egui::PointerButton::Primary) && self.dragging {
            // Continue drag
            if let Some(axis) = self.active_axis {
                self.update_drag(mouse_pos, world, entity, axis, rect, view_matrix, projection_matrix);
                handled = true;
            }
        } else if response.drag_stopped_by(egui::PointerButton::Primary) && self.dragging {
            // End drag
            eprintln!("3D GIZMO INPUT: Ending drag");
            self.end_drag();
            handled = true;
        }
        
        // If we're not dragging but another button is being used, don't interfere
        if !self.dragging && response.dragged_by(egui::PointerButton::Secondary) {
            handled = false;
        }
        
        handled
    }
    
    fn start_drag(&mut self, axis: Axis, mouse_pos: egui::Pos2, world_pos: Vec3, view_matrix: Mat4) {
        self.dragging = true;
        self.active_axis = Some(axis);
        self.drag_start_mouse = Some(mouse_pos);
        self.drag_start_world_pos = Some(world_pos);
        
        // Calculate drag plane
        let camera_pos = view_matrix.inverse().w_axis.truncate();
        let to_camera = (camera_pos - world_pos).normalize();
        
        let axis_direction = match axis {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
        };
        
        // Create a plane that contains the axis and faces the camera
        let plane_normal = if axis_direction.dot(to_camera).abs() > 0.95 {
            // Camera looking along axis - use fallback
            if axis == Axis::Y {
                Vec3::Z
            } else {
                Vec3::Y
            }
        } else {
            // Normal case
            let cross = axis_direction.cross(to_camera);
            if cross.length() < 0.001 {
                if axis == Axis::Y {
                    Vec3::Z
                } else {
                    Vec3::Y
                }
            } else {
                axis_direction.cross(cross).normalize()
            }
        };
        
        self.drag_plane_normal = Some(plane_normal);
        self.drag_plane_point = Some(world_pos);
    }
    
    fn update_drag(
        &mut self,
        mouse_pos: egui::Pos2,
        world: &mut World,
        entity: Entity,
        axis: Axis,
        rect: egui::Rect,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) {
        let Some(start_pos) = self.drag_start_world_pos else { return };
        let Some(plane_normal) = self.drag_plane_normal else { return };
        let Some(plane_point) = self.drag_plane_point else { return };
        
        // Convert mouse to ray
        let ray_origin = view_matrix.inverse().w_axis.truncate();
        let ray_direction = self.screen_to_world_ray(mouse_pos, rect, view_matrix, projection_matrix);
        
        // Intersect ray with drag plane
        if let Some(intersection) = self.ray_plane_intersection(ray_origin, ray_direction, plane_point, plane_normal) {
            let axis_direction = match axis {
                Axis::X => Vec3::X,
                Axis::Y => Vec3::Y,
                Axis::Z => Vec3::Z,
            };
            
            // Project movement onto axis
            let delta = intersection - start_pos;
            let movement = delta.dot(axis_direction) * axis_direction;
            let new_position = start_pos + movement;
            
            // Update transform
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                eprintln!("3D Gizmo: Moving from {:?} to {:?}", transform.position, new_position);
                transform.position = new_position.to_array();
            }
        }
    }
    
    fn end_drag(&mut self) {
        self.dragging = false;
        self.active_axis = None;
        self.drag_start_mouse = None;
        self.drag_start_world_pos = None;
        self.drag_plane_normal = None;
        self.drag_plane_point = None;
    }
    
    fn reset(&mut self) {
        self.dragging = false;
        self.active_axis = None;
        self.drag_start_mouse = None;
        self.drag_start_world_pos = None;
        self.drag_plane_normal = None;
        self.drag_plane_point = None;
    }
    
    fn hit_test_gizmo(
        &self,
        mouse_pos: egui::Pos2,
        gizmo_pos: Vec3,
        rect: egui::Rect,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> Option<Axis> {
        eprintln!("3D GIZMO HIT TEST: Testing at gizmo pos {:?}, mouse pos {:?}", gizmo_pos, mouse_pos);
        
        // Calculate screen positions of gizmo axes
        let center_screen = self.world_to_screen(gizmo_pos, rect, view_matrix, projection_matrix)?;
        eprintln!("3D GIZMO HIT TEST: Gizmo center on screen: {:?}", center_screen);
        
        // Calculate gizmo scale - must match gizmo_3d.wgsl
        // The shader now uses: base_scale = uniforms.viewport_size.z * 0.01
        // Where viewport_size.z is the gizmo_size (150.0)
        let gizmo_size = 150.0;
        let scale = gizmo_size * 0.01; // Fixed world-space scale matching shader
        eprintln!("3D GIZMO HIT TEST: Using fixed scale: {}", scale);
        
        // Test each axis
        let axes = [
            (Axis::X, Vec3::X),
            (Axis::Y, Vec3::Y),
            (Axis::Z, Vec3::Z),
        ];
        
        for (axis, direction) in axes {
            let end_world = gizmo_pos + direction * self.axis_length * scale;
            if let Some(end_screen) = self.world_to_screen(end_world, rect, view_matrix, projection_matrix) {
                let distance = self.point_to_line_distance(mouse_pos, center_screen, end_screen);
                eprintln!("3D GIZMO HIT TEST: Axis {:?} - end screen: {:?}, distance: {}", axis, end_screen, distance);
                if distance < self.hit_threshold {
                    eprintln!("3D GIZMO HIT TEST: HIT on axis {:?}!", axis);
                    return Some(axis);
                }
            }
        }
        
        eprintln!("3D GIZMO HIT TEST: No axis hit");
        None
    }
    
    fn world_to_screen(
        &self,
        world_pos: Vec3,
        rect: egui::Rect,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> Option<egui::Pos2> {
        let clip_pos = projection_matrix * view_matrix * world_pos.extend(1.0);
        
        if clip_pos.w <= 0.0 {
            return None;
        }
        
        let ndc = clip_pos.truncate() / clip_pos.w;
        
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
            return None;
        }
        
        let x = rect.left() + (ndc.x + 1.0) * 0.5 * rect.width();
        let y = rect.top() + (1.0 - ndc.y) * 0.5 * rect.height();
        
        Some(egui::pos2(x, y))
    }
    
    fn screen_to_world_ray(
        &self,
        screen_pos: egui::Pos2,
        rect: egui::Rect,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> Vec3 {
        // Convert screen to NDC
        let ndc_x = (screen_pos.x - rect.left()) / rect.width() * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y - rect.top()) / rect.height() * 2.0;
        
        // Unproject
        let clip_pos = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let eye_pos = projection_matrix.inverse() * clip_pos;
        let eye_dir = Vec4::new(eye_pos.x, eye_pos.y, -1.0, 0.0);
        let world_dir = view_matrix.inverse() * eye_dir;
        
        world_dir.truncate().normalize()
    }
    
    fn ray_plane_intersection(
        &self,
        ray_origin: Vec3,
        ray_dir: Vec3,
        plane_point: Vec3,
        plane_normal: Vec3,
    ) -> Option<Vec3> {
        let denominator = plane_normal.dot(ray_dir);
        
        if denominator.abs() < 0.0001 {
            return None;
        }
        
        let t = plane_normal.dot(plane_point - ray_origin) / denominator;
        
        if t < 0.0 {
            return None;
        }
        
        Some(ray_origin + ray_dir * t)
    }
    
    fn point_to_line_distance(&self, point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        
        let line_len_sq = line_vec.length_sq();
        if line_len_sq < 0.0001 {
            return point_vec.length();
        }
        
        let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
        let projection = line_start + t * line_vec;
        
        (point - projection).length()
    }
}