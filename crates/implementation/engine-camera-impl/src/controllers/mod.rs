//! Camera controller implementations
//!
//! This module provides camera control schemes, starting with FPS controller.

use engine_components_3d::Transform;
use glam::{Vec3, Quat};

pub mod fps;

pub use fps::FPSCameraController;

/// Input state for camera controllers
#[derive(Debug, Clone, Default)]
pub struct CameraInput {
    /// Mouse delta movement
    pub mouse_delta: [f32; 2],
    
    /// Mouse scroll delta
    pub scroll_delta: f32,
    
    /// Movement input (x: right, y: up, z: forward)
    pub movement: [f32; 3],
    
    /// Whether the primary mouse button is pressed
    pub primary_mouse: bool,
    
    /// Whether the secondary mouse button is pressed
    pub secondary_mouse: bool,
    
    /// Whether the middle mouse button is pressed
    pub middle_mouse: bool,
    
    /// Modifier keys
    pub modifiers: ModifierKeys,
}

/// Modifier key state
#[derive(Debug, Clone, Default)]
pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Trait for camera controllers
pub trait CameraController: Send + Sync {
    /// Update the camera transform based on input
    fn update(
        &mut self,
        transform: &mut Transform,
        input: &CameraInput,
        delta_time: f32,
    );
    
    /// Get controller configuration for UI/debugging
    fn get_config(&self) -> ControllerConfig;
    
    /// Set controller configuration
    fn set_config(&mut self, config: ControllerConfig);
    
    /// Reset the controller to default state
    fn reset(&mut self);
    
    /// Get controller type name
    fn type_name(&self) -> &'static str;
}

/// Generic controller configuration
#[derive(Debug, Clone)]
pub struct ControllerConfig {
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    
    /// Movement speed in units per second
    pub movement_speed: f32,
    
    /// Whether input is enabled
    pub enabled: bool,
    
    /// Controller-specific settings as key-value pairs
    pub custom_settings: Vec<(String, f32)>,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            movement_speed: 5.0,
            enabled: true,
            custom_settings: Vec::new(),
        }
    }
}

/// Helper functions for camera math
pub mod helpers {
    use super::*;
    
    /// Convert Euler angles to quaternion using YXZ order (standard for cameras)
    pub fn euler_to_quaternion(euler: Vec3) -> Quat {
        Quat::from_euler(glam::EulerRot::YXZ, euler.y, euler.x, euler.z)
    }
    
    /// Convert quaternion to Euler angles
    pub fn quaternion_to_euler(q: Quat) -> Vec3 {
        let (y, x, z) = q.to_euler(glam::EulerRot::YXZ);
        Vec3::new(x, y, z)
    }
    
    /// Clamp angle to -PI..PI range
    pub fn wrap_angle(angle: f32) -> f32 {
        use std::f32::consts::PI;
        let mut a = angle % (2.0 * PI);
        if a > PI {
            a -= 2.0 * PI;
        } else if a < -PI {
            a += 2.0 * PI;
        }
        a
    }
    
    /// Calculate forward vector from rotation
    pub fn get_forward(rotation: Quat) -> Vec3 {
        rotation * -Vec3::Z
    }
    
    /// Calculate right vector from rotation
    pub fn get_right(rotation: Quat) -> Vec3 {
        rotation * Vec3::X
    }
    
    /// Calculate up vector from rotation
    pub fn get_up(rotation: Quat) -> Vec3 {
        rotation * Vec3::Y
    }
}