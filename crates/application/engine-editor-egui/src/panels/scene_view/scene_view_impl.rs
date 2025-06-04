// Scene view implementation - full scene rendering logic

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material, Light, Visibility, MeshFilter, MeshRenderer};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::Camera;
use crate::types::{SceneNavigation, GizmoSystem, PlayState};
use crate::editor_state::ConsoleMessage;
use super::object_renderer;

pub struct SceneViewRenderer {
    last_rendered_entity_count: usize,
}

impl SceneViewRenderer {
    pub fn new() -> Self {
        Self {
            last_rendered_entity_count: 0,
        }
    }
    
    /// Main scene rendering function - Phase 10.1: WGPU Integration
    pub fn draw_scene(
        &mut self,
        world: &World,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        scene_navigation: &mut SceneNavigation,
        selected_entity: Option<Entity>,
        play_state: PlayState,
    ) {
        let painter = ui.painter();
        
        
        // Get camera position and rotation for view transformation
        let camera_pos = scene_navigation.scene_camera_transform.position;
        let camera_rot = scene_navigation.scene_camera_transform.rotation;
        
        
        // Draw scene objects using FAKE 2D PROJECTION (Temporary Phase 10 approach)
        let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        
        // DEBUG: Direct entity count check
        let total_entity_count = world.entity_count();
        
        // SUPER DEBUG: Check specific component counts
        let transform_count = world.query_legacy::<Transform>().count();
        let mesh_count_direct = world.query_legacy::<MeshFilter>().count();
        let material_count = world.query_legacy::<Material>().count();
        
        
        // Track entity changes
        let current_entity_count = entities_with_transforms.len();
        if current_entity_count != self.last_rendered_entity_count {
            self.last_rendered_entity_count = current_entity_count;
        }
        
        
        // COMMENTED OUT: Force debug cube
        /*
        let debug_screen_pos = rect.center() + egui::Vec2::new(100.0, 0.0);
        self.render_enhanced_cube(
            painter, 
            debug_screen_pos, 
            80.0, 
            [0.0, 0.0, 0.0], 
            egui::Color32::from_rgb(0, 255, 0), 
            "FORCED DEBUG CUBE"
        );
        */
        
        // Count and render mesh entities
        let mut mesh_count = 0;
        let mut debug_entity_info = Vec::new();
        
        for (idx, entity) in entities_with_transforms.iter().enumerate() {
            // Debug: Show first 3 entities on screen
            if idx < 3 {
                if let Some(transform) = world.get_component::<Transform>(*entity) {
                    let name = world.get_component::<engine_components_ui::Name>(*entity)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| format!("Entity {}", entity.id()));
                    let has_mesh = world.get_component::<MeshFilter>(*entity).is_some();
                    debug_entity_info.push(format!("{}: Mesh={}, Pos=[{:.1},{:.1},{:.1}]", 
                        name, has_mesh, transform.position[0], transform.position[1], transform.position[2]));
                }
            }
            
            // Check for new mesh components (MeshFilter + MeshRenderer)
            if let Some(mesh_filter) = world.get_component::<MeshFilter>(*entity) {
                if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(*entity) {
                    if mesh_renderer.enabled {
                        mesh_count += 1;
                        // Phase 10.1: Render with enhanced visibility
                        self.render_mesh_entity_enhanced(
                            world,
                            painter,
                            rect,
                            *entity,
                            camera_pos,
                            camera_rot,
                            selected_entity,
                        );
                    }
                }
            } else if world.get_component::<Mesh>(*entity).is_some() {
                // Fallback for old Mesh component (will be removed)
                mesh_count += 1;
                self.render_mesh_entity_enhanced(
                    world,
                    painter,
                    rect,
                    *entity,
                    camera_pos,
                    camera_rot,
                    selected_entity,
                );
            } else {
                // Non-mesh entities (cameras, lights, etc)
                self.render_entity(
                    world,
                    painter,
                    rect,
                    *entity,
                    camera_pos,
                    camera_rot,
                    selected_entity,
                );
            }
        }
        
        
        
        // Render sprites separately (they need different handling)
        self.render_sprites(world, painter, rect, camera_pos, camera_rot, selected_entity);
        
        // Draw grid background (after objects for depth)
        self.draw_grid(painter, rect, camera_pos, camera_rot);
        
        // Draw scene camera indicator
        self.draw_scene_camera_indicator(painter, rect, camera_pos);
        
        // Draw scene overlay info
        self.draw_scene_overlay(ui, rect, world, selected_entity, play_state, scene_navigation);
        // Scene drawn
    }
    
    /// Phase 10.1: Enhanced mesh entity rendering with guaranteed visibility
    /// Render a proper 3D cube with all vertices transformed
    fn render_3d_cube(
        &self,
        painter: &egui::Painter,
        world_pos: [f32; 3],
        size: f32,
        rotation: [f32; 3],
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        rect: egui::Rect,
        color: egui::Color32,
        is_selected: bool,
    ) {
        let half_size = size * 0.5;
        
        // Define cube vertices in local space
        let vertices = [
            // Front face
            [-half_size, -half_size, -half_size], // 0: front bottom left
            [ half_size, -half_size, -half_size], // 1: front bottom right
            [ half_size,  half_size, -half_size], // 2: front top right
            [-half_size,  half_size, -half_size], // 3: front top left
            // Back face
            [-half_size, -half_size,  half_size], // 4: back bottom left
            [ half_size, -half_size,  half_size], // 5: back bottom right
            [ half_size,  half_size,  half_size], // 6: back top right
            [-half_size,  half_size,  half_size], // 7: back top left
        ];
        
        // Apply object rotation to vertices
        let cos_x = rotation[0].cos();
        let sin_x = rotation[0].sin();
        let cos_y = rotation[1].cos();
        let sin_y = rotation[1].sin();
        let cos_z = rotation[2].cos();
        let sin_z = rotation[2].sin();
        
        let mut transformed_vertices = Vec::new();
        let mut screen_vertices = Vec::new();
        let mut depths = Vec::new();
        
        for vertex in &vertices {
            // Apply rotation around Z axis
            let x1 = vertex[0] * cos_z - vertex[1] * sin_z;
            let y1 = vertex[0] * sin_z + vertex[1] * cos_z;
            let z1 = vertex[2];
            
            // Apply rotation around Y axis
            let x2 = x1 * cos_y + z1 * sin_y;
            let y2 = y1;
            let z2 = -x1 * sin_y + z1 * cos_y;
            
            // Apply rotation around X axis
            let x3 = x2;
            let y3 = y2 * cos_x - z2 * sin_x;
            let z3 = y2 * sin_x + z2 * cos_x;
            
            // Translate to world position
            let world_vertex = [
                x3 + world_pos[0],
                y3 + world_pos[1],
                z3 + world_pos[2],
            ];
            
            transformed_vertices.push(world_vertex);
            
            // Project to screen
            let (screen_pos, depth) = self.world_to_screen_enhanced(
                world_vertex,
                camera_pos,
                camera_rot,
                rect.center(),
                rect,
            );
            
            screen_vertices.push(screen_pos);
            depths.push(depth);
        }
        
        // Define cube faces (indices into vertices array)
        let faces = [
            [0, 1, 2, 3], // Front
            [5, 4, 7, 6], // Back
            [4, 0, 3, 7], // Left
            [1, 5, 6, 2], // Right
            [3, 2, 6, 7], // Top
            [4, 5, 1, 0], // Bottom
        ];
        
        // Calculate face normals and sort by depth
        let mut face_depths = Vec::new();
        for (i, face) in faces.iter().enumerate() {
            // Calculate face center depth
            let center_depth = (depths[face[0]] + depths[face[1]] + depths[face[2]] + depths[face[3]]) / 4.0;
            
            // Calculate face normal in camera space to determine if it's facing camera
            let v0 = &transformed_vertices[face[0]];
            let v1 = &transformed_vertices[face[1]];
            let v2 = &transformed_vertices[face[2]];
            
            // Two edge vectors
            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            
            // Cross product for normal
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];
            
            // View vector from face center to camera
            let face_center = [
                (v0[0] + v1[0] + v2[0] + transformed_vertices[face[3]][0]) / 4.0,
                (v0[1] + v1[1] + v2[1] + transformed_vertices[face[3]][1]) / 4.0,
                (v0[2] + v1[2] + v2[2] + transformed_vertices[face[3]][2]) / 4.0,
            ];
            
            let view_vec = [
                camera_pos[0] - face_center[0],
                camera_pos[1] - face_center[1],
                camera_pos[2] - face_center[2],
            ];
            
            // Dot product to check if face is facing camera
            let dot = normal[0] * view_vec[0] + normal[1] * view_vec[1] + normal[2] * view_vec[2];
            
            if dot > 0.0 {
                face_depths.push((i, center_depth));
            }
        }
        
        // Sort faces by depth (far to near)
        face_depths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Draw faces
        for (face_idx, _) in face_depths {
            let face = &faces[face_idx];
            let face_vertices: Vec<egui::Pos2> = face.iter()
                .map(|&i| screen_vertices[i])
                .collect();
            
            // Determine face color based on which face it is
            let face_color = match face_idx {
                0 | 1 => color, // Front/back
                2 | 3 => color.gamma_multiply(0.8), // Left/right
                4 => color.gamma_multiply(0.9), // Top (slightly lighter)
                5 => color.gamma_multiply(0.6), // Bottom (darker)
                _ => color,
            };
            
            let final_color = if is_selected {
                egui::Color32::YELLOW.gamma_multiply(0.8)
            } else {
                face_color
            };
            
            painter.add(egui::Shape::convex_polygon(
                face_vertices,
                final_color,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            ));
        }
    }

    fn render_mesh_entity_enhanced(
        &self,
        world: &World,
        painter: &egui::Painter,
        rect: egui::Rect,
        entity: Entity,
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        selected_entity: Option<Entity>,
    ) {
        let Some(transform) = world.get_component::<Transform>(entity) else { return; };
        
        // Determine mesh type - check new components first, then fall back to old
        let mesh_type = if let Some(_mesh_filter) = world.get_component::<MeshFilter>(entity) {
            // TODO: In the future, get mesh type from resource handle
            // For now, default to Cube for all MeshFilter entities
            MeshType::Cube
        } else if let Some(mesh) = world.get_component::<Mesh>(entity) {
            mesh.mesh_type.clone()
        } else {
            return; // No mesh component found
        };
        
        // Enhanced world-to-screen projection for better visibility
        let (screen_pos, depth) = self.world_to_screen_enhanced(
            transform.position,
            camera_pos,
            camera_rot,
            rect.center(),
            rect,
        );
        
        let name = world.get_component::<engine_components_ui::Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
        
        // Always render mesh entities - no culling for Phase 10 debugging
        // Force render at center for first entity to test
        let screen_pos = if name.contains("GREEN TEST") {
            rect.center() + egui::Vec2::new(0.0, -100.0) // Force position for test cube
        } else {
            screen_pos
        };
        
        
        // Get material color with enhanced visibility
        let base_color = if let Some(material) = world.get_component::<Material>(entity) {
            egui::Color32::from_rgba_unmultiplied(
                (material.color[0] * 255.0) as u8,
                (material.color[1] * 255.0) as u8,
                (material.color[2] * 255.0) as u8,
                255, // Force full opacity
            )
        } else {
            egui::Color32::from_rgb(200, 200, 200)
        };
        
        let color = if selected_entity == Some(entity) { 
            egui::Color32::YELLOW 
        } else { 
            base_color 
        };
        
        // Enhanced size calculation - always visible
        let base_size = 40.0; // Larger base size
        let distance_from_camera = ((transform.position[0] - camera_pos[0]).powi(2) + 
                                   (transform.position[1] - camera_pos[1]).powi(2) + 
                                   (transform.position[2] - camera_pos[2]).powi(2)).sqrt();
        
        // Scale based on distance and object scale
        let perspective_scale = 100.0 / (distance_from_camera + 1.0).max(1.0);
        let size = base_size * transform.scale[0] * perspective_scale.max(0.5); // Minimum scale
        
        match mesh_type {
            MeshType::Cube => {
                // Use proper 3D cube rendering
                self.render_3d_cube(
                    painter,
                    transform.position,
                    transform.scale[0], // Use X scale as size
                    transform.rotation,
                    camera_pos,
                    camera_rot,
                    rect,
                    base_color,
                    selected_entity == Some(entity),
                );
            },
            MeshType::Sphere => {
                // Enhanced sphere rendering
                painter.circle_filled(screen_pos, size, color);
                painter.circle_stroke(screen_pos, size, egui::Stroke::new(2.0, egui::Color32::BLACK));
            },
            MeshType::Plane => {
                // Enhanced plane rendering
                let half_size = size;
                painter.rect_filled(
                    egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(half_size * 2.0)),
                    4.0,
                    color,
                );
            },
            MeshType::Custom(_) => {
                // Default to 3D cube for custom meshes
                self.render_3d_cube(
                    painter,
                    transform.position,
                    transform.scale[0],
                    transform.rotation,
                    camera_pos,
                    camera_rot,
                    rect,
                    base_color,
                    selected_entity == Some(entity),
                );
            }
        }
        
        // Always render name label for debugging
        painter.text(
            screen_pos + egui::Vec2::new(size + 5.0, -10.0),
            egui::Align2::LEFT_CENTER,
            &name,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }
    
    /// Enhanced world-to-screen projection with proper perspective
    fn world_to_screen_enhanced(
        &self,
        world_pos: [f32; 3],
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        view_center: egui::Pos2,
        viewport_rect: egui::Rect,
    ) -> (egui::Pos2, f32) {
        // Calculate relative position from camera
        let relative_pos = [
            world_pos[0] - camera_pos[0],
            world_pos[1] - camera_pos[1], 
            world_pos[2] - camera_pos[2]
        ];
        
        // Apply camera rotation (Unity-style: Y-axis yaw first, then X-axis pitch)
        let yaw = camera_rot[1];
        let pitch = camera_rot[0];
        
        // Rotate around Y-axis (yaw)
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = relative_pos[0] * cos_yaw + relative_pos[2] * sin_yaw;
        let rotated_z = -relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
        
        // Apply pitch rotation around X-axis
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let final_y = relative_pos[1] * cos_pitch - rotated_z * sin_pitch;
        let final_z = relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
        
        // Ensure objects are always visible by forcing positive depth
        let depth = final_z.max(0.1); // Minimum depth
        
        // Proper perspective projection with FOV and aspect ratio
        let fov_radians = 60.0_f32.to_radians(); // 60 degree FOV
        let aspect_ratio = viewport_rect.width() / viewport_rect.height();
        let projection_scale = viewport_rect.height() / (2.0 * (fov_radians / 2.0).tan());
        
        // Project to NDC space
        let ndc_x = (rotated_x / depth) * (1.0 / aspect_ratio);
        let ndc_y = final_y / depth;
        
        // Convert to screen coordinates
        let screen_x = view_center.x + ndc_x * projection_scale;
        let screen_y = view_center.y - ndc_y * projection_scale;
        
        (egui::pos2(screen_x, screen_y), depth)
    }
    
    /// Enhanced cube rendering with better visibility
    fn render_enhanced_cube(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        size: f32,
        _rotation: [f32; 3],
        color: egui::Color32,
        _name: &str,
    ) {
        // For now, just draw a simple square to indicate cube position
        // The real issue is that we need proper 3D vertices transformation
        let half_size = size * 0.5;
        
        // Draw a square with a cross to indicate it's a cube
        let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(size));
        painter.rect_filled(rect, 0.0, color);
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::BLACK));
        
        // Draw diagonals to show it's a 3D object placeholder
        painter.line_segment(
            [rect.left_top(), rect.right_bottom()],
            egui::Stroke::new(1.0, egui::Color32::BLACK.gamma_multiply(0.5)),
        );
        painter.line_segment(
            [rect.right_top(), rect.left_bottom()],
            egui::Stroke::new(1.0, egui::Color32::BLACK.gamma_multiply(0.5)),
        );
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, camera_pos: [f32; 3], camera_rot: [f32; 3]) {
        use super::improved_grid::*;
        // Get appropriate grid level based on camera height
        let camera_height = camera_pos[1];
        let level = get_grid_level(camera_height);
        
        // Calculate grid bounds
        let grid_extent = (level.extent / level.spacing) as i32;
        
        // Calculate grid bounds centered on origin for simplicity
        let grid_left = -grid_extent;
        let grid_right = grid_extent;
        let grid_top = -grid_extent;
        let grid_bottom = grid_extent;
        
        let view_center = rect.center();
        
        // Draw grid lines running along X axis (parallel to X)
        for z in grid_top..=grid_bottom {
            let world_z = z as f32 * level.spacing;
            
            // Start and end points of the line in world space at Y=0
            let start_world = [grid_left as f32 * level.spacing, 0.0, world_z];
            let end_world = [grid_right as f32 * level.spacing, 0.0, world_z];
            
            // Transform to screen space
            let (start_screen, start_depth) = self.world_to_screen(start_world, camera_pos, camera_rot, view_center, rect);
            let (end_screen, end_depth) = self.world_to_screen(end_world, camera_pos, camera_rot, view_center, rect);
            
            // Clip line to near plane if necessary
            let clipped = match clip_line_to_near_plane(start_world, end_world, start_depth, end_depth, 0.1) {
                Some((clipped_start, clipped_end, new_start_depth, new_end_depth)) => {
                    // Re-project clipped points if they changed
                    let start_s = if clipped_start[0] == start_world[0] && clipped_start[1] == start_world[1] && clipped_start[2] == start_world[2] {
                        start_screen
                    } else {
                        self.world_to_screen(clipped_start, camera_pos, camera_rot, view_center, rect).0
                    };
                    let end_s = if clipped_end[0] == end_world[0] && clipped_end[1] == end_world[1] && clipped_end[2] == end_world[2] {
                        end_screen
                    } else {
                        self.world_to_screen(clipped_end, camera_pos, camera_rot, view_center, rect).0
                    };
                    Some((start_s, end_s, new_start_depth, new_end_depth))
                },
                None => None,
            };
            
            let (start_screen, end_screen, start_depth, end_depth) = match clipped {
                Some(data) => data,
                None => continue,
            };
            
            // Check if line is within reasonable screen bounds
            if !is_line_in_bounds(start_screen, end_screen, rect, 200.0) {
                continue;
            }
            
            // Calculate distance for fading
            let mid_point = [
                (start_world[0] + end_world[0]) * 0.5,
                (start_world[1] + end_world[1]) * 0.5,
                (start_world[2] + end_world[2]) * 0.5,
            ];
            let distance = ((mid_point[0] - camera_pos[0]).powi(2) + 
                           (mid_point[1] - camera_pos[1]).powi(2) + 
                           (mid_point[2] - camera_pos[2]).powi(2)).sqrt();
            
            // Get line style based on grid coordinate and distance
            let (stroke_width, color) = get_line_style(z, false, distance, &level, camera_height);
            
            // Skip fully transparent lines
            if color.a() == 0 {
                continue;
            }
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(stroke_width, color)
            );
        }
        
        // Draw grid lines running along Z axis (parallel to Z)
        for x in grid_left..=grid_right {
            let world_x = x as f32 * level.spacing;
            
            // Start and end points of the line in world space at Y=0
            let start_world = [world_x, 0.0, grid_top as f32 * level.spacing];
            let end_world = [world_x, 0.0, grid_bottom as f32 * level.spacing];
            
            // Transform to screen space
            let (start_screen, start_depth) = self.world_to_screen(start_world, camera_pos, camera_rot, view_center, rect);
            let (end_screen, end_depth) = self.world_to_screen(end_world, camera_pos, camera_rot, view_center, rect);
            
            // Clip line to near plane if necessary
            let clipped = match clip_line_to_near_plane(start_world, end_world, start_depth, end_depth, 0.1) {
                Some((clipped_start, clipped_end, new_start_depth, new_end_depth)) => {
                    // Re-project clipped points if they changed
                    let start_s = if clipped_start[0] == start_world[0] && clipped_start[1] == start_world[1] && clipped_start[2] == start_world[2] {
                        start_screen
                    } else {
                        self.world_to_screen(clipped_start, camera_pos, camera_rot, view_center, rect).0
                    };
                    let end_s = if clipped_end[0] == end_world[0] && clipped_end[1] == end_world[1] && clipped_end[2] == end_world[2] {
                        end_screen
                    } else {
                        self.world_to_screen(clipped_end, camera_pos, camera_rot, view_center, rect).0
                    };
                    Some((start_s, end_s, new_start_depth, new_end_depth))
                },
                None => None,
            };
            
            let (start_screen, end_screen, start_depth, end_depth) = match clipped {
                Some(data) => data,
                None => continue,
            };
            
            // Check if line is within reasonable screen bounds
            if !is_line_in_bounds(start_screen, end_screen, rect, 200.0) {
                continue;
            }
            
            // Calculate distance for fading
            let mid_point = [
                (start_world[0] + end_world[0]) * 0.5,
                (start_world[1] + end_world[1]) * 0.5,
                (start_world[2] + end_world[2]) * 0.5,
            ];
            let distance = ((mid_point[0] - camera_pos[0]).powi(2) + 
                           (mid_point[1] - camera_pos[1]).powi(2) + 
                           (mid_point[2] - camera_pos[2]).powi(2)).sqrt();
            
            // Get line style based on grid coordinate and distance
            let (stroke_width, color) = get_line_style(x, true, distance, &level, camera_height);
            
            // Skip fully transparent lines
            if color.a() == 0 {
                continue;
            }
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(stroke_width, color)
            );
        }
        
        // Draw origin marker at (0, 0, 0) if visible
        let (origin_screen, origin_depth) = self.world_to_screen([0.0, 0.0, 0.0], camera_pos, camera_rot, view_center, rect);
        
        if origin_depth > 0.1 && rect.contains(origin_screen) {
            // Draw a small circle at the origin
            painter.circle_filled(
                origin_screen,
                4.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200)
            );
            painter.circle_stroke(
                origin_screen,
                4.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 255))
            );
            
            // Draw axis labels near origin
            let label_offset = 20.0;
            painter.text(
                origin_screen + egui::vec2(label_offset, 0.0),
                egui::Align2::LEFT_CENTER,
                "X",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgba_unmultiplied(200, 100, 100, 180)
            );
            painter.text(
                origin_screen + egui::vec2(0.0, -label_offset),
                egui::Align2::CENTER_BOTTOM,
                "Z",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgba_unmultiplied(100, 100, 200, 180)
            );
        }
    }
    
    fn render_entity(
        &self,
        world: &World,
        painter: &egui::Painter,
        rect: egui::Rect,
        entity: Entity,
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        selected_entity: Option<Entity>,
    ) {
        let Some(transform) = world.get_component::<Transform>(entity) else { return; };
        
        // Calculate screen position
        let (screen_pos, depth) = self.world_to_screen(
            transform.position,
            camera_pos,
            camera_rot,
            rect.center(),
            rect,
        );
        
        // Skip if behind camera (negative depth means behind camera)
        let name = world.get_component::<engine_components_ui::Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
        
        if depth <= 0.001 {  // Much smaller threshold
            return;
        }
        
        let is_selected = selected_entity == Some(entity);
        
        // Render based on entity type
        if world.get_component::<Camera>(entity).is_some() {
            object_renderer::render_camera(painter, screen_pos, &name, is_selected);
        } else {
            // Default object rendering
            let color = if is_selected { 
                egui::Color32::YELLOW 
            } else { 
                egui::Color32::from_rgb(150, 150, 150) 
            };
            painter.circle_filled(screen_pos, 8.0, color);
            painter.text(
                screen_pos + egui::vec2(10.0, -8.0),
                egui::Align2::LEFT_CENTER,
                format!("📍 {}", name),
                egui::FontId::proportional(12.0),
                color
            );
        }
    }
    
    fn render_mesh(
        &self,
        world: &World,
        painter: &egui::Painter,
        entity: Entity,
        mesh: &Mesh,
        transform: &Transform,
        screen_pos: egui::Pos2,
        depth: f32,
        camera_rot: [f32; 3],
        name: &str,
        is_selected: bool,
    ) {
        // Get material color
        let base_color = if let Some(material) = world.get_component::<Material>(entity) {
            egui::Color32::from_rgba_unmultiplied(
                (material.color[0] * 255.0) as u8,
                (material.color[1] * 255.0) as u8,
                (material.color[2] * 255.0) as u8,
                (material.color[3] * 255.0) as u8,
            )
        } else {
            egui::Color32::from_rgb(200, 200, 200)
        };
        
        let color = if is_selected { egui::Color32::YELLOW } else { base_color };
        
        // Calculate perspective scale
        let fov_scale = 100.0;
        let perspective_scale = fov_scale / depth;
        let base_size = 20.0;
        let size = base_size * transform.scale[0] * (perspective_scale / 2.0);
        
        match mesh.mesh_type {
            MeshType::Cube => {
                object_renderer::render_cube(
                    painter,
                    screen_pos,
                    size,
                    transform.rotation,
                    color,
                    camera_rot,
                    name,
                );
            }
            MeshType::Sphere => {
                let radius = 15.0 * transform.scale[0] * (perspective_scale / 2.0);
                object_renderer::render_sphere(
                    painter,
                    screen_pos,
                    radius,
                    transform.rotation,
                    color,
                    name,
                );
            }
            MeshType::Plane => {
                object_renderer::render_plane(
                    painter,
                    screen_pos,
                    size,
                    transform.rotation,
                    color,
                    name,
                );
            }
            _ => {
                // Default mesh representation
                painter.circle_filled(screen_pos, 10.0, color);
                painter.text(
                    screen_pos + egui::vec2(15.0, -10.0),
                    egui::Align2::LEFT_CENTER,
                    format!("📦 {}", name),
                    egui::FontId::proportional(12.0),
                    color
                );
            }
        }
    }
    
    fn render_sprites(
        &self,
        world: &World,
        painter: &egui::Painter,
        rect: egui::Rect,
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        selected_entity: Option<Entity>,
    ) {
        for (entity, _transform) in world.query_legacy::<Transform>() {
            if let Some(sprite_renderer) = world.get_component::<SpriteRenderer>(entity) {
                let transform = _transform;
                
                // Calculate screen position
                let (screen_pos, depth) = self.world_to_screen(
                    transform.position,
                    camera_pos,
                    camera_rot,
                    rect.center(),
                    rect,
                );
                
                // Skip if behind camera
                if depth <= 0.1 {
                    continue;
                }
                
                // Calculate sprite size
                let fov_scale = 100.0;
                let perspective_scale = fov_scale / depth;
                let world_scale = (transform.scale[0] + transform.scale[1]) * 0.5;
                let base_size = 32.0;
                let sprite_size = egui::vec2(
                    base_size * world_scale * (perspective_scale / 2.0), 
                    base_size * world_scale * (perspective_scale / 2.0)
                );
                
                // Get sprite color
                let sprite_color = egui::Color32::from_rgba_unmultiplied(
                    (sprite_renderer.sprite.color[0] * 255.0) as u8,
                    (sprite_renderer.sprite.color[1] * 255.0) as u8,
                    (sprite_renderer.sprite.color[2] * 255.0) as u8,
                    (sprite_renderer.sprite.color[3] * 255.0) as u8,
                );
                
                let is_selected = selected_entity == Some(entity);
                
                let name = world.get_component::<Name>(entity)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("Sprite {}", entity.id()));
                
                object_renderer::render_sprite(
                    painter,
                    screen_pos,
                    sprite_size,
                    sprite_color,
                    is_selected,
                    &name,
                );
            }
        }
    }
    
    fn world_to_screen(
        &self,
        world_pos: [f32; 3],
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        view_center: egui::Pos2,
        viewport_rect: egui::Rect,
    ) -> (egui::Pos2, f32) {
        // Calculate relative position from camera
        let relative_pos = [
            world_pos[0] - camera_pos[0],
            world_pos[1] - camera_pos[1], 
            world_pos[2] - camera_pos[2]
        ];
        
        // Apply camera rotation (Unity-style: Y-axis yaw first, then X-axis pitch)
        let yaw = camera_rot[1];
        let pitch = camera_rot[0];
        
        // Rotate around Y-axis (yaw)
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = relative_pos[0] * cos_yaw + relative_pos[2] * sin_yaw;
        let rotated_z = -relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
        
        // Apply pitch rotation around X-axis
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let final_y = relative_pos[1] * cos_pitch - rotated_z * sin_pitch;
        let final_z = relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
        
        // Depth for culling
        let depth = final_z;
        
        // Proper perspective projection with FOV and aspect ratio
        let fov_radians = 60.0_f32.to_radians(); // 60 degree FOV
        let aspect_ratio = viewport_rect.width() / viewport_rect.height();
        let projection_scale = viewport_rect.height() / (2.0 * (fov_radians / 2.0).tan());
        
        // Project to NDC space
        let ndc_x = (rotated_x / depth.max(0.1)) * (1.0 / aspect_ratio);
        let ndc_y = final_y / depth.max(0.1);
        
        // Convert to screen coordinates
        let screen_x = view_center.x + ndc_x * projection_scale;
        let screen_y = view_center.y - ndc_y * projection_scale;
        
        (egui::pos2(screen_x, screen_y), depth)
    }
    
    fn draw_scene_camera_indicator(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        camera_pos: [f32; 3],
    ) {
        let view_center = rect.center();
        painter.circle_filled(
            view_center, 
            8.0, 
            egui::Color32::from_rgba_unmultiplied(255, 255, 0, 200)
        );
        painter.text(
            view_center + egui::vec2(12.0, -8.0),
            egui::Align2::LEFT_CENTER,
            format!("📷 Scene Camera [{:.1}, {:.1}, {:.1}]", 
                camera_pos[0], camera_pos[1], camera_pos[2]),
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE
        );
    }
    
    fn draw_scene_overlay(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        world: &World,
        selected_entity: Option<Entity>,
        play_state: PlayState,
        scene_navigation: &SceneNavigation,
    ) {
        // Scene info overlay
        ui.allocate_ui_at_rect(egui::Rect::from_min_size(rect.min, egui::vec2(300.0, 120.0)), |ui| {
            ui.vertical(|ui| {
                match play_state {
                    PlayState::Editing => {
                        ui.label("🎨 Scene View (Editor Mode)");
                    }
                    PlayState::Playing => {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), 
                            "▶️ Scene View (Playing)");
                    }
                    PlayState::Paused => {
                        ui.colored_label(egui::Color32::from_rgb(200, 200, 100), 
                            "⏸️ Scene View (Paused)");
                    }
                };
                
                ui.label(format!("📦 {} objects", world.entity_count()));
                if let Some(entity) = selected_entity {
                    if let Some(transform) = world.get_component::<Transform>(entity) {
                        ui.label(format!("📍 Selected: {:.1}, {:.1}, {:.1}", 
                            transform.position[0], transform.position[1], transform.position[2]));
                    }
                }
                
                match play_state {
                    PlayState::Editing => ui.small("Click objects to select • Drag to orbit camera"),
                    PlayState::Playing => ui.small("Game running • Properties locked in Inspector"),
                    PlayState::Paused => ui.small("Game paused • Limited editing available"),
                };
            });
        });
    }
}