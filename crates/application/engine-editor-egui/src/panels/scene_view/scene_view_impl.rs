// Scene view implementation - 3D scene rendering with engine-renderer-3d

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Mesh, MeshType, Material, Light, Visibility, MeshFilter, MeshRenderer};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_renderer_3d::{Camera, EguiRenderWidget, EcsRenderBridge, CameraController, Renderer3D};
use crate::types::{SceneNavigation, GizmoSystem, PlayState};
use crate::editor_state::ConsoleMessage;
use super::object_renderer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SceneViewRenderer {
    last_rendered_entity_count: usize,
    render_widget: Option<EguiRenderWidget>,
    game_render_widget: Option<EguiRenderWidget>, // Separate widget for game view
    ecs_bridge: Option<EcsRenderBridge>,
    camera_controller: CameraController,
    pub editor_camera: super::ecs_camera_bridge::EditorCameraManager,
}

impl SceneViewRenderer {
    pub fn new() -> Self {
        Self {
            last_rendered_entity_count: 0,
            render_widget: None,
            game_render_widget: None,
            ecs_bridge: None,
            camera_controller: CameraController::new(Camera::default()),
            editor_camera: super::ecs_camera_bridge::EditorCameraManager::new(),
        }
    }
    
    /// Initialize the 3D renderer (call this when we have wgpu context)
    pub fn initialize_renderer(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Result<(), anyhow::Error> {
        // Create 3D renderer
        let device_clone = device.clone();
        let queue_clone = queue.clone();
        let renderer = pollster::block_on(async {
            Renderer3D::new(device_clone, queue_clone, 800, 600).await
        })?;
        
        // Get the actual mesh and material IDs from the renderer
        let cube_mesh_id = renderer.get_default_mesh_id("cube").unwrap_or(0);
        let sphere_mesh_id = renderer.get_default_mesh_id("sphere").unwrap_or(1);
        let plane_mesh_id = renderer.get_default_mesh_id("plane").unwrap_or(2);
        let default_material_id = renderer.get_default_material_id("default").unwrap_or(0);
        
        log::info!("Mesh IDs - cube: {}, sphere: {}, plane: {}", cube_mesh_id, sphere_mesh_id, plane_mesh_id);
        
        // Create render widgets - one for scene view, one for game view
        let renderer = Arc::new(Mutex::new(renderer));
        self.render_widget = Some(EguiRenderWidget::new(renderer.clone()));
        
        // Create a second renderer for the game view
        let mut game_renderer = pollster::block_on(async {
            Renderer3D::new(device, queue, 800, 600).await
        })?;
        
        // Disable grid for game view - it's an editor-only feature
        game_renderer.set_grid_enabled(false);
        
        let game_renderer = Arc::new(Mutex::new(game_renderer));
        self.game_render_widget = Some(EguiRenderWidget::new(game_renderer));
        
        // Create ECS bridge with actual mappings
        let mut mesh_mappings = HashMap::new();
        mesh_mappings.insert("cube".to_string(), cube_mesh_id);
        mesh_mappings.insert("sphere".to_string(), sphere_mesh_id);
        mesh_mappings.insert("plane".to_string(), plane_mesh_id);
        
        let mut material_mappings = HashMap::new();
        material_mappings.insert("default".to_string(), default_material_id);
        
        self.ecs_bridge = Some(EcsRenderBridge::new(
            mesh_mappings,
            material_mappings, 
            cube_mesh_id, // default mesh ID
            default_material_id, // default material ID
        ));
        
        Ok(())
    }
    
    /// Main scene rendering function - Using engine-renderer-3d
    pub fn draw_scene(
        &mut self,
        world: &mut World,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        scene_navigation: &mut SceneNavigation,
        selected_entity: Option<Entity>,
        play_state: PlayState,
    ) {
        // Initialize editor camera in ECS if needed
        self.editor_camera.initialize(world);
        
        // Handle camera input
        let delta_time = ui.input(|i| i.stable_dt);
        self.editor_camera.handle_input(world, ui, response, delta_time);
        
        // Sync camera with scene navigation (for compatibility)
        self.editor_camera.sync_to_scene_navigation(world, scene_navigation);
        
        // Update camera matrices
        engine_camera_impl::camera_update_system(world);
        
        // Get camera from ECS for rendering
        let camera = if let Some(entity) = self.editor_camera.camera_entity {
            if let (Some(transform), Some(cam)) = (
                world.get_component::<engine_components_3d::Transform>(entity),
                world.get_component::<engine_components_3d::Camera>(entity)
            ) {
                Camera::from_position_rotation(
                    transform.position,
                    transform.rotation,
                    rect.aspect_ratio(),
                )
            } else {
                // Fallback
                Camera::from_position_rotation(
                    scene_navigation.scene_camera_transform.position,
                    scene_navigation.scene_camera_transform.rotation,
                    rect.aspect_ratio(),
                )
            }
        } else {
            // Fallback
            Camera::from_position_rotation(
                scene_navigation.scene_camera_transform.position,
                scene_navigation.scene_camera_transform.rotation,
                rect.aspect_ratio(),
            )
        };
        
        self.camera_controller.camera = camera;
        
        // If we have the renderer initialized, use it
        if let (Some(render_widget), Some(ecs_bridge)) = (&mut self.render_widget, &self.ecs_bridge) {
            log::info!("SCENE VIEW: Using 3D renderer with camera at pos={:?}", self.camera_controller.camera.position);
            
            // Convert ECS world to render scene
            let render_scene = ecs_bridge.world_to_render_scene(world, self.camera_controller.camera.clone());
            log::info!("Created render scene with {} objects", render_scene.objects.len());
            
            // Render the scene
            if let Err(e) = render_widget.render_scene(&render_scene) {
                // Fallback to 2D rendering on error
                log::error!("3D rendering failed: {}, falling back to 2D", e);
                self.draw_fallback_2d_scene(world, ui, rect, scene_navigation);
                return;
            }
            
            log::info!("Rendering complete, adding widget to UI");
            // Display the rendered result
            let response = ui.add(render_widget);
            log::info!("Widget added, response rect: {:?}", response.rect);
            
        } else {
            log::warn!("3D renderer not initialized, using 2D fallback");
            // Fallback to 2D rendering if 3D renderer not initialized
            self.draw_fallback_2d_scene(world, ui, rect, scene_navigation);
        }
        
        // Track entity count for debugging
        let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        let current_entity_count = entities_with_transforms.len();
        if current_entity_count != self.last_rendered_entity_count {
            self.last_rendered_entity_count = current_entity_count;
        }
    }
    
    /// Render from a specific camera (used for game view)
    pub fn render_game_camera_view(
        &mut self,
        world: &mut World,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        camera: Camera,
    ) {
        log::info!("render_game_camera_view called with camera pos: {:?}, target: {:?}", camera.position, camera.target);
        
        // IMPORTANT: We need to render the scene with the game camera, not reuse the scene view's render
        // The game view should always show what the main camera sees
        
        // Use the dedicated game render widget
        if let (Some(render_widget), Some(ecs_bridge)) = (&mut self.game_render_widget, &self.ecs_bridge) {
            log::info!("Rendering game view from main camera perspective");
            
            // Convert ECS world to render scene with the GAME CAMERA
            let render_scene = ecs_bridge.world_to_render_scene(world, camera);
            log::info!("Created game view render scene with {} objects", render_scene.objects.len());
            
            // Render the scene with the game camera
            if let Err(e) = render_widget.render_scene(&render_scene) {
                log::error!("Game view 3D rendering failed: {}", e);
                // Could fall back to 2D here if needed
            } else {
                log::info!("Game view render succeeded");
                
                // Display the rendered result at the specific rect
                ui.allocate_ui_at_rect(rect, |ui| {
                    // Force the UI to use the full rect
                    ui.set_min_size(rect.size());
                    ui.set_max_size(rect.size());
                    let response = ui.add(render_widget);
                    log::info!("Game view widget added, response rect: {:?}", response.rect);
                });
            }
        } else {
            log::warn!("Renderer not initialized for game view");
        }
    }
    
    /// Fallback 2D scene rendering when 3D renderer is unavailable
    fn draw_fallback_2d_scene(
        &mut self,
        world: &World,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        scene_navigation: &SceneNavigation,
    ) {
        let painter = ui.painter();
        
        // Draw basic grid background
        let view_center = rect.center();
        let camera_pos = scene_navigation.scene_camera_transform.position;
        let camera_offset_x = -camera_pos[0] * 50.0;
        let camera_offset_y = camera_pos[2] * 50.0;
        
        // Grid lines
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
        
        // Draw simple 2D representation of entities
        let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        for (entity, transform) in world.query_legacy::<Transform>() {
            if world.get_component::<MeshFilter>(entity).is_some() {
                // Draw a simple square for mesh entities
                let pos = [
                    view_center.x + camera_offset_x + transform.position[0] * 50.0,
                    view_center.y + camera_offset_y - transform.position[2] * 50.0,
                ];
                let size = 10.0 * transform.scale[0];
                
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(pos[0], pos[1]),
                        egui::Vec2::splat(size)
                    ),
                    0.0,
                    egui::Color32::from_rgb(150, 150, 200)
                );
            }
        }
    }
}
