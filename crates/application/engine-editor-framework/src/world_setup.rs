// World initialization and default entity creation

use crate::types::{HierarchyObject, ObjectType};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_3d::{
    Camera, CameraMatrices, MainCamera, Material, Mesh, MeshFilter, MeshRenderer, MeshType,
    Transform, Visibility,
};
use engine_components_ui::Name;
use engine_ecs_core::{Entity, World, WorldBundleExt};
use engine_geometry_core::{MeshData, Vertex};
use engine_resource_core::{ResourceHandle, ResourceId};
use engine_scripting::components::{LuaScript, TypeScriptScript};
use glam::{Vec2, Vec3};

/// Creates a default world with sample entities for the editor
pub fn create_default_world() -> (World, Entity) {
    let mut world = World::new();

    // Register all component types
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<Camera>();
    engine_ecs_core::register_component::<MainCamera>();
    engine_ecs_core::register_component::<CameraMatrices>();
    engine_ecs_core::register_component::<Name>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Visibility>();
    engine_ecs_core::register_component::<Sprite>();
    engine_ecs_core::register_component::<SpriteRenderer>();
    engine_ecs_core::register_component::<MeshFilter>();
    engine_ecs_core::register_component::<MeshRenderer>();
    engine_ecs_core::register_component::<Mesh>();
    engine_ecs_core::register_component::<LuaScript>();
    engine_ecs_core::register_component::<TypeScriptScript>();

    // Create camera entity with bundle - SIMPLIFIED for coordinate system testing
    let camera_entity = world.spawn_bundle((
        Transform {
            position: [0.0, 2.0, 5.0], // Camera positioned behind cube, looking straight at it
            rotation: [0.0, 0.0, 0.0], // No rotation - looking straight down -Z axis
            scale: [1.0, 1.0, 1.0],
        },
        Camera::perspective(60.0, 0.1, 1000.0),
        Name::new("Main Camera"),
    ));

    // Add MainCamera tag to make it the main camera
    world.add_component(camera_entity, MainCamera).unwrap();

    // Create a cube with the new mesh component system
    let cube_entity = world.spawn();

    // Add transform
    world
        .add_component(
            cube_entity,
            Transform {
                position: [0.0, 0.5, 0.0], // At origin, slightly above ground
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
        )
        .unwrap();

    // Generate cube mesh data directly
    let _mesh_data = create_cube_mesh_data(1.0);

    // Create mesh handle (in a real system, this would be managed by a resource manager)
    let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(1));

    // Add Mesh component for rendering
    world
        .add_component(
            cube_entity,
            Mesh {
                mesh_type: MeshType::Cube,
            },
        )
        .unwrap();

    // Add MeshFilter component
    world
        .add_component(cube_entity, MeshFilter::new(mesh_handle))
        .unwrap();

    // Add MeshRenderer component with default material
    world
        .add_component(cube_entity, MeshRenderer::default())
        .unwrap();

    // Add material component
    world
        .add_component(
            cube_entity,
            Material {
                color: [0.8, 0.8, 0.8, 1.0], // Light gray cube
                metallic: 0.0,
                roughness: 0.5,
                emissive: [0.0, 0.0, 0.0],
            },
        )
        .unwrap();

    world
        .add_component(cube_entity, Visibility::default())
        .unwrap();
    world.add_component(cube_entity, Name::new("Cube")).unwrap();

    // Try to get all entities with Transform components
    let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().collect();

    for (entity, _transform) in entities_with_transforms.iter().take(5) {
        let _name = world
            .get_component::<Name>(*entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
    }

    // FINAL DEBUG: Verify entities exist
    let _final_count = world.entity_count();
    let _mesh_entities: Vec<_> = world
        .query_legacy::<Transform>()
        .filter(|(e, _)| world.get_component::<MeshFilter>(*e).is_some())
        .collect();

    (world, cube_entity)
}

/// Creates test sprite entities
fn _create_test_sprites(world: &mut World) {
    // Red sprite
    let red_sprite_entity = world.spawn();
    world
        .add_component(
            red_sprite_entity,
            Transform {
                position: [-2.0, 0.5, 4.5], // Positive Z = in front of camera
                rotation: [0.0, 0.0, 0.0],
                scale: [1.5, 1.5, 1.0],
            },
        )
        .unwrap();
    world
        .add_component(red_sprite_entity, Name::new("Red Sprite"))
        .unwrap();
    world
        .add_component(
            red_sprite_entity,
            SpriteRenderer {
                sprite: Sprite::new()
                    .with_texture(1001)
                    .with_color(1.0, 0.8, 0.8, 1.0),
                layer: 0,
                material_override: None,
                enabled: true,
            },
        )
        .unwrap();
    world
        .add_component(red_sprite_entity, Visibility::default())
        .unwrap();

    // Blue sprite
    let blue_sprite_entity = world.spawn();
    world
        .add_component(
            blue_sprite_entity,
            Transform {
                position: [2.0, 0.5, 4.5], // Positive Z = in front of camera
                rotation: [0.0, 0.0, 15.0],
                scale: [1.0, 2.0, 1.0],
            },
        )
        .unwrap();
    world
        .add_component(blue_sprite_entity, Name::new("Blue Sprite"))
        .unwrap();
    world
        .add_component(
            blue_sprite_entity,
            SpriteRenderer {
                sprite: Sprite::new().with_texture(1003),
                layer: 1,
                material_override: None,
                enabled: true,
            },
        )
        .unwrap();
    world
        .add_component(blue_sprite_entity, Visibility::default())
        .unwrap();

    // Yellow sprite
    let yellow_sprite_entity = world.spawn();
    world
        .add_component(
            yellow_sprite_entity,
            Transform {
                position: [0.0, 2.0, 4.5], // Positive Z = in front of camera
                rotation: [0.0, 0.0, 0.0],
                scale: [0.8, 0.8, 1.0],
            },
        )
        .unwrap();
    world
        .add_component(yellow_sprite_entity, Name::new("Yellow Sprite"))
        .unwrap();
    world
        .add_component(
            yellow_sprite_entity,
            SpriteRenderer {
                sprite: Sprite::new()
                    .with_texture(1004)
                    .with_color(1.0, 1.0, 0.5, 0.9),
                layer: 2,
                material_override: None,
                enabled: true,
            },
        )
        .unwrap();
    world
        .add_component(yellow_sprite_entity, Visibility::default())
        .unwrap();
}

/// Creates default hierarchy objects for the editor UI
pub fn create_default_hierarchy() -> Vec<HierarchyObject> {
    vec![
        HierarchyObject::new("Main Camera", ObjectType::Camera),
        HierarchyObject::new("Cube", ObjectType::GameObject),
    ]
}

/// Create cube mesh data
fn create_cube_mesh_data(size: f32) -> MeshData {
    let half_size = size * 0.5;
    let vertices = vec![
        // Front face
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
        // Back face
        Vertex::new(Vec3::new(half_size, -half_size, -half_size))
            .with_normal(Vec3::NEG_Z)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size))
            .with_normal(Vec3::NEG_Z)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size))
            .with_normal(Vec3::NEG_Z)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size))
            .with_normal(Vec3::NEG_Z)
            .with_uv(Vec2::new(0.0, 1.0)),
        // Left face
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size))
            .with_normal(Vec3::NEG_X)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, half_size))
            .with_normal(Vec3::NEG_X)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, half_size, half_size))
            .with_normal(Vec3::NEG_X)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size))
            .with_normal(Vec3::NEG_X)
            .with_uv(Vec2::new(0.0, 1.0)),
        // Right face
        Vertex::new(Vec3::new(half_size, -half_size, half_size))
            .with_normal(Vec3::X)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, -half_size))
            .with_normal(Vec3::X)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size))
            .with_normal(Vec3::X)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size))
            .with_normal(Vec3::X)
            .with_uv(Vec2::new(0.0, 1.0)),
        // Top face
        Vertex::new(Vec3::new(-half_size, half_size, half_size))
            .with_normal(Vec3::Y)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size))
            .with_normal(Vec3::Y)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size))
            .with_normal(Vec3::Y)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size))
            .with_normal(Vec3::Y)
            .with_uv(Vec2::new(0.0, 1.0)),
        // Bottom face
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size))
            .with_normal(Vec3::NEG_Y)
            .with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, -half_size))
            .with_normal(Vec3::NEG_Y)
            .with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, half_size))
            .with_normal(Vec3::NEG_Y)
            .with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, half_size))
            .with_normal(Vec3::NEG_Y)
            .with_uv(Vec2::new(0.0, 1.0)),
    ];

    let indices = vec![
        // Front face
        0, 1, 2, 0, 2, 3, // Back face
        4, 5, 6, 4, 6, 7, // Left face
        8, 9, 10, 8, 10, 11, // Right face
        12, 13, 14, 12, 14, 15, // Top face
        16, 17, 18, 16, 18, 19, // Bottom face
        20, 21, 22, 20, 22, 23,
    ];

    MeshData::new("Cube".to_string(), vertices, indices)
}
