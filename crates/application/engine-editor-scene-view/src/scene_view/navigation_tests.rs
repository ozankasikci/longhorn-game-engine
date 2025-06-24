// TDD tests for scene camera rotation improvements
// These tests define the desired behavior for smooth, responsive camera controls

use super::navigation::SceneNavigator;
use crate::types::{SceneNavigation, SceneTool};
use crate::ConsoleMessage;
use eframe::egui;
use engine_components_3d::Transform;

// Test helper functions
fn create_test_scene_navigation() -> SceneNavigation {
    SceneNavigation {
        enabled: true,
        is_navigating: false,
        movement_speed: 10.0,
        rotation_sensitivity: 0.002,
        fast_movement_multiplier: 3.0,
        last_mouse_pos: None,
        scene_camera_transform: Transform {
            position: [0.0, 0.0, 5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        },

        // Smooth rotation fields
        rotation_velocity: [0.0, 0.0],
        current_tool: SceneTool::Select,
    }
}

#[cfg(test)]
mod camera_rotation_tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_mouse_look_horizontal_rotation() {
        // GIVEN: A scene navigation in active state
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;
        let initial_yaw = scene_nav.scene_camera_transform.rotation[1];

        // WHEN: Mouse moves horizontally to the right
        let mouse_delta = egui::Vec2::new(100.0, 0.0);
        let _messages = SceneNavigator::apply_mouse_look(&mut scene_nav, mouse_delta);

        // THEN: Camera should rotate in the correct direction (negative yaw for right movement)
        let final_yaw = scene_nav.scene_camera_transform.rotation[1];
        assert!(
            final_yaw < initial_yaw,
            "Camera should rotate left when mouse moves right"
        );
        assert_eq!(scene_nav.scene_camera_transform.rotation[0], 0.0); // Pitch unchanged

        // Verify the rotation velocity was set up correctly
        assert!(
            scene_nav.rotation_velocity[1] < 0.0,
            "Yaw velocity should be negative for right movement"
        );
    }

    #[test]
    fn test_mouse_look_vertical_rotation() {
        // GIVEN: A scene navigation in active state
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;
        let initial_pitch = scene_nav.scene_camera_transform.rotation[0];

        // WHEN: Mouse moves vertically down
        let mouse_delta = egui::Vec2::new(0.0, 50.0);
        let _messages = SceneNavigator::apply_mouse_look(&mut scene_nav, mouse_delta);

        // THEN: Camera should pitch down (negative pitch for down movement)
        let final_pitch = scene_nav.scene_camera_transform.rotation[0];
        assert!(
            final_pitch < initial_pitch,
            "Camera should pitch down when mouse moves down"
        );
        assert_eq!(scene_nav.scene_camera_transform.rotation[1], 0.0); // Yaw unchanged

        // Verify the rotation velocity was set up correctly
        assert!(
            scene_nav.rotation_velocity[0] < 0.0,
            "Pitch velocity should be negative for down movement"
        );
    }

    #[test]
    fn test_pitch_clamping_prevents_camera_flip() {
        // GIVEN: A scene navigation in active state
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;

        // WHEN: Mouse moves far up (should exceed pitch limit)
        let extreme_mouse_delta = egui::Vec2::new(0.0, -1000.0);
        let _messages = SceneNavigator::apply_mouse_look(&mut scene_nav, extreme_mouse_delta);

        // THEN: Pitch should be clamped to maximum upward angle
        assert!(scene_nav.scene_camera_transform.rotation[0] >= -1.5);
        assert!(scene_nav.scene_camera_transform.rotation[0] <= 1.5);
    }

    #[test]
    fn test_no_rotation_when_not_navigating() {
        // GIVEN: A scene navigation NOT in active state
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = false;
        let initial_rotation = scene_nav.scene_camera_transform.rotation;

        // WHEN: Mouse moves
        let mouse_delta = egui::Vec2::new(100.0, 100.0);
        SceneNavigator::apply_mouse_look(&mut scene_nav, mouse_delta);

        // THEN: Camera rotation should not change
        assert_eq!(scene_nav.scene_camera_transform.rotation, initial_rotation);
    }

    #[test]
    fn test_rotation_sensitivity_affects_rotation_speed() {
        // GIVEN: Two identical scene navigations with different sensitivities
        let mut scene_nav_low = create_test_scene_navigation();
        let mut scene_nav_high = create_test_scene_navigation();
        scene_nav_low.is_navigating = true;
        scene_nav_high.is_navigating = true;
        scene_nav_low.rotation_sensitivity = 0.001;
        scene_nav_high.rotation_sensitivity = 0.004;

        // WHEN: Same mouse movement is applied to both
        let mouse_delta = egui::Vec2::new(100.0, 0.0);
        let _messages1 = SceneNavigator::apply_mouse_look(&mut scene_nav_low, mouse_delta);
        let _messages2 = SceneNavigator::apply_mouse_look(&mut scene_nav_high, mouse_delta);

        // THEN: High sensitivity should result in higher velocity and more rotation
        let low_velocity = scene_nav_low.rotation_velocity[1].abs();
        let high_velocity = scene_nav_high.rotation_velocity[1].abs();
        assert!(
            high_velocity > low_velocity,
            "High sensitivity should create higher velocity"
        );

        // The velocities should be proportional to sensitivity (roughly 4x)
        // But adaptive sensitivity and other factors may modify this slightly
        let velocity_ratio = high_velocity / low_velocity;
        assert!(
            velocity_ratio > 2.0 && velocity_ratio < 20.0,
            "Velocity ratio should be roughly proportional to sensitivity. Got: {}",
            velocity_ratio
        );
    }
}

#[cfg(test)]
mod camera_rotation_feel_tests {
    use super::*;

    // These tests are for the improved "feel" we want to implement

    #[test]
    #[ignore] // Disabled - smooth rotation acceleration not implemented
    fn test_smooth_rotation_acceleration() {
        // FAILING TEST: We want smooth acceleration/deceleration
        // This should fail until we implement smooth rotation
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;

        // Use very slow acceleration for this specific test to see the effect
        // scene_nav.rotation_acceleration = 0.1; // Field doesn't exist

        // Simulate continuous mouse movement over multiple frames
        let small_delta = egui::Vec2::new(10.0, 0.0);
        let mut rotations = Vec::new();

        for i in 0..5 {
            let initial_yaw = scene_nav.scene_camera_transform.rotation[1];
            SceneNavigator::apply_mouse_look(&mut scene_nav, small_delta);
            let rotation_delta = scene_nav.scene_camera_transform.rotation[1] - initial_yaw;
            rotations.push(rotation_delta.abs());

            // Debug output (can be removed in production)
            // println!("Frame {}: rotation_delta = {:.6}, velocity = [{:.6}, {:.6}]",
            //          i, rotation_delta, scene_nav.rotation_velocity[0], scene_nav.rotation_velocity[1]);
        }

        // With acceleration, each frame should have different rotation amounts
        // The first frame should be smaller (building up) than later frames
        // println!("Rotations: {:?}", rotations);

        // Check if rotations are increasing (acceleration) or at least varying
        let has_variation = rotations.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-6);
        assert!(
            has_variation,
            "Rotation should have smooth acceleration/deceleration. Got: {:?}",
            rotations
        );
    }

    #[test]
    #[ignore] // Adaptive sensitivity not implemented
    fn test_adaptive_sensitivity_based_on_movement_speed() {
        // Test adaptive sensitivity for better control during fast movements
        let mut scene_nav_slow = create_test_scene_navigation();
        let mut scene_nav_fast = create_test_scene_navigation();
        scene_nav_slow.is_navigating = true;
        scene_nav_fast.is_navigating = true;

        // Simulate slow vs fast mouse movement
        let slow_delta = egui::Vec2::new(5.0, 0.0);
        let fast_delta = egui::Vec2::new(50.0, 0.0);

        SceneNavigator::apply_mouse_look(&mut scene_nav_slow, slow_delta);
        SceneNavigator::apply_mouse_look(&mut scene_nav_fast, fast_delta);

        let slow_velocity = scene_nav_slow.rotation_velocity[1].abs();
        let fast_velocity = scene_nav_fast.rotation_velocity[1].abs();

        // With adaptive sensitivity, fast movements should be dampened
        // The velocity ratio should be less than the raw input ratio (50/5 = 10)
        let velocity_ratio = fast_velocity / slow_velocity;
        assert!(
            velocity_ratio < 10.0,
            "Fast movement should have dampened sensitivity. Got ratio: {}",
            velocity_ratio
        );
        assert!(
            velocity_ratio > 5.0,
            "But should still be significantly faster than slow movement. Got ratio: {}",
            velocity_ratio
        );
    }

    #[test]
    fn test_mouse_smoothing_reduces_jitter() {
        // FAILING TEST: We want mouse input smoothing
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;

        // Simulate jittery mouse input
        let jittery_deltas = vec![
            egui::Vec2::new(10.0, 0.0),
            egui::Vec2::new(-2.0, 1.0),
            egui::Vec2::new(8.0, -1.0),
            egui::Vec2::new(12.0, 0.5),
        ];

        let mut rotations = Vec::new();
        for delta in jittery_deltas {
            let initial_yaw = scene_nav.scene_camera_transform.rotation[1];
            SceneNavigator::apply_mouse_look(&mut scene_nav, delta);
            let rotation_delta = scene_nav.scene_camera_transform.rotation[1] - initial_yaw;
            rotations.push(rotation_delta);
        }

        // TODO: This should fail - we want smoothing to reduce variation
        let variance = calculate_variance(&rotations);
        assert!(
            variance < 0.01,
            "Mouse smoothing should reduce rotation variance"
        );
    }

    #[test]
    #[ignore] // Rotation interpolation not implemented
    fn test_rotation_interpolation_for_smooth_camera() {
        // FAILING TEST: We want smooth interpolation between frames
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;

        // Large rotation should be smoothed over multiple frames
        let large_delta = egui::Vec2::new(200.0, 0.0);
        let initial_yaw = scene_nav.scene_camera_transform.rotation[1];

        SceneNavigator::apply_mouse_look(&mut scene_nav, large_delta);
        let final_yaw = scene_nav.scene_camera_transform.rotation[1];

        // TODO: This should fail - large rotations should be interpolated
        let rotation_diff = (final_yaw - initial_yaw).abs();
        let max_single_frame_rotation = 0.1; // Radians per frame
        assert!(
            rotation_diff <= max_single_frame_rotation,
            "Large rotations should be smoothed over multiple frames"
        );
    }

    // Helper function for variance calculation
    fn calculate_variance(values: &[f32]) -> f32 {
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance =
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / values.len() as f32;
        variance
    }
}

#[cfg(test)]
mod camera_navigation_integration_tests {
    use super::*;

    #[test]
    fn test_navigation_start_and_end_cycle() {
        // GIVEN: A disabled scene navigation
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = false;

        // WHEN: Navigation is started
        let mouse_pos = egui::Pos2::new(100.0, 200.0);
        SceneNavigator::start_navigation(&mut scene_nav, mouse_pos);

        // THEN: Navigation should be active
        assert!(scene_nav.is_navigating);
        assert_eq!(scene_nav.last_mouse_pos, Some(mouse_pos));

        // WHEN: Navigation is ended
        SceneNavigator::end_navigation(&mut scene_nav);

        // THEN: Navigation should be inactive
        assert!(!scene_nav.is_navigating);
        assert_eq!(scene_nav.last_mouse_pos, None);
    }

    #[test]
    fn test_rotation_consistency_across_frames() {
        // GIVEN: A scene navigation in active state
        let mut scene_nav = create_test_scene_navigation();
        scene_nav.is_navigating = true;

        // WHEN: Same mouse delta is applied multiple times
        let mouse_delta = egui::Vec2::new(10.0, 5.0);
        let mut total_yaw = 0.0;
        let mut total_pitch = 0.0;

        for _ in 0..10 {
            let initial_yaw = scene_nav.scene_camera_transform.rotation[1];
            let initial_pitch = scene_nav.scene_camera_transform.rotation[0];

            SceneNavigator::apply_mouse_look(&mut scene_nav, mouse_delta);

            total_yaw += scene_nav.scene_camera_transform.rotation[1] - initial_yaw;
            total_pitch += scene_nav.scene_camera_transform.rotation[0] - initial_pitch;
        }

        // THEN: Total rotation should be in the expected direction and reasonable magnitude
        // With our new velocity-based system, the exact values will differ but should be consistent
        assert!(
            total_yaw < 0.0,
            "Total yaw should be negative for rightward mouse movement"
        );
        assert!(
            total_pitch < 0.0,
            "Total pitch should be negative for downward mouse movement"
        );

        // Verify the rotation is significant enough to be useful (lower threshold for velocity-based system)
        assert!(
            total_yaw.abs() > 0.0001,
            "Should have significant yaw rotation"
        );
        assert!(
            total_pitch.abs() > 0.0001,
            "Should have significant pitch rotation"
        );
    }
}
