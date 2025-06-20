// Dead simple gizmo implementation that actually works
use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use glam::{Mat4, Vec3, Vec4};

pub struct SimpleGizmo {
    dragging_axis: Option<GizmoAxis>,
    drag_start_pos: Option<egui::Pos2>,
    drag_start_transform: Option<[f32; 3]>,
    // Cache for debug visualization
    view_matrix_cache: Option<Mat4>,
    projection_matrix_cache: Option<Mat4>,
    rect_cache: egui::Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

impl SimpleGizmo {
    pub fn new() -> Self {
        Self {
            dragging_axis: None,
            drag_start_pos: None,
            drag_start_transform: None,
            view_matrix_cache: None,
            projection_matrix_cache: None,
            rect_cache: egui::Rect::NOTHING,
        }
    }

    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        rect: egui::Rect,
        world: &mut World,
        selected_entity: Option<Entity>,
        view_matrix: Option<Mat4>,
        projection_matrix: Option<Mat4>,
    ) -> bool {
        let Some(entity) = selected_entity else { 
            eprintln!("GIZMO: No entity selected");
            return false 
        };
        let Some(transform) = world.get_component::<Transform>(entity).cloned() else { 
            eprintln!("GIZMO: No transform on entity");
            return false 
        };

        // Project object's 3D position to screen coordinates using proper camera matrices
        let world_pos = Vec3::from_array(transform.position);
        
        // Cache matrices for debug visualization
        self.view_matrix_cache = view_matrix;
        self.projection_matrix_cache = projection_matrix;
        self.rect_cache = rect;
        
        // Project object position to screen coordinates
        let gizmo_center = if let (Some(view), Some(proj)) = (view_matrix, projection_matrix) {
            if let Some(screen_pos) = self.world_to_screen(world_pos, view, proj, rect) {
                screen_pos
            } else {
                // If projection fails, place at screen center as fallback
                rect.center()
            }
        } else {
            // No camera matrices - fallback to center
            rect.center()
        };
        let axis_length = 60.0;
        
        // Draw gizmos
        self.draw_gizmos(ui, gizmo_center, axis_length, world_pos);

        // Handle input
        if let Some(mouse_pos) = response.hover_pos() {
            if response.drag_started() {
                // Check which axis was clicked
                if let Some(axis) = self.hit_test(mouse_pos, gizmo_center, axis_length) {
                    eprintln!("GIZMO: Started dragging axis {:?}", axis);
                    self.dragging_axis = Some(axis);
                    self.drag_start_pos = Some(mouse_pos);
                    self.drag_start_transform = Some(transform.position);
                    return true;
                }
            } else if response.dragged() && self.dragging_axis.is_some() {
                // Handle dragging
                if let (Some(start_pos), Some(start_transform), Some(axis)) = (
                    self.drag_start_pos,
                    self.drag_start_transform,
                    self.dragging_axis
                ) {
                    let delta = mouse_pos - start_pos;
                    let movement_scale = 0.1; // Increased sensitivity
                    
                    let mut new_pos = start_transform;
                    match axis {
                        GizmoAxis::X => new_pos[0] = start_transform[0] + delta.x * movement_scale,
                        GizmoAxis::Y => new_pos[1] = start_transform[1] - delta.y * movement_scale, // Flip Y for screen space
                        GizmoAxis::Z => new_pos[2] = start_transform[2] - delta.y * movement_scale, // Z movement uses Y mouse delta
                    }
                    
                    eprintln!("GIZMO: Dragging {:?} - delta: {:?}, old pos: {:?}, new pos: {:?}", 
                        axis, delta, start_transform, new_pos);
                    
                    // Update transform directly
                    if let Some(transform_mut) = world.get_component_mut::<Transform>(entity) {
                        transform_mut.position = new_pos;
                        eprintln!("GIZMO: Updated transform position to {:?}", new_pos);
                    } else {
                        eprintln!("GIZMO: ERROR - Could not get mutable transform!");
                    }
                    return true;
                }
            } else if response.drag_stopped() {
                if self.dragging_axis.is_some() {
                    self.dragging_axis = None;
                    self.drag_start_pos = None;
                    self.drag_start_transform = None;
                    return true;
                }
            }
        }

        false
    }

    fn draw_gizmos(&self, ui: &mut egui::Ui, center: egui::Pos2, length: f32, _world_pos: Vec3) {
        let painter = ui.painter();
        
        // Draw a bright background circle to make gizmos more visible
        painter.circle_filled(center, 30.0, egui::Color32::from_rgba_unmultiplied(50, 50, 50, 200));
        
        // X axis (red) - horizontal line to the right
        let x_color = if self.dragging_axis == Some(GizmoAxis::X) {
            egui::Color32::YELLOW
        } else {
            egui::Color32::from_rgb(255, 100, 100) // Bright red
        };
        let x_end = center + egui::vec2(length, 0.0);
        painter.line_segment(
            [center, x_end],
            egui::Stroke::new(4.0, x_color)
        );
        
        // X axis arrow head
        let arrow_size = 8.0;
        painter.line_segment(
            [x_end, x_end + egui::vec2(-arrow_size, -arrow_size/2.0)],
            egui::Stroke::new(3.0, x_color)
        );
        painter.line_segment(
            [x_end, x_end + egui::vec2(-arrow_size, arrow_size/2.0)],
            egui::Stroke::new(3.0, x_color)
        );
        
        // Y axis (green) - vertical line upward
        let y_color = if self.dragging_axis == Some(GizmoAxis::Y) {
            egui::Color32::YELLOW
        } else {
            egui::Color32::from_rgb(100, 255, 100) // Bright green
        };
        let y_end = center + egui::vec2(0.0, -length);
        painter.line_segment(
            [center, y_end],
            egui::Stroke::new(4.0, y_color)
        );
        
        // Y axis arrow head
        painter.line_segment(
            [y_end, y_end + egui::vec2(-arrow_size/2.0, arrow_size)],
            egui::Stroke::new(3.0, y_color)
        );
        painter.line_segment(
            [y_end, y_end + egui::vec2(arrow_size/2.0, arrow_size)],
            egui::Stroke::new(3.0, y_color)
        );
        
        // Z axis (blue) - diagonal line (perspective)
        let z_color = if self.dragging_axis == Some(GizmoAxis::Z) {
            egui::Color32::YELLOW
        } else {
            egui::Color32::from_rgb(100, 100, 255) // Bright blue
        };
        let z_end = center + egui::vec2(-length * 0.7, length * 0.7);
        painter.line_segment(
            [center, z_end],
            egui::Stroke::new(4.0, z_color)
        );
        
        // Z axis arrow head
        let z_dir = (z_end - center).normalized();
        painter.line_segment(
            [z_end, z_end - z_dir * arrow_size + egui::vec2(-arrow_size/2.0, arrow_size/2.0)],
            egui::Stroke::new(3.0, z_color)
        );
        painter.line_segment(
            [z_end, z_end - z_dir * arrow_size + egui::vec2(arrow_size/2.0, -arrow_size/2.0)],
            egui::Stroke::new(3.0, z_color)
        );
        
        // Center dot
        painter.circle_filled(center, 6.0, egui::Color32::WHITE);
        painter.circle_stroke(center, 6.0, egui::Stroke::new(2.0, egui::Color32::BLACK));
    }

    fn hit_test(&self, mouse_pos: egui::Pos2, center: egui::Pos2, length: f32) -> Option<GizmoAxis> {
        let threshold = 15.0;
        
        // X axis
        let x_end = center + egui::vec2(length, 0.0);
        let x_dist = self.point_to_line_distance(mouse_pos, center, x_end);
        if x_dist < threshold {
            return Some(GizmoAxis::X);
        }
        
        // Y axis  
        let y_end = center + egui::vec2(0.0, -length);
        let y_dist = self.point_to_line_distance(mouse_pos, center, y_end);
        if y_dist < threshold {
            return Some(GizmoAxis::Y);
        }
        
        // Z axis
        let z_end = center + egui::vec2(-length * 0.7, length * 0.7);
        let z_dist = self.point_to_line_distance(mouse_pos, center, z_end);
        if z_dist < threshold {
            return Some(GizmoAxis::Z);
        }
        None
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

    /// Projects a 3D world position to 2D screen coordinates
    fn world_to_screen(
        &self,
        world_pos: Vec3,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        viewport_rect: egui::Rect,
    ) -> Option<egui::Pos2> {
        // Transform world position to clip space
        let world_pos4 = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        let clip_pos = projection_matrix * view_matrix * world_pos4;
        
        // Check if behind camera
        if clip_pos.w <= 0.0 {
            return None;
        }
        
        // Perspective divide to get NDC coordinates
        let ndc = Vec3::new(
            clip_pos.x / clip_pos.w,
            clip_pos.y / clip_pos.w,
            clip_pos.z / clip_pos.w,
        );
        
        // Check if within view frustum
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
            return None;
        }
        
        // Convert NDC to screen coordinates
        let screen_x = viewport_rect.left() + (ndc.x + 1.0) * 0.5 * viewport_rect.width();
        let screen_y = viewport_rect.top() + (1.0 - ndc.y) * 0.5 * viewport_rect.height();
        
        Some(egui::pos2(screen_x, screen_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use eframe::egui;
    
    #[test]
    fn test_world_to_screen_center() {
        // Create a camera looking at origin from positive Z
        let camera_pos = Vec3::new(0.0, 0.0, 5.0);
        let target = Vec3::ZERO;
        let view_matrix = Mat4::look_at_rh(camera_pos, target, Vec3::Y);
        
        let projection_matrix = Mat4::perspective_rh(
            60.0_f32.to_radians(), // fov
            800.0 / 600.0,         // aspect ratio
            0.1,                   // near
            100.0,                 // far
        );
        
        let viewport = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0), 
            egui::vec2(800.0, 600.0)
        );
        
        // Create a simple gizmo to test the projection
        let gizmo = SimpleGizmo::new();
        
        // Object at origin should appear at screen center
        let world_pos = Vec3::ZERO;
        let screen_pos = gizmo.world_to_screen(world_pos, view_matrix, projection_matrix, viewport);
        
        println!("World pos: {:?}", world_pos);
        println!("Screen pos: {:?}", screen_pos);
        
        assert!(screen_pos.is_some(), "Origin should be visible");
        let pos = screen_pos.unwrap();
        
        // Should be close to screen center (400, 300)
        let center_x = 400.0;
        let center_y = 300.0;
        let tolerance = 5.0;
        
        assert!(
            (pos.x - center_x).abs() < tolerance,
            "X position {} should be close to center {}", pos.x, center_x
        );
        assert!(
            (pos.y - center_y).abs() < tolerance,
            "Y position {} should be close to center {}", pos.y, center_y
        );
    }
    
    #[test]
    fn test_world_to_screen_offset() {
        // Camera at origin looking in +Z direction (engine convention)
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let forward = Vec3::new(0.0, 0.0, 1.0); // +Z is forward in this engine
        let target = camera_pos + forward;
        let view_matrix = Mat4::look_at_rh(camera_pos, target, Vec3::Y);
        
        let projection_matrix = Mat4::perspective_rh(
            60.0_f32.to_radians(),
            800.0 / 600.0,
            0.1,
            100.0,
        );
        
        let viewport = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0), 
            egui::vec2(800.0, 600.0)
        );
        
        let gizmo = SimpleGizmo::new();
        
        // Test object at (1, 0, 5) - should be to the right of center
        let world_pos = Vec3::new(1.0, 0.0, 5.0);
        let screen_pos = gizmo.world_to_screen(world_pos, view_matrix, projection_matrix, viewport);
        
        println!("World pos: {:?}", world_pos);
        println!("Screen pos: {:?}", screen_pos);
        
        assert!(screen_pos.is_some(), "Object should be visible");
        let pos = screen_pos.unwrap();
        
        // Should be to the right of center (x > 400)
        assert!(pos.x > 400.0, "Object to the right should appear right of center, got x={}", pos.x);
        
        // Test object at (-1, 0, 5) - should be to the left of center
        let world_pos = Vec3::new(-1.0, 0.0, 5.0);
        let screen_pos = gizmo.world_to_screen(world_pos, view_matrix, projection_matrix, viewport);
        
        assert!(screen_pos.is_some(), "Object should be visible");
        let pos = screen_pos.unwrap();
        
        // Should be to the left of center (x < 400)
        assert!(pos.x < 400.0, "Object to the left should appear left of center, got x={}", pos.x);
    }
    
    #[test]
    fn test_engine_coordinate_system() {
        // Test the engine's coordinate system where +Z is forward
        let camera_pos = Vec3::new(0.0, 1.0, -5.0); // Camera behind and above origin
        let forward = Vec3::new(0.0, 0.0, 1.0); // +Z is forward
        let target = camera_pos + forward;
        let view_matrix = Mat4::look_at_rh(camera_pos, target, Vec3::Y);
        
        let projection_matrix = Mat4::perspective_rh(60.0_f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);
        let viewport = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        
        let gizmo = SimpleGizmo::new();
        
        // Object at origin should be visible
        let world_pos = Vec3::ZERO;
        let screen_pos = gizmo.world_to_screen(world_pos, view_matrix, projection_matrix, viewport);
        
        println!("Engine coords - Camera pos: {:?}", camera_pos);
        println!("Engine coords - Target: {:?}", target);
        println!("Engine coords - World pos: {:?}", world_pos);
        println!("Engine coords - Screen pos: {:?}", screen_pos);
        
        assert!(screen_pos.is_some(), "Origin should be visible from camera");
        
        // The origin should appear below center since camera is above it
        let pos = screen_pos.unwrap();
        assert!(pos.y > 300.0, "Origin should appear below center when camera is above, got y={}", pos.y);
    }
}