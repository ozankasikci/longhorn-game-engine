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
    
    /// Apply mouse look rotation to the camera
    pub fn apply_mouse_look(scene_nav: &mut SceneNavigation, mouse_delta: egui::Vec2) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        if !scene_nav.is_navigating {
            return messages;
        }
        
        // Apply rotation with sensitivity
        scene_nav.scene_camera_transform.rotation[1] -= mouse_delta.x * scene_nav.rotation_sensitivity;
        scene_nav.scene_camera_transform.rotation[0] -= mouse_delta.y * scene_nav.rotation_sensitivity;
        
        // Clamp pitch to prevent camera flipping
        scene_nav.scene_camera_transform.rotation[0] = scene_nav.scene_camera_transform.rotation[0]
            .clamp(-1.5, 1.5); // ~85 degrees
        
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
}