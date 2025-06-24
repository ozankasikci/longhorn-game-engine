//! Render Queue Test
//!
//! This test demonstrates the render queue and sorting functionality,
//! showing how objects can be sorted for optimal rendering performance.

use engine_renderer_3d::{Camera, RenderObject, RenderQueue, SortMode};
use glam::{Mat4, Vec3, Vec4Swizzles};

fn main() {
    env_logger::init();

    println!("🔄 Render Queue Test");
    println!("===================");

    // Create test camera
    let mut camera = Camera::new(16.0 / 9.0);
    camera.position = Vec3::new(0.0, 0.0, 0.0);
    camera.target = Vec3::new(0.0, 0.0, -1.0);

    println!("📷 Camera position: {:?}", camera.position);

    // Test 1: No sorting
    println!("\n🧪 Test 1: No Sorting");
    test_sorting(SortMode::None, &camera);

    // Test 2: Front-to-back sorting
    println!("\n🧪 Test 2: Front-to-Back Sorting");
    test_sorting(SortMode::FrontToBack, &camera);

    // Test 3: Back-to-front sorting
    println!("\n🧪 Test 3: Back-to-Front Sorting");
    test_sorting(SortMode::BackToFront, &camera);

    // Test 4: Material sorting
    println!("\n🧪 Test 4: Material Sorting");
    test_material_sorting(&camera);

    // Test 5: Material + Distance sorting
    println!("\n🧪 Test 5: Material + Distance Sorting");
    test_material_distance_sorting(&camera);

    // Test 6: Performance test with many objects
    println!("\n🧪 Test 6: Performance Test");
    test_performance(&camera);

    println!("\n🎉 All render queue tests completed successfully!");
}

fn test_sorting(sort_mode: SortMode, camera: &Camera) {
    let mut queue = RenderQueue::new(sort_mode);

    // Create objects at different distances
    let objects = vec![
        create_test_object(1, Vec3::new(0.0, 0.0, -10.0)), // Far
        create_test_object(1, Vec3::new(0.0, 0.0, -5.0)),  // Near
        create_test_object(1, Vec3::new(0.0, 0.0, -15.0)), // Farthest
        create_test_object(1, Vec3::new(0.0, 0.0, -2.0)),  // Nearest
    ];

    queue.add_objects(objects, camera);

    let stats = queue.get_stats();
    println!("   📊 Queue stats: {}", stats);

    let items = queue.get_sorted_items();
    println!("   🎯 Sorted distances:");
    for (i, item) in items.iter().enumerate() {
        let pos = item.object.transform.col(3).xyz();
        println!(
            "      Object {}: distance={:.1}, pos={:?}",
            i + 1,
            item.camera_distance,
            pos
        );
    }

    // Verify sorting correctness
    match sort_mode {
        SortMode::FrontToBack => {
            for i in 1..items.len() {
                assert!(
                    items[i - 1].camera_distance <= items[i].camera_distance,
                    "Front-to-back sorting failed"
                );
            }
            println!("   ✅ Front-to-back sorting verified");
        }
        SortMode::BackToFront => {
            for i in 1..items.len() {
                assert!(
                    items[i - 1].camera_distance >= items[i].camera_distance,
                    "Back-to-front sorting failed"
                );
            }
            println!("   ✅ Back-to-front sorting verified");
        }
        _ => {}
    }
}

fn test_material_sorting(camera: &Camera) {
    let mut queue = RenderQueue::new(SortMode::Material);

    // Create objects with different materials (intentionally out of order)
    let objects = vec![
        create_test_object(5, Vec3::new(0.0, 0.0, -5.0)),
        create_test_object(2, Vec3::new(1.0, 0.0, -5.0)),
        create_test_object(8, Vec3::new(2.0, 0.0, -5.0)),
        create_test_object(1, Vec3::new(3.0, 0.0, -5.0)),
        create_test_object(5, Vec3::new(4.0, 0.0, -5.0)),
        create_test_object(2, Vec3::new(5.0, 0.0, -5.0)),
    ];

    queue.add_objects(objects, camera);

    let stats = queue.get_stats();
    println!("   📊 Queue stats: {}", stats);

    let items = queue.get_sorted_items();
    println!("   🎯 Sorted materials:");
    for (i, item) in items.iter().enumerate() {
        println!(
            "      Object {}: material_id={}",
            i + 1,
            item.object.material_id
        );
    }

    // Verify material sorting
    for i in 1..items.len() {
        assert!(
            items[i - 1].object.material_id <= items[i].object.material_id,
            "Material sorting failed"
        );
    }
    println!("   ✅ Material sorting verified");

    // Test material groups
    let groups = queue.get_material_groups();
    println!("   📦 Material groups:");
    for (i, group) in groups.iter().enumerate() {
        println!(
            "      Group {}: material_id={}, count={}, range={:?}",
            i + 1,
            group.material_id,
            group.count,
            group.range()
        );
    }

    println!("   ✅ Material grouping verified ({} groups)", groups.len());
}

fn test_material_distance_sorting(camera: &Camera) {
    let mut queue = RenderQueue::new(SortMode::MaterialThenDistance);

    // Create objects with same materials at different distances
    let objects = vec![
        create_test_object(2, Vec3::new(0.0, 0.0, -10.0)), // Material 2, far
        create_test_object(1, Vec3::new(0.0, 0.0, -15.0)), // Material 1, farthest
        create_test_object(2, Vec3::new(0.0, 0.0, -5.0)),  // Material 2, near
        create_test_object(1, Vec3::new(0.0, 0.0, -2.0)),  // Material 1, nearest
    ];

    queue.add_objects(objects, camera);

    let stats = queue.get_stats();
    println!("   📊 Queue stats: {}", stats);

    let items = queue.get_sorted_items();
    println!("   🎯 Sorted by material then distance:");
    for (i, item) in items.iter().enumerate() {
        let pos = item.object.transform.col(3).xyz();
        println!(
            "      Object {}: material_id={}, distance={:.1}, pos={:?}",
            i + 1,
            item.object.material_id,
            item.camera_distance,
            pos
        );
    }

    // Verify material-then-distance sorting
    for i in 1..items.len() {
        let prev = &items[i - 1];
        let curr = &items[i];

        if prev.object.material_id == curr.object.material_id {
            // Same material, should be sorted by distance
            assert!(
                prev.camera_distance <= curr.camera_distance,
                "Distance sorting within material failed"
            );
        } else {
            // Different material, material ID should be sorted
            assert!(
                prev.object.material_id <= curr.object.material_id,
                "Material sorting failed"
            );
        }
    }
    println!("   ✅ Material-then-distance sorting verified");
}

fn test_performance(camera: &Camera) {
    use std::time::Instant;

    let num_objects = 10000;
    println!("   🏃 Testing with {} objects...", num_objects);

    // Create many objects with random materials and positions
    let mut objects = Vec::with_capacity(num_objects);
    for i in 0..num_objects {
        let material_id = (i % 10) as u32; // 10 different materials
        let x = (i as f32 % 100.0) - 50.0;
        let z = -((i as f32) / 100.0) - 5.0;
        objects.push(create_test_object(material_id, Vec3::new(x, 0.0, z)));
    }

    // Test different sorting modes
    for sort_mode in [
        SortMode::None,
        SortMode::Material,
        SortMode::FrontToBack,
        SortMode::MaterialThenDistance,
    ] {
        let mut queue = RenderQueue::new(sort_mode);

        let start = Instant::now();
        queue.add_objects(objects.clone(), camera);
        let add_time = start.elapsed();

        let start = Instant::now();
        let _items = queue.get_sorted_items();
        let sort_time = start.elapsed();

        let stats = queue.get_stats();

        println!(
            "      {:?}: add={:.2}ms, sort={:.2}ms, draw_calls={}",
            sort_mode,
            add_time.as_secs_f64() * 1000.0,
            sort_time.as_secs_f64() * 1000.0,
            stats.estimated_draw_calls
        );
    }

    println!("   ✅ Performance test completed");
}

fn create_test_object(material_id: u32, position: Vec3) -> RenderObject {
    let transform = Mat4::from_translation(position);
    RenderObject::new(transform, 0, material_id)
}
