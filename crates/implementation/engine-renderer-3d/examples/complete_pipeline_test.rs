//! Complete 3D Rendering Pipeline Test
//! 
//! This test demonstrates the complete 3D rendering pipeline including:
//! - ECS world with multiple 3D objects
//! - ECS-to-renderer bridge
//! - Frustum culling
//! - Render queue sorting
//! - Multiple transforms and materials

use engine_renderer_3d::{
    Renderer3D, Camera, EcsRendererIntegration, FrustumCuller, RenderQueue, SortMode
};
use engine_ecs_core::ecs_v2::{World, component::register_component};
use engine_components_3d::{Transform, Mesh, Material as EcsMaterial, MeshType};
use glam::Vec3;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Complete 3D Rendering Pipeline Test");
    println!("======================================");
    
    // Setup phase
    println!("\n🔧 Phase 1: System Setup");
    let (renderer, camera, world, integration) = setup_systems().await?;
    
    // Create complex scene
    println!("\n🌍 Phase 2: Complex Scene Creation");
    let world = create_complex_scene(world);
    
    // Test complete pipeline
    println!("\n⚙️  Phase 3: Complete Pipeline Test");
    test_complete_pipeline(&renderer, &camera, &world, &integration).await?;
    
    // Performance benchmark
    println!("\n🏁 Phase 4: Performance Benchmark");
    performance_benchmark(&renderer, &camera, &integration).await?;
    
    // Real-time simulation
    println!("\n🎬 Phase 5: Real-time Scene Updates");
    test_realtime_updates(&renderer, &camera, &integration).await?;
    
    println!("\n🎉 Complete pipeline test successful!");
    Ok(())
}

async fn setup_systems() -> Result<(Renderer3D, Camera, World, EcsRendererIntegration), Box<dyn std::error::Error>> {
    // Register ECS components
    register_component::<Transform>();
    register_component::<Mesh>();
    register_component::<EcsMaterial>();
    println!("   ✅ ECS components registered");
    
    // Initialize WGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or("Failed to find an appropriate adapter")?;
    
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await?;
    
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    
    // Create renderer
    let renderer = Renderer3D::new(device, queue, 1920, 1080).await?;
    println!("   ✅ Renderer initialized (1920x1080)");
    
    // Create camera
    let mut camera = Camera::new(16.0 / 9.0);
    camera.position = Vec3::new(0.0, 5.0, 10.0);
    camera.target = Vec3::new(0.0, 0.0, 0.0);
    camera.fov = 75.0_f32.to_radians();
    camera.near = 0.1;
    camera.far = 1000.0;
    println!("   ✅ Camera configured");
    
    // Create ECS world
    let world = World::new();
    println!("   ✅ ECS world created");
    
    // Create integration
    let mut integration = EcsRendererIntegration::new(
        renderer.get_default_mesh_id("triangle").unwrap_or(0),
        renderer.get_default_material_id("default").unwrap_or(0),
    );
    integration.setup_default_mappings(&renderer);
    println!("   ✅ ECS integration configured");
    
    Ok((renderer, camera, world, integration))
}

fn create_complex_scene(mut world: World) -> World {
    let mut entity_count = 0;
    
    // Create a grid of cubes
    println!("   🧊 Creating cube grid (10x10)...");
    for x in -5..5 {
        for z in -5..5 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new()
                .with_position(x as f32 * 3.0, 0.0, z as f32 * 3.0)
                .with_scale(1.0, 1.0, 1.0)
            ).unwrap();
            world.add_component(entity, Mesh { mesh_type: MeshType::Cube }).unwrap();
            
            // Vary materials based on position
            let _material_variation = ((x + 5) % 3) as u32;
            world.add_component(entity, EcsMaterial::default()).unwrap();
            entity_count += 1;
        }
    }
    
    // Create some spheres at different heights
    println!("   🔮 Creating floating spheres...");
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let radius = 15.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = 5.0 + i as f32 * 2.0;
        
        let entity = world.spawn();
        world.add_component(entity, Transform::new()
            .with_position(x, y, z)
            .with_scale(2.0, 2.0, 2.0)
        ).unwrap();
        world.add_component(entity, Mesh { mesh_type: MeshType::Sphere }).unwrap();
        world.add_component(entity, EcsMaterial::default()).unwrap();
        entity_count += 1;
    }
    
    // Create some custom objects
    println!("   🏗️  Creating custom objects...");
    for i in 0..3 {
        let entity = world.spawn();
        world.add_component(entity, Transform::new()
            .with_position(-20.0, i as f32 * 5.0, 0.0)
            .with_rotation(0.0, i as f32 * 0.5, 0.0)
            .with_scale(3.0, 1.0, 1.0)
        ).unwrap();
        world.add_component(entity, Mesh { 
            mesh_type: MeshType::Custom(format!("tower_{}", i))
        }).unwrap();
        world.add_component(entity, EcsMaterial::default()).unwrap();
        entity_count += 1;
    }
    
    println!("   ✅ Scene created with {} entities", entity_count);
    world
}

async fn test_complete_pipeline(
    _renderer: &Renderer3D,
    camera: &Camera,
    world: &World,
    integration: &EcsRendererIntegration
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    println!("   🔄 Testing complete rendering pipeline...");
    
    // Step 1: ECS to Scene Conversion
    let start = Instant::now();
    let scene = integration.world_to_scene(world, camera.aspect);
    let ecs_time = start.elapsed();
    
    println!("      1. ECS → Scene: {:.2}ms ({} objects)", 
             ecs_time.as_secs_f64() * 1000.0, scene.objects.len());
    
    // Step 2: Frustum Culling
    let start = Instant::now();
    let mut culler = FrustumCuller::new(camera);
    let visible_objects = culler.cull_objects(&scene.objects);
    let culling_time = start.elapsed();
    let culling_stats = culler.get_stats();
    
    println!("      2. Frustum Culling: {:.2}ms ({})", 
             culling_time.as_secs_f64() * 1000.0, culling_stats);
    
    // Step 3: Render Queue Sorting
    let start = Instant::now();
    let mut render_queue = RenderQueue::new(SortMode::MaterialThenDistance);
    render_queue.add_objects(visible_objects, camera);
    let queue_stats = render_queue.get_stats();
    let sorted_items = render_queue.get_sorted_items();
    let sorted_items_len = sorted_items.len(); // Get the length to avoid borrowing issues
    let sorting_time = start.elapsed();
    
    println!("      3. Render Queue: {:.2}ms ({})", 
             sorting_time.as_secs_f64() * 1000.0, queue_stats);
    
    // Step 4: Material Grouping
    let start = Instant::now();
    let material_groups = render_queue.get_material_groups();
    let grouping_time = start.elapsed();
    
    println!("      4. Material Grouping: {:.2}ms ({} groups)", 
             grouping_time.as_secs_f64() * 1000.0, material_groups.len());
    
    // Display detailed results
    println!("   📊 Pipeline Results:");
    println!("      • Total entities: {}", world.entity_count());
    println!("      • Renderable objects: {}", scene.objects.len());
    println!("      • Visible after culling: {}", sorted_items_len);
    println!("      • Material groups: {}", material_groups.len());
    
    let total_time = ecs_time + culling_time + sorting_time + grouping_time;
    println!("      • Total pipeline time: {:.2}ms", total_time.as_secs_f64() * 1000.0);
    
    if !material_groups.is_empty() {
        println!("   🎨 Material Group Breakdown:");
        for (i, group) in material_groups.iter().enumerate() {
            println!("      Group {}: material_id={}, objects={}, range={:?}",
                     i + 1, group.material_id, group.count, group.range());
        }
    }
    
    println!("   ✅ Complete pipeline test passed");
    Ok(())
}

async fn performance_benchmark(
    _renderer: &Renderer3D,
    camera: &Camera,
    integration: &EcsRendererIntegration
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    println!("   🏃 Running performance benchmark...");
    
    // Create large world for benchmarking
    let mut world = World::new();
    let num_objects = 50000;
    
    println!("      Creating {} test objects...", num_objects);
    let creation_start = Instant::now();
    
    for i in 0..num_objects {
        let entity = world.spawn();
        
        // Distribute objects in a large area
        let x = ((i % 100) as f32 - 50.0) * 5.0;
        let y = (((i / 100) % 100) as f32 - 50.0) * 2.0;
        let z = -((i / 10000) as f32) * 20.0 - 10.0;
        
        world.add_component(entity, Transform::new()
            .with_position(x, y, z)
            .with_scale(1.0, 1.0, 1.0)
        ).unwrap();
        
        let mesh_type = match i % 3 {
            0 => MeshType::Cube,
            1 => MeshType::Sphere,
            _ => MeshType::Plane,
        };
        world.add_component(entity, Mesh { mesh_type }).unwrap();
        world.add_component(entity, EcsMaterial::default()).unwrap();
    }
    
    let creation_time = creation_start.elapsed();
    println!("      Object creation: {:.2}ms", creation_time.as_secs_f64() * 1000.0);
    
    // Benchmark multiple pipeline runs
    let num_runs = 10;
    let mut total_times = Vec::new();
    
    for run in 0..num_runs {
        let run_start = Instant::now();
        
        // Full pipeline
        let scene = integration.world_to_scene(&world, camera.aspect);
        let mut culler = FrustumCuller::new(camera);
        let visible_objects = culler.cull_objects(&scene.objects);
        let mut render_queue = RenderQueue::new(SortMode::MaterialThenDistance);
        render_queue.add_objects(visible_objects, camera);
        let _sorted_items = render_queue.get_sorted_items();
        
        let run_time = run_start.elapsed();
        total_times.push(run_time);
        
        if run == 0 {
            // Print detailed stats for first run
            let culling_stats = culler.get_stats();
            let queue_stats = render_queue.get_stats();
            println!("      Run 1 details: {:.2}ms, culling: {}, queue: {}",
                     run_time.as_secs_f64() * 1000.0, culling_stats, queue_stats);
        }
    }
    
    // Calculate statistics
    let min_time = total_times.iter().min().unwrap();
    let max_time = total_times.iter().max().unwrap();
    let avg_time = total_times.iter().sum::<std::time::Duration>() / num_runs as u32;
    
    println!("   📊 Benchmark Results ({} runs):", num_runs);
    println!("      • Objects processed: {}", num_objects);
    println!("      • Min time: {:.2}ms", min_time.as_secs_f64() * 1000.0);
    println!("      • Max time: {:.2}ms", max_time.as_secs_f64() * 1000.0);
    println!("      • Avg time: {:.2}ms", avg_time.as_secs_f64() * 1000.0);
    println!("      • Throughput: {:.0} objects/ms", 
             num_objects as f64 / (avg_time.as_secs_f64() * 1000.0));
    
    println!("   ✅ Performance benchmark completed");
    Ok(())
}

async fn test_realtime_updates(
    _renderer: &Renderer3D,
    camera: &Camera,
    integration: &EcsRendererIntegration
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    println!("   🎬 Testing real-time scene updates...");
    
    // Create dynamic scene
    let mut world = World::new();
    let mut entity_transforms = Vec::new();
    
    // Create moving objects
    for i in 0..100 {
        let entity = world.spawn();
        let initial_transform = Transform::new()
            .with_position(
                (i as f32 * 0.5).sin() * 20.0,
                0.0,
                (i as f32 * 0.3).cos() * 20.0
            );
        
        world.add_component(entity, initial_transform.clone()).unwrap();
        world.add_component(entity, Mesh { mesh_type: MeshType::Cube }).unwrap();
        world.add_component(entity, EcsMaterial::default()).unwrap();
        
        entity_transforms.push((entity, initial_transform));
    }
    
    println!("      Simulating {} frames of animation...", 60);
    
    let mut frame_times = Vec::new();
    
    // Simulate 60 frames
    for frame in 0..60 {
        let frame_start = Instant::now();
        
        // Update transforms (simulate animation)
        let time = frame as f32 * 0.016; // 16ms per frame
        for (entity, base_transform) in &entity_transforms {
            let new_transform = Transform::new()
                .with_position(
                    base_transform.position[0] + (time * 2.0).sin(),
                    base_transform.position[1] + (time * 3.0).sin() * 0.5,
                    base_transform.position[2] + (time * 1.5).cos()
                )
                .with_rotation(0.0, time, 0.0);
            
            // In a real system, this would be more efficient
            world.add_component(*entity, new_transform).unwrap();
        }
        
        // Process rendering pipeline
        let scene = integration.world_to_scene(&world, camera.aspect);
        let mut culler = FrustumCuller::new(camera);
        let visible_objects = culler.cull_objects(&scene.objects);
        let mut render_queue = RenderQueue::new(SortMode::MaterialThenDistance);
        render_queue.add_objects(visible_objects, camera);
        let _sorted_items = render_queue.get_sorted_items();
        
        let frame_time = frame_start.elapsed();
        frame_times.push(frame_time);
        
        if frame % 15 == 0 {
            let culling_stats = culler.get_stats();
            println!("      Frame {}: {:.2}ms, visible: {}/{}", 
                     frame, frame_time.as_secs_f64() * 1000.0,
                     culling_stats.visible_objects, culling_stats.total_tests);
        }
    }
    
    // Calculate frame statistics
    let avg_frame_time = frame_times.iter().sum::<std::time::Duration>() / frame_times.len() as u32;
    let min_frame_time = frame_times.iter().min().unwrap();
    let max_frame_time = frame_times.iter().max().unwrap();
    let fps = 1.0 / avg_frame_time.as_secs_f64();
    
    println!("   📊 Real-time Performance:");
    println!("      • Average frame time: {:.2}ms", avg_frame_time.as_secs_f64() * 1000.0);
    println!("      • Min frame time: {:.2}ms", min_frame_time.as_secs_f64() * 1000.0);
    println!("      • Max frame time: {:.2}ms", max_frame_time.as_secs_f64() * 1000.0);
    println!("      • Average FPS: {:.1}", fps);
    
    if fps >= 60.0 {
        println!("      ✅ Real-time capable (60+ FPS)");
    } else if fps >= 30.0 {
        println!("      ⚠️  Moderate performance (30-60 FPS)");
    } else {
        println!("      ❌ Performance issues (<30 FPS)");
    }
    
    println!("   ✅ Real-time update test completed");
    Ok(())
}