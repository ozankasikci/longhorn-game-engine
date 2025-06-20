// Tests for the gizmo system
#[cfg(test)]
mod tests {
    use super::super::gizmos::*;
    use crate::types::{GizmoComponent, GizmoAxis, GizmoPlane};
    use eframe::egui;
    
    #[test]
    fn test_gizmo_hit_detection_x_axis() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_move_gizmo([0.0, 0.0, 0.0]);
        
        let gizmo_center = egui::pos2(400.0, 300.0);
        let axis_length = 80.0;
        
        // Test hitting X axis (extends to the right)
        let x_axis_point = egui::pos2(440.0, 300.0); // Middle of X axis
        let hit = gizmo_system.test_gizmo_hit(x_axis_point, gizmo_center);
        assert_eq!(hit, Some(GizmoComponent::Axis(GizmoAxis::X)));
        
        // Test missing X axis
        let miss_point = egui::pos2(440.0, 320.0); // Too far from axis
        let hit = gizmo_system.test_gizmo_hit(miss_point, gizmo_center);
        assert_ne!(hit, Some(GizmoComponent::Axis(GizmoAxis::X)));
    }
    
    #[test]
    fn test_gizmo_hit_detection_y_axis() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_move_gizmo([0.0, 0.0, 0.0]);
        
        let gizmo_center = egui::pos2(400.0, 300.0);
        
        // Test hitting Y axis (extends upward - negative Y in screen coords)
        let y_axis_point = egui::pos2(400.0, 260.0); // Middle of Y axis
        let hit = gizmo_system.test_gizmo_hit(y_axis_point, gizmo_center);
        assert_eq!(hit, Some(GizmoComponent::Axis(GizmoAxis::Y)));
        
        // Test missing Y axis
        let miss_point = egui::pos2(420.0, 260.0); // Too far from axis
        let hit = gizmo_system.test_gizmo_hit(miss_point, gizmo_center);
        assert_ne!(hit, Some(GizmoComponent::Axis(GizmoAxis::Y)));
    }
    
    #[test]
    fn test_gizmo_hit_detection_z_axis() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_move_gizmo([0.0, 0.0, 0.0]);
        
        let gizmo_center = egui::pos2(400.0, 300.0);
        
        // Test hitting Z axis (diagonal: -0.7x, +0.7y in screen coords)
        // Z axis end point should be at approximately (344, 356)
        let z_axis_point = egui::pos2(372.0, 328.0); // Middle of Z axis
        let hit = gizmo_system.test_gizmo_hit(z_axis_point, gizmo_center);
        assert_eq!(hit, Some(GizmoComponent::Axis(GizmoAxis::Z)));
    }
    
    #[test]
    fn test_gizmo_hit_detection_center() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.enable_move_gizmo([0.0, 0.0, 0.0]);
        
        let gizmo_center = egui::pos2(400.0, 300.0);
        
        // Test hitting center sphere
        let hit = gizmo_system.test_gizmo_hit(gizmo_center, gizmo_center);
        assert_eq!(hit, Some(GizmoComponent::Center));
        
        // Test point very close to center
        let near_center = egui::pos2(405.0, 305.0);
        let hit = gizmo_system.test_gizmo_hit(near_center, gizmo_center);
        assert_eq!(hit, Some(GizmoComponent::Center));
    }
    
    #[test]
    fn test_gizmo_movement_x_axis() {
        let gizmo_system = GizmoSystem::new();
        let start_pos = [1.0, 2.0, 3.0];
        let mouse_delta = egui::vec2(50.0, 0.0); // Move 50 pixels right
        let scale = 50.0; // 50 pixels per world unit
        
        let new_pos = gizmo_system.calculate_new_position(
            start_pos,
            mouse_delta,
            GizmoComponent::Axis(GizmoAxis::X),
            scale
        );
        
        // X should increase by 1.0 (50 pixels / 50 scale)
        assert_eq!(new_pos[0], 2.0);
        assert_eq!(new_pos[1], 2.0); // Y unchanged
        assert_eq!(new_pos[2], 3.0); // Z unchanged
    }
    
    #[test]
    fn test_gizmo_movement_y_axis() {
        let gizmo_system = GizmoSystem::new();
        let start_pos = [1.0, 2.0, 3.0];
        let mouse_delta = egui::vec2(0.0, 50.0); // Move 50 pixels down
        let scale = 50.0;
        
        let new_pos = gizmo_system.calculate_new_position(
            start_pos,
            mouse_delta,
            GizmoComponent::Axis(GizmoAxis::Y),
            scale
        );
        
        // Y should decrease by 1.0 (inverted screen coords)
        assert_eq!(new_pos[0], 1.0); // X unchanged
        assert_eq!(new_pos[1], 1.0); // Y decreased
        assert_eq!(new_pos[2], 3.0); // Z unchanged
    }
    
    #[test]
    fn test_gizmo_movement_z_axis() {
        let gizmo_system = GizmoSystem::new();
        let start_pos = [1.0, 2.0, 3.0];
        // Move along the visual Z axis direction
        let mouse_delta = egui::vec2(-35.0, 35.0); // Diagonal movement
        let scale = 50.0;
        
        let new_pos = gizmo_system.calculate_new_position(
            start_pos,
            mouse_delta,
            GizmoComponent::Axis(GizmoAxis::Z),
            scale
        );
        
        assert_eq!(new_pos[0], 1.0); // X unchanged
        assert_eq!(new_pos[1], 2.0); // Y unchanged
        // Z should change based on the diagonal movement
        // (-35 * 0.7 + 35 * 0.7) / 50 = 0.98
        assert!((new_pos[2] - 3.98).abs() < 0.01);
    }
    
    #[test]
    fn test_gizmo_movement_center_screen_space() {
        let gizmo_system = GizmoSystem::new();
        let start_pos = [1.0, 2.0, 3.0];
        let mouse_delta = egui::vec2(50.0, 50.0);
        let scale = 50.0;
        
        let new_pos = gizmo_system.calculate_new_position(
            start_pos,
            mouse_delta,
            GizmoComponent::Center,
            scale
        );
        
        // Center movement affects X and Z (screen-space movement)
        assert_eq!(new_pos[0], 2.0); // X increased
        assert_eq!(new_pos[1], 2.0); // Y unchanged
        assert_eq!(new_pos[2], 2.0); // Z decreased (inverted Y)
    }
    
    #[test]
    fn test_gizmo_movement_with_snapping() {
        let mut gizmo_system = GizmoSystem::new();
        gizmo_system.toggle_snap(); // Enable snapping
        gizmo_system.set_snap_increment(0.5);
        
        let start_pos = [1.1, 2.2, 3.3];
        let mouse_delta = egui::vec2(37.0, 0.0); // Would move to 1.84 without snapping
        let scale = 50.0;
        
        let new_pos = gizmo_system.calculate_new_position(
            start_pos,
            mouse_delta,
            GizmoComponent::Axis(GizmoAxis::X),
            scale
        );
        
        // Should snap to nearest 0.5
        assert_eq!(new_pos[0], 2.0); // Snapped to 2.0
        assert_eq!(new_pos[1], 2.0); // Snapped from 2.2
        assert_eq!(new_pos[2], 3.5); // Snapped from 3.3
    }
    
    #[test]
    fn test_point_to_line_distance() {
        let gizmo_system = GizmoSystem::new();
        
        // Test perpendicular distance
        let point = egui::pos2(5.0, 5.0);
        let line_start = egui::pos2(0.0, 0.0);
        let line_end = egui::pos2(10.0, 0.0);
        
        let distance = gizmo_system.point_to_line_distance(point, line_start, line_end);
        assert_eq!(distance, 5.0);
        
        // Test point on line
        let point_on_line = egui::pos2(5.0, 0.0);
        let distance = gizmo_system.point_to_line_distance(point_on_line, line_start, line_end);
        assert_eq!(distance, 0.0);
        
        // Test point past line end
        let point_past_end = egui::pos2(15.0, 5.0);
        let distance = gizmo_system.point_to_line_distance(point_past_end, line_start, line_end);
        let expected = ((15.0_f32 - 10.0).powi(2) + 5.0_f32.powi(2)).sqrt();
        assert!((distance - expected).abs() < 0.001);
    }
}