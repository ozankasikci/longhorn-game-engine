// Scene navigation and camera controls

use eframe::egui;
use crate::types::SceneNavigation;
use engine_components_3d::Transform;

/// Scene navigation helper that manages camera movement in the editor
pub struct SceneNavigator;

impl SceneNavigator {
    /// Start scene navigation (right mouse button pressed)
    pub fn start_navigation(scene_nav: &mut SceneNavigation, mouse_pos: egui::Pos2)  {
        if !scene_nav.enabled {
            return;
        }
        
        scene_nav.is_navigating = true;
        scene_nav.last_mouse_pos = Some(mouse_pos);
        
        // Done
    }
    
    /// End scene navigation (right mouse button released)
    pub fn end_navigation(scene_nav: &mut SceneNavigation)  {
        if scene_nav.is_navigating {
            scene_nav.is_navigating = false;
            scene_nav.last_mouse_pos = None;
        }
        
        // Done
    }
    
    /// Apply mouse look rotation to the camera - Simple direct rotation
    pub fn apply_mouse_look(scene_nav: &mut SceneNavigation, mouse_delta: egui::Vec2)  {
        if !scene_nav.is_navigating {
            return;
        }
        
        // Simple, direct rotation calculation
        let pitch_delta = -mouse_delta.y * scene_nav.rotation_sensitivity;  // Negative: mouse down = look down
        let yaw_delta = -mouse_delta.x * scene_nav.rotation_sensitivity;    // Negative: mouse right = turn left
        
        // Update rotation velocity (for tests that check this)
        scene_nav.rotation_velocity[0] = pitch_delta / 0.016; // Convert to velocity (assuming 60 FPS)
        scene_nav.rotation_velocity[1] = yaw_delta / 0.016;
        
        // Update camera rotation directly
        scene_nav.scene_camera_transform.rotation[0] += pitch_delta;
        scene_nav.scene_camera_transform.rotation[1] += yaw_delta;
        
        // Clamp pitch to prevent camera flipping
        scene_nav.scene_camera_transform.rotation[0] = scene_nav.scene_camera_transform.rotation[0]
            .clamp(-1.5, 1.5); // ~85 degrees
        
        // Done
    }
    
    
    /// Handle WASD movement input
    pub fn handle_wasd_movement(scene_nav: &mut SceneNavigation, ui: &egui::Ui, delta_time: f32)  {
        if !scene_nav.is_navigating {
            return;
        }
        
        // Check if any movement keys are actually pressed
        let any_movement_key = ui.input(|i| {
            i.key_down(egui::Key::W) || i.key_down(egui::Key::A) || 
            i.key_down(egui::Key::S) || i.key_down(egui::Key::D) ||
            i.key_down(egui::Key::Q) || i.key_down(egui::Key::E)
        });
        
        if !any_movement_key {
            // No movement keys pressed, don't move
            return;
        }
        
        let mut movement = [0.0, 0.0, 0.0];
        let speed = if ui.input(|i| i.modifiers.shift) {
            scene_nav.movement_speed * scene_nav.fast_movement_multiplier
        } else {
            scene_nav.movement_speed
        };
        
        // WASD movement - Camera looks in +Z direction
        if ui.input(|i| i.key_down(egui::Key::W)) {
            movement[2] += 1.0;  // W moves forward (+Z in camera space)
        }
        if ui.input(|i| i.key_down(egui::Key::S)) {
            movement[2] -= 1.0;  // S moves backward (-Z in camera space)
        }
        if ui.input(|i| i.key_down(egui::Key::A)) {
            movement[0] -= 1.0;  // A moves left (-X in camera space)
        }
        if ui.input(|i| i.key_down(egui::Key::D)) {
            movement[0] += 1.0;  // D moves right (+X in camera space)
        }
        
        // Vertical movement (Q/E) - Fixed direction (standard Y-up coordinate system)
        if ui.input(|i| i.key_down(egui::Key::Q)) {
            movement[1] -= 1.0;  // Q moves DOWN (-Y in world space)
        }
        if ui.input(|i| i.key_down(egui::Key::E)) {
            movement[1] += 1.0;  // E moves UP (+Y in world space)
        }
        
        // Apply movement if any
        let movement_magnitude = ((movement[0] * movement[0] + movement[1] * movement[1] + movement[2] * movement[2]) as f32).sqrt();
        if movement_magnitude > 0.001 {
            // Normalize movement vector
            movement[0] /= movement_magnitude;
            movement[1] /= movement_magnitude;
            movement[2] /= movement_magnitude;
            
            // Scale by speed and delta time
            movement[0] *= speed * delta_time;
            movement[1] *= speed * delta_time;
            movement[2] *= speed * delta_time;
            
            // Transform movement by camera rotation
            let transformed_movement = super::camera_movement::transform_movement_by_camera(&scene_nav.scene_camera_transform, movement);
            
            // Apply movement to camera position
            scene_nav.scene_camera_transform.position[0] += transformed_movement[0];
            scene_nav.scene_camera_transform.position[1] += transformed_movement[1];
            scene_nav.scene_camera_transform.position[2] += transformed_movement[2];
        }
        
        // Done
    }
    
    /// Handle navigation speed control with scroll wheel
    pub fn handle_navigation_speed_control(scene_nav: &mut SceneNavigation, ui: &egui::Ui)  {
        if !scene_nav.is_navigating {
            return;
        }
        
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            // Adjust speed with mouse wheel
            let speed_multiplier = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
            scene_nav.movement_speed = (scene_nav.movement_speed * speed_multiplier)
                .clamp(0.5, 50.0);
        }
        
        // Done
    }
    
    /// Handle scene navigation (right mouse button + WASD)
    pub fn handle_scene_navigation(
        scene_navigation: &mut SceneNavigation,
        ui: &egui::Ui,
        response: &egui::Response,
        rect: egui::Rect,
    ) -> Vec<crate::editor_state::ConsoleMessage> {
        // Use the response object's drag detection for reliable input in docked panels
        let is_dragging = response.dragged_by(egui::PointerButton::Secondary);
        let drag_started = response.drag_started_by(egui::PointerButton::Secondary);
        let drag_stopped = response.drag_stopped_by(egui::PointerButton::Secondary);
        
        // Get pointer position from context
        let ctx = ui.ctx();
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        
        // Check if mouse is in rect
        let mouse_in_rect = pointer_pos.map_or(false, |pos| rect.contains(pos));
        
        // Start navigation on drag start
        if drag_started && mouse_in_rect && !scene_navigation.is_navigating {
            if let Some(mouse_pos) = pointer_pos {
                Self::start_navigation(scene_navigation, mouse_pos);
                
                // Hide cursor during navigation
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            }
        }
        
        // End navigation on drag stop
        if drag_stopped && scene_navigation.is_navigating {
            Self::end_navigation(scene_navigation);
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        
        // Handle navigation input during active navigation
        if scene_navigation.is_navigating && is_dragging {
            // Calculate mouse delta using multiple methods
            if let Some(current_pos) = pointer_pos {
                let effective_delta = if let Some(last_pos) = scene_navigation.last_mouse_pos {
                    // Always calculate manual delta
                    let manual_delta = egui::Vec2::new(
                        current_pos.x - last_pos.x,
                        current_pos.y - last_pos.y
                    );
                    
                    // Filter out tiny movements to reduce jitter
                    // Only process movement if it's above a small threshold
                    if manual_delta.length() > 0.2 {  // Small threshold to filter micro-movements
                        manual_delta
                    } else {
                        egui::Vec2::ZERO
                    }
                } else {
                    // First frame of navigation, no delta yet
                    egui::Vec2::ZERO
                };
                
                // Apply rotation if there's movement
                if effective_delta.length() > 0.0 {  // Process any non-zero delta since we already filtered
                    Self::apply_mouse_look(scene_navigation, effective_delta);
                }
                
                // Update last mouse position
                scene_navigation.last_mouse_pos = Some(current_pos);
            }
            
            // Handle WASD movement
            // Note: delta_time should be passed in from the parent
            let delta_time = ui.input(|i| i.stable_dt);
            Self::handle_wasd_movement(scene_navigation, ui, delta_time);
        }
        
        // Handle scroll wheel for speed adjustment (even when not actively navigating)
        Self::handle_navigation_speed_control(scene_navigation, ui);
        
        Vec::new() // No console messages from navigation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_components_3d::Transform;
    
    #[test]
    fn test_w_key_moves_camera_in_look_direction() {
        // Camera looks in +Z direction when rotation is [0,0,0]
        // W key should move the camera forward (in the direction it's looking)
        
        // Test 1: Camera with no rotation (looking in +Z)
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0], 
            scale: [1.0, 1.0, 1.0],
        };
        
        // W key produces movement[2] = +1.0 (forward in camera space)
        let movement = [0.0, 0.0, 1.0];
        let transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, movement);
        
        // Camera should move forward in world space (+Z since that's where it's looking)
        assert!(transformed[2] > 0.0, "W key with no rotation should move +Z");
        
        // Test 2: Camera rotated 180 degrees (looking in -Z)
        let camera_transform_180 = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, std::f32::consts::PI, 0.0], // 180 degree yaw
            scale: [1.0, 1.0, 1.0],
        };
        
        let transformed_180 = SceneNavigator::transform_movement_by_camera(&camera_transform_180, movement);
        
        // When rotated 180, W should move us in -Z direction
        assert!(transformed_180[2] < 0.0, "W key when rotated 180 should move -Z");
    }
    
    #[test]
    fn test_forward_movement_with_yaw_rotation() {
        // Given a camera rotated 90 degrees to the right (looking along +X)
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, std::f32::consts::PI / 2.0, 0.0], // 90 degrees yaw
            scale: [1.0, 1.0, 1.0],
        };
        
        // When moving forward (W key produces +1.0)
        let movement = [0.0, 0.0, 1.0];
        let transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, movement);
        
        // Then camera should move along +X axis (where it's looking)
        assert!(transformed[0] > 0.9, "Forward movement should be along +X when rotated right");
        assert!(transformed[2].abs() < 0.1, "Minimal Z movement expected");
        assert_eq!(transformed[1], 0.0, "No Y movement expected");
    }
    
    #[test]
    fn test_forward_movement_with_pitch_rotation() {
        // Given a camera looking up 45 degrees
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [std::f32::consts::PI / 4.0, 0.0, 0.0], // 45 degrees pitch up
            scale: [1.0, 1.0, 1.0],
        };
        
        // When moving forward (W key produces +1.0)
        let movement = [0.0, 0.0, 1.0];
        let transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, movement);
        
        // Then camera should move forward (negative Z) and up
        assert!(transformed[2] < 0.0, "Should still move forward (negative Z)");
        assert!(transformed[1] > 0.0, "Should also move up when pitched up");
        assert!(transformed[0].abs() < 0.001, "No X movement expected");
    }
    
    #[test]
    fn test_backward_movement_is_opposite_of_forward() {
        // Given a camera with some rotation
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.2, 0.5, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        // When moving forward and backward (correct directions)
        let forward_movement = [0.0, 0.0, 1.0];   // W key
        let backward_movement = [0.0, 0.0, -1.0]; // S key
        
        let forward_transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, forward_movement);
        let backward_transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, backward_movement);
        
        // Then backward should be opposite of forward
        assert!((forward_transformed[0] + backward_transformed[0]).abs() < 0.001);
        assert!((forward_transformed[1] + backward_transformed[1]).abs() < 0.001);
        assert!((forward_transformed[2] + backward_transformed[2]).abs() < 0.001);
    }
    
    #[test]
    fn test_strafe_movement_perpendicular_to_forward() {
        // Given a camera looking forward
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        // When strafing right
        let movement = [1.0, 0.0, 0.0];
        let transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, movement);
        
        // Then should move along +X axis
        assert!(transformed[0] > 0.9, "Right strafe should be +X");
        assert_eq!(transformed[1], 0.0, "No Y movement expected");
        assert_eq!(transformed[2], 0.0, "No Z movement expected");
    }
}