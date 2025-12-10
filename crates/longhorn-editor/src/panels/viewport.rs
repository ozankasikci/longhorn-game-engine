use egui::{Ui, TextureId, Sense};
use crate::styling::Colors;
use crate::CameraInput;
use crate::{GizmoState, GizmoConfig};
use crate::gizmo::{hit_test_gizmo, update_transform_from_drag, draw_gizmo};
use glam::Vec2;
use longhorn_core::{Transform, GlobalTransform, Sprite, World};

pub struct ViewportPanel {}

/// Actions that can be triggered from the viewport
#[derive(Debug, Clone, Default)]
pub struct ViewportAction {
    pub frame_selected: bool,
    pub transform_update: Option<Transform>,
    pub entity_clicked: Option<longhorn_core::EntityId>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        texture_id: Option<TextureId>,
        texture_size: Vec2,
        gizmo_state: &mut GizmoState,
        gizmo_config: &GizmoConfig,
        selected_transform: Option<Transform>,
        selected_global_transform: Option<GlobalTransform>,
        camera_pos: Vec2,
        camera_zoom: f32,
        world: &World,
    ) -> (CameraInput, ViewportAction) {
        ui.heading("Scene View (Editor Camera)");
        ui.separator();

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());

        if let Some(texture_id) = texture_id {
            // Draw the rendered game texture
            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Colors::TEXT_ON_ACCENT,
            );
        } else {
            // Placeholder when no texture is set
            ui.painter().rect_filled(
                rect,
                0.0,
                Colors::BG_VIEWPORT,
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Game Viewport",
                egui::FontId::proportional(20.0),
                Colors::TEXT_SECONDARY,
            );
        }

        // Capture camera input and gizmo interaction
        let mut camera_input = CameraInput::default();
        let mut action = ViewportAction::default();

        // Convert world position to screen position (matching Camera::world_to_screen)
        let world_to_screen = |world_pos: Vec2| -> Vec2 {
            let half_width = (texture_size.x / 2.0) / camera_zoom;
            let half_height = (texture_size.y / 2.0) / camera_zoom;

            // Convert from world space to NDC space
            let ndc_x = (world_pos.x - camera_pos.x) / half_width;
            let ndc_y = (world_pos.y - camera_pos.y) / half_height;

            // Convert from NDC space to texture space
            let texture_x = (ndc_x + 1.0) * texture_size.x / 2.0;
            let texture_y = (1.0 - ndc_y) * texture_size.y / 2.0;

            // Map from texture space to screen (egui rect) space
            let scale_x = rect.width() / texture_size.x;
            let scale_y = rect.height() / texture_size.y;
            Vec2::new(
                rect.left() + texture_x * scale_x,
                rect.top() + texture_y * scale_y,
            )
        };

        // Convert screen position to world position (matching Camera::screen_to_world)
        let screen_to_world = |screen_pos: Vec2| -> Vec2 {
            // Map from screen (egui rect) space to texture space
            let scale_x = texture_size.x / rect.width();
            let scale_y = texture_size.y / rect.height();
            let texture_x = (screen_pos.x - rect.left()) * scale_x;
            let texture_y = (screen_pos.y - rect.top()) * scale_y;

            // Convert from texture space (0,0 at top-left) to NDC space (-1 to 1)
            let ndc_x = (texture_x / texture_size.x) * 2.0 - 1.0;
            let ndc_y = 1.0 - (texture_y / texture_size.y) * 2.0;

            // Apply zoom and camera position
            let half_width = (texture_size.x / 2.0) / camera_zoom;
            let half_height = (texture_size.y / 2.0) / camera_zoom;

            Vec2::new(
                camera_pos.x + ndc_x * half_width,
                camera_pos.y + ndc_y * half_height,
            )
        };

        // Handle gizmo interaction if entity is selected
        if let (Some(transform), Some(global_transform)) = (selected_transform, selected_global_transform) {
            // Use GlobalTransform for rendering position (correct world position)
            let screen_pos = world_to_screen(global_transform.position);

            // Get mouse position in viewport
            let mouse_pos = ui.input(|i| i.pointer.hover_pos())
                .map(|p| Vec2::new(p.x, p.y));

            if let Some(mouse_pos) = mouse_pos {
                // Update hover state
                if !gizmo_state.is_dragging() {
                    gizmo_state.hover_handle = hit_test_gizmo(
                        mouse_pos,
                        screen_pos,
                        gizmo_state.mode,
                        gizmo_config,
                    );
                }

                // Handle drag start
                if response.drag_started() && gizmo_state.hover_handle.is_some() {
                    gizmo_state.begin_drag(
                        gizmo_state.hover_handle.unwrap(),
                        mouse_pos,
                        transform,
                    );
                }

                // Handle dragging
                if gizmo_state.is_dragging() {
                    if let (Some(drag_start_pos), Some(drag_start_transform), Some(active_handle)) = (
                        gizmo_state.drag_start_pos,
                        gizmo_state.drag_start_transform,
                        gizmo_state.active_handle,
                    ) {
                        // Convert both positions to world space for accurate tracking
                        let drag_start_world = screen_to_world(drag_start_pos);
                        let current_world = screen_to_world(mouse_pos);
                        let world_delta = current_world - drag_start_world;

                        let new_transform = update_transform_from_drag(
                            active_handle,
                            drag_start_transform,
                            world_delta,
                        );
                        action.transform_update = Some(new_transform);
                    }
                }

                // Handle drag end
                if response.drag_stopped() {
                    gizmo_state.end_drag();
                }
            }

            // Draw the gizmo
            draw_gizmo(
                ui.painter(),
                gizmo_config,
                gizmo_state.mode,
                screen_pos,
                gizmo_state.hover_handle,
                gizmo_state.active_handle,
            );
        }

        // Capture camera input when hovered (but not when dragging gizmo)
        if response.hovered() && !gizmo_state.is_dragging() {
            camera_input.mmb_held = ui.input(|i| {
                i.pointer.button_down(egui::PointerButton::Middle)
            });
            camera_input.rmb_held = ui.input(|i| {
                i.pointer.button_down(egui::PointerButton::Secondary)
            });

            // Get mouse delta - need to check both primary drag (for gizmos) and manual delta for RMB
            let drag_delta = response.drag_delta();

            // For RMB, we need to manually get the pointer delta since Sense doesn't track secondary button drags
            let pointer_delta = ui.input(|i| i.pointer.delta());

            // Use pointer delta if RMB is held, otherwise use the response drag delta
            if camera_input.rmb_held {
                camera_input.mouse_delta = Vec2::new(pointer_delta.x, pointer_delta.y);
            } else {
                camera_input.mouse_delta = Vec2::new(drag_delta.x, drag_delta.y);
            }

            camera_input.scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);

            // Check for F key to frame selected entity
            action.frame_selected = ui.input(|i| i.key_pressed(egui::Key::F));
        }

        // Handle entity selection by clicking
        // Only process clicks that aren't on gizmos and aren't drags
        if response.clicked() && !gizmo_state.is_dragging() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                let click_pos = Vec2::new(click_pos.x, click_pos.y);

                // Check if click is on gizmo first
                let clicked_on_gizmo = if let Some(_transform) = selected_transform {
                    gizmo_state.hover_handle.is_some()
                } else {
                    false
                };

                // If not clicking on gizmo, check for entity selection
                if !clicked_on_gizmo {
                    let world_click_pos = screen_to_world(click_pos);

                    // Find entity at click position
                    // We iterate in reverse order so topmost (last drawn) entities are checked first
                    let mut clicked_entity = None;
                    for (entity_id, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
                        let entity_pos = transform.position;
                        let half_size = sprite.size / 2.0;

                        // Check if click is within sprite bounds
                        if world_click_pos.x >= entity_pos.x - half_size.x
                            && world_click_pos.x <= entity_pos.x + half_size.x
                            && world_click_pos.y >= entity_pos.y - half_size.y
                            && world_click_pos.y <= entity_pos.y + half_size.y
                        {
                            clicked_entity = Some(entity_id);
                            // Don't break - keep going to find the topmost entity
                        }
                    }

                    action.entity_clicked = clicked_entity;
                }
            }
        }

        (camera_input, action)
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
