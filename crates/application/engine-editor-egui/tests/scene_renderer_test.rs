// Tests for scene renderer using new mesh components

use engine_components_3d::{Material, MeshFilter, MeshRenderer, Transform};
use engine_components_ui::Name;
use engine_ecs_core::World;
use engine_geometry_core::{MeshData, Vertex};
use engine_resource_core::{ResourceHandle, ResourceId};
use glam::{Vec2, Vec3};

/// Helper function to create a simple cube mesh data
#[allow(dead_code)]
fn create_test_cube_mesh() -> MeshData {
    let half_size = 0.5;
    let vertices = vec![
        // Front face (simplified - just 4 vertices)
        Vertex::new(Vec3::new(-half_size, -half_size, half_size))
            .with_normal(Vec3::Z)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, half_size))
            .with_normal(Vec3::Z)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size))
            .with_normal(Vec3::Z)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, half_size))
            .with_normal(Vec3::Z)
            .with_uv(Vec2::new(0.0, 1.0)),
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    MeshData::new("TestCube".to_string(), vertices, indices)
}

/// Mock scene renderer that simulates the query pattern
struct MockSceneRenderer;

impl MockSceneRenderer {
    /// Simulates the scene renderer's entity collection logic
    fn collect_renderable_entities(
        world: &World,
    ) -> Vec<(engine_ecs_core::Entity, [f32; 3], ResourceId, [f32; 4])> {
        let mut renderable_entities = Vec::new();

        for (entity, transform) in world.query_legacy::<Transform>() {
            // Check for MeshFilter (contains mesh data reference)
            if let Some(mesh_filter) = world.get_component::<MeshFilter>(entity) {
                // Check for MeshRenderer (contains rendering settings)
                if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                    // Only render if enabled
                    if mesh_renderer.enabled {
                        // Get material or use default
                        let material = world
                            .get_component::<Material>(entity)
                            .cloned()
                            .unwrap_or_default();

                        renderable_entities.push((
                            entity,
                            transform.position,
                            mesh_filter.mesh.id(),
                            material.color,
                        ));
                    }
                }
            }
        }

        renderable_entities
    }
}

#[test]
fn test_scene_renderer_finds_new_mesh_entities() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Name>();

    // Create test entities
    let cube1 = world.spawn();
    world
        .add_component(
            cube1,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
        )
        .unwrap();
    world
        .add_component(
            cube1,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1))),
        )
        .unwrap();
    world.add_component(cube1, MeshRenderer::default()).unwrap();
    world
        .add_component(
            cube1,
            Material {
                color: [1.0, 0.0, 0.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();
    world.add_component(cube1, Name::new("Red Cube")).unwrap();

    // Create another cube
    let cube2 = world.spawn();
    world
        .add_component(
            cube2,
            Transform {
                position: [2.0, 0.0, 0.0],
                rotation: [0.0, 45.0, 0.0],
                scale: [0.5, 0.5, 0.5],
            },
        )
        .unwrap();
    world
        .add_component(
            cube2,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2))),
        )
        .unwrap();
    world.add_component(cube2, MeshRenderer::default()).unwrap();
    world
        .add_component(
            cube2,
            Material {
                color: [0.0, 1.0, 0.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();
    world.add_component(cube2, Name::new("Green Cube")).unwrap();

    // Create disabled entity (should not render)
    let disabled_cube = world.spawn();
    world
        .add_component(disabled_cube, Transform::default())
        .unwrap();
    world
        .add_component(
            disabled_cube,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(3))),
        )
        .unwrap();
    world
        .add_component(
            disabled_cube,
            MeshRenderer {
                enabled: false,
                ..Default::default()
            },
        )
        .unwrap();
    world
        .add_component(disabled_cube, Material::default())
        .unwrap();
    world
        .add_component(disabled_cube, Name::new("Disabled Cube"))
        .unwrap();

    // Collect renderable entities
    let renderable = MockSceneRenderer::collect_renderable_entities(&world);

    assert_eq!(renderable.len(), 2, "Should find 2 enabled mesh entities");

    // Verify entity properties
    for (entity, position, _mesh_id, color) in &renderable {
        let name = world.get_component::<Name>(*entity).unwrap();

        match name.name.as_str() {
            "Red Cube" => {
                assert_eq!(*position, [0.0, 0.0, 0.0]);
                assert_eq!(*color, [1.0, 0.0, 0.0, 1.0]);
            }
            "Green Cube" => {
                assert_eq!(*position, [2.0, 0.0, 0.0]);
                assert_eq!(*color, [0.0, 1.0, 0.0, 1.0]);
            }
            _ => panic!("Unexpected entity name: {}", name.name),
        }
    }
}

#[test]
fn test_scene_renderer_handles_missing_components() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();

    // Entity with all components (should render)
    let complete_entity = world.spawn();
    world
        .add_component(complete_entity, Transform::default())
        .unwrap();
    world
        .add_component(
            complete_entity,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1))),
        )
        .unwrap();
    world
        .add_component(complete_entity, MeshRenderer::default())
        .unwrap();

    // Entity missing MeshRenderer (should not render)
    let no_renderer = world.spawn();
    world
        .add_component(no_renderer, Transform::default())
        .unwrap();
    world
        .add_component(
            no_renderer,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2))),
        )
        .unwrap();

    // Entity missing MeshFilter (should not render)
    let no_filter = world.spawn();
    world
        .add_component(no_filter, Transform::default())
        .unwrap();
    world
        .add_component(no_filter, MeshRenderer::default())
        .unwrap();

    // Entity with only Transform (should not render)
    let transform_only = world.spawn();
    world
        .add_component(transform_only, Transform::default())
        .unwrap();

    // Collect renderable entities
    let renderable = MockSceneRenderer::collect_renderable_entities(&world);

    assert_eq!(renderable.len(), 1, "Should find only 1 complete entity");
    assert_eq!(
        renderable[0].0, complete_entity,
        "Should be the complete entity"
    );
}

#[test]
fn test_scene_renderer_respects_layer_mask() {
    let mut world = World::new();

    // Register components
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Material>();

    // Entity on default layer
    let default_layer = world.spawn();
    world
        .add_component(default_layer, Transform::default())
        .unwrap();
    world
        .add_component(
            default_layer,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(1))),
        )
        .unwrap();
    world
        .add_component(
            default_layer,
            MeshRenderer {
                layer_mask: 1, // Default layer
                enabled: true,
                ..Default::default()
            },
        )
        .unwrap();

    // Entity on UI layer
    let ui_layer = world.spawn();
    world.add_component(ui_layer, Transform::default()).unwrap();
    world
        .add_component(
            ui_layer,
            MeshFilter::new(ResourceHandle::<MeshData>::new(ResourceId::new(2))),
        )
        .unwrap();
    world
        .add_component(
            ui_layer,
            MeshRenderer {
                layer_mask: 32, // UI layer (bit 5)
                enabled: true,
                ..Default::default()
            },
        )
        .unwrap();

    // Simulate filtering by layer
    let mut default_layer_entities = Vec::new();
    let mut ui_layer_entities = Vec::new();

    for (entity, _) in world.query_legacy::<Transform>() {
        if let Some(_mesh_filter) = world.get_component::<MeshFilter>(entity) {
            if let Some(mesh_renderer) = world.get_component::<MeshRenderer>(entity) {
                if mesh_renderer.enabled {
                    if mesh_renderer.layer_mask & 1 != 0 {
                        default_layer_entities.push(entity);
                    }
                    if mesh_renderer.layer_mask & 32 != 0 {
                        ui_layer_entities.push(entity);
                    }
                }
            }
        }
    }

    assert_eq!(
        default_layer_entities.len(),
        1,
        "Should have 1 entity on default layer"
    );
    assert_eq!(
        ui_layer_entities.len(),
        1,
        "Should have 1 entity on UI layer"
    );
}
