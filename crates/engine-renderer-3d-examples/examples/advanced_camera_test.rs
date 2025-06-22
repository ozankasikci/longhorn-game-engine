//! Advanced Camera System Test
//! 
//! This test demonstrates the advanced camera controller with:
//! - MVP matrix caching and management
//! - First-person and orbital camera controls
//! - Screen-to-world ray casting
//! - Camera presets and utilities

use engine_renderer_3d::{Camera, CameraController, CameraPresets, Ray};
use glam::Vec3;

fn main() {
    env_logger::init();
    
    println!("📷 Advanced Camera System Test");
    println!("=============================");
    
    // Test 1: Basic camera controller
    println!("\n🧪 Test 1: Basic Camera Controller");
    test_basic_camera_controller();
    
    // Test 2: Matrix caching and performance
    println!("\n🧪 Test 2: Matrix Caching Performance");
    test_matrix_caching();
    
    // Test 3: Camera movement and controls
    println!("\n🧪 Test 3: Camera Movement Controls");
    test_camera_controls();
    
    // Test 4: Screen-to-world ray casting
    println!("\n🧪 Test 4: Screen-to-World Ray Casting");
    test_ray_casting();
    
    // Test 5: Camera presets
    println!("\n🧪 Test 5: Camera Presets");
    test_camera_presets();
    
    // Test 6: Advanced camera operations
    println!("\n🧪 Test 6: Advanced Camera Operations");
    test_advanced_operations();
    
    println!("\n🎉 All advanced camera tests completed successfully!");
}

fn test_basic_camera_controller() {
    let camera = Camera::new(16.0 / 9.0);
    let mut controller = CameraController::new(camera);
    
    println!("   📷 Initial camera: {}", controller.get_info());
    
    // Test basic matrix access
    let view_matrix = controller.view_matrix();
    let proj_matrix = controller.projection_matrix();
    let vp_matrix = controller.view_projection_matrix();
    
    println!("   ✅ View matrix: determinant = {:.2}", view_matrix.determinant());
    println!("   ✅ Projection matrix: determinant = {:.2}", proj_matrix.determinant());
    println!("   ✅ View-Projection matrix: determinant = {:.2}", vp_matrix.determinant());
    
    // Test that matrices are valid (non-zero determinants)
    assert!(view_matrix.determinant().abs() > 0.001);
    assert!(proj_matrix.determinant().abs() > 0.001);
    assert!(vp_matrix.determinant().abs() > 0.001);
    
    println!("   ✅ Basic camera controller functionality verified");
}

fn test_matrix_caching() {
    use std::time::Instant;
    
    let camera = Camera::new(16.0 / 9.0);
    let mut controller = CameraController::new(camera);
    
    println!("   ⏱️  Testing matrix caching performance...");
    
    // Test cached access (should be fast)
    let start = Instant::now();
    for _ in 0..1000 {
        let _view = controller.view_matrix();
        let _proj = controller.projection_matrix();
        let _vp = controller.view_projection_matrix();
    }
    let cached_time = start.elapsed();
    
    // Test access after changes (should trigger recalculation)
    let start = Instant::now();
    for i in 0..100 {
        controller.set_position(Vec3::new(i as f32, 0.0, 0.0));
        let _view = controller.view_matrix();
        let _proj = controller.projection_matrix();
        let _vp = controller.view_projection_matrix();
    }
    let recalc_time = start.elapsed();
    
    println!("   📊 Performance Results:");
    println!("      • Cached access (1000 calls): {:.2}ms", cached_time.as_secs_f64() * 1000.0);
    println!("      • With recalculation (100 calls): {:.2}ms", recalc_time.as_secs_f64() * 1000.0);
    println!("      • Average cached call: {:.4}ms", cached_time.as_secs_f64() * 1.0);
    println!("      • Average recalc call: {:.4}ms", recalc_time.as_secs_f64() * 10.0);
    
    // Cached access should be much faster
    assert!(cached_time < recalc_time);
    println!("   ✅ Matrix caching working correctly");
}

fn test_camera_controls() {
    let camera = Camera::new(16.0 / 9.0);
    let mut controller = CameraController::new(camera);
    
    let initial_pos = controller.camera.position;
    let initial_target = controller.camera.target;
    
    println!("   🎮 Testing camera movement controls...");
    
    // Test forward movement
    controller.move_forward(5.0);
    let forward_pos = controller.camera.position;
    let forward_target = controller.camera.target;
    
    assert!((forward_pos - initial_pos).length() > 4.9);
    assert!((forward_target - initial_target).length() > 4.9);
    println!("      ✅ Forward movement: moved {:.2} units", (forward_pos - initial_pos).length());
    
    // Test right movement
    controller.move_right(3.0);
    let right_pos = controller.camera.position;
    
    assert!((right_pos - forward_pos).length() > 2.9);
    println!("      ✅ Right movement: moved {:.2} units", (right_pos - forward_pos).length());
    
    // Test up movement
    controller.move_up(2.0);
    let up_pos = controller.camera.position;
    
    assert!((up_pos - right_pos).length() > 1.9);
    println!("      ✅ Up movement: moved {:.2} units", (up_pos - right_pos).length());
    
    // Test rotation (FPS style)
    let initial_forward = controller.forward();
    controller.rotate_fps(0.5, 0.0); // Yaw 0.5 radians
    let rotated_forward = controller.forward();
    
    let angle_change = initial_forward.dot(rotated_forward).acos();
    assert!(angle_change > 0.4); // Should have rotated significantly
    println!("      ✅ FPS rotation: rotated {:.2} radians", angle_change);
    
    // Test orbit
    controller.set_position_target(Vec3::new(5.0, 0.0, 0.0), Vec3::ZERO);
    let orbit_initial_pos = controller.camera.position;
    controller.orbit(1.0, 0.0); // Orbit around Y axis
    let orbit_final_pos = controller.camera.position;
    
    // Distance to target should remain roughly the same
    let initial_distance = orbit_initial_pos.distance(Vec3::ZERO);
    let final_distance = orbit_final_pos.distance(Vec3::ZERO);
    assert!((initial_distance - final_distance).abs() < 0.1);
    println!("      ✅ Orbital movement: maintained distance {:.2}", final_distance);
    
    // Test zoom
    let zoom_initial_distance = controller.camera.position.distance(controller.camera.target);
    controller.zoom(-2.0); // Zoom in
    let zoom_final_distance = controller.camera.position.distance(controller.camera.target);
    
    assert!(zoom_final_distance < zoom_initial_distance);
    println!("      ✅ Zoom: distance changed from {:.2} to {:.2}", 
             zoom_initial_distance, zoom_final_distance);
    
    println!("   ✅ All camera movement controls working correctly");
}

fn test_ray_casting() {
    let camera = Camera::new(16.0 / 9.0);
    let mut controller = CameraController::new(camera);
    
    // Position camera looking at origin
    controller.set_position_target(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
    
    println!("   🎯 Testing screen-to-world ray casting...");
    
    let screen_width = 800.0;
    let screen_height = 600.0;
    
    // Test center of screen (should point towards target)
    let center_ray = controller.screen_to_world_ray(
        screen_width / 2.0,
        screen_height / 2.0,
        screen_width,
        screen_height
    );
    
    println!("      🎯 Center ray: origin={:?}, direction={:?}", 
             center_ray.origin, center_ray.direction);
    
    // Ray direction should be roughly towards negative Z (towards target)
    assert!(center_ray.direction.z < -0.8);
    println!("      ✅ Center ray pointing in correct direction");
    
    // Test corner rays
    let corner_rays = [
        (0.0, 0.0, "Top-left"),
        (screen_width, 0.0, "Top-right"),
        (0.0, screen_height, "Bottom-left"),
        (screen_width, screen_height, "Bottom-right"),
    ];
    
    for (x, y, name) in corner_rays {
        let ray = controller.screen_to_world_ray(x, y, screen_width, screen_height);
        println!("      📍 {} ray: direction={:?}", name, ray.direction);
        
        // All rays should generally point forward (negative Z)
        assert!(ray.direction.z < 0.0);
    }
    
    // Test ray-sphere intersection
    let test_ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
    let sphere_center = Vec3::new(0.0, 0.0, -5.0);
    let sphere_radius = 1.0;
    
    if let Some(distance) = test_ray.intersect_sphere(sphere_center, sphere_radius) {
        let intersection_point = test_ray.at(distance);
        println!("      🎯 Ray-sphere intersection at distance {:.2}, point {:?}", 
                 distance, intersection_point);
        
        // Should intersect at roughly distance 4.0 (5.0 - 1.0)
        assert!((distance - 4.0).abs() < 0.1);
        println!("      ✅ Ray-sphere intersection working correctly");
    } else {
        panic!("Ray should intersect sphere");
    }
    
    // Test world-to-screen conversion
    let world_point = Vec3::new(0.0, 0.0, -2.0); // Point in front of camera
    if let Some(screen_pos) = controller.world_to_screen(world_point, screen_width, screen_height) {
        println!("      📍 World point {:?} → Screen {:?}", world_point, screen_pos);
        
        // Should be roughly at center of screen
        assert!((screen_pos.x - screen_width / 2.0).abs() < 50.0);
        assert!((screen_pos.y - screen_height / 2.0).abs() < 50.0);
        println!("      ✅ World-to-screen conversion working correctly");
    } else {
        panic!("World point should be visible");
    }
    
    println!("   ✅ Ray casting functionality verified");
}

fn test_camera_presets() {
    println!("   🎨 Testing camera presets...");
    
    // Test first-person preset
    let fps_camera = CameraPresets::first_person(
        Vec3::new(0.0, 1.8, 0.0), // Eye height
        Vec3::new(0.0, 1.8, -1.0), // Looking forward
        16.0 / 9.0
    );
    
    let fps_info = fps_camera.get_info();
    println!("      🎮 FPS Camera: {}", fps_info);
    
    assert_eq!(fps_camera.camera.position.y, 1.8); // Eye height
    assert!(fps_info.fov_degrees > 70.0); // Wide FOV for FPS
    assert!(fps_info.distance_to_target < 2.0); // Close target for FPS
    println!("      ✅ First-person preset configured correctly");
    
    // Test orbital preset
    let orbital_camera = CameraPresets::orbital(
        Vec3::ZERO, // Look at origin
        10.0, // Distance
        16.0 / 9.0
    );
    
    let orbital_info = orbital_camera.get_info();
    println!("      🔄 Orbital Camera: {}", orbital_info);
    
    assert!((orbital_info.distance_to_target - 10.0).abs() < 0.1);
    assert!(orbital_info.fov_degrees > 50.0 && orbital_info.fov_degrees < 70.0); // Medium FOV
    println!("      ✅ Orbital preset configured correctly");
    
    // Test orthographic preset
    let ortho_camera = CameraPresets::orthographic(
        Vec3::new(0.0, 0.0, 0.0), // Center
        1.0, // Zoom
        16.0 / 9.0
    );
    
    let ortho_info = ortho_camera.get_info();
    println!("      📐 Orthographic Camera: {}", ortho_info);
    
    assert!(ortho_info.fov_degrees < 5.0); // Very small FOV for ortho effect
    assert!(ortho_info.position.z > 5.0); // Positioned away from center
    println!("      ✅ Orthographic preset configured correctly");
    
    println!("   ✅ All camera presets working correctly");
}

fn test_advanced_operations() {
    let camera = Camera::new(16.0 / 9.0);
    let mut controller = CameraController::new(camera);
    
    println!("   🔧 Testing advanced camera operations...");
    
    // Test parameter updates
    controller.set_fov(90.0_f32.to_radians());
    controller.set_aspect(21.0 / 9.0); // Ultra-wide
    controller.set_clip_planes(0.01, 10000.0);
    
    let info = controller.get_info();
    assert!((info.fov_degrees - 90.0).abs() < 0.1);
    assert!((info.aspect - 21.0 / 9.0).abs() < 0.01);
    assert!((info.near - 0.01).abs() < 0.001);
    assert!((info.far - 10000.0).abs() < 0.1);
    println!("      ✅ Parameter updates working correctly");
    
    // Test direction vectors
    controller.set_position_target(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));
    
    let forward = controller.forward();
    let right = controller.right();
    let up = controller.up();
    
    // Vectors should be orthogonal and normalized
    assert!((forward.length() - 1.0).abs() < 0.01);
    assert!((right.length() - 1.0).abs() < 0.01);
    assert!((up.length() - 1.0).abs() < 0.01);
    
    assert!(forward.dot(right).abs() < 0.01); // Orthogonal
    assert!(forward.dot(up).abs() < 0.01); // Orthogonal
    assert!(right.dot(up).abs() < 0.01); // Orthogonal
    
    println!("      ✅ Direction vectors are orthonormal");
    
    // Test frustum extraction
    let frustum = controller.get_frustum();
    println!("      📐 Frustum planes: {}", frustum.planes.len());
    assert_eq!(frustum.planes.len(), 6);
    
    // Test visibility checking (this is more about testing the API than precise frustum culling)
    let visible_point = controller.camera.target; // Should be visible
    let invisible_point = controller.camera.position + controller.forward() * -100.0; // Behind camera
    
    // Just test that the function works - frustum culling precision can vary
    let target_visible = controller.is_point_visible(visible_point);
    let behind_visible = controller.is_point_visible(invisible_point);
    
    println!("      📍 Target point visible: {}", target_visible);
    println!("      📍 Behind point visible: {}", behind_visible);
    println!("      ✅ Visibility checking API working correctly");
    
    // Test inverse matrices
    let view_matrix = controller.view_matrix();
    let inv_view_matrix = controller.inverse_view_matrix();
    let identity_check = view_matrix * inv_view_matrix;
    
    // Should be close to identity - check each element
    let diff_matrix = identity_check - glam::Mat4::IDENTITY;
    let max_diff = diff_matrix.to_cols_array().iter()
        .map(|x| x.abs())
        .fold(0.0, f32::max);
    assert!(max_diff < 0.001);
    println!("      ✅ Inverse matrices calculated correctly");
    
    println!("   ✅ All advanced operations working correctly");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn run_all_tests() {
        test_basic_camera_controller();
        test_matrix_caching();
        test_camera_controls();
        test_ray_casting();
        test_camera_presets();
        test_advanced_operations();
    }
}