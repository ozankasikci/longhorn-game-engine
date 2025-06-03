// World initialization and default entity creation

use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Visibility, Material, Mesh, MeshType};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::Camera;
use crate::editor_state::ConsoleMessage;

/// Creates a default world with sample entities for the editor
pub fn create_default_world() -> (World, Entity, Vec<ConsoleMessage>) {
    let mut world = World::new();
    let mut messages = Vec::new();
    
    // Create camera entity
    let camera_entity = world.spawn_with(Transform {
        position: [0.0, 0.0, 5.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.add_component(camera_entity, Name::new("Main Camera")).unwrap();
    world.add_component(camera_entity, Camera::default()).unwrap();
    
    // Create cube entity with mesh and material
    let cube_entity = world.spawn_with(Transform {
        position: [1.0, 0.0, 0.0],
        rotation: [0.0, 45.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.add_component(cube_entity, Name::new("Cube")).unwrap();
    world.add_component(cube_entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    world.add_component(cube_entity, Material {
        color: [0.8, 0.2, 0.2, 1.0], // Red cube
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(cube_entity, Visibility::default()).unwrap();
    
    // Create sphere entity with mesh and material
    let sphere_entity = world.spawn_with(Transform {
        position: [-1.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.5, 1.5, 1.5],
    });
    world.add_component(sphere_entity, Name::new("Sphere")).unwrap();
    world.add_component(sphere_entity, Mesh {
        mesh_type: MeshType::Sphere,
    }).unwrap();
    world.add_component(sphere_entity, Material {
        color: [0.2, 0.8, 0.2, 1.0], // Green sphere
        metallic: 0.1,
        roughness: 0.3,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(sphere_entity, Visibility::default()).unwrap();
    
    // Create plane entity (ground)
    let plane_entity = world.spawn_with(Transform {
        position: [0.0, -1.5, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [5.0, 1.0, 5.0],
    });
    world.add_component(plane_entity, Name::new("Ground Plane")).unwrap();
    world.add_component(plane_entity, Mesh {
        mesh_type: MeshType::Plane,
    }).unwrap();
    world.add_component(plane_entity, Material {
        color: [0.6, 0.6, 0.6, 1.0], // Gray ground
        metallic: 0.0,
        roughness: 0.8,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(plane_entity, Visibility::default()).unwrap();
    
    // Create sprite entities
    create_test_sprites(&mut world);
    
    messages.push(ConsoleMessage::info("🚀 ECS v2 World created with default entities!"));
    
    (world, camera_entity, messages)
}

/// Creates test sprite entities
fn create_test_sprites(world: &mut World) {
    // Red sprite
    let red_sprite_entity = world.spawn_with(Transform {
        position: [-2.0, 0.5, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.5, 1.5, 1.0],
    });
    world.add_component(red_sprite_entity, Name::new("Red Sprite")).unwrap();
    world.add_component(red_sprite_entity, SpriteRenderer {
        sprite: Sprite::new().with_texture(1001).with_color(1.0, 0.8, 0.8, 1.0),
        layer: 0,
        material_override: None,
        enabled: true,
    }).unwrap();
    world.add_component(red_sprite_entity, Visibility::default()).unwrap();
    
    // Blue sprite
    let blue_sprite_entity = world.spawn_with(Transform {
        position: [2.0, 0.5, 0.0],
        rotation: [0.0, 0.0, 15.0],
        scale: [1.0, 2.0, 1.0],
    });
    world.add_component(blue_sprite_entity, Name::new("Blue Sprite")).unwrap();
    world.add_component(blue_sprite_entity, SpriteRenderer {
        sprite: Sprite::new().with_texture(1003),
        layer: 1,
        material_override: None,
        enabled: true,
    }).unwrap();
    world.add_component(blue_sprite_entity, Visibility::default()).unwrap();
    
    // Yellow sprite
    let yellow_sprite_entity = world.spawn_with(Transform {
        position: [0.0, 2.0, -1.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [0.8, 0.8, 1.0],
    });
    world.add_component(yellow_sprite_entity, Name::new("Yellow Sprite")).unwrap();
    world.add_component(yellow_sprite_entity, SpriteRenderer {
        sprite: Sprite::new().with_texture(1004).with_color(1.0, 1.0, 0.5, 0.9),
        layer: 2,
        material_override: None,
        enabled: true,
    }).unwrap();
    world.add_component(yellow_sprite_entity, Visibility::default()).unwrap();
}

/// Creates default hierarchy objects for the editor UI
pub fn create_default_hierarchy() -> Vec<crate::types::HierarchyObject> {
    use crate::types::{HierarchyObject, ObjectType};
    
    vec![
        HierarchyObject::new("📱 Main Camera", ObjectType::Camera),
        HierarchyObject::new("☀️ Directional Light", ObjectType::Light),
        HierarchyObject::parent("📦 Game Objects", vec![
            HierarchyObject::new("🧊 Cube", ObjectType::GameObject),
            HierarchyObject::new("⚽ Sphere", ObjectType::GameObject),
            HierarchyObject::new("🔺 Plane", ObjectType::GameObject),
        ]),
    ]
}

/// Creates default project assets for the editor UI
pub fn create_default_project_assets() -> Vec<crate::types::ProjectAsset> {
    use crate::types::ProjectAsset;
    
    vec![
        ProjectAsset::folder("📁 Scripts", vec![
            ProjectAsset::file("📄 PlayerController.cs"),
            ProjectAsset::file("📄 GameManager.cs"),
            ProjectAsset::file("📄 UIController.cs"),
        ]),
        ProjectAsset::folder("📁 Materials", vec![
            ProjectAsset::file("🎨 DefaultMaterial.mat"),
            ProjectAsset::file("🎨 WoodTexture.mat"),
            ProjectAsset::file("🎨 MetalSurface.mat"),
        ]),
        ProjectAsset::folder("📁 Textures", vec![
            ProjectAsset::file("🖼️ grass.png"),
            ProjectAsset::file("🖼️ brick_wall.jpg"),
            ProjectAsset::file("🖼️ sky_gradient.png"),
        ]),
    ]
}