// Unity-style transform gizmos implementation
// Based on research of Unity's gizmo system and best practices

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use glam::{Mat4, Vec3, Vec4};

pub struct UnityStyleGizmo {
    // Current interaction state
    pub active_axis: Option<Axis>,
    drag_start_mouse: Option<egui::Pos2>,
    drag_start_world_pos: Option<Vec3>,
    pub drag_plane_normal: Option<Vec3>,
    drag_plane_point: Option<Vec3>,
    
    // Gizmo settings
    gizmo_size: f32,  // Size in pixels (constant screen size)
    axis_length: f32, // Length of axis lines in world units
    
    // Cached axis endpoints for hit testing
    pub axis_endpoints: Option<AxisEndpoints>,
    
    // Track the currently selected entity to detect changes
    pub last_selected_entity: Option<Entity>,
}

#[cfg_attr(test, derive(Debug))]
pub struct AxisEndpoints {
    pub center: egui::Pos2,
    pub x_end: Option<egui::Pos2>,
    pub y_end: Option<egui::Pos2>,
    pub z_end: Option<egui::Pos2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl UnityStyleGizmo {
    pub fn new() -> Self {
        Self {
            active_axis: None,
            drag_start_mouse: None,
            drag_start_world_pos: None,
            drag_plane_normal: None,
            drag_plane_point: None,
            gizmo_size: 100.0,
            axis_length: 1.0,
            axis_endpoints: None,
            last_selected_entity: None,
        }
    }
    
    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        rect: egui::Rect,
        world: &mut World,
        selected_entity: Option<Entity>,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) {
        let Some(entity) = selected_entity else { 
            // Clear cached endpoints when no entity is selected
            self.axis_endpoints = None;
            self.last_selected_entity = None;
            return;
        };
        
        // Check if selected entity changed
        if self.last_selected_entity != Some(entity) {
            eprintln!("Unity Gizmo: Selected entity changed to {:?}", entity);
            self.axis_endpoints = None;  // Clear cache to force recalculation
            self.last_selected_entity = Some(entity);
        }
        
        let Some(transform) = world.get_component::<Transform>(entity).cloned() else { return };
        
        let world_pos = Vec3::from_array(transform.position);
        
        // Debug: Log initial position
        if self.axis_endpoints.is_none() {
            eprintln!("Unity Gizmo: Initial render for entity at position {:?}", world_pos);
        }
        
        // Project world position to screen
        let Some(screen_pos) = self.world_to_screen(world_pos, view_matrix, projection_matrix, rect) else {
            eprintln!("Unity Gizmo: Failed to project world position {:?} to screen", world_pos);
            return;
        };
        
        // Calculate screen-space gizmo size (constant size regardless of distance)
        let gizmo_scale = self.calculate_screen_space_scale(world_pos, view_matrix, projection_matrix);
        
        // IMPORTANT: Always draw the gizmo first to ensure axis_endpoints are updated
        // This must happen before input handling for proper hit testing
        self.draw_gizmo(ui, screen_pos, world_pos, view_matrix, projection_matrix, rect, gizmo_scale);
        
        // Handle input (after drawing so axis_endpoints are available)
        if let Some(mouse_pos) = response.hover_pos() {
            if response.drag_started() {
                // Hit test axes using the endpoints that were just calculated in draw_gizmo
                if let Some(axis) = self.hit_test_axes(mouse_pos, screen_pos, gizmo_scale) {
                    self.start_drag(axis, mouse_pos, world_pos, view_matrix);
                }
            } else if response.dragged() && self.active_axis.is_some() {
                // Continue drag
                self.update_drag(mouse_pos, world, entity, view_matrix, projection_matrix, rect);
            } else if response.drag_stopped() {
                // End drag
                self.end_drag();
            }
        }
    }
    
    pub fn start_drag(&mut self, axis: Axis, mouse_pos: egui::Pos2, world_pos: Vec3, view_matrix: Mat4) {
        eprintln!("Unity Gizmo: Starting drag on axis {:?}", axis);
        self.active_axis = Some(axis);
        self.drag_start_mouse = Some(mouse_pos);
        self.drag_start_world_pos = Some(world_pos);
        
        // Calculate drag plane based on camera view and selected axis
        let camera_forward = -view_matrix.z_axis.truncate().normalize();
        let camera_pos = view_matrix.inverse().w_axis.truncate();
        
        // Unity's approach: Create a plane that passes through the object and faces the camera
        // but is constrained to allow movement only along the selected axis
        
        let axis_direction = match axis {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
        };
        
        // The key insight: we need a plane that:
        // 1. Contains the axis we want to move along
        // 2. Faces towards the camera as much as possible
        
        // Calculate the vector from object to camera
        let to_camera = (camera_pos - world_pos).normalize();
        
        // The plane normal should be perpendicular to the axis
        // and as aligned with the camera direction as possible
        let plane_normal = if axis_direction.dot(to_camera).abs() > 0.95 {
            // Camera is looking along the axis - use a fallback plane
            // Choose the world axis that's most perpendicular to our movement axis
            if axis == Axis::Y {
                Vec3::Z
            } else {
                Vec3::Y
            }
        } else {
            // Normal case: plane normal is perpendicular to both axis and roughly facing camera
            let cross = axis_direction.cross(to_camera);
            if cross.length() < 0.001 {
                // Degenerate case, use fallback
                if axis == Axis::Y {
                    Vec3::Z
                } else {
                    Vec3::Y
                }
            } else {
                // Now we have a vector perpendicular to both the axis and camera direction
                // Cross it again with the axis to get a plane that contains the axis
                // and faces the camera as much as possible
                axis_direction.cross(cross).normalize()
            }
        };
        
        eprintln!("Unity Gizmo: Axis {:?}, camera_forward: {:?}, plane_normal: {:?}", 
                 axis, camera_forward, plane_normal);
        self.drag_plane_normal = Some(plane_normal);
        self.drag_plane_point = Some(world_pos);
    }
    
    pub fn update_drag(
        &mut self,
        mouse_pos: egui::Pos2,
        world: &mut World,
        entity: Entity,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        rect: egui::Rect,
    ) {
        let Some(axis) = self.active_axis else { return };
        let Some(start_pos) = self.drag_start_world_pos else { return };
        let Some(plane_normal) = self.drag_plane_normal else { return };
        let Some(plane_point) = self.drag_plane_point else { return };
        
        // Convert mouse position to ray
        let ray_origin = self.screen_to_world_ray_origin(mouse_pos, view_matrix, projection_matrix, rect);
        let ray_direction = self.screen_to_world_ray_direction(mouse_pos, view_matrix, projection_matrix, rect);
        
        // Intersect ray with drag plane
        if let Some(intersection) = self.ray_plane_intersection(ray_origin, ray_direction, plane_point, plane_normal) {
            // Project intersection onto axis
            let axis_direction = match axis {
                Axis::X => Vec3::X,
                Axis::Y => Vec3::Y,
                Axis::Z => Vec3::Z,
            };
            
            let delta = intersection - start_pos;
            let movement = delta.dot(axis_direction) * axis_direction;
            let new_position = start_pos + movement;
            
            eprintln!("Unity Gizmo: Ray intersection at {:?}, delta: {:?}, movement: {:?}", 
                     intersection, delta, movement);
            
            // Update transform
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                eprintln!("Unity Gizmo: Moving {:?} from {:?} to {:?}", axis, transform.position, new_position.to_array());
                transform.position = new_position.to_array();
            }
        }
    }
    
    fn end_drag(&mut self) {
        self.active_axis = None;
        self.drag_start_mouse = None;
        self.drag_start_world_pos = None;
        self.drag_plane_normal = None;
        self.drag_plane_point = None;
    }
    
    fn draw_gizmo(
        &mut self,
        ui: &mut egui::Ui,
        screen_pos: egui::Pos2,
        world_pos: Vec3,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        rect: egui::Rect,
        scale: f32,
    ) {
        let painter = ui.painter();
        
        // Calculate depth of each axis endpoint for proper draw order
        let camera_pos = view_matrix.inverse().w_axis.truncate();
        let view_dir = (world_pos - camera_pos).normalize();
        
        // Calculate screen-space directions for each axis
        let x_end_world = world_pos + Vec3::X * self.axis_length * scale;
        let y_end_world = world_pos + Vec3::Y * self.axis_length * scale;
        let z_end_world = world_pos + Vec3::Z * self.axis_length * scale;
        
        let x_end_screen = self.world_to_screen(x_end_world, view_matrix, projection_matrix, rect);
        let y_end_screen = self.world_to_screen(y_end_world, view_matrix, projection_matrix, rect);
        let z_end_screen = self.world_to_screen(z_end_world, view_matrix, projection_matrix, rect);
        
        // Store endpoints for hit testing
        self.axis_endpoints = Some(AxisEndpoints {
            center: screen_pos,
            x_end: x_end_screen,
            y_end: y_end_screen,
            z_end: z_end_screen,
        });
        
        // Draw function that adds outline for visibility
        let draw_axis_with_outline = |painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2, color: egui::Color32, width: f32| {
            // Draw semi-transparent white background for extra visibility
            painter.line_segment(
                [start, end],
                egui::Stroke::new(width + 4.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100))
            );
            // Draw black outline (thicker)
            painter.line_segment(
                [start, end],
                egui::Stroke::new(width + 2.0, egui::Color32::BLACK)
            );
            // Draw colored line on top
            painter.line_segment(
                [start, end],
                egui::Stroke::new(width, color)
            );
        };
        
        // X axis (red)
        if let Some(end) = x_end_screen {
            let color = if self.active_axis == Some(Axis::X) {
                egui::Color32::YELLOW
            } else {
                egui::Color32::from_rgb(255, 0, 0)
            };
            
            draw_axis_with_outline(&painter, screen_pos, end, color, 3.0);
            
            let dir = (end - screen_pos).normalized();
            self.draw_arrow_head_with_outline(painter, end, dir, color);
        }
        
        // Y axis (green)
        if let Some(end) = y_end_screen {
            let color = if self.active_axis == Some(Axis::Y) {
                egui::Color32::YELLOW
            } else {
                egui::Color32::from_rgb(0, 255, 0)
            };
            
            draw_axis_with_outline(&painter, screen_pos, end, color, 3.0);
            
            let dir = (end - screen_pos).normalized();
            self.draw_arrow_head_with_outline(painter, end, dir, color);
        }
        
        // Z axis (blue)
        if let Some(end) = z_end_screen {
            let color = if self.active_axis == Some(Axis::Z) {
                egui::Color32::YELLOW
            } else {
                egui::Color32::from_rgb(0, 0, 255)
            };
            
            draw_axis_with_outline(&painter, screen_pos, end, color, 3.0);
            
            let dir = (end - screen_pos).normalized();
            self.draw_arrow_head_with_outline(painter, end, dir, color);
        }
        
        // Center with enhanced visibility
        painter.circle_filled(screen_pos, 7.0, egui::Color32::BLACK);  // Black background
        painter.circle_filled(screen_pos, 5.0, egui::Color32::WHITE);  // White center
        painter.circle_stroke(screen_pos, 5.0, egui::Stroke::new(1.0, egui::Color32::from_gray(128))); // Gray border
    }
    
    fn draw_arrow_head(&self, painter: &egui::Painter, tip: egui::Pos2, direction: egui::Vec2, color: egui::Color32) {
        let size = 10.0;
        let angle = 150.0_f32.to_radians();
        
        let back = -direction * size;
        let right = egui::vec2(-back.y, back.x);
        
        let p1 = tip + back + right * angle.sin();
        let p2 = tip + back - right * angle.sin();
        
        painter.line_segment([tip, p1], egui::Stroke::new(2.0, color));
        painter.line_segment([tip, p2], egui::Stroke::new(2.0, color));
    }
    
    fn draw_arrow_head_with_outline(&self, painter: &egui::Painter, tip: egui::Pos2, direction: egui::Vec2, color: egui::Color32) {
        let size = 10.0;
        let angle = 150.0_f32.to_radians();
        
        let back = -direction * size;
        let right = egui::vec2(-back.y, back.x);
        
        let p1 = tip + back + right * angle.sin();
        let p2 = tip + back - right * angle.sin();
        
        // Draw semi-transparent white background for visibility
        painter.line_segment([tip, p1], egui::Stroke::new(6.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)));
        painter.line_segment([tip, p2], egui::Stroke::new(6.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)));
        
        // Draw black outline
        painter.line_segment([tip, p1], egui::Stroke::new(4.0, egui::Color32::BLACK));
        painter.line_segment([tip, p2], egui::Stroke::new(4.0, egui::Color32::BLACK));
        
        // Draw colored arrow on top
        painter.line_segment([tip, p1], egui::Stroke::new(2.0, color));
        painter.line_segment([tip, p2], egui::Stroke::new(2.0, color));
    }
    
    pub fn hit_test_axes(&self, mouse_pos: egui::Pos2, gizmo_center: egui::Pos2, scale: f32) -> Option<Axis> {
        let threshold = 10.0;
        
        // Use the actual projected axis endpoints
        if let Some(endpoints) = &self.axis_endpoints {
            // Test X axis
            if let Some(x_end) = endpoints.x_end {
                if self.point_to_line_distance(mouse_pos, endpoints.center, x_end) < threshold {
                    return Some(Axis::X);
                }
            }
            
            // Test Y axis
            if let Some(y_end) = endpoints.y_end {
                if self.point_to_line_distance(mouse_pos, endpoints.center, y_end) < threshold {
                    return Some(Axis::Y);
                }
            }
            
            // Test Z axis
            if let Some(z_end) = endpoints.z_end {
                if self.point_to_line_distance(mouse_pos, endpoints.center, z_end) < threshold {
                    return Some(Axis::Z);
                }
            }
        }
        
        None
    }
    
    pub fn calculate_screen_space_scale(&self, world_pos: Vec3, view_matrix: Mat4, projection_matrix: Mat4) -> f32 {
        // Calculate constant screen-space scale
        let camera_pos = view_matrix.inverse().w_axis.truncate();
        let distance = (world_pos - camera_pos).length();
        
        // This ensures gizmo appears same size on screen regardless of distance
        distance * 0.1
    }
    
    pub fn world_to_screen(&self, world_pos: Vec3, view_matrix: Mat4, proj_matrix: Mat4, viewport: egui::Rect) -> Option<egui::Pos2> {
        let world_pos4 = world_pos.extend(1.0);
        let view_pos = view_matrix * world_pos4;
        let clip_pos = proj_matrix * view_pos;
        
        // Debug first frame projection
        if self.axis_endpoints.is_none() {
            eprintln!("Unity Gizmo world_to_screen debug:");
            eprintln!("  World pos: {:?}", world_pos);
            eprintln!("  View pos: {:?}", view_pos);
            eprintln!("  Clip pos: {:?}", clip_pos);
            eprintln!("  Viewport: {:?}", viewport);
        }
        
        if clip_pos.w <= 0.0 {
            if self.axis_endpoints.is_none() {
                eprintln!("  Failed: Behind camera (w = {})", clip_pos.w);
            }
            return None;
        }
        
        let ndc = clip_pos.truncate() / clip_pos.w;
        
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
            if self.axis_endpoints.is_none() {
                eprintln!("  Failed: Outside NDC bounds: {:?}", ndc);
            }
            return None;
        }
        
        let x = viewport.left() + (ndc.x + 1.0) * 0.5 * viewport.width();
        let y = viewport.top() + (1.0 - ndc.y) * 0.5 * viewport.height();
        
        if self.axis_endpoints.is_none() {
            eprintln!("  Success: Screen pos ({}, {})", x, y);
        }
        
        Some(egui::pos2(x, y))
    }
    
    fn screen_to_world_ray_origin(&self, screen_pos: egui::Pos2, view_matrix: Mat4, proj_matrix: Mat4, viewport: egui::Rect) -> Vec3 {
        // For perspective projection, ray starts at camera position
        view_matrix.inverse().w_axis.truncate()
    }
    
    fn screen_to_world_ray_direction(&self, screen_pos: egui::Pos2, view_matrix: Mat4, proj_matrix: Mat4, viewport: egui::Rect) -> Vec3 {
        // Convert screen to NDC
        let ndc_x = (screen_pos.x - viewport.left()) / viewport.width() * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y - viewport.top()) / viewport.height() * 2.0;
        
        // Unproject
        let clip_pos = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let eye_pos = proj_matrix.inverse() * clip_pos;
        let eye_dir = Vec4::new(eye_pos.x, eye_pos.y, -1.0, 0.0);
        let world_dir = view_matrix.inverse() * eye_dir;
        
        world_dir.truncate().normalize()
    }
    
    pub fn ray_plane_intersection(&self, ray_origin: Vec3, ray_dir: Vec3, plane_point: Vec3, plane_normal: Vec3) -> Option<Vec3> {
        let denominator = plane_normal.dot(ray_dir);
        
        if denominator.abs() < 0.0001 {
            return None; // Ray parallel to plane
        }
        
        let t = plane_normal.dot(plane_point - ray_origin) / denominator;
        
        if t < 0.0 {
            return None; // Intersection behind ray origin
        }
        
        Some(ray_origin + ray_dir * t)
    }
    
    pub fn point_to_line_distance(&self, point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
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