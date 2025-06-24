//! First-person shooter (FPS) camera controller
//!
//! Provides standard FPS controls with mouse look and WASD movement.

use super::{helpers, CameraController, CameraInput, ControllerConfig};
use engine_components_3d::Transform;
use glam::Vec3;

/// FPS camera controller with mouse look and WASD movement
#[derive(Debug, Clone)]
pub struct FPSCameraController {
    /// Mouse look sensitivity
    pub mouse_sensitivity: f32,

    /// Movement speed in units per second
    pub movement_speed: f32,

    /// Sprint speed multiplier
    pub sprint_multiplier: f32,

    /// Minimum pitch angle in radians (looking down)
    pub min_pitch: f32,

    /// Maximum pitch angle in radians (looking up)
    pub max_pitch: f32,

    /// Current rotation as Euler angles (pitch, yaw, roll)
    euler_angles: Vec3,

    /// Smoothing factor for movement (0 = no smoothing, 1 = infinite smoothing)
    pub movement_smoothing: f32,

    /// Smoothing factor for rotation
    pub rotation_smoothing: f32,

    /// Velocity for smooth movement
    velocity: Vec3,
}

impl Default for FPSCameraController {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            movement_speed: 5.0,
            sprint_multiplier: 2.0,
            min_pitch: -1.5, // ~85 degrees down
            max_pitch: 1.5,  // ~85 degrees up
            euler_angles: Vec3::ZERO,
            movement_smoothing: 0.1,
            rotation_smoothing: 0.0, // Usually rotation should be instant
            velocity: Vec3::ZERO,
        }
    }
}

impl FPSCameraController {
    /// Create a new FPS camera controller
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial rotation from a transform
    pub fn sync_rotation(&mut self, transform: &Transform) {
        self.euler_angles = Vec3::from_array(transform.rotation);
    }
}

impl CameraController for FPSCameraController {
    fn update(&mut self, transform: &mut Transform, input: &CameraInput, delta_time: f32) {
        // Update rotation from mouse input
        if input.secondary_mouse {
            // Apply mouse delta to rotation
            self.euler_angles.x -= input.mouse_delta[1] * self.mouse_sensitivity;
            self.euler_angles.y -= input.mouse_delta[0] * self.mouse_sensitivity;

            // Clamp pitch
            self.euler_angles.x = self.euler_angles.x.clamp(self.min_pitch, self.max_pitch);

            // Wrap yaw to prevent numerical issues
            self.euler_angles.y = helpers::wrap_angle(self.euler_angles.y);
        }

        // Apply rotation to transform
        if self.rotation_smoothing > 0.0 {
            // Smooth rotation (rarely used for FPS)
            let current_rot = Vec3::from_array(transform.rotation);
            let smoothed_rot = current_rot.lerp(
                self.euler_angles,
                1.0 - self.rotation_smoothing.powf(delta_time),
            );
            transform.rotation = smoothed_rot.into();
        } else {
            // Instant rotation
            transform.rotation = self.euler_angles.into();
        }

        // Calculate movement
        let mut movement = Vec3::ZERO;

        // Get movement input
        movement.x = input.movement[0]; // Right/Left
        movement.y = input.movement[1]; // Up/Down
        movement.z = input.movement[2]; // Forward/Backward

        // Apply speed
        let speed = if input.modifiers.shift {
            self.movement_speed * self.sprint_multiplier
        } else {
            self.movement_speed
        };

        // Transform movement to world space
        if movement.length_squared() > 0.001 {
            movement = movement.normalize() * speed;

            // Get camera rotation as quaternion
            let rotation = helpers::euler_to_quaternion(self.euler_angles);

            // Transform movement by camera rotation
            let forward = helpers::get_forward(rotation);
            let right = helpers::get_right(rotation);
            let up = Vec3::Y; // World up for FPS camera

            let world_movement = right * movement.x + up * movement.y + forward * movement.z;

            // Apply movement with smoothing
            if self.movement_smoothing > 0.0 {
                let target_velocity = world_movement;
                self.velocity = self.velocity.lerp(
                    target_velocity,
                    1.0 - self.movement_smoothing.powf(delta_time),
                );

                let position = Vec3::from_array(transform.position);
                transform.position = (position + self.velocity * delta_time).into();
            } else {
                // Instant movement
                let position = Vec3::from_array(transform.position);
                transform.position = (position + world_movement * delta_time).into();
            }
        } else if self.movement_smoothing > 0.0 {
            // Decelerate when no input
            self.velocity = self
                .velocity
                .lerp(Vec3::ZERO, 1.0 - self.movement_smoothing.powf(delta_time));

            if self.velocity.length_squared() > 0.0001 {
                let position = Vec3::from_array(transform.position);
                transform.position = (position + self.velocity * delta_time).into();
            }
        }
    }

    fn get_config(&self) -> ControllerConfig {
        ControllerConfig {
            mouse_sensitivity: self.mouse_sensitivity,
            movement_speed: self.movement_speed,
            enabled: true,
            custom_settings: vec![
                ("sprint_multiplier".to_string(), self.sprint_multiplier),
                ("min_pitch".to_string(), self.min_pitch),
                ("max_pitch".to_string(), self.max_pitch),
                ("movement_smoothing".to_string(), self.movement_smoothing),
                ("rotation_smoothing".to_string(), self.rotation_smoothing),
            ],
        }
    }

    fn set_config(&mut self, config: ControllerConfig) {
        self.mouse_sensitivity = config.mouse_sensitivity;
        self.movement_speed = config.movement_speed;

        for (key, value) in config.custom_settings {
            match key.as_str() {
                "sprint_multiplier" => self.sprint_multiplier = value,
                "min_pitch" => self.min_pitch = value,
                "max_pitch" => self.max_pitch = value,
                "movement_smoothing" => self.movement_smoothing = value,
                "rotation_smoothing" => self.rotation_smoothing = value,
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.euler_angles = Vec3::ZERO;
        self.velocity = Vec3::ZERO;
    }

    fn type_name(&self) -> &'static str {
        "FPS Camera"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_controller_creation() {
        let controller = FPSCameraController::new();
        assert_eq!(controller.mouse_sensitivity, 0.002);
        assert_eq!(controller.movement_speed, 5.0);
        assert_eq!(controller.euler_angles, Vec3::ZERO);
    }

    #[test]
    fn test_pitch_clamping() {
        let mut controller = FPSCameraController::new();
        let mut transform = Transform::default();

        // Try to look too far up
        let input = CameraInput {
            mouse_delta: [0.0, -1000.0], // Large upward movement
            secondary_mouse: true,
            ..Default::default()
        };

        controller.update(&mut transform, &input, 1.0 / 60.0);

        // Check that pitch is clamped
        assert!(transform.rotation[0] <= controller.max_pitch);
    }

    #[test]
    fn test_movement_transform() {
        let mut controller = FPSCameraController::new();
        controller.movement_smoothing = 0.0; // Disable smoothing for test
        let mut transform = Transform::default();

        // Set camera to look along +X axis (90 degree yaw)
        controller.euler_angles.y = std::f32::consts::FRAC_PI_2;

        // Move forward
        let input = CameraInput {
            movement: [0.0, 0.0, 1.0], // Forward
            ..Default::default()
        };

        controller.update(&mut transform, &input, 1.0);

        // With yaw = PI/2, camera looks along -X (left), so forward is -X
        assert!(
            transform.position[0] < -4.9,
            "Expected X < -4.9, got {}",
            transform.position[0]
        );
        assert!(
            transform.position[2].abs() < 0.1,
            "Expected minimal Z movement, got {}",
            transform.position[2]
        );
    }
}
