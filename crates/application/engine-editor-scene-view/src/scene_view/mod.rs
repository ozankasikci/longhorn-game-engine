// Scene view module - handles 3D scene rendering and interaction

pub mod camera_movement;
pub mod debug_overlay;
pub mod ecs_camera_bridge;
pub mod gizmo_3d_input;
pub mod improved_grid;
pub mod navigation;
pub mod object_renderer;
pub mod rendering;
pub mod scene_view_impl;

#[cfg(test)]
mod camera_movement_tests;
#[cfg(test)]
mod navigation_tests;

use crate::types::{ConsoleMessage, GizmoSystem, SceneNavigation};
use eframe::egui;
use engine_components_3d::{MeshFilter, Transform};
use engine_components_ui::Name;
use engine_ecs_core::{Entity, World};
use glam::{Mat4, Vec3};

/// Focus the scene camera on the selected object
fn focus_on_selected_object(
    world: &World,
    selected_entity: Entity,
    scene_navigation: &mut SceneNavigation,
) {
    if let Some(transform) = world.get_component::<Transform>(selected_entity) {
        // Get object position
        let object_pos = transform.position;

        // The camera actually looks in +Z direction when rotation is [0,0,0]
        // So to look at an object, we need to place the camera in FRONT (negative Z)

        scene_navigation.scene_camera_transform.position = [
            object_pos[0],       // Same X as object
            object_pos[1] + 1.5, // 1.5 units above
            object_pos[2] - 5.0, // 5 units in FRONT (negative Z)
        ];

        // With camera in front looking back (+Z direction), no rotation needed
        scene_navigation.scene_camera_transform.rotation = [0.0, 0.0, 0.0];

        // Debug output
        let name = world
            .get_component::<Name>(selected_entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", selected_entity.id()));

        eprintln!("\n=== FOCUS: Fixed for +Z look direction ===");
        eprintln!(
            "Object '{}' at: [{:.2}, {:.2}, {:.2}]",
            name, object_pos[0], object_pos[1], object_pos[2]
        );
        eprintln!(
            "Camera pos: [{:.2}, {:.2}, {:.2}] (in front, looking back)",
            scene_navigation.scene_camera_transform.position[0],
            scene_navigation.scene_camera_transform.position[1],
            scene_navigation.scene_camera_transform.position[2]
        );
        eprintln!("Rotation: [0, 0, 0] (looking in +Z direction)");
        eprintln!("==================\n");
    }
}

/// Get camera view and projection matrices for 3D projection
fn get_camera_matrices(
    world: &World,
    scene_navigation: &SceneNavigation,
    viewport_rect: egui::Rect,
) -> (Option<Mat4>, Option<Mat4>) {
    // Create view matrix from scene camera transform
    let camera_transform = &scene_navigation.scene_camera_transform;
    let camera_pos = Vec3::from_array(camera_transform.position);
    let camera_rot = Vec3::from_array(camera_transform.rotation);

    // The renderer uses -Z as forward direction (standard for graphics)
    // Calculate the proper view matrix matching the renderer's convention

    // Always use rotation-based view matrix for consistency
    // Create quaternion from Euler angles (YXZ order, matching renderer)
    let quat = glam::Quat::from_euler(
        glam::EulerRot::YXZ,
        camera_rot.y,
        camera_rot.x,
        camera_rot.z,
    );

    // Forward is -Z (matching renderer convention)
    let forward = quat * Vec3::NEG_Z;
    let target = camera_pos + forward;
    let up = quat * Vec3::Y;

    let view_matrix = Mat4::look_at_rh(camera_pos, target, up);

    // Create projection matrix
    let aspect_ratio = viewport_rect.width() / viewport_rect.height();
    let fov = 60.0_f32.to_radians();
    let projection_matrix = Mat4::perspective_rh(fov, aspect_ratio, 0.1, 1000.0);

    (Some(view_matrix), Some(projection_matrix))
}

/// Scene view panel for 3D scene rendering and manipulation
pub struct SceneViewPanel {
    pub scene_view_active: bool,
    gizmo_3d_input: gizmo_3d_input::Gizmo3DInput,
}

impl SceneViewPanel {
    pub fn new() -> Self {
        Self {
            scene_view_active: true,
            gizmo_3d_input: gizmo_3d_input::Gizmo3DInput::new(),
        }
    }

    /// Main entry point for rendering the scene view
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: Option<Entity>,
        scene_navigation: &mut SceneNavigation,
        gizmo_system: &mut dyn GizmoSystem,
        scene_renderer: &mut scene_view_impl::SceneViewRenderer,
        play_state: crate::PlayState,
    ) -> Vec<ConsoleMessage> {
        // Scene view toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.scene_view_active, true, "Scene");
            ui.selectable_value(&mut self.scene_view_active, false, "Game");

            ui.separator();

            // Simple mode indicator
            if selected_entity.is_some() {
                ui.label("🎯 MOVE MODE (Simple Gizmos Active)");
            } else {
                ui.label("Select an object to show gizmos");
            }

            ui.separator();

            if ui
                .button("🔍")
                .on_hover_text("Focus on selected (F)")
                .clicked()
            {
                if let Some(entity) = selected_entity {
                    focus_on_selected_object(world, entity, scene_navigation);
                }
            }
        });

        ui.separator();

        // Main view area - allocate space first
        let available_size = ui.available_size();
        let (rect, mut response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        // CRITICAL: Create an interactive area that captures mouse events
        // This ensures the scene view gets mouse input even in a docked panel
        response = ui.interact(rect, response.id, egui::Sense::click_and_drag());

        // Debug input events (only important ones)
        if response.clicked() {
            eprintln!("Scene view clicked");
        }
        if response.drag_started() {
            eprintln!("Scene view drag started");
        }

        // Force focus when hovering to ensure we get input priority
        if response.hovered() {
            response.request_focus();
        }

        // Get camera matrices for 3D projection
        let (camera_view_matrix, camera_projection_matrix) =
            get_camera_matrices(world, scene_navigation, rect);

        // Handle 3D gizmo input before navigation
        let gizmo_handled = if !scene_navigation.is_navigating {
            if let (Some(view), Some(proj)) = (camera_view_matrix, camera_projection_matrix) {
                self.gizmo_3d_input.handle_input(
                    world,
                    selected_entity,
                    &response,
                    rect,
                    view,
                    proj,
                )
            } else {
                false
            }
        } else {
            false
        };

        // Handle navigation only if gizmo didn't handle the input
        let mut console_messages = if gizmo_handled {
            // If gizmo handled input, ensure we maintain focus
            response.request_focus();
            Vec::new()
        } else {
            navigation::SceneNavigator::handle_scene_navigation(
                scene_navigation,
                ui,
                &response,
                rect,
            )
        };

        // Draw background
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(35, 35, 35),
        );

        // Draw 3D scene first
        if self.scene_view_active {
            scene_renderer.draw_scene(
                world,
                ui,
                rect,
                &response,
                scene_navigation,
                selected_entity,
                play_state,
            );
        }

        // Scene content overlay - CRITICAL: This must come AFTER 3D scene
        ui.allocate_ui_at_rect(rect, |ui| {
            if self.scene_view_active {
                // Draw debug overlay
                debug_overlay::draw_movement_debug_overlay(ui, rect, scene_navigation);

                // Disabled 2D overlay gizmos - using true 3D gizmos instead
                // if !scene_navigation.is_navigating {
                //     if let (Some(view), Some(proj)) = (camera_view_matrix, camera_projection_matrix) {
                //         self.gizmo.update(
                //             ui,
                //             &response,
                //             rect,
                //             world,
                //             selected_entity,
                //             view,
                //             proj,
                //         );
                //     }
                // }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Runtime game preview");
                        ui.small("Press Play to see game running");
                    });
                });
            }
        });

        // Handle keyboard shortcuts for scene view
        ui.input(|i| {
            // F key to focus on selected object
            if i.key_pressed(egui::Key::F) && selected_entity.is_some() {
                if let Some(entity) = selected_entity {
                    focus_on_selected_object(world, entity, scene_navigation);
                }
            }
        });

        console_messages
    }
}
