// World initialization and default entity creation

use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Visibility, Material, Mesh, MeshType, GameObject3DBundle};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::{Camera, CameraBundle};
use crate::editor_state::ConsoleMessage;

/// Creates a default world with sample entities for the editor
pub fn create_default_world() -> (World, Entity, Vec<ConsoleMessage>) {
    let mut world = World::new();
    let mut messages = Vec::new();
    
    // Create camera entity with bundle
    let camera_entity = world.spawn_bundle(CameraBundle {
        transform: Transform {
            position: [5.0, 5.0, 15.0],  // Move camera back and up for better view
            rotation: [-0.2, -0.3, 0.0],  // Slight downward and leftward angle
            scale: [1.0, 1.0, 1.0],
        },
        camera: Camera::default(),
        name: Name::new("Main Camera"),
    }).expect("Failed to create camera entity");
    
    messages.push(ConsoleMessage::info("✅ Created camera with bundle"));
    
    // Create multiple 3D objects for camera rotation testing
    
    // BRIGHT GREEN CUBE - RIGHT IN FRONT OF CAMERA FOR VISIBILITY TEST
    let _test_cube_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [0.0, 2.0, 5.0],  // Same Y as camera, 3 units in front
            rotation: [0.0, 0.0, 0.0],  // No rotation
            scale: [3.0, 3.0, 3.0],     // Very large and visible
        },
        mesh: Mesh {
            mesh_type: MeshType::Cube,
        },
        material: Material {
            color: [0.0, 1.0, 0.0, 1.0], // Bright green - very visible
            metallic: 0.0,
            roughness: 0.3,
            emissive: [0.1, 0.3, 0.1],   // Slight green glow
        },
        visibility: Visibility::default(),
    }).expect("Failed to create green cube");
    
    // Red cube
    let _red_cube_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [1.0, 2.0, 6.0],  // Test positive Z (should be in front)
            rotation: [0.0, 45.0, 0.0],
            scale: [2.0, 2.0, 2.0],  // Make bigger
        },
        mesh: Mesh {
            mesh_type: MeshType::Cube,
        },
        material: Material {
            color: [0.8, 0.2, 0.2, 1.0], // Red cube
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create red cube");
    
    // Green sphere
    let _green_sphere_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [-2.0, 0.5, 5.0],  // Positive Z = in front of camera
            rotation: [0.0, 0.0, 0.0],
            scale: [1.2, 1.2, 1.2],
        },
        mesh: Mesh {
            mesh_type: MeshType::Sphere,
        },
        material: Material {
            color: [0.2, 0.8, 0.2, 1.0], // Green sphere
            metallic: 0.1,
            roughness: 0.3,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create green sphere");
    
    // Blue cube
    let _blue_cube_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [0.0, 2.0, 3.0],  // Still in front but further away
            rotation: [15.0, 30.0, 0.0],
            scale: [0.8, 0.8, 0.8],
        },
        mesh: Mesh {
            mesh_type: MeshType::Cube,
        },
        material: Material {
            color: [0.2, 0.4, 0.9, 1.0], // Blue cube
            metallic: 0.3,
            roughness: 0.2,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create blue cube");
    
    // Yellow sphere on the right side
    let _yellow_sphere_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [3.5, 1.0, 4.0],  // Positive Z = in front of camera
            rotation: [0.0, 0.0, 0.0],
            scale: [0.7, 0.7, 0.7],
        },
        mesh: Mesh {
            mesh_type: MeshType::Sphere,
        },
        material: Material {
            color: [0.9, 0.8, 0.1, 1.0], // Yellow sphere
            metallic: 0.0,
            roughness: 0.4,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create yellow sphere");
    
    // Large ground plane
    let _plane_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [0.0, -1.5, 4.0],  // Positive Z = in front of camera
            rotation: [0.0, 0.0, 0.0],
            scale: [8.0, 1.0, 8.0],
        },
        mesh: Mesh {
            mesh_type: MeshType::Plane,
        },
        material: Material {
            color: [0.6, 0.6, 0.6, 1.0], // Gray ground
            metallic: 0.0,
            roughness: 0.8,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create ground plane");
    
    messages.push(ConsoleMessage::info("✅ Created 6 3D objects using bundles"));
    messages.push(ConsoleMessage::info("🚀 ECS v2 World with proper multi-component entities!"));
    messages.push(ConsoleMessage::info("🎮 Objects should now render with actual meshes"));
    
    // DEBUG: Force log all entities that were just created
    messages.push(ConsoleMessage::info(&format!("🔍 DEBUG: World has {} entities total", world.entity_count())));
    
    // Try to get all entities with Transform components
    let entities_with_transforms: Vec<_> = world.query_legacy::<Transform>().collect();
    messages.push(ConsoleMessage::info(&format!("🔍 DEBUG: Found {} entities with Transform", entities_with_transforms.len())));
    
    for (entity, transform) in entities_with_transforms.iter().take(5) {
        let name = world.get_component::<engine_components_ui::Name>(*entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
        messages.push(ConsoleMessage::info(&format!(
            "  📦 Entity: {} at [{:.1}, {:.1}, {:.1}]",
            name, transform.position[0], transform.position[1], transform.position[2]
        )));
    }
    
    // FINAL DEBUG: Verify entities exist
    let final_count = world.entity_count();
    let mesh_entities: Vec<_> = world.query_legacy::<Transform>()
        .filter(|(e, _)| world.get_component::<Mesh>(*e).is_some())
        .collect();
    
    messages.push(ConsoleMessage::info(&format!(
        "🎯 WORLD SETUP COMPLETE: {} total entities, {} with mesh components",
        final_count, mesh_entities.len()
    )));
    
    (world, camera_entity, messages)
}

/// Creates test sprite entities
fn create_test_sprites(world: &mut World) {
    // Red sprite
    let red_sprite_entity = world.spawn_with(Transform {
        position: [-2.0, 0.5, 4.5],  // Positive Z = in front of camera
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
        position: [2.0, 0.5, 4.5],  // Positive Z = in front of camera
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
        position: [0.0, 2.0, 4.5],  // Positive Z = in front of camera
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
        HierarchyObject::parent("📦 3D Objects", vec![
            HierarchyObject::new("🟢 BRIGHT GREEN TEST CUBE", ObjectType::GameObject),
            HierarchyObject::new("🔴 Red Cube", ObjectType::GameObject),
            HierarchyObject::new("🟢 Green Sphere", ObjectType::GameObject),
            HierarchyObject::new("🔵 Blue Cube", ObjectType::GameObject),
            HierarchyObject::new("🟡 Yellow Sphere", ObjectType::GameObject),
            HierarchyObject::new("🟣 Purple Cube", ObjectType::GameObject),
            HierarchyObject::new("🟠 Center Reference", ObjectType::GameObject),
            HierarchyObject::new("⬜ Ground Plane", ObjectType::GameObject),
        ]),
        HierarchyObject::parent("🎨 Sprites", vec![
            HierarchyObject::new("🔴 Red Sprite", ObjectType::GameObject),
            HierarchyObject::new("🔵 Blue Sprite", ObjectType::GameObject),
            HierarchyObject::new("🟡 Yellow Sprite", ObjectType::GameObject),
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