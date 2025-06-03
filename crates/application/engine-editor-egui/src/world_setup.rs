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
    
    // SIMPLE TEST: Create one entity with just Transform
    let simple_entity = world.spawn_with(Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [2.0, 2.0, 2.0],
    });
    
    // Create camera entity - TRANSFORM ONLY for now
    let camera_entity = world.spawn_with(Transform {
        position: [0.0, 2.0, 8.0],  // Move camera back to see objects
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    
    // Create multiple 3D objects for camera rotation testing
    
    // BRIGHT GREEN CUBE - RIGHT IN FRONT OF CAMERA FOR VISIBILITY TEST
    let test_cube_entity = world.spawn_with(Transform {
        position: [0.0, 2.0, 5.0],  // Same Y as camera, 3 units in front
        rotation: [0.0, 0.0, 0.0],  // No rotation
        scale: [3.0, 3.0, 3.0],     // Very large and visible
    });
    
    // Red cube - TRANSFORM ONLY
    let cube_entity = world.spawn_with(Transform {
        position: [1.0, 2.0, 6.0],  // Test positive Z (should be in front)
        rotation: [0.0, 45.0, 0.0],
        scale: [2.0, 2.0, 2.0],  // Make bigger
    });
    
    // SKIP other complex entities for now due to migration issue
    /*
    
    // Blue cube elevated behind
    let blue_cube_entity = world.spawn_with(Transform {
        position: [0.0, 2.0, 3.0],  // Still in front but further away
        rotation: [15.0, 30.0, 0.0],
        scale: [0.8, 0.8, 0.8],
    });
    world.add_component(blue_cube_entity, Name::new("Blue Cube")).unwrap();
    world.add_component(blue_cube_entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    world.add_component(blue_cube_entity, Material {
        color: [0.2, 0.4, 0.9, 1.0], // Blue cube
        metallic: 0.3,
        roughness: 0.2,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(blue_cube_entity, Visibility::default()).unwrap();
    
    // Yellow sphere on the right side
    let yellow_sphere_entity = world.spawn_with(Transform {
        position: [3.5, 1.0, 4.0],  // Positive Z = in front of camera
        rotation: [0.0, 0.0, 0.0],
        scale: [0.7, 0.7, 0.7],
    });
    world.add_component(yellow_sphere_entity, Name::new("Yellow Sphere")).unwrap();
    world.add_component(yellow_sphere_entity, Mesh {
        mesh_type: MeshType::Sphere,
    }).unwrap();
    world.add_component(yellow_sphere_entity, Material {
        color: [0.9, 0.8, 0.1, 1.0], // Yellow sphere
        metallic: 0.0,
        roughness: 0.4,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(yellow_sphere_entity, Visibility::default()).unwrap();
    
    // Purple cube on the left side  
    let purple_cube_entity = world.spawn_with(Transform {
        position: [-3.0, 0.5, 6.0],  // Positive Z = in front of camera
        rotation: [0.0, -20.0, 10.0],
        scale: [1.3, 0.6, 1.3],
    });
    world.add_component(purple_cube_entity, Name::new("Purple Cube")).unwrap();
    world.add_component(purple_cube_entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    world.add_component(purple_cube_entity, Material {
        color: [0.7, 0.3, 0.8, 1.0], // Purple cube
        metallic: 0.2,
        roughness: 0.6,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(purple_cube_entity, Visibility::default()).unwrap();
    
    // Large ground plane
    let plane_entity = world.spawn_with(Transform {
        position: [0.0, -1.5, 4.0],  // Positive Z = in front of camera
        rotation: [0.0, 0.0, 0.0],
        scale: [8.0, 1.0, 8.0],
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
    
    // TEST CUBE - AT SAME POSITION AS CAMERA
    let center_cube_entity = world.spawn_with(Transform {
        position: [0.0, 2.0, 8.0],  // EXACTLY same as camera position
        rotation: [0.0, 0.0, 0.0],
        scale: [5.0, 5.0, 5.0],  // HUGE cube
    });
    world.add_component(center_cube_entity, Name::new("Center Reference")).unwrap();
    world.add_component(center_cube_entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    world.add_component(center_cube_entity, Material {
        color: [1.0, 0.5, 0.1, 1.0], // Orange cube
        metallic: 0.1,
        roughness: 0.3,
        emissive: [0.1, 0.05, 0.0], // Slight glow
    }).unwrap();
    world.add_component(center_cube_entity, Visibility::default()).unwrap();
    
    // Create sprite entities
    create_test_sprites(&mut world);
    
    // SIMPLE TEST: Create one guaranteed cube
    let simple_test_entity = world.spawn();
    world.add_component(simple_test_entity, Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    }).unwrap();
    world.add_component(simple_test_entity, Name::new("SIMPLE TEST CUBE")).unwrap();
    world.add_component(simple_test_entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    world.add_component(simple_test_entity, Material {
        color: [1.0, 0.0, 1.0, 1.0], // Magenta
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    world.add_component(simple_test_entity, Visibility::default()).unwrap();
    */
    
    messages.push(ConsoleMessage::info("⚠️ Component migration not implemented - using Transform-only entities"));
    messages.push(ConsoleMessage::info("🚀 ECS v2 World created with 4 Transform-only entities"));
    messages.push(ConsoleMessage::info("🎮 These should now be visible in queries"));
    
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