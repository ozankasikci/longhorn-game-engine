// Integration test for the new mesh system without old Mesh component

use engine_components_3d::{Material, MeshFilter, MeshRenderer, Transform, Visibility};
use engine_components_ui::Name;
use engine_ecs_core::World;
use engine_geometry_core::{MeshData, Vertex};
use engine_resource_core::{ResourceHandle, ResourceId};
use glam::{Vec2, Vec3};

/// Test that entities with new mesh components render correctly
#[test]
fn test_entities_render_with_new_mesh_system() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Visibility>();
    engine_ecs_core::register_component::<Name>();

    // Create a cube entity
    let cube = world.spawn();
    world
        .add_component(
            cube,
            Transform {
                position: [0.0, 0.5, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
        )
        .unwrap();

    // Add MeshFilter with cube mesh handle
    let cube_mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(1));
    world
        .add_component(cube, MeshFilter::new(cube_mesh_handle))
        .unwrap();

    // Add MeshRenderer
    world.add_component(cube, MeshRenderer::default()).unwrap();

    // Add Material
    world
        .add_component(
            cube,
            Material {
                color: [1.0, 0.0, 0.0, 1.0], // Red
                metallic: 0.0,
                roughness: 0.5,
                emissive: [0.0, 0.0, 0.0],
            },
        )
        .unwrap();

    world.add_component(cube, Visibility::default()).unwrap();
    world.add_component(cube, Name::new("Red Cube")).unwrap();

    // Create a sphere entity
    let sphere = world.spawn();
    world
        .add_component(
            sphere,
            Transform {
                position: [2.0, 0.5, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
        )
        .unwrap();

    // Add MeshFilter with sphere mesh handle
    let sphere_mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(2));
    world
        .add_component(sphere, MeshFilter::new(sphere_mesh_handle))
        .unwrap();

    // Add MeshRenderer with shadows disabled
    world
        .add_component(
            sphere,
            MeshRenderer {
                cast_shadows: false,
                receive_shadows: false,
                ..Default::default()
            },
        )
        .unwrap();

    // Add Material
    world
        .add_component(
            sphere,
            Material {
                color: [0.0, 1.0, 0.0, 1.0], // Green
                metallic: 0.5,
                roughness: 0.2,
                emissive: [0.0, 0.0, 0.0],
            },
        )
        .unwrap();

    world.add_component(sphere, Visibility::default()).unwrap();
    world
        .add_component(sphere, Name::new("Green Sphere"))
        .unwrap();

    // Query all renderable entities
    let mut renderable_count = 0;
    for (entity, transform) in world.query_legacy::<Transform>() {
        if let Some(mesh_filter) = world.get_component::<MeshFilter>(entity) {
            if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                if mesh_renderer.enabled {
                    renderable_count += 1;

                    // Verify all rendering components exist
                    assert!(
                        world.get_component::<Material>(entity).is_some(),
                        "Material component missing"
                    );
                    assert!(
                        world.get_component::<Visibility>(entity).is_some(),
                        "Visibility component missing"
                    );

                    let name = world.get_component::<Name>(entity).unwrap();
                    let material = world.get_component::<Material>(entity).unwrap();

                    match name.name.as_str() {
                        "Red Cube" => {
                            assert_eq!(transform.position, [0.0, 0.5, 0.0]);
                            assert_eq!(material.color, [1.0, 0.0, 0.0, 1.0]);
                            assert!(mesh_renderer.cast_shadows);
                        }
                        "Green Sphere" => {
                            assert_eq!(transform.position, [2.0, 0.5, 0.0]);
                            assert_eq!(material.color, [0.0, 1.0, 0.0, 1.0]);
                            assert!(!mesh_renderer.cast_shadows);
                        }
                        _ => panic!("Unexpected entity: {}", name.name),
                    }
                }
            }
        }
    }

    assert_eq!(renderable_count, 2, "Should have 2 renderable entities");
}

/// Test disabled entities are not rendered
#[test]
fn test_disabled_entities_not_rendered() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Visibility>();

    // Create enabled entity
    let enabled = world.spawn();
    world.add_component(enabled, Transform::default()).unwrap();
    world
        .add_component(
            enabled,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1))),
        )
        .unwrap();
    world
        .add_component(
            enabled,
            MeshRenderer {
                enabled: true,
                ..Default::default()
            },
        )
        .unwrap();

    // Create disabled entity
    let disabled = world.spawn();
    world.add_component(disabled, Transform::default()).unwrap();
    world
        .add_component(
            disabled,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2))),
        )
        .unwrap();
    world
        .add_component(
            disabled,
            MeshRenderer {
                enabled: false,
                ..Default::default()
            },
        )
        .unwrap();

    // Create invisible entity (Visibility component)
    let invisible = world.spawn();
    world
        .add_component(invisible, Transform::default())
        .unwrap();
    world
        .add_component(
            invisible,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(3))),
        )
        .unwrap();
    world
        .add_component(invisible, MeshRenderer::default())
        .unwrap();
    world
        .add_component(invisible, Visibility { visible: false })
        .unwrap();

    // Count renderable entities
    let mut renderable_count = 0;
    for (entity, _) in world.query_legacy::<Transform>() {
        if let Some(_mesh_filter) = world.get_component::<MeshFilter>(entity) {
            if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                if mesh_renderer.enabled {
                    // Check visibility component
                    let is_visible = world
                        .get_component::<Visibility>(entity)
                        .map(|v| v.visible)
                        .unwrap_or(true); // Default to visible if no Visibility component

                    if is_visible {
                        renderable_count += 1;
                    }
                }
            }
        }
    }

    assert_eq!(renderable_count, 1, "Only 1 entity should be renderable");
}

/// Test layer mask filtering
#[test]
fn test_layer_mask_filtering() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();

    // Default layer entity
    let default_entity = world.spawn();
    world
        .add_component(default_entity, Transform::default())
        .unwrap();
    world
        .add_component(
            default_entity,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1))),
        )
        .unwrap();
    world
        .add_component(
            default_entity,
            MeshRenderer {
                layer_mask: 1, // Layer 0
                ..Default::default()
            },
        )
        .unwrap();

    // UI layer entity
    let ui_entity = world.spawn();
    world
        .add_component(ui_entity, Transform::default())
        .unwrap();
    world
        .add_component(
            ui_entity,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2))),
        )
        .unwrap();
    world
        .add_component(
            ui_entity,
            MeshRenderer {
                layer_mask: 32, // Layer 5 (UI layer)
                ..Default::default()
            },
        )
        .unwrap();

    // Water layer entity
    let water_entity = world.spawn();
    world
        .add_component(water_entity, Transform::default())
        .unwrap();
    world
        .add_component(
            water_entity,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(3))),
        )
        .unwrap();
    world
        .add_component(
            water_entity,
            MeshRenderer {
                layer_mask: 16, // Layer 4 (Water layer)
                ..Default::default()
            },
        )
        .unwrap();

    // Simulate camera culling mask (Default + Water layers)
    let camera_culling_mask = 1 | 16; // Layers 0 and 4

    let mut visible_count = 0;
    for (entity, _) in world.query_legacy::<Transform>() {
        if let Some(_mesh_filter) = world.get_component::<MeshFilter>(entity) {
            if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                if mesh_renderer.enabled && (mesh_renderer.layer_mask & camera_culling_mask) != 0 {
                    visible_count += 1;
                }
            }
        }
    }

    assert_eq!(
        visible_count, 2,
        "Should render 2 entities (default and water layers)"
    );
}
