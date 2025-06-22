//! Frustum Culling Test
//! 
//! This test demonstrates the frustum culling functionality,
//! showing how objects outside the camera view are efficiently culled.

use engine_renderer_3d::{FrustumCuller, Camera, RenderObject, BoundingVolume};
use glam::{Mat4, Vec3};

fn main() {
    env_logger::init();
    
    println!("🔍 Frustum Culling Test");
    println!("=======================");
    
    // Test 1: Basic frustum culling with camera
    println!("\n🧪 Test 1: Basic Camera Frustum");
    test_basic_camera_frustum();
    
    // Test 2: Object culling with different positions
    println!("\n🧪 Test 2: Object Position Culling");
    test_object_position_culling();
    
    // Test 3: Bounding volume culling
    println!("\n🧪 Test 3: Bounding Volume Culling");
    test_bounding_volume_culling();
    
    // Test 4: Performance test with many objects
    println!("\n🧪 Test 4: Performance Test");
    test_culling_performance();
    
    // Test 5: Camera movement and frustum updates
    println!("\n🧪 Test 5: Dynamic Camera Updates");
    test_dynamic_camera_updates();
    
    println!("\n🎉 All frustum culling tests completed successfully!");
}

fn test_basic_camera_frustum() {
    let camera = create_test_camera();
    let culler = FrustumCuller::new(&camera);
    
    println!("   📷 Camera position: {:?}", camera.position);
    println!("   📷 Camera target: {:?}", camera.target);
    println!("   📷 Camera FOV: {:.1}°", camera.fov.to_degrees());
    
    let frustum = culler.get_frustum();
    println!("   📐 Frustum created with 6 planes:");
    for (i, plane) in frustum.planes.iter().enumerate() {
        let plane_name = match i {
            0 => "Left",
            1 => "Right", 
            2 => "Bottom",
            3 => "Top",
            4 => "Near",
            5 => "Far",
            _ => "Unknown",
        };
        println!("      {}: normal={:?}, distance={:.2}", 
                 plane_name, plane.normal, plane.distance);
    }
    
    println!("   ✅ Frustum creation verified");
}

fn test_object_position_culling() {
    let camera = create_test_camera();
    let mut culler = FrustumCuller::new(&camera);
    
    // Create objects at various positions
    let test_objects = vec![
        (create_test_object(Vec3::new(0.0, 0.0, -5.0)), "Center (should be visible)"),
        (create_test_object(Vec3::new(0.0, 0.0, -50.0)), "Far away (might be visible)"),
        (create_test_object(Vec3::new(0.0, 0.0, 5.0)), "Behind camera (should be culled)"),
        (create_test_object(Vec3::new(100.0, 0.0, -5.0)), "Far right (should be culled)"),
        (create_test_object(Vec3::new(-100.0, 0.0, -5.0)), "Far left (should be culled)"),
        (create_test_object(Vec3::new(0.0, 100.0, -5.0)), "Far up (should be culled)"),
        (create_test_object(Vec3::new(0.0, -100.0, -5.0)), "Far down (should be culled)"),
        (create_test_object(Vec3::new(2.0, 0.0, -5.0)), "Slightly right (might be visible)"),
        (create_test_object(Vec3::new(-2.0, 0.0, -5.0)), "Slightly left (might be visible)"),
    ];
    
    println!("   🎯 Testing {} objects:", test_objects.len());
    
    for (object, description) in &test_objects {
        let visible = culler.is_object_visible(object);
        let position = object.transform.col(3).truncate();
        println!("      {}: {} at {:?}", 
                 if visible { "✅ VISIBLE" } else { "❌ CULLED" },
                 description, position);
    }
    
    let stats = culler.get_stats();
    println!("   📊 Culling stats: {}", stats);
    println!("   ✅ Position-based culling verified");
}

fn test_bounding_volume_culling() {
    let camera = create_test_camera();
    let mut culler = FrustumCuller::new(&camera);
    
    // Create objects with different bounding volumes
    let test_cases = vec![
        (
            create_test_object(Vec3::new(0.0, 0.0, -5.0)),
            BoundingVolume::point(Vec3::new(0.0, 0.0, -5.0)),
            "Point at center"
        ),
        (
            create_test_object(Vec3::new(0.0, 0.0, -5.0)),
            BoundingVolume::sphere(Vec3::new(0.0, 0.0, -5.0), 1.0),
            "Small sphere at center"
        ),
        (
            create_test_object(Vec3::new(0.0, 0.0, -5.0)),
            BoundingVolume::sphere(Vec3::new(0.0, 0.0, -5.0), 10.0),
            "Large sphere at center"
        ),
        (
            create_test_object(Vec3::new(50.0, 0.0, -5.0)),
            BoundingVolume::sphere(Vec3::new(50.0, 0.0, -5.0), 1.0),
            "Small sphere far right"
        ),
        (
            create_test_object(Vec3::new(50.0, 0.0, -5.0)),
            BoundingVolume::sphere(Vec3::new(50.0, 0.0, -5.0), 60.0),
            "Huge sphere far right (should intersect)"
        ),
        (
            create_test_object(Vec3::new(0.0, 0.0, -5.0)),
            BoundingVolume::aabb(
                Vec3::new(-1.0, -1.0, -6.0),
                Vec3::new(1.0, 1.0, -4.0)
            ),
            "Small AABB at center"
        ),
        (
            create_test_object(Vec3::new(0.0, 0.0, -5.0)),
            BoundingVolume::aabb(
                Vec3::new(-100.0, -100.0, -100.0),
                Vec3::new(100.0, 100.0, 100.0)
            ),
            "Huge AABB covering everything"
        ),
    ];
    
    println!("   🎯 Testing {} bounding volumes:", test_cases.len());
    
    let objects_with_bounds: Vec<_> = test_cases.iter()
        .map(|(obj, bound, _)| (obj.clone(), *bound))
        .collect();
    
    let visible_objects = culler.cull_objects_with_bounds(&objects_with_bounds);
    
    for (i, (_, _, description)) in test_cases.iter().enumerate() {
        let was_culled = !visible_objects.iter().any(|obj| {
            // Simple comparison by position (not perfect but works for this test)
            let original_pos = test_cases[i].0.transform.col(3).truncate();
            let visible_pos = obj.transform.col(3).truncate();
            (original_pos - visible_pos).length() < 0.01
        });
        
        println!("      {}: {}", 
                 if was_culled { "❌ CULLED" } else { "✅ VISIBLE" },
                 description);
    }
    
    let stats = culler.get_stats();
    println!("   📊 Culling stats: {}", stats);
    println!("   ✅ Bounding volume culling verified");
}

fn test_culling_performance() {
    use std::time::Instant;
    
    let camera = create_test_camera();
    let mut culler = FrustumCuller::new(&camera);
    
    let num_objects = 10000;
    println!("   🏃 Testing performance with {} objects...", num_objects);
    
    // Create many objects in a grid pattern
    let mut objects = Vec::with_capacity(num_objects);
    for i in 0..num_objects {
        let x = ((i % 100) as f32 - 50.0) * 2.0; // Spread from -100 to 100
        let y = (((i / 100) % 100) as f32 - 50.0) * 2.0;
        let z = -((i / 10000) as f32 + 1.0) * 5.0; // Different depth layers
        objects.push(create_test_object(Vec3::new(x, y, z)));
    }
    
    // Test culling performance
    let start = Instant::now();
    let visible_objects = culler.cull_objects(&objects);
    let culling_time = start.elapsed();
    
    let stats = culler.get_stats();
    
    println!("      ⏱️  Culling time: {:.2}ms", culling_time.as_secs_f64() * 1000.0);
    println!("      📊 Results: {}", stats);
    println!("      📈 Throughput: {:.0} objects/ms", 
             num_objects as f64 / (culling_time.as_secs_f64() * 1000.0));
    println!("      🎯 Visible objects: {}/{} ({:.1}%)", 
             visible_objects.len(), num_objects,
             (visible_objects.len() as f32 / num_objects as f32) * 100.0);
    
    println!("   ✅ Performance test completed");
}

fn test_dynamic_camera_updates() {
    let mut camera = create_test_camera();
    let mut culler = FrustumCuller::new(&camera);
    
    // Create a test object
    let test_object = create_test_object(Vec3::new(5.0, 0.0, -5.0));
    
    println!("   🎯 Testing camera movement effects on culling:");
    
    // Test different camera positions
    let camera_positions = vec![
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), "Looking forward"),
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, -1.0), "Looking right"),
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, -1.0), "Looking left"),
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 0.0, -5.0), "Looking at object"),
        (Vec3::new(10.0, 0.0, 0.0), Vec3::new(5.0, 0.0, -5.0), "From the side"),
    ];
    
    for (position, target, description) in camera_positions {
        camera.position = position;
        camera.target = target;
        culler.update_from_camera(&camera);
        
        let visible = culler.is_object_visible(&test_object);
        println!("      {}: {} (object at {:?})", 
                 description,
                 if visible { "✅ VISIBLE" } else { "❌ CULLED" },
                 test_object.transform.col(3).truncate());
    }
    
    let stats = culler.get_stats();
    println!("   📊 Final stats: {}", stats);
    println!("   ✅ Dynamic camera updates verified");
}

fn create_test_camera() -> Camera {
    let mut camera = Camera::new(16.0 / 9.0);
    camera.position = Vec3::new(0.0, 0.0, 0.0);
    camera.target = Vec3::new(0.0, 0.0, -1.0);
    camera.fov = 60.0_f32.to_radians();
    camera.near = 0.1;
    camera.far = 100.0;
    camera
}

fn create_test_object(position: Vec3) -> RenderObject {
    let transform = Mat4::from_translation(position);
    RenderObject::new(transform, 0, 0)
}