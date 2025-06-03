// Scene navigation and camera controls

use eframe::egui;
use crate::types::SceneNavigation;
use crate::editor_state::ConsoleMessage;
use engine_components_3d::Transform;

/// Scene navigation helper that manages camera movement in the editor
pub struct SceneNavigator;

impl SceneNavigator {
    /// Start scene navigation (right mouse button pressed)
    pub fn start_navigation(scene_nav: &mut SceneNavigation, mouse_pos: egui::Pos2) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.enabled {
            return messages;
        }
        
        scene_nav.is_navigating = true;
        scene_nav.last_mouse_pos = Some(mouse_pos);
        messages.push(ConsoleMessage::info("🔄 Scene navigation started (WASD + Mouse)"));
        
        messages
    }
    
    /// End scene navigation (right mouse button released)
    pub fn end_navigation(scene_nav: &mut SceneNavigation) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if scene_nav.is_navigating {
            scene_nav.is_navigating = false;
            scene_nav.last_mouse_pos = None;
            scene_nav.rotation_smoothing_samples.clear(); // Clear smoothing samples
            messages.push(ConsoleMessage::info("✔️ Scene navigation ended"));
        }
        
        messages
    }
    
    /// Apply mouse look rotation to the camera - Unity-style free look (FPS-style) with smoothing
    pub fn apply_mouse_look(scene_nav: &mut SceneNavigation, mouse_delta: egui::Vec2) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating {
            return messages;
        }
        
        // Calculate raw rotation deltas (FIXED: Only invert pitch)
        let raw_pitch_delta = -mouse_delta.y * scene_nav.rotation_sensitivity;  // Invert pitch: up = look up
        let raw_yaw_delta = mouse_delta.x * scene_nav.rotation_sensitivity;     // Normal yaw: right = turn right
        
        // Add to smoothing samples (keep last 5 samples for averaging)
        scene_nav.rotation_smoothing_samples.push([raw_pitch_delta, raw_yaw_delta]);
        if scene_nav.rotation_smoothing_samples.len() > 5 {
            scene_nav.rotation_smoothing_samples.remove(0);
        }
        
        // Apply simple exponential smoothing to reduce jitter
        // Use weighted average with more weight on recent samples
        let smoothing_factor: f32 = 0.85; // Higher = less smoothing, more responsive (0.85 for good balance)
        let (pitch_delta, yaw_delta) = if scene_nav.rotation_smoothing_samples.len() > 1 {
            let mut weighted_pitch = 0.0;
            let mut weighted_yaw = 0.0;
            let mut total_weight = 0.0;
            
            for (i, sample) in scene_nav.rotation_smoothing_samples.iter().enumerate() {
                let weight = smoothing_factor.powi(scene_nav.rotation_smoothing_samples.len() as i32 - i as i32 - 1);
                weighted_pitch += sample[0] * weight;
                weighted_yaw += sample[1] * weight;
                total_weight += weight;
            }
            
            (weighted_pitch / total_weight, weighted_yaw / total_weight)
        } else {
            (raw_pitch_delta, raw_yaw_delta)
        };
        
        // Update camera rotation with smoothed values
        scene_nav.scene_camera_transform.rotation[0] += pitch_delta;
        scene_nav.scene_camera_transform.rotation[1] += yaw_delta;
        
        // Clamp pitch to prevent camera flipping
        scene_nav.scene_camera_transform.rotation[0] = scene_nav.scene_camera_transform.rotation[0]
            .clamp(-1.5, 1.5); // ~85 degrees
        
        // Only log significant rotations
        if pitch_delta.abs() > 0.01 || yaw_delta.abs() > 0.01 {
            messages.push(ConsoleMessage::info(&format!(
                "🔄 Free look: pitch={:.1}°, yaw={:.1}°",
                scene_nav.scene_camera_transform.rotation[0].to_degrees(),
                scene_nav.scene_camera_transform.rotation[1].to_degrees()
            )));
        }
        
        messages
    }
    
    /// Calculate smoothed rotation input from recent samples
    fn calculate_smoothed_rotation_input(samples: &[[f32; 2]]) -> [f32; 2] {
        if samples.is_empty() {
            return [0.0, 0.0];
        }
        
        // Use exponential moving average for smoothing
        let mut smoothed = [0.0, 0.0];
        let mut weight_sum = 0.0;
        
        for (i, sample) in samples.iter().enumerate() {
            let weight = 0.5_f32.powi(samples.len() as i32 - i as i32 - 1);
            smoothed[0] += sample[0] * weight;
            smoothed[1] += sample[1] * weight;
            weight_sum += weight;
        }
        
        if weight_sum > 0.0 {
            smoothed[0] /= weight_sum;
            smoothed[1] /= weight_sum;
        }
        
        smoothed
    }
    
    /// Calculate adaptive sensitivity based on movement speed
    fn calculate_adaptive_sensitivity(mouse_speed: f32, base_sensitivity: f32) -> f32 {
        // Apply logarithmic scaling for fast movements to improve control
        let speed_factor = if mouse_speed > 20.0 {
            (20.0_f32.ln() / mouse_speed.ln()).max(0.5)
        } else {
            1.0
        };
        
        base_sensitivity * speed_factor
    }
    
    /// Approach target velocity with acceleration and max speed limiting
    fn approach_velocity(current: f32, target: f32, acceleration: f32, max_speed: f32, delta_time: f32) -> f32 {
        // Clamp target to max speed
        let clamped_target = target.clamp(-max_speed, max_speed);
        
        // Calculate the difference
        let diff = clamped_target - current;
        
        // Apply acceleration limiting - much more conservative
        let max_change = acceleration * delta_time;
        let change = if diff.abs() < max_change {
            diff
        } else {
            diff.signum() * max_change
        };
        
        current + change
    }
    
    /// Update smooth rotation with acceleration/deceleration
    pub fn update_smooth_rotation(scene_nav: &mut SceneNavigation, delta_time: f32) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating {
            // Apply damping when not navigating
            let damping_factor = (-scene_nav.rotation_damping * delta_time).exp();
            scene_nav.rotation_velocity[0] *= damping_factor;
            scene_nav.rotation_velocity[1] *= damping_factor;
            
            // Clear smoothing samples when not navigating
            scene_nav.rotation_smoothing_samples.clear();
            scene_nav.target_rotation_delta = [0.0, 0.0];
            
            return messages;
        }
        
        // Apply any remaining target rotation with interpolation
        if scene_nav.target_rotation_delta[0].abs() > 0.001 || scene_nav.target_rotation_delta[1].abs() > 0.001 {
            let interp_speed = 5.0; // How quickly to interpolate remaining rotation
            let interp_factor = (1.0 - (-interp_speed * delta_time).exp()).min(1.0);
            
            let pitch_step = scene_nav.target_rotation_delta[0] * interp_factor;
            let yaw_step = scene_nav.target_rotation_delta[1] * interp_factor;
            
            scene_nav.scene_camera_transform.rotation[0] += pitch_step;
            scene_nav.scene_camera_transform.rotation[1] += yaw_step;
            
            // Clamp pitch
            scene_nav.scene_camera_transform.rotation[0] = scene_nav.scene_camera_transform.rotation[0]
                .clamp(-1.5, 1.5);
            
            // Reduce target rotation
            scene_nav.target_rotation_delta[0] -= pitch_step;
            scene_nav.target_rotation_delta[1] -= yaw_step;
        }
        
        messages
    }
    
    /// Handle WASD movement input
    pub fn handle_wasd_movement(scene_nav: &mut SceneNavigation, ui: &egui::Ui, delta_time: f32) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating || !ui.ctx().wants_keyboard_input() {
            return messages;
        }
        
        let mut movement = [0.0, 0.0, 0.0];
        let speed = if ui.input(|i| i.modifiers.shift) {
            scene_nav.movement_speed * scene_nav.fast_movement_multiplier
        } else {
            scene_nav.movement_speed
        };
        
        // WASD movement - Fixed: Swap W/S to fix direction issue
        if ui.input(|i| i.key_down(egui::Key::W)) {
            movement[2] -= 1.0;  // W moves forward (swapped)
        }
        if ui.input(|i| i.key_down(egui::Key::S)) {
            movement[2] += 1.0;  // S moves backward (swapped)
        }
        if ui.input(|i| i.key_down(egui::Key::A)) {
            movement[0] += 1.0;  // Left is positive X in local space
        }
        if ui.input(|i| i.key_down(egui::Key::D)) {
            movement[0] -= 1.0;  // Right is negative X in local space
        }
        
        // Vertical movement (Q/E)
        if ui.input(|i| i.key_down(egui::Key::Q)) {
            movement[1] -= 1.0;
        }
        if ui.input(|i| i.key_down(egui::Key::E)) {
            movement[1] += 1.0;
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
            let transformed_movement = Self::transform_movement_by_camera(&scene_nav.scene_camera_transform, movement);
            
            // Apply movement to camera position
            scene_nav.scene_camera_transform.position[0] += transformed_movement[0];
            scene_nav.scene_camera_transform.position[1] += transformed_movement[1];
            scene_nav.scene_camera_transform.position[2] += transformed_movement[2];
            
            // Log movement for debugging
            messages.push(ConsoleMessage::info(&format!(
                "🎮 Moving: [{:.2}, {:.2}, {:.2}] @ speed {:.1}",
                transformed_movement[0], transformed_movement[1], transformed_movement[2], speed
            )));
        }
        
        messages
    }
    
    /// Transform movement vector by camera rotation
    fn transform_movement_by_camera(camera_transform: &Transform, movement: [f32; 3]) -> [f32; 3] {
        // Use same coordinate system as world_to_screen for consistency
        let yaw = -camera_transform.rotation[1];  // Negative for correct rotation direction
        let pitch = -camera_transform.rotation[0];
        
        // Calculate rotation values
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        
        // Calculate camera basis vectors for FPS-style movement
        // Forward vector (horizontal only - ignore pitch for W/S movement)
        // FIXED: Use negative to move toward what you're looking at
        let forward_x = -sin_yaw;
        let forward_y = 0.0;  // Keep movement horizontal
        let forward_z = -cos_yaw;
        
        // Right vector (perpendicular to forward, horizontal only)
        // Cross product of up × forward = (0,1,0) × (-sin(yaw), 0, -cos(yaw)) = (cos(yaw), 0, -sin(yaw))
        let right_x = cos_yaw;
        let right_y = 0.0;
        let right_z = -sin_yaw;
        
        // Up vector is always world up
        let up_x = 0.0;
        let up_y = 1.0;
        let up_z = 0.0;
        
        // Transform movement from local camera space to world space
        let transformed_x = movement[0] * right_x + movement[1] * up_x + movement[2] * forward_x;
        let transformed_y = movement[0] * right_y + movement[1] * up_y + movement[2] * forward_y;
        let transformed_z = movement[0] * right_z + movement[1] * up_z + movement[2] * forward_z;
        
        [transformed_x, transformed_y, transformed_z]
    }
    
    /// Handle navigation speed control with scroll wheel
    pub fn handle_navigation_speed_control(scene_nav: &mut SceneNavigation, ui: &egui::Ui) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating {
            return messages;
        }
        
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            // Adjust speed with mouse wheel
            let speed_multiplier = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
            scene_nav.movement_speed = (scene_nav.movement_speed * speed_multiplier)
                .clamp(0.5, 50.0);
            
            messages.push(ConsoleMessage::info(&format!(
                "🌍 Navigation speed: {:.1} units/sec",
                scene_nav.movement_speed
            )));
        }
        
        messages
    }
    
    /// Handle scene navigation (right mouse button + WASD)
    pub fn handle_scene_navigation(
        scene_navigation: &mut SceneNavigation,
        ui: &egui::Ui,
        response: &egui::Response,
        rect: egui::Rect,
    ) -> Vec<ConsoleMessage> {
        let mut console_messages = Vec::new();
        
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
                console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 Starting navigation at ({:.1}, {:.1})", mouse_pos.x, mouse_pos.y
                )));
                let messages = Self::start_navigation(scene_navigation, mouse_pos);
                console_messages.extend(messages);
                
                // Hide cursor during navigation
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            }
        }
        
        // End navigation on drag stop
        if drag_stopped && scene_navigation.is_navigating {
            let messages = Self::end_navigation(scene_navigation);
            console_messages.extend(messages);
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
                    let messages = Self::apply_mouse_look(scene_navigation, effective_delta);
                    console_messages.extend(messages);
                }
                
                // Update last mouse position
                scene_navigation.last_mouse_pos = Some(current_pos);
            }
            
            // Handle WASD movement
            // Note: delta_time should be passed in from the parent
            let delta_time = ui.input(|i| i.stable_dt);
            let messages = Self::handle_wasd_movement(scene_navigation, ui, delta_time);
            console_messages.extend(messages);
        }
        
        // Smooth rotation disabled for Unity-style direct response
        // let delta_time = ui.input(|i| i.stable_dt);
        // let messages = Self::update_smooth_rotation(scene_navigation, delta_time);
        // console_messages.extend(messages);
        
        // Handle scroll wheel for speed adjustment (even when not actively navigating)
        let messages = Self::handle_navigation_speed_control(scene_navigation, ui);
        console_messages.extend(messages);
        
        console_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_components_3d::Transform;
    
    #[test]
    fn test_w_key_moves_camera_in_look_direction() {
        // In a typical FPS/Unity setup:
        // - Camera at origin looks down -Z (forward is -Z in world space)
        // - W key should move the camera forward (in the direction it's looking)
        
        // Test 1: Camera with no rotation (looking down -Z)
        let camera_transform = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0], 
            scale: [1.0, 1.0, 1.0],
        };
        
        // W key now produces movement[2] = +1.0 (correct forward movement)
        let movement = [0.0, 0.0, 1.0];
        let transformed = SceneNavigator::transform_movement_by_camera(&camera_transform, movement);
        
        // Camera should move forward in world space (negative Z for forward)
        assert!(transformed[2] < 0.0, "W key with no rotation should move -Z");
        
        // Test 2: Camera rotated 180 degrees (looking down +Z)
        let camera_transform_180 = Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, std::f32::consts::PI, 0.0], // 180 degree yaw
            scale: [1.0, 1.0, 1.0],
        };
        
        let transformed_180 = SceneNavigator::transform_movement_by_camera(&camera_transform_180, movement);
        
        // When rotated 180, W should move us backward in world space (+Z)
        assert!(transformed_180[2] > 0.0, "W key when rotated 180 should move +Z");
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
        
        // Then camera should move along -X axis (looking left when rotated right)
        assert!(transformed[0] < -0.9, "Forward movement should be along -X when rotated right");
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