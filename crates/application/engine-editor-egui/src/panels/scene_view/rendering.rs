// Scene rendering logic

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material, Light, Visibility};
use engine_components_2d::SpriteRenderer;
use engine_renderer_3d::Camera;
use crate::types::SceneNavigation;
use crate::editor_state::ConsoleMessage;
use engine_components_ui::Name;

/// Helper for rendering 3D scenes in the editor
pub struct SceneRenderer {}

impl SceneRenderer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Draw depth reference lines in the scene view
    pub fn draw_depth_reference_lines(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        center: egui::Pos2,
        scale: f32,
    ) {
        let painter = ui.painter();
        let depth_line_color = egui::Color32::from_rgba_unmultiplied(80, 80, 80, 60);
        
        // Draw depth reference lines
        for depth in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            let offset = depth * scale * 0.3;
            let y_pos = center.y + offset;
            
            painter.line_segment(
                [
                    egui::pos2(rect.left() + 20.0, y_pos),
                    egui::pos2(rect.right() - 20.0, y_pos),
                ],
                egui::Stroke::new(1.0, depth_line_color),
            );
            
            // Depth label
            painter.text(
                egui::pos2(rect.left() + 5.0, y_pos),
                egui::Align2::LEFT_CENTER,
                format!("Z={:.0}", -depth * 5.0),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100),
            );
        }
    }
    
    /// Show message when no camera is available
    pub fn show_no_camera_message(ui: &mut egui::Ui, rect: egui::Rect, message: &str) {
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Camera Issue");
                    ui.label(message);
                    ui.small("Add a Camera component to an entity");
                    ui.small("Set 'is_main' to true for main camera");
                });
            });
        });
    }
    
    /// Find the main camera entity in the scene
    pub fn find_main_camera_entity(world: &World) -> Option<Entity> {
        // Look for entity with Camera component that has is_main = true
        for (entity, _transform) in world.query_legacy::<Transform>() {
            if let Some(camera) = world.get_component::<Camera>(entity) {
                if camera.is_main {
                    return Some(entity);
                }
            }
        }
        
        // If no main camera found, return the first camera entity
        for (entity, _transform) in world.query_legacy::<Transform>() {
            if world.get_component::<Camera>(entity).is_some() {
                return Some(entity);
            }
        }
        
        None
    }
    
    /// Render the scene from the camera's perspective using perspective projection
    pub fn render_scene_from_camera(
        world: &World,
        ui: &mut egui::Ui, 
        rect: egui::Rect, 
        camera_transform: &Transform, 
        camera: &Camera, 
        view_center: egui::Pos2
    ) {
        let painter = ui.painter();
        
        // Camera position and view parameters
        let camera_pos = camera_transform.position;
        let fov_rad = camera.fov.to_radians();
        let aspect_ratio = rect.width() / rect.height();
        
        // Calculate view frustum dimensions at different depths
        let render_scale = 100.0; // Scale factor for rendering
        
        // Render all entities with transforms
        for (entity, transform) in world.query_legacy::<Transform>() {
            // Skip the camera itself
            if let Some(camera_check) = world.get_component::<Camera>(entity) {
                if camera_check.is_main {
                    continue;
                }
            }
            
            // Calculate relative position from camera
            let relative_pos = [
                transform.position[0] - camera_pos[0],
                transform.position[1] - camera_pos[1], 
                transform.position[2] - camera_pos[2]
            ];
            
            // Simple perspective projection (assuming camera looks down -Z axis)
            let depth = -relative_pos[2]; // Distance from camera
            
            // Skip objects behind camera or too far away
            if depth <= camera.near || depth > camera.far {
                continue;
            }
            
            // Perspective projection to screen space
            let proj_x = relative_pos[0] / depth;
            let proj_y = relative_pos[1] / depth;
            
            // Convert to screen coordinates
            let screen_x = view_center.x + proj_x * render_scale;
            let screen_y = view_center.y - proj_y * render_scale; // Flip Y for screen space
            
            // Check if object is within screen bounds (frustum culling)
            let screen_pos = egui::pos2(screen_x, screen_y);
            if !rect.contains(screen_pos) {
                continue;
            }
            
            // Calculate object size based on depth (perspective scaling)
            let base_size = 20.0;
            let size_scale = camera.near / depth; // Objects get smaller with distance
            let object_size = base_size * size_scale * transform.scale[0];
            
            // Get entity info for rendering
            let name = world.get_component::<Name>(entity)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
            // Determine object color and shape based on components
            let (color, shape) = if let Some(sprite_renderer) = world.get_component::<SpriteRenderer>(entity) {
                // Render sprite
                let sprite_color = egui::Color32::from_rgba_unmultiplied(
                    (sprite_renderer.sprite.color[0] * 255.0) as u8,
                    (sprite_renderer.sprite.color[1] * 255.0) as u8,
                    (sprite_renderer.sprite.color[2] * 255.0) as u8,
                    (sprite_renderer.sprite.color[3] * 255.0) as u8,
                );
                (sprite_color, "square") // Sprites as squares
            } else if let Some(_mesh) = world.get_component::<Mesh>(entity) {
                // Render mesh object
                (egui::Color32::from_rgb(180, 180, 180), "circle") // Meshes as circles
            } else {
                // Default object
                (egui::Color32::YELLOW, "circle")
            };
            
            // Draw object
            if shape == "square" {
                let size_vec = egui::vec2(object_size, object_size);
                let obj_rect = egui::Rect::from_center_size(screen_pos, size_vec);
                painter.rect_filled(obj_rect, egui::Rounding::same(2.0), color);
            } else {
                painter.circle_filled(screen_pos, object_size * 0.5, color);
            }
            
            // Draw object label (smaller for distant objects)
            let label_size = (12.0 * size_scale).max(8.0);
            painter.text(
                screen_pos + egui::vec2(object_size * 0.5 + 2.0, -object_size * 0.5),
                egui::Align2::LEFT_CENTER,
                &name,
                egui::FontId::proportional(label_size),
                color
            );
        }
        
        // Draw depth indication lines for reference
        Self::draw_depth_reference_lines(ui, rect, view_center, render_scale);
    }
    
    /// Render the scene from the main camera's perspective
    pub fn render_camera_perspective(
        world: &World,
        ui: &mut egui::Ui, 
        rect: egui::Rect
    ) {
        let painter = ui.painter();
        
        // Draw background
        painter.rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(25, 25, 35) // Darker background for game view
        );
        
        // Find the main camera
        let main_camera_entity = Self::find_main_camera_entity(world);
        
        if let Some(camera_entity) = main_camera_entity {
            if let Some(camera_transform) = world.get_component::<Transform>(camera_entity).cloned() {
                if let Some(camera) = world.get_component::<Camera>(camera_entity).cloned() {
                    // Calculate camera projection parameters
                    let aspect_ratio = rect.width() / rect.height();
                    let view_center = rect.center();
                    
                    // Render objects from camera perspective
                    Self::render_scene_from_camera(
                        world, ui, rect, &camera_transform, &camera, view_center
                    );
                    
                    // Draw camera info overlay
                    ui.allocate_ui_at_rect(egui::Rect::from_min_size(rect.min, egui::vec2(250.0, 80.0)), |ui| {
                        ui.vertical(|ui| {
                            ui.colored_label(egui::Color32::WHITE, 
                                format!("📷 Camera View (FOV: {:.0}°)", camera.fov));
                            ui.small(format!("Position: [{:.1}, {:.1}, {:.1}]", 
                                camera_transform.position[0], 
                                camera_transform.position[1], 
                                camera_transform.position[2]));
                            ui.small(format!("Aspect: {:.2} | Near: {:.1} | Far: {:.0}", 
                                aspect_ratio, camera.near, camera.far));
                        });
                    });
                } else {
                    Self::show_no_camera_message(ui, rect, "Camera component missing");
                }
            } else {
                Self::show_no_camera_message(ui, rect, "Camera transform missing");
            }
        } else {
            Self::show_no_camera_message(ui, rect, "No main camera found");
        }
    }
}