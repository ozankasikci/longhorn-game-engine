//! Tests with real engine components to ensure migration works with actual types

use engine_components_3d::{Light, Material, Mesh, Transform, Visibility};
use engine_ecs_core::{register_component, World};
// TODO: Add these components when crates are available
// use engine_camera_core::Camera;
// use engine_components_ui::Name;

#[test]
fn test_real_components_migration() {
    // Register all component types
    register_component::<Transform>();
    register_component::<Mesh>();
    register_component::<Material>();
    register_component::<Visibility>();
    register_component::<Light>();
    // register_component::<Camera>();
    // register_component::<Name>();

    let mut world = World::new();

    // Create game object with transform
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();

    // Add mesh component (triggers migration)
    world.add_component(entity, Mesh::default()).unwrap();

    // Add material component (another migration)
    world.add_component(entity, Material::default()).unwrap();

    // Verify all components exist
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<Mesh>(entity).is_some());
    assert!(world.get_component::<Material>(entity).is_some());

    // Test removing a component
    world.remove_component::<Mesh>(entity).unwrap();
    assert!(world.get_component::<Mesh>(entity).is_none());
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<Material>(entity).is_some());
}

// #[test]
// fn test_camera_entity_creation() {
//     register_component::<Transform>();
//     // register_component::<Camera>();
//     // register_component::<Name>();
//
//     let mut world = World::new();
//
//     // Create camera dynamically
//     let camera = world.spawn();
//     world.add_component(camera, Transform::default()).unwrap();
//     world.add_component(camera, Camera::default()).unwrap();
//     world.add_component(camera, Name::new("Main Camera")).unwrap();
//
//     // Verify
//     let name = world.get_component::<Name>(camera).unwrap();
//     assert_eq!(name.name, "Main Camera");
// }

#[test]
fn test_complex_entity_evolution() {
    register_component::<Transform>();
    register_component::<Mesh>();
    register_component::<Material>();
    register_component::<Light>();

    let mut world = World::new();

    // Start with just transform
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();

    // Add mesh and material
    world.add_component(entity, Mesh::default()).unwrap();
    world.add_component(entity, Material::default()).unwrap();

    // Remove mesh, add light (entity becomes a light source)
    world.remove_component::<Mesh>(entity).unwrap();
    world.add_component(entity, Light::default()).unwrap();

    // Verify final state
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<Mesh>(entity).is_none());
    assert!(world.get_component::<Material>(entity).is_some());
    assert!(world.get_component::<Light>(entity).is_some());
}
