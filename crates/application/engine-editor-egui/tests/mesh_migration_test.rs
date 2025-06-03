// Tests for migrating from old Mesh component to new MeshFilter/MeshRenderer system

use engine_ecs_core::World;
use engine_components_3d::{Transform, Mesh, MeshType, MeshFilter, MeshRenderer, Material};
use engine_components_ui::Name;
use engine_geometry_core::{MeshData, Vertex};
use engine_resource_core::{ResourceId, ResourceHandle};
use glam::{Vec3, Vec2};

/// Test that entities with MeshFilter and MeshRenderer can be queried correctly
#[test]
fn test_query_entities_with_new_mesh_components() {
    let mut world = World::new();
    
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Name>();
    
    // Create entity with new mesh components
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();
    world.add_component(entity, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1)))).unwrap();
    world.add_component(entity, MeshRenderer::default()).unwrap();
    world.add_component(entity, Material::default()).unwrap();
    world.add_component(entity, Name::new("Test Cube")).unwrap();
    
    // Query entities with both MeshFilter and Transform
    let mut count = 0;
    for (e, _transform) in world.query_legacy::<Transform>() {
        if world.get_component::<MeshFilter>(e).is_some() {
            count += 1;
            
            // Verify all components exist
            assert!(world.get_component::<MeshRenderer>(e).is_some());
            assert!(world.get_component::<Material>(e).is_some());
            assert!(world.get_component::<Name>(e).is_some());
        }
    }
    
    assert_eq!(count, 1, "Should find exactly one entity with new mesh components");
}

/// Test that old Mesh component queries return empty when only new components exist
#[test]
fn test_old_mesh_queries_return_empty_with_new_components() {
    let mut world = World::new();
    
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Mesh>(); // Old component
    
    // Create entity with ONLY new mesh components
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();
    world.add_component(entity, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1)))).unwrap();
    world.add_component(entity, MeshRenderer::default()).unwrap();
    
    // Query for old Mesh component should return nothing
    let old_mesh_count = world.query_legacy::<Mesh>().count();
    assert_eq!(old_mesh_count, 0, "Should find no entities with old Mesh component");
    
    // Query for new components should find the entity
    let mut new_mesh_count = 0;
    for (e, _) in world.query_legacy::<Transform>() {
        if world.get_component::<MeshFilter>(e).is_some() {
            new_mesh_count += 1;
        }
    }
    assert_eq!(new_mesh_count, 1, "Should find entity with new mesh components");
}

/// Test multiple entities with different mesh types using new system
#[test]
fn test_multiple_entities_with_different_mesh_types() {
    let mut world = World::new();
    
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Name>();
    
    // Create cube
    let cube = world.spawn();
    world.add_component(cube, Transform::default()).unwrap();
    world.add_component(cube, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1)))).unwrap();
    world.add_component(cube, MeshRenderer::default()).unwrap();
    world.add_component(cube, Material { color: [1.0, 0.0, 0.0, 1.0], ..Default::default() }).unwrap();
    world.add_component(cube, Name::new("Cube")).unwrap();
    
    // Create sphere
    let sphere = world.spawn();
    world.add_component(sphere, Transform::default()).unwrap();
    world.add_component(sphere, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2)))).unwrap();
    world.add_component(sphere, MeshRenderer::default()).unwrap();
    world.add_component(sphere, Material { color: [0.0, 1.0, 0.0, 1.0], ..Default::default() }).unwrap();
    world.add_component(sphere, Name::new("Sphere")).unwrap();
    
    // Create plane
    let plane = world.spawn();
    world.add_component(plane, Transform::default()).unwrap();
    world.add_component(plane, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(3)))).unwrap();
    world.add_component(plane, MeshRenderer::default()).unwrap();
    world.add_component(plane, Material { color: [0.0, 0.0, 1.0, 1.0], ..Default::default() }).unwrap();
    world.add_component(plane, Name::new("Plane")).unwrap();
    
    // Count entities with mesh components
    let mut mesh_entities = Vec::new();
    for (entity, _) in world.query_legacy::<Transform>() {
        if world.get_component::<MeshFilter>(entity).is_some() {
            let name = world.get_component::<Name>(entity).unwrap();
            let material = world.get_component::<Material>(entity).unwrap();
            mesh_entities.push((name.name.clone(), material.color));
        }
    }
    
    assert_eq!(mesh_entities.len(), 3, "Should have 3 mesh entities");
    assert!(mesh_entities.iter().any(|(name, _)| name == "Cube"));
    assert!(mesh_entities.iter().any(|(name, _)| name == "Sphere"));
    assert!(mesh_entities.iter().any(|(name, _)| name == "Plane"));
}

/// Test that renderer can find entities by querying MeshFilter
#[test]
fn test_renderer_query_pattern() {
    let mut world = World::new();
    
    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    
    // Create multiple entities with different component combinations
    // Entity 1: Has all components
    let e1 = world.spawn();
    world.add_component(e1, Transform::default()).unwrap();
    world.add_component(e1, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1)))).unwrap();
    world.add_component(e1, MeshRenderer::default()).unwrap();
    world.add_component(e1, Material::default()).unwrap();
    
    // Entity 2: Missing MeshRenderer (should not render)
    let e2 = world.spawn();
    world.add_component(e2, Transform::default()).unwrap();
    world.add_component(e2, MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2)))).unwrap();
    world.add_component(e2, Material::default()).unwrap();
    
    // Entity 3: Missing MeshFilter (should not render)
    let e3 = world.spawn();
    world.add_component(e3, Transform::default()).unwrap();
    world.add_component(e3, MeshRenderer::default()).unwrap();
    world.add_component(e3, Material::default()).unwrap();
    
    // Entity 4: Only Transform (should not render)
    let e4 = world.spawn();
    world.add_component(e4, Transform::default()).unwrap();
    
    // Simulate renderer query pattern
    let mut renderable_entities = Vec::new();
    for (entity, _transform) in world.query_legacy::<Transform>() {
        if let Some(mesh_filter) = world.get_component::<MeshFilter>(entity) {
            if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                if mesh_renderer.enabled {
                    renderable_entities.push(entity);
                }
            }
        }
    }
    
    assert_eq!(renderable_entities.len(), 1, "Only one entity should be renderable");
    assert_eq!(renderable_entities[0], e1, "Entity 1 should be the renderable one");
}

/// Test resource handle functionality for mesh data
#[test]
fn test_mesh_resource_handles() {
    let mut world = World::new();
    
    engine_ecs_core::register_component::<MeshFilter>();
    
    // Create entities with different mesh handles
    let e1 = world.spawn();
    let handle1 = ResourceHandle::<MeshData>::new(ResourceId::new(100));
    world.add_component(e1, MeshFilter::new(handle1.clone())).unwrap();
    
    let e2 = world.spawn();
    let handle2 = ResourceHandle::<MeshData>::new(ResourceId::new(200));
    world.add_component(e2, MeshFilter::new(handle2.clone())).unwrap();
    
    // Verify handles are different
    let filter1 = world.get_component::<MeshFilter>(e1).unwrap();
    let filter2 = world.get_component::<MeshFilter>(e2).unwrap();
    
    assert_ne!(filter1.mesh.id(), filter2.mesh.id(), "Mesh handles should be different");
    assert_eq!(filter1.mesh.id(), handle1.id(), "Handle 1 should match");
    assert_eq!(filter2.mesh.id(), handle2.id(), "Handle 2 should match");
}