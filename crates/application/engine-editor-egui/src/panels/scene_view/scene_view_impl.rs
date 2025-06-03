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
    
    /// Main scene rendering function
    pub fn draw_scene(
        &mut self,
        world: &World,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        scene_navigation: &SceneNavigation,
        selected_entity: Option<Entity>,
        play_state: PlayState,
    ) -> Vec<ConsoleMessage> {
        let mut console_messages = Vec::new();
        let painter = ui.painter();
        
        // Get camera position and rotation for view transformation
        let camera_pos = scene_navigation.scene_camera_transform.position;
        let camera_rot = scene_navigation.scene_camera_transform.rotation;
        
        // Draw grid background
        self.draw_grid(painter, rect, camera_pos);
        
        // Draw scene objects
        let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        
        // Track entity changes
        let current_entity_count = entities_with_transforms.len();
        if current_entity_count != self.last_rendered_entity_count {
            console_messages.push(ConsoleMessage::info(&format!(
                "🎮 Phase 10.2: Total entities with Transform: {}",
                current_entity_count
            )));
            self.last_rendered_entity_count = current_entity_count;
        }
        
        // Render all entities
        for entity in entities_with_transforms {
            if let Some(messages) = self.render_entity(
                world,
                painter,
                rect,
                entity,
                camera_pos,
                camera_rot,
                selected_entity,
            ) {
                console_messages.extend(messages);
            }
        }
        
        // Render sprites separately (they need different handling)
        self.render_sprites(world, painter, rect, camera_pos, camera_rot, selected_entity);
        
        // Draw scene camera indicator
        self.draw_scene_camera_indicator(painter, rect, camera_pos);
        
        // Draw scene overlay info
        self.draw_scene_overlay(ui, rect, world, selected_entity, play_state);
        
        // TODO: Add gizmo rendering here if move tool is active
        
        console_messages
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, camera_pos: [f32; 3]) {
        let view_center = rect.center();
        
        // Apply camera offset to grid rendering
        let camera_offset_x = -camera_pos[0] * 50.0; // 50 pixels per world unit
        let camera_offset_y = camera_pos[2] * 50.0;  // Z becomes Y in screen space
        
        // Draw grid lines with camera offset
        painter.line_segment(
            [egui::pos2(rect.left(), view_center.y + camera_offset_y), 
             egui::pos2(rect.right(), view_center.y + camera_offset_y)],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
        painter.line_segment(
            [egui::pos2(view_center.x + camera_offset_x, rect.top()), 
             egui::pos2(view_center.x + camera_offset_x, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
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
        
        // Skip if behind camera
        if depth <= 0.1 {
            return None;
        }
        
        // Get entity info
        let name = world.get_component::<Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
        
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
        
        // Apply camera rotation
        let yaw = camera_rot[1];
        let pitch = camera_rot[0];
        
        // Rotate around Y-axis (yaw)
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = relative_pos[0] * cos_yaw + relative_pos[2] * sin_yaw;
        let rotated_z = -relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
        
        // Apply pitch rotation
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let final_y = relative_pos[1] * cos_pitch + rotated_z * sin_pitch;
        let final_z = -relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
        
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