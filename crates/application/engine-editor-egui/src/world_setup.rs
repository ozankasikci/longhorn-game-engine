// World initialization and default entity creation

use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Visibility, Material, Mesh, MeshType, GameObject3DBundle};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::Name;
use engine_camera::Camera;
use engine_component_traits::{Bundle, ComponentClone};
use crate::editor_state::ConsoleMessage;

/// Bundle for camera entities
pub struct CameraBundle {
    pub transform: Transform,
    pub camera: Camera,
    pub name: Name,
}

impl Bundle for CameraBundle {
    fn component_ids() -> Vec<std::any::TypeId> where Self: Sized {
        vec![
            std::any::TypeId::of::<Transform>(),
            std::any::TypeId::of::<Camera>(),
            std::any::TypeId::of::<Name>(),
        ]
    }
    
    fn into_components(self) -> Vec<(std::any::TypeId, Box<dyn ComponentClone>)> {
        vec![
            (std::any::TypeId::of::<Transform>(), Box::new(self.transform)),
            (std::any::TypeId::of::<Camera>(), Box::new(self.camera)),
            (std::any::TypeId::of::<Name>(), Box::new(self.name)),
        ]
    }
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            camera: Camera::default(),
            name: Name::new("Camera"),
        }
    }
}

/// Creates a default world with sample entities for the editor
pub fn create_default_world() -> (World, Entity, Vec<ConsoleMessage>) {
    let mut world = World::new();
    let mut messages = Vec::new();
    
    // Register all component types
    engine_ecs_core::register_component::<Transform>();
    engine_ecs_core::register_component::<Camera>();
    engine_ecs_core::register_component::<Name>();
    engine_ecs_core::register_component::<Mesh>();
    engine_ecs_core::register_component::<Material>();
    engine_ecs_core::register_component::<Visibility>();
    engine_ecs_core::register_component::<Sprite>();
    engine_ecs_core::register_component::<SpriteRenderer>();
    
    messages.push(ConsoleMessage::info("📝 Registered all component types"));
    
    // Create camera entity with bundle - SIMPLIFIED for coordinate system testing
    let camera_entity = world.spawn_bundle(CameraBundle {
        transform: Transform {
            position: [0.0, 2.0, 5.0],  // Camera positioned behind cube, looking straight at it
            rotation: [0.0, 0.0, 0.0],  // No rotation - looking straight down -Z axis
            scale: [1.0, 1.0, 1.0],
        },
        camera: Camera::default(),
        name: Name::new("Main Camera"),
    }).expect("Failed to create camera entity");
    
    messages.push(ConsoleMessage::info("✅ Created camera with bundle"));
    
    // Create a single cube in front of the camera
    let _cube_entity = world.spawn_bundle(GameObject3DBundle {
        transform: Transform {
            position: [0.0, 0.5, 0.0],  // At origin, slightly above ground
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        },
        mesh: Mesh {
            mesh_type: MeshType::Cube,
        },
        material: Material {
            color: [0.8, 0.8, 0.8, 1.0], // Light gray cube
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        },
        visibility: Visibility::default(),
    }).expect("Failed to create cube");
    
    world.add_component(_cube_entity, Name::new("Cube")).unwrap();
    
    messages.push(ConsoleMessage::info("✅ Created single cube object"));
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
        HierarchyObject::new("📦 Cube", ObjectType::GameObject),
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