// Scene view implementation - full scene rendering logic

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material, Light, Visibility};
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
    ) -> Vec<ConsoleMessage> {
        let mut console_messages = Vec::new();
        let painter = ui.painter();
        
        
        // Get camera position and rotation for view transformation
        let camera_pos = scene_navigation.scene_camera_transform.position;
        let camera_rot = scene_navigation.scene_camera_transform.rotation;
        
        // Log camera state for debugging
        if camera_rot[0].abs() > 0.01 || camera_rot[1].abs() > 0.01 {
            console_messages.push(ConsoleMessage::info(&format!(
                "🎥 Camera: pos=[{:.1}, {:.1}, {:.1}], rot=[pitch={:.1}°, yaw={:.1}°]",
                camera_pos[0], camera_pos[1], camera_pos[2],
                camera_rot[0].to_degrees(), camera_rot[1].to_degrees()
            )));
        }
        
        
        // Draw scene objects using FAKE 2D PROJECTION (Temporary Phase 10 approach)
        let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        
        // DEBUG: Direct entity count check
        let total_entity_count = world.entity_count();
        
        // SUPER DEBUG: Check specific component counts
        let transform_count = world.query_legacy::<Transform>().count();
        let mesh_count_direct = world.query_legacy::<Mesh>().count();
        let material_count = world.query_legacy::<Material>().count();
        
        
        // Track entity changes and debug object positions
        let current_entity_count = entities_with_transforms.len();
        if current_entity_count != self.last_rendered_entity_count {
            console_messages.push(ConsoleMessage::info(&format!(
                "🚀 Phase 10: WGPU Integration Ready! {} entities | Camera at [{:.1}, {:.1}, {:.1}]",
                current_entity_count, camera_pos[0], camera_pos[1], camera_pos[2]
            )));
            
            // Debug: List entities with mesh components
            for entity in entities_with_transforms.iter().take(5) {
                if let Some(transform) = world.get_component::<Transform>(*entity) {
                    let name = world.get_component::<engine_components_ui::Name>(*entity)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| format!("Entity {}", entity.id()));
                    
                    let has_mesh = world.get_component::<Mesh>(*entity).is_some();
                    let has_material = world.get_component::<Material>(*entity).is_some();
                    let has_visibility = world.get_component::<Visibility>(*entity).is_some();
                    
                    console_messages.push(ConsoleMessage::info(&format!(
                        "  📦 {}: pos=[{:.1}, {:.1}, {:.1}] | Mesh: {} | Material: {} | Visible: {}",
                        name, transform.position[0], transform.position[1], transform.position[2],
                        has_mesh, has_material, has_visibility
                    )));
                }
            }
            
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
                    let has_mesh = world.get_component::<Mesh>(*entity).is_some();
                    debug_entity_info.push(format!("{}: Mesh={}, Pos=[{:.1},{:.1},{:.1}]", 
                        name, has_mesh, transform.position[0], transform.position[1], transform.position[2]));
                }
            }
            
            if world.get_component::<Mesh>(*entity).is_some() {
                mesh_count += 1;
                // Phase 10.1: Render with enhanced visibility
                if let Some(messages) = self.render_mesh_entity_enhanced(
                    world,
                    painter,
                    rect,
                    *entity,
                    camera_pos,
                    camera_rot,
                    selected_entity,
                ) {
                    console_messages.extend(messages);
                }
            } else {
                // Non-mesh entities (cameras, lights, etc)
                if let Some(messages) = self.render_entity(
                    world,
                    painter,
                    rect,
                    *entity,
                    camera_pos,
                    camera_rot,
                    selected_entity,
                ) {
                    console_messages.extend(messages);
                }
            }
        }
        
        
        
        // Render sprites separately (they need different handling)
        self.render_sprites(world, painter, rect, camera_pos, camera_rot, selected_entity);
        
        // Draw grid background (after objects for depth)
        self.draw_grid(painter, rect, camera_pos, camera_rot);
        
        // Draw scene camera indicator
        self.draw_scene_camera_indicator(painter, rect, camera_pos);
        
        // Draw scene overlay info
        self.draw_scene_overlay(ui, rect, world, selected_entity, play_state);
        
        console_messages
    }
    
    /// Phase 10.1: Enhanced mesh entity rendering with guaranteed visibility
    fn render_mesh_entity_enhanced(
        &self,
        world: &World,
        painter: &egui::Painter,
        rect: egui::Rect,
        entity: Entity,
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        selected_entity: Option<Entity>,
    ) -> Option<Vec<ConsoleMessage>> {
        let transform = world.get_component::<Transform>(entity)?;
        let mesh = world.get_component::<Mesh>(entity)?;
        
        // Enhanced world-to-screen projection for better visibility
        let (screen_pos, depth) = self.world_to_screen_enhanced(
            transform.position,
            camera_pos,
            camera_rot,
            rect.center(),
        );
        
        let name = world.get_component::<engine_components_ui::Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
        
        let debug_messages = Vec::new();
        
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
        
        match mesh.mesh_type {
            MeshType::Cube => {
                // Enhanced cube rendering
                self.render_enhanced_cube(painter, screen_pos, size, transform.rotation, color, &name);
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
                // Default to cube for custom meshes
                self.render_enhanced_cube(painter, screen_pos, size, transform.rotation, color, &name);
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
        
        Some(debug_messages)
    }
    
    /// Enhanced world-to-screen projection that never culls objects
    fn world_to_screen_enhanced(
        &self,
        world_pos: [f32; 3],
        camera_pos: [f32; 3],
        camera_rot: [f32; 3],
        view_center: egui::Pos2,
    ) -> (egui::Pos2, f32) {
        // Calculate relative position from camera
        let relative_pos = [
            world_pos[0] - camera_pos[0],
            world_pos[1] - camera_pos[1], 
            world_pos[2] - camera_pos[2]
        ];
        
        // Apply camera rotation (Unity-style: Y-axis yaw first, then X-axis pitch)
        // Note: We use the rotation values directly now since mouse input already handles the sign
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
        
        // Project to screen with perspective
        let screen_scale = 200.0;
        let screen_x = view_center.x + (rotated_x / depth) * screen_scale;
        let screen_y = view_center.y - (final_y / depth) * screen_scale;
        
        (egui::pos2(screen_x, screen_y), depth)
    }
    
    /// Enhanced cube rendering with better visibility
    fn render_enhanced_cube(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        size: f32,
        rotation: [f32; 3],
        color: egui::Color32,
        name: &str,
    ) {
        let half_size = size * 0.5;
        
        // Simple cube projection with isometric style
        let front_face = [
            center + egui::Vec2::new(-half_size, -half_size),
            center + egui::Vec2::new(half_size, -half_size),
            center + egui::Vec2::new(half_size, half_size),
            center + egui::Vec2::new(-half_size, half_size),
        ];
        
        // Draw main face
        painter.add(egui::Shape::convex_polygon(
            front_face.to_vec(),
            color,
            egui::Stroke::new(2.0, egui::Color32::BLACK),
        ));
        
        // Draw isometric edges for 3D effect
        let depth_offset = half_size * 0.3;
        let top_right = center + egui::Vec2::new(half_size + depth_offset, -half_size - depth_offset);
        let bottom_right = center + egui::Vec2::new(half_size + depth_offset, half_size - depth_offset);
        
        // Right face
        painter.add(egui::Shape::convex_polygon(
            vec![
                center + egui::Vec2::new(half_size, -half_size),
                top_right,
                bottom_right,
                center + egui::Vec2::new(half_size, half_size),
            ],
            color.gamma_multiply(0.7), // Darker for side face
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        ));
        
        // Top face
        painter.add(egui::Shape::convex_polygon(
            vec![
                center + egui::Vec2::new(-half_size, -half_size),
                center + egui::Vec2::new(half_size, -half_size),
                top_right,
                center + egui::Vec2::new(-half_size + depth_offset, -half_size - depth_offset),
            ],
            color.gamma_multiply(0.8), // Lighter for top face
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        ));
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, camera_pos: [f32; 3], camera_rot: [f32; 3]) {
        // Unity-style grid parameters
        let grid_size = 1.0; // 1 unit grid
        let major_grid_interval = 10; // Major lines every 10 units
        let pixels_per_unit = 50.0; // Scale factor
        
        // Draw a larger grid to ensure coverage
        let grid_extent = 50; // Fixed grid size: 50x50 units
        
        // Calculate grid bounds centered on origin for simplicity
        let grid_left = -grid_extent;
        let grid_right = grid_extent;
        let grid_top = -grid_extent;
        let grid_bottom = grid_extent;
        
        let view_center = rect.center();
        
        // Draw grid lines running along X axis (parallel to X)
        for z in grid_top..=grid_bottom {
            let world_z = z as f32 * grid_size;
            
            // Start and end points of the line in world space at Y=0
            let start_world = [grid_left as f32 * grid_size, 0.0, world_z];
            let end_world = [grid_right as f32 * grid_size, 0.0, world_z];
            
            // Transform to screen space
            let (start_screen, start_depth) = self.world_to_screen(start_world, camera_pos, camera_rot, view_center);
            let (end_screen, end_depth) = self.world_to_screen(end_world, camera_pos, camera_rot, view_center);
            
            // Skip if both points are behind camera
            if start_depth <= 0.1 && end_depth <= 0.1 {
                continue;
            }
            
            // Skip lines that would appear too distorted
            if start_depth <= 0.5 || end_depth <= 0.5 {
                continue;
            }
            
            // Also skip if line endpoints are too far outside the view
            if !rect.expand(100.0).contains(start_screen) && !rect.expand(100.0).contains(end_screen) {
                continue;
            }
            
            // Determine line style
            let (stroke_width, color) = if z == 0 {
                // Z-axis line (blue)
                (2.0, egui::Color32::from_rgba_unmultiplied(100, 100, 200, 180))
            } else if z % major_grid_interval == 0 {
                // Major grid lines
                (1.0, egui::Color32::from_rgba_unmultiplied(120, 120, 120, 120))
            } else {
                // Minor grid lines
                (0.5, egui::Color32::from_rgba_unmultiplied(80, 80, 80, 60))
            };
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(stroke_width, color)
            );
        }
        
        // Draw grid lines running along Z axis (parallel to Z)
        for x in grid_left..=grid_right {
            let world_x = x as f32 * grid_size;
            
            // Start and end points of the line in world space at Y=0
            let start_world = [world_x, 0.0, grid_top as f32 * grid_size];
            let end_world = [world_x, 0.0, grid_bottom as f32 * grid_size];
            
            // Transform to screen space
            let (start_screen, start_depth) = self.world_to_screen(start_world, camera_pos, camera_rot, view_center);
            let (end_screen, end_depth) = self.world_to_screen(end_world, camera_pos, camera_rot, view_center);
            
            // Skip if both points are behind camera
            if start_depth <= 0.1 && end_depth <= 0.1 {
                continue;
            }
            
            // Skip lines that would appear too distorted
            if start_depth <= 0.5 || end_depth <= 0.5 {
                continue;
            }
            
            // Also skip if line endpoints are too far outside the view
            if !rect.expand(100.0).contains(start_screen) && !rect.expand(100.0).contains(end_screen) {
                continue;
            }
            
            // Determine line style
            let (stroke_width, color) = if x == 0 {
                // X-axis line (red)
                (2.0, egui::Color32::from_rgba_unmultiplied(200, 100, 100, 180))
            } else if x % major_grid_interval == 0 {
                // Major grid lines
                (1.0, egui::Color32::from_rgba_unmultiplied(120, 120, 120, 120))
            } else {
                // Minor grid lines
                (0.5, egui::Color32::from_rgba_unmultiplied(80, 80, 80, 60))
            };
            
            painter.line_segment(
                [start_screen, end_screen],
                egui::Stroke::new(stroke_width, color)
            );
        }
        
        // Draw origin marker at (0, 0, 0) if visible
        let (origin_screen, origin_depth) = self.world_to_screen([0.0, 0.0, 0.0], camera_pos, camera_rot, view_center);
        
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
    ) -> Option<Vec<ConsoleMessage>> {
        let transform = world.get_component::<Transform>(entity)?;
        
        // Calculate screen position
        let (screen_pos, depth) = self.world_to_screen(
            transform.position,
            camera_pos,
            camera_rot,
            rect.center(),
        );
        
        // Skip if behind camera (negative depth means behind camera)
        let name = world.get_component::<engine_components_ui::Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
        
        if depth <= 0.001 {  // Much smaller threshold
            return None;
        }
        
        let is_selected = selected_entity == Some(entity);
        
        // Render based on entity type
        if world.get_component::<Camera>(entity).is_some() {
            object_renderer::render_camera(painter, screen_pos, &name, is_selected);
        } else if let Some(mesh) = world.get_component::<Mesh>(entity) {
            self.render_mesh(
                world,
                painter,
                entity,
                mesh,
                transform,
                screen_pos,
                depth,
                camera_rot,
                &name,
                is_selected,
            );
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
        
        None
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
    ) -> (egui::Pos2, f32) {
        // Calculate relative position from camera
        let relative_pos = [
            world_pos[0] - camera_pos[0],
            world_pos[1] - camera_pos[1], 
            world_pos[2] - camera_pos[2]
        ];
        
        // Apply camera rotation (Unity-style: Y-axis yaw first, then X-axis pitch)
        // Note: We use the rotation values directly now since mouse input already handles the sign
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
        
        // Simple perspective projection
        let depth = final_z;
        let fov_scale = 100.0;
        let perspective_scale = fov_scale / depth.max(0.1);
        
        let screen_x = view_center.x + (rotated_x * perspective_scale);
        let screen_y = view_center.y - (final_y * perspective_scale);
        
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