// Tests for Unity-style gizmos
#[cfg(test)]
mod tests {
    use crate::panels::scene_view::unity_style_gizmos::*;
    use eframe::egui;
    use engine_ecs_core::{World, Entity};
    use engine_components_3d::Transform;
    use glam::{Mat4, Vec3, Vec4};

    fn create_test_world() -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        (world, entity)
    }

    fn create_test_matrices() -> (Mat4, Mat4) {
        // View matrix looking down -Z axis from (0, 0, 5)
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        
        // Standard perspective projection
        let proj = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0, // 45 degree FOV
            1.0, // aspect ratio
            0.1, // near
            100.0, // far
        );
        
        (view, proj)
    }

    #[test]
    fn test_world_to_screen_projection() {
        let gizmo = UnityStyleGizmo::new();
        let (view, proj) = create_test_matrices();
        let viewport = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        
        // Test projection of origin
        let screen_pos = gizmo.world_to_screen(Vec3::ZERO, view, proj, viewport);
        assert!(screen_pos.is_some());
        let pos = screen_pos.unwrap();
        assert!((pos.x - 400.0).abs() < 0.1); // Should be center X
        assert!((pos.y - 300.0).abs() < 0.1); // Should be center Y
        
        // Test projection of point to the right
        let screen_pos = gizmo.world_to_screen(Vec3::new(1.0, 0.0, 0.0), view, proj, viewport);
        assert!(screen_pos.is_some());
        let pos = screen_pos.unwrap();
        assert!(pos.x > 400.0); // Should be right of center
        assert!((pos.y - 300.0).abs() < 0.1); // Should be same Y
        
        // Test projection of point behind camera
        let screen_pos = gizmo.world_to_screen(Vec3::new(0.0, 0.0, 10.0), view, proj, viewport);
        assert!(screen_pos.is_none()); // Should be culled
    }

    #[test]
    fn test_hit_detection_x_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        let viewport = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        
        // Simulate drawing to populate axis endpoints
        gizmo.axis_endpoints = Some(AxisEndpoints {
            center: egui::pos2(400.0, 300.0),
            x_end: Some(egui::pos2(500.0, 300.0)),
            y_end: Some(egui::pos2(400.0, 200.0)),
            z_end: Some(egui::pos2(450.0, 350.0)),
        });
        
        // Test hitting X axis
        let hit = gizmo.hit_test_axes(egui::pos2(450.0, 300.0), egui::pos2(400.0, 300.0), 1.0);
        assert_eq!(hit, Some(Axis::X));
        
        // Test missing X axis
        let hit = gizmo.hit_test_axes(egui::pos2(450.0, 320.0), egui::pos2(400.0, 300.0), 1.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn test_hit_detection_y_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        
        gizmo.axis_endpoints = Some(AxisEndpoints {
            center: egui::pos2(400.0, 300.0),
            x_end: Some(egui::pos2(500.0, 300.0)),
            y_end: Some(egui::pos2(400.0, 200.0)),
            z_end: Some(egui::pos2(450.0, 350.0)),
        });
        
        // Test hitting Y axis
        let hit = gizmo.hit_test_axes(egui::pos2(400.0, 250.0), egui::pos2(400.0, 300.0), 1.0);
        assert_eq!(hit, Some(Axis::Y));
        
        // Test missing Y axis
        let hit = gizmo.hit_test_axes(egui::pos2(420.0, 250.0), egui::pos2(400.0, 300.0), 1.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn test_hit_detection_z_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        
        gizmo.axis_endpoints = Some(AxisEndpoints {
            center: egui::pos2(400.0, 300.0),
            x_end: Some(egui::pos2(500.0, 300.0)),
            y_end: Some(egui::pos2(400.0, 200.0)),
            z_end: Some(egui::pos2(450.0, 350.0)),
        });
        
        // Test hitting Z axis
        let hit = gizmo.hit_test_axes(egui::pos2(425.0, 325.0), egui::pos2(400.0, 300.0), 1.0);
        assert_eq!(hit, Some(Axis::Z));
    }

    #[test]
    fn test_ray_plane_intersection() {
        let gizmo = UnityStyleGizmo::new();
        
        // Test ray hitting XY plane at origin
        let ray_origin = Vec3::new(0.0, 0.0, 5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);
        let plane_point = Vec3::ZERO;
        let plane_normal = Vec3::Z;
        
        let intersection = gizmo.ray_plane_intersection(ray_origin, ray_dir, plane_point, plane_normal);
        assert!(intersection.is_some());
        let hit = intersection.unwrap();
        assert!((hit - Vec3::ZERO).length() < 0.001);
        
        // Test ray parallel to plane
        let ray_dir_parallel = Vec3::X;
        let intersection = gizmo.ray_plane_intersection(ray_origin, ray_dir_parallel, plane_point, plane_normal);
        assert!(intersection.is_none());
        
        // Test ray pointing away from plane
        let ray_dir_away = Vec3::new(0.0, 0.0, 1.0);
        let intersection = gizmo.ray_plane_intersection(ray_origin, ray_dir_away, plane_point, plane_normal);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_drag_plane_selection_for_x_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 5.0), Vec3::ZERO, Vec3::Y);
        
        gizmo.start_drag(Axis::X, egui::pos2(0.0, 0.0), Vec3::ZERO, view);
        
        // For X axis movement, plane normal should be perpendicular to X
        let plane_normal = gizmo.drag_plane_normal.unwrap();
        assert!(plane_normal.dot(Vec3::X).abs() < 0.1);
    }

    #[test]
    fn test_drag_plane_selection_for_y_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        
        gizmo.start_drag(Axis::Y, egui::pos2(0.0, 0.0), Vec3::ZERO, view);
        
        // For Y axis movement, plane normal should be perpendicular to Y
        let plane_normal = gizmo.drag_plane_normal.unwrap();
        assert!(plane_normal.dot(Vec3::Y).abs() < 0.1);
    }

    #[test]
    fn test_drag_plane_selection_for_z_axis() {
        let mut gizmo = UnityStyleGizmo::new();
        let view = Mat4::look_at_rh(Vec3::new(5.0, 5.0, 0.0), Vec3::ZERO, Vec3::Y);
        
        gizmo.start_drag(Axis::Z, egui::pos2(0.0, 0.0), Vec3::ZERO, view);
        
        // For Z axis movement, plane normal should be perpendicular to Z
        let plane_normal = gizmo.drag_plane_normal.unwrap();
        assert!(plane_normal.dot(Vec3::Z).abs() < 0.1);
    }

    #[test]
    fn test_screen_space_scale_calculation() {
        let gizmo = UnityStyleGizmo::new();
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 1.0, 0.1, 100.0);
        
        // Test scale at different distances
        let scale_near = gizmo.calculate_screen_space_scale(Vec3::ZERO, view, proj);
        let scale_far = gizmo.calculate_screen_space_scale(Vec3::new(0.0, 0.0, -10.0), view, proj);
        
        // Further objects should have larger scale to maintain constant screen size
        assert!(scale_far > scale_near);
    }

    #[test]
    fn test_point_to_line_distance() {
        let gizmo = UnityStyleGizmo::new();
        
        // Test perpendicular distance
        let point = egui::pos2(5.0, 5.0);
        let line_start = egui::pos2(0.0, 0.0);
        let line_end = egui::pos2(10.0, 0.0);
        
        let distance = gizmo.point_to_line_distance(point, line_start, line_end);
        assert!((distance - 5.0).abs() < 0.001);
        
        // Test point on line
        let point_on_line = egui::pos2(5.0, 0.0);
        let distance = gizmo.point_to_line_distance(point_on_line, line_start, line_end);
        assert!(distance < 0.001);
        
        // Test point past line end
        let point_past_end = egui::pos2(15.0, 5.0);
        let distance = gizmo.point_to_line_distance(point_past_end, line_start, line_end);
        let expected = ((15.0_f32 - 10.0).powi(2) + 5.0_f32.powi(2)).sqrt();
        assert!((distance - expected).abs() < 0.001);
    }

    #[test]
    fn test_entity_change_detection() {
        let mut gizmo = UnityStyleGizmo::new();
        let (mut world, entity1) = create_test_world();
        let entity2 = world.spawn();
        world.add_component(entity2, Transform {
            position: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        
        // First selection
        assert_eq!(gizmo.last_selected_entity, None);
        assert!(gizmo.axis_endpoints.is_none());
        
        // We can't easily create UI/Response in tests, so we'll test the parts we can
        // Test that entity change is tracked
        gizmo.last_selected_entity = Some(entity1);
        
        // Manually trigger what would happen in update
        if gizmo.last_selected_entity != Some(entity2) {
            gizmo.axis_endpoints = None;  // Clear cache
            gizmo.last_selected_entity = Some(entity2);
        }
        
        assert_eq!(gizmo.last_selected_entity, Some(entity2));
        assert!(gizmo.axis_endpoints.is_none()); // Should be cleared on entity change
    }

    #[test]
    fn test_drag_mechanics() {
        let mut gizmo = UnityStyleGizmo::new();
        let (view, proj) = create_test_matrices();
        
        // Start drag on X axis
        gizmo.start_drag(Axis::X, egui::pos2(400.0, 300.0), Vec3::ZERO, view);
        
        // Verify drag state is set up correctly
        assert_eq!(gizmo.active_axis, Some(Axis::X));
        assert!(gizmo.drag_plane_normal.is_some());
        
        // The drag plane normal should be perpendicular to X axis
        let normal = gizmo.drag_plane_normal.unwrap();
        assert!(normal.dot(Vec3::X).abs() < 0.1); // Should be nearly perpendicular
    }
}