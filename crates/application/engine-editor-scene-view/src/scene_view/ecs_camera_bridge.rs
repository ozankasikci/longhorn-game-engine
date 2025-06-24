//! Bridge between editor camera navigation and ECS camera system

use crate::types::SceneNavigation;
use eframe::egui;
use engine_camera_impl::{CameraController, CameraInput, FPSCameraController};
use engine_components_3d::{Camera, MainCamera, Transform};
use engine_ecs_core::{Entity, World, WorldBundleExt};

/// Manages the editor camera as an ECS entity
pub struct EditorCameraManager {
    /// The editor camera entity
    pub camera_entity: Option<Entity>,

    /// FPS controller for the editor camera
    pub fps_controller: FPSCameraController,
}

impl Default for EditorCameraManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCameraManager {
    pub fn new() -> Self {
        let mut fps_controller = FPSCameraController::new();
        fps_controller.mouse_sensitivity = 0.002;
        fps_controller.movement_speed = 5.0;
        fps_controller.movement_smoothing = 0.0; // Instant movement for editor

        Self {
            camera_entity: None,
            fps_controller,
        }
    }

    /// Initialize the editor camera in the ECS world
    pub fn initialize(&mut self, world: &mut World) {
        // Check if we already have a camera
        if self.camera_entity.is_some() {
            return;
        }

        // Create camera components
        // Position camera to see the cube
        // With fixed forward calculation, rotation (0,0,0) looks down -Z
        let transform = Transform::default()
            .with_position(0.0, 2.0, 5.0)
            .with_rotation(0.0, 0.0, 0.0); // Looking down -Z axis

        let camera = Camera::perspective(60.0, 0.1, 1000.0)
            .with_priority(100) // High priority for editor camera
            .with_clear_color([0.1, 0.1, 0.1, 1.0]);

        // Spawn the camera entity with a tuple bundle
        let entity = world.spawn_bundle((transform, camera));

        // Add MainCamera tag to make it the default camera
        world.add_component(entity, MainCamera).unwrap();

        self.camera_entity = Some(entity);
    }

    /// Update camera from legacy scene navigation (for compatibility)
    pub fn sync_from_scene_navigation(&mut self, world: &mut World, scene_nav: &SceneNavigation) {
        if let Some(entity) = self.camera_entity {
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.position = scene_nav.scene_camera_transform.position;
                transform.rotation = scene_nav.scene_camera_transform.rotation;

                // Sync controller rotation
                self.fps_controller.sync_rotation(transform);
            }
        }
    }

    /// Update scene navigation from camera (for compatibility)
    pub fn sync_to_scene_navigation(&self, world: &World, scene_nav: &mut SceneNavigation) {
        if let Some(entity) = self.camera_entity {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                scene_nav.scene_camera_transform = transform.clone();
            }
        }
    }

    /// Handle camera input using the FPS controller
    pub fn handle_input(
        &mut self,
        world: &mut World,
        ui: &egui::Ui,
        response: &egui::Response,
        delta_time: f32,
    ) {
        if let Some(entity) = self.camera_entity {
            // Get current transform
            let mut transform = world
                .get_component::<Transform>(entity)
                .cloned()
                .unwrap_or_default();

            // Convert egui input to CameraInput
            let is_navigating = response.dragged_by(egui::PointerButton::Secondary);

            let mouse_delta = if is_navigating {
                ui.input(|i| i.pointer.delta())
            } else {
                egui::Vec2::ZERO
            };

            let mut movement = [0.0, 0.0, 0.0];
            if is_navigating {
                if ui.input(|i| i.key_down(egui::Key::W)) {
                    movement[2] += 1.0;
                }
                if ui.input(|i| i.key_down(egui::Key::S)) {
                    movement[2] -= 1.0;
                }
                if ui.input(|i| i.key_down(egui::Key::A)) {
                    movement[0] -= 1.0;
                }
                if ui.input(|i| i.key_down(egui::Key::D)) {
                    movement[0] += 1.0;
                }
                if ui.input(|i| i.key_down(egui::Key::Q)) {
                    movement[1] -= 1.0;
                }
                if ui.input(|i| i.key_down(egui::Key::E)) {
                    movement[1] += 1.0;
                }
            }

            let camera_input = CameraInput {
                mouse_delta: [mouse_delta.x, mouse_delta.y],
                movement,
                secondary_mouse: is_navigating,
                modifiers: engine_camera_impl::controllers::ModifierKeys {
                    shift: ui.input(|i| i.modifiers.shift),
                    ctrl: ui.input(|i| i.modifiers.ctrl),
                    alt: ui.input(|i| i.modifiers.alt),
                },
                ..Default::default()
            };

            // Update transform with controller
            self.fps_controller
                .update(&mut transform, &camera_input, delta_time);

            // Apply transform back to entity
            if let Some(transform_comp) = world.get_component_mut::<Transform>(entity) {
                *transform_comp = transform;
            }
        }
    }

    /// Update camera aspect ratio
    pub fn update_aspect_ratio(&self, world: &mut World, _aspect_ratio: f32) {
        if let Some(entity) = self.camera_entity {
            if let Some(_camera) = world.get_component_mut::<Camera>(entity) {
                // The aspect ratio is calculated dynamically from viewport,
                // but we can store a hint here if needed
            }
        }
    }
}
