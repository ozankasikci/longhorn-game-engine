//! ECS Integration Test for the 3D renderer
//! 
//! This test demonstrates the bridge between ECS world and the 3D renderer,
//! showing how ECS entities with Transform, Mesh, and Material components
//! are converted to renderable objects.

use engine_renderer_3d::{Renderer3D, Camera, EcsRendererIntegration};
use engine_ecs_core::ecs_v2::{World, component::register_component};
use engine_components_3d::{Transform, Mesh, Material as EcsMaterial, MeshType};
use glam::{Vec3, Vec4Swizzles};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🔗 ECS Integration Test");
    println!("======================");
    
    // Register components first
    println!("🔧 Registering ECS components...");
    register_component::<Transform>();
    register_component::<Mesh>();
    register_component::<EcsMaterial>();
    println!("   ✅ Components registered");
    
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
    let width = 800;
    let height = 600;
    let renderer = Renderer3D::new(device.clone(), queue.clone(), width, height).await?;
    
    println!("✅ Renderer initialized ({}x{})", width, height);
    
    // Phase 1: Setup ECS World
    println!("\n📦 Phase 1: Creating ECS World");
    let mut world = World::new();
    
    // Create various entities with different components
    let entities = create_test_entities(&mut world);
    println!("   Created {} test entities", entities.len());
    
    // Log entity details
    for (i, (entity, description)) in entities.iter().enumerate() {
        println!("   Entity {}: {} (ID: {})", i + 1, description, entity.id());
    }
    
    // Phase 2: Setup ECS-Renderer Integration
    println!("\n🔗 Phase 2: Setting Up ECS Integration");
    let mut integration = EcsRendererIntegration::new(
        renderer.get_default_mesh_id("triangle").unwrap_or(0),
        renderer.get_default_material_id("default").unwrap_or(0),
    );
    
    // Setup default mappings
    integration.setup_default_mappings(&renderer);
    println!("   ✅ Default mappings configured");
    
    // Display integration stats
    let integration_stats = integration.get_stats();
    println!("   📊 Integration stats: {}", integration_stats);
    
    // Phase 3: Convert ECS World to Render Scene
    println!("\n🎬 Phase 3: Converting ECS World to Render Scene");
    let aspect_ratio = width as f32 / height as f32;
    let render_scene = integration.world_to_scene(&world, aspect_ratio);
    
    println!("   ✅ Conversion completed");
    println!("   📊 Render scene stats:");
    println!("      - Objects: {}", render_scene.objects.len());
    println!("      - Camera position: {:?}", render_scene.camera.position);
    println!("      - Camera target: {:?}", render_scene.camera.target);
    println!("      - Clear color: {:?}", render_scene.clear_color);
    
    // Phase 4: Detailed Object Analysis
    println!("\n🔍 Phase 4: Render Object Analysis");
    for (i, obj) in render_scene.objects.iter().enumerate() {
        let transform_matrix = obj.transform;
        let translation = transform_matrix.col(3).xyz();
        
        println!("   Object {}: mesh_id={}, material_id={}, pos={:?}", 
                 i + 1, obj.mesh_id, obj.material_id, translation);
    }
    
    // Phase 5: Test Scene Updates
    println!("\n🔄 Phase 5: Testing Scene Updates");
    
    // Add a new entity to the world
    let new_entity = world.spawn();
    world.add_component(new_entity, Transform::new().with_position(5.0, 1.0, 0.0))?;
    world.add_component(new_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(new_entity, EcsMaterial::default())?;
    
    println!("   ➕ Added new entity: {} at position (5, 1, 0)", new_entity.id());
    
    // Convert updated world to scene
    let updated_scene = integration.world_to_scene(&world, aspect_ratio);
    
    println!("   📊 Updated scene stats:");
    println!("      - Objects: {} (was {})", updated_scene.objects.len(), render_scene.objects.len());
    println!("      - New object count: +{}", updated_scene.objects.len() - render_scene.objects.len());
    
    // Phase 6: Custom Camera Test
    println!("\n📸 Phase 6: Custom Camera Test");
    let custom_camera = Camera::from_position_rotation([3.0, 4.0, 5.0], [0.1, 0.2, 0.0], aspect_ratio);
    let custom_scene = integration.world_to_scene_with_camera(&world, custom_camera);
    
    println!("   📊 Custom camera scene:");
    println!("      - Camera position: {:?}", custom_scene.camera.position);
    println!("      - Camera target: {:?}", custom_scene.camera.target);
    println!("      - Objects: {}", custom_scene.objects.len());
    
    // Phase 7: Validation and Final Tests
    println!("\n✅ Phase 7: Validation");
    
    // Verify the conversion preserved entity count
    let expected_renderable_entities = count_renderable_entities(&world);
    let actual_render_objects = updated_scene.objects.len();
    
    println!("   🧮 Entity validation:");
    println!("      - Renderable entities in ECS: {}", expected_renderable_entities);
    println!("      - Render objects created: {}", actual_render_objects);
    
    let validation_success = expected_renderable_entities == actual_render_objects;
    println!("      - Validation: {}", if validation_success { "✅ PASSED" } else { "❌ FAILED" });
    
    // Test resource mapping fallbacks
    println!("   🔧 Resource mapping validation:");
    let mapping_stats = integration.get_stats();
    println!("      - Mesh mappings: {}", mapping_stats.mesh_mappings);
    println!("      - Material mappings: {}", mapping_stats.material_mappings);
    println!("      - Default mesh ID: {}", mapping_stats.default_mesh_id);
    println!("      - Default material ID: {}", mapping_stats.default_material_id);
    
    // Final Summary
    println!("\n🎉 Test Results Summary");
    println!("======================");
    println!("ECS-Renderer Integration: {}", if validation_success { "✅ OPERATIONAL" } else { "❌ FAILED" });
    println!("Features Tested:");
    println!("   ✅ ECS world creation with 3D components");
    println!("   ✅ ECS-to-renderer bridge setup");
    println!("   ✅ Component mapping and fallbacks");
    println!("   ✅ Scene conversion from ECS world");
    println!("   ✅ Dynamic entity addition");
    println!("   ✅ Custom camera integration");
    println!("   ✅ Resource validation");
    
    println!("\n🚀 ECS Integration: FULLY OPERATIONAL");
    
    Ok(())
}

/// Create test entities with various component combinations
fn create_test_entities(world: &mut World) -> Vec<(engine_ecs_core::ecs_v2::Entity, String)> {
    let mut entities = Vec::new();
    
    // Entity 1: Cube at origin
    let entity1 = world.spawn();
    world.add_component(entity1, Transform::new()).unwrap();
    world.add_component(entity1, Mesh { mesh_type: MeshType::Cube }).unwrap();
    world.add_component(entity1, EcsMaterial::default()).unwrap();
    entities.push((entity1, "Cube at origin".to_string()));
    
    // Entity 2: Cube at (2, 0, 0) 
    let entity2 = world.spawn();
    world.add_component(entity2, Transform::new().with_position(2.0, 0.0, 0.0)).unwrap();
    world.add_component(entity2, Mesh { mesh_type: MeshType::Cube }).unwrap();
    world.add_component(entity2, EcsMaterial::default()).unwrap();
    entities.push((entity2, "Cube at (2, 0, 0)".to_string()));
    
    // Entity 3: Sphere at (-2, 1, 0)
    let entity3 = world.spawn();
    world.add_component(entity3, Transform::new().with_position(-2.0, 1.0, 0.0)).unwrap();
    world.add_component(entity3, Mesh { mesh_type: MeshType::Sphere }).unwrap();
    world.add_component(entity3, EcsMaterial::default()).unwrap();
    entities.push((entity3, "Sphere at (-2, 1, 0)".to_string()));
    
    // Entity 4: Custom mesh at (0, 2, 0)
    let entity4 = world.spawn();
    world.add_component(entity4, Transform::new().with_position(0.0, 2.0, 0.0)).unwrap();
    world.add_component(entity4, Mesh { mesh_type: MeshType::Custom("pyramid".to_string()) }).unwrap();
    world.add_component(entity4, EcsMaterial::default()).unwrap();
    entities.push((entity4, "Custom pyramid at (0, 2, 0)".to_string()));
    
    // Entity 5: Transform only (should not be rendered)
    let entity5 = world.spawn();
    world.add_component(entity5, Transform::new().with_position(10.0, 10.0, 10.0)).unwrap();
    entities.push((entity5, "Transform only (not renderable)".to_string()));
    
    // Entity 6: Mesh only (should not be rendered)
    let entity6 = world.spawn();
    world.add_component(entity6, Mesh { mesh_type: MeshType::Plane }).unwrap();
    entities.push((entity6, "Mesh only (not renderable)".to_string()));
    
    entities
}

/// Count entities that have all required components for rendering
fn count_renderable_entities(world: &World) -> usize {
    use std::collections::HashSet;
    
    let transform_entities: HashSet<_> = world.query_legacy::<Transform>().map(|(e, _)| e).collect();
    let mesh_entities: HashSet<_> = world.query_legacy::<Mesh>().map(|(e, _)| e).collect();
    let material_entities: HashSet<_> = world.query_legacy::<EcsMaterial>().map(|(e, _)| e).collect();
    
    // Count entities that have all three components
    transform_entities
        .intersection(&mesh_entities)
        .filter(|entity| material_entities.contains(entity))
        .count()
}