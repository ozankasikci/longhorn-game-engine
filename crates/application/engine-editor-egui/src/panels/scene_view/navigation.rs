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
            messages.push(ConsoleMessage::info("✔️ Scene navigation ended"));
        }
        
        messages
    }
    
    /// Apply mouse look rotation to the camera with smooth interpolation
    pub fn apply_mouse_look(scene_nav: &mut SceneNavigation, mouse_delta: egui::Vec2) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating {
            return messages;
        }
        
        // Calculate raw rotation input
        let raw_pitch_delta = -mouse_delta.y * scene_nav.rotation_sensitivity;
        let raw_yaw_delta = -mouse_delta.x * scene_nav.rotation_sensitivity;
        
        // Add to smoothing samples for jitter reduction
        scene_nav.rotation_smoothing_samples.push([raw_pitch_delta, raw_yaw_delta]);
        
        // Keep only recent samples (last 5 frames)
        if scene_nav.rotation_smoothing_samples.len() > 5 {
            scene_nav.rotation_smoothing_samples.remove(0);
        }
        
        // Calculate smoothed input using exponential moving average
        let smoothed_input = Self::calculate_smoothed_rotation_input(&scene_nav.rotation_smoothing_samples);
        
        // Apply adaptive sensitivity based on movement speed
        let adaptive_sensitivity = Self::calculate_adaptive_sensitivity(mouse_delta.length(), scene_nav.rotation_sensitivity);
        
        // Calculate target velocity from input
        let target_velocity = [
            smoothed_input[0] * adaptive_sensitivity * 60.0, // Convert to rad/s (assuming 60fps)
            smoothed_input[1] * adaptive_sensitivity * 60.0,
        ];
        
        // Update rotation velocity with acceleration towards target
        let delta_time = 1.0 / 60.0; // Assume 60fps for consistent behavior
        scene_nav.rotation_velocity[0] = Self::approach_velocity(
            scene_nav.rotation_velocity[0],
            target_velocity[0],
            scene_nav.rotation_acceleration,
            scene_nav.max_rotation_speed,
            delta_time
        );
        scene_nav.rotation_velocity[1] = Self::approach_velocity(
            scene_nav.rotation_velocity[1], 
            target_velocity[1],
            scene_nav.rotation_acceleration,
            scene_nav.max_rotation_speed,
            delta_time
        );
        
        // Calculate rotation delta from velocity
        let velocity_pitch_delta = scene_nav.rotation_velocity[0] * delta_time;
        let velocity_yaw_delta = scene_nav.rotation_velocity[1] * delta_time;
        
        // For large rotations, add them to target delta for interpolation
        scene_nav.target_rotation_delta[0] += velocity_pitch_delta;
        scene_nav.target_rotation_delta[1] += velocity_yaw_delta;
        
        // Use final deltas for immediate application
        let final_pitch_delta = velocity_pitch_delta;
        let final_yaw_delta = velocity_yaw_delta;
        
        // Apply rotation with interpolation for large movements
        let max_single_frame_rotation = 0.1; // Radians per frame
        
        let pitch_delta = if final_pitch_delta.abs() > max_single_frame_rotation {
            final_pitch_delta.signum() * max_single_frame_rotation
        } else {
            final_pitch_delta
        };
        
        let yaw_delta = if final_yaw_delta.abs() > max_single_frame_rotation {
            final_yaw_delta.signum() * max_single_frame_rotation
        } else {
            final_yaw_delta
        };
        
        // Apply rotation to camera
        scene_nav.scene_camera_transform.rotation[0] += pitch_delta;
        scene_nav.scene_camera_transform.rotation[1] += yaw_delta;
        
        // DEBUG: Log rotation changes - ALWAYS log to debug rotation issue
        messages.push(ConsoleMessage::info(&format!(
            "🔄 Camera rotation: pitch_delta={:.4}, yaw_delta={:.4}, new_rot=[{:.2}, {:.2}]°, mouse_delta=[{:.1}, {:.1}]",
            pitch_delta.to_degrees(), yaw_delta.to_degrees(),
            scene_nav.scene_camera_transform.rotation[0].to_degrees(),
            scene_nav.scene_camera_transform.rotation[1].to_degrees(),
            mouse_delta.x, mouse_delta.y
        )));
        
        // Clamp pitch to prevent camera flipping
        scene_nav.scene_camera_transform.rotation[0] = scene_nav.scene_camera_transform.rotation[0]
            .clamp(-1.5, 1.5); // ~85 degrees
        
        // Update target rotation delta (subtract what we applied)
        scene_nav.target_rotation_delta[0] -= pitch_delta;
        scene_nav.target_rotation_delta[1] -= yaw_delta;
        
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
        
        // WASD movement
        if ui.input(|i| i.key_down(egui::Key::W)) {
            movement[2] -= 1.0;
        }
        if ui.input(|i| i.key_down(egui::Key::S)) {
            movement[2] += 1.0;
        }
        if ui.input(|i| i.key_down(egui::Key::A)) {
            movement[0] -= 1.0;
        }
        if ui.input(|i| i.key_down(egui::Key::D)) {
            movement[0] += 1.0;
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
        let yaw = camera_transform.rotation[1];
        let pitch = camera_transform.rotation[0];
        
        // Apply yaw rotation
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        
        // Apply pitch for forward/backward movement
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        
        // Transform the movement vector
        let mut transformed = [0.0; 3];
        
        // Right/left movement (strafing) - only affected by yaw
        transformed[0] = movement[0] * cos_yaw - movement[2] * sin_yaw;
        transformed[2] = movement[0] * sin_yaw + movement[2] * cos_yaw;
        
        // Up/down movement - not affected by rotation
        transformed[1] = movement[1];
        
        // Apply pitch to forward/backward movement
        if movement[2].abs() > 0.001 {
            let forward_horizontal = transformed[2] * cos_pitch;
            let forward_vertical = -transformed[2] * sin_pitch;
            
            transformed[2] = forward_horizontal;
            transformed[1] += forward_vertical;
        }
        
        transformed
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
        
        // Check for secondary mouse button click on the response
        if response.secondary_clicked() {
            console_messages.push(ConsoleMessage::info("🖱️ Response detected secondary click!"));
        }
        
        // Check for right mouse button to start navigation - use ctx directly for dock compatibility
        let ctx = ui.ctx();
        let (rmb_down, rmb_pressed, pointer_pos, pointer_delta) = ctx.input(|i| {
            (i.pointer.secondary_down(), 
             i.pointer.secondary_pressed(),
             i.pointer.hover_pos(),
             i.pointer.delta())
        });
        
        // Alternative: Check response for drag with secondary button
        let is_dragging_secondary = response.dragged_by(egui::PointerButton::Secondary);
        
        // Only handle navigation if mouse is within the scene view
        let mouse_in_rect = pointer_pos.map_or(false, |pos| rect.contains(pos));
        
        // DEBUG: Log mouse rect check
        if rmb_pressed || response.secondary_clicked() {
            console_messages.push(ConsoleMessage::info(&format!(
                "🖱️ RMB pressed - mouse_in_rect: {}, pointer_pos: {:?}, rect: {:?}, is_navigating: {}, rmb_pressed: {}",
                mouse_in_rect, pointer_pos, rect, scene_navigation.is_navigating, rmb_pressed
            )));
        }
        
        // Start navigation if RMB is pressed (not just down) and mouse is in rect
        if (rmb_pressed || response.drag_started_by(egui::PointerButton::Secondary)) 
            && mouse_in_rect && !scene_navigation.is_navigating {
            if let Some(mouse_pos) = pointer_pos {
                console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 Starting navigation at ({:.1}, {:.1})", mouse_pos.x, mouse_pos.y
                )));
                let messages = Self::start_navigation(scene_navigation, mouse_pos);
                console_messages.extend(messages);
                
                // Capture mouse to prevent interference from other UI elements
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            }
        }
        
        // Check for right mouse button release or drag end
        if (!rmb_down && scene_navigation.is_navigating) || 
           (scene_navigation.is_navigating && response.drag_stopped_by(egui::PointerButton::Secondary)) {
            let messages = Self::end_navigation(scene_navigation);
            console_messages.extend(messages);
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        
        // Handle navigation input during active navigation
        if scene_navigation.is_navigating && (rmb_down || is_dragging_secondary) {
            // Use pointer delta for more accurate mouse movement tracking
            if pointer_delta != egui::Vec2::ZERO {
                console_messages.push(ConsoleMessage::info(&format!(
                    "🖱️ Mouse delta: ({:.2}, {:.2}), dragging_secondary: {}",
                    pointer_delta.x, pointer_delta.y, is_dragging_secondary
                )));
                let messages = Self::apply_mouse_look(scene_navigation, pointer_delta);
                console_messages.extend(messages);
            }
            
            // Handle WASD movement
            // Note: delta_time should be passed in from the parent
            let delta_time = ui.input(|i| i.stable_dt);
            let messages = Self::handle_wasd_movement(scene_navigation, ui, delta_time);
            console_messages.extend(messages);
        }
        
        // Always update smooth rotation (for interpolation and damping)
        let delta_time = ui.input(|i| i.stable_dt);
        let messages = Self::update_smooth_rotation(scene_navigation, delta_time);
        console_messages.extend(messages);
        
        // Handle scroll wheel for speed adjustment (even when not actively navigating)
        let messages = Self::handle_navigation_speed_control(scene_navigation, ui);
        console_messages.extend(messages);
        
        console_messages
    }
}