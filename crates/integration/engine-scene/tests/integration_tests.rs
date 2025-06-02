//! Integration tests for engine-scene

use engine_scene::*;
use engine_materials_core::Color;
use glam::{Vec3, Quat, Mat4};

#[test]
fn test_transform_creation_and_operations() {
    // Test default transform
    let default_transform = Transform::default();
    assert_eq!(default_transform.position, Vec3::ZERO);
    assert_eq!(default_transform.rotation, Quat::IDENTITY);
    assert_eq!(default_transform.scale, Vec3::ONE);

    // Test transform creation methods
    let position_transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(position_transform.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(position_transform.scale, Vec3::ONE);

    let scale_transform = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
    assert_eq!(scale_transform.scale, Vec3::new(2.0, 2.0, 2.0));

    let uniform_scale_transform = Transform::from_uniform_scale(3.0);
    assert_eq!(uniform_scale_transform.scale, Vec3::splat(3.0));

    // Test builder pattern
    let complex_transform = Transform::new()
        .with_position(Vec3::new(5.0, 10.0, 15.0))
        .with_scale(Vec3::new(2.0, 3.0, 4.0));
    
    assert_eq!(complex_transform.position, Vec3::new(5.0, 10.0, 15.0));
    assert_eq!(complex_transform.scale, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn test_transform_operations() {
    let mut transform = Transform::new();

    // Test translation
    transform.translate(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));

    // Test rotation
    transform.rotate_y(std::f32::consts::PI / 2.0); // 90 degrees
    
    // Test scaling
    transform.scale_uniform(2.0);
    assert_eq!(transform.scale, Vec3::splat(2.0));

    transform.scale_by(Vec3::new(0.5, 1.0, 1.5));
    assert_eq!(transform.scale, Vec3::new(1.0, 2.0, 3.0));

    // Test directional methods
    let forward = transform.forward();
    let right = transform.right();
    let up = transform.up();
    
    // These should be approximately orthogonal unit vectors
    assert!((forward.length() - 1.0).abs() < 0.001);
    assert!((right.length() - 1.0).abs() < 0.001);
    assert!((up.length() - 1.0).abs() < 0.001);
}

#[test]
fn test_transform_matrix_operations() {
    let transform = Transform::new()
        .with_position(Vec3::new(1.0, 2.0, 3.0))
        .with_scale(Vec3::new(2.0, 2.0, 2.0));

    // Test matrix calculation
    let matrix = transform.matrix();
    let inverse = transform.inverse_matrix();
    
    // Test that matrix * inverse ≈ identity
    let identity_test = matrix * inverse;
    let identity = Mat4::IDENTITY;
    
    // Check that the result is close to identity (allowing for floating point errors)
    for i in 0..4 {
        for j in 0..4 {
            let diff = (identity_test.col(i)[j] - identity.col(i)[j]).abs();
            assert!(diff < 0.001, "Matrix inverse test failed at ({}, {}): diff = {}", i, j, diff);
        }
    }

    // Test point transformation
    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed_point = transform.transform_point(point);
    
    // Point should be scaled by 2 and translated
    let expected = Vec3::new(3.0, 2.0, 3.0); // (1*2 + 1, 0*2 + 2, 0*2 + 3)
    assert!((transformed_point - expected).length() < 0.001);
}

#[test]
fn test_scene_node_creation_and_hierarchy() {
    let mut hierarchy = NodeHierarchy::new();

    // Create root node
    let root_node = SceneNode::new("Root");
    let root_id = hierarchy.add_node(root_node);

    // Create child nodes
    let child1_node = SceneNode::new("Child1");
    let child1_id = hierarchy.add_node(child1_node);

    let child2_node = SceneNode::new("Child2");
    let child2_id = hierarchy.add_node(child2_node);

    // Set up hierarchy
    assert!(hierarchy.set_parent(child1_id, Some(root_id)).is_ok());
    assert!(hierarchy.set_parent(child2_id, Some(root_id)).is_ok());

    // Test hierarchy queries
    assert_eq!(hierarchy.node_count(), 3);
    
    let root = hierarchy.get_node(root_id).unwrap();
    assert_eq!(root.name, "Root");
    assert_eq!(root.children.len(), 2);
    assert!(root.children.contains(&child1_id));
    assert!(root.children.contains(&child2_id));

    let child1 = hierarchy.get_node(child1_id).unwrap();
    assert_eq!(child1.parent, Some(root_id));

    // Test finding nodes
    let found_nodes = hierarchy.find_by_name("Child1");
    assert_eq!(found_nodes.len(), 1);
    assert_eq!(found_nodes[0].id, child1_id);
}

#[test]
fn test_scene_node_components() {
    let mut node = SceneNode::new("TestNode");
    
    // Initially no components
    assert!(!node.has_mesh());
    assert!(!node.has_material());
    assert!(!node.has_camera());
    assert!(!node.has_light());
    assert!(!node.is_renderable());

    // Add mesh and material
    node.components.mesh = Some(123);
    node.components.material = Some(456);

    assert!(node.has_mesh());
    assert!(node.has_material());
    assert!(node.is_renderable());

    // Add camera
    node.components.camera = Some(Camera::default());
    assert!(node.has_camera());
}

#[test]
fn test_camera_creation_and_properties() {
    // Test default camera
    let default_camera = Camera::default();
    assert!(default_camera.active);
    assert!(default_camera.clear_color.is_some());
    assert_eq!(default_camera.clear_depth, 1.0);

    // Test perspective camera
    let perspective_camera = Camera::perspective(
        45.0_f32.to_radians(),
        16.0 / 9.0,
        0.1,
        1000.0
    );
    
    match perspective_camera.projection {
        CameraProjection::Perspective { fov_y, aspect_ratio, near, far } => {
            assert_eq!(fov_y, 45.0_f32.to_radians());
            assert_eq!(aspect_ratio, 16.0 / 9.0);
            assert_eq!(near, 0.1);
            assert_eq!(far, 1000.0);
        }
        _ => panic!("Expected perspective projection"),
    }

    // Test orthographic camera
    let ortho_camera = Camera::orthographic(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
    
    match ortho_camera.projection {
        CameraProjection::Orthographic { left, right, bottom, top, near, far } => {
            assert_eq!(left, -10.0);
            assert_eq!(right, 10.0);
            assert_eq!(bottom, -10.0);
            assert_eq!(top, 10.0);
            assert_eq!(near, 0.1);
            assert_eq!(far, 100.0);
        }
        _ => panic!("Expected orthographic projection"),
    }
}

#[test]
fn test_camera_matrices() {
    let camera = Camera::perspective(
        90.0_f32.to_radians(), // 90 degree FOV
        1.0, // Square aspect ratio
        1.0, // Near plane
        100.0 // Far plane
    );

    // Test projection matrix
    let proj_matrix = camera.projection_matrix();
    
    // For a 90-degree FOV with 1:1 aspect ratio, the projection should be non-identity
    assert_ne!(proj_matrix, Mat4::IDENTITY);

    // Test view matrix (identity when no look-at target)
    let camera_position = Vec3::new(0.0, 0.0, 5.0);
    let view_matrix = camera.view_matrix(camera_position);
    
    // Without a look-at target, view matrix should be identity
    assert_eq!(view_matrix, Mat4::IDENTITY);

    // Test with look-at target
    let camera_with_target = camera.look_at(Vec3::new(0.0, 0.0, 0.0));
    let view_matrix_with_target = camera_with_target.view_matrix(camera_position);
    
    // With a look-at target, view matrix should not be identity
    assert_ne!(view_matrix_with_target, Mat4::IDENTITY);
}

#[test]
fn test_light_creation_and_properties() {
    // Test directional light
    let directional = Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Color::WHITE,
        1.0
    );
    
    assert!(directional.enabled);
    assert!(directional.cast_shadows);
    assert_eq!(directional.color, Color::WHITE);
    assert_eq!(directional.intensity, 1.0);
    
    match directional.light_type {
        LightType::Directional(ref dir_light) => {
            let expected_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
            assert!((dir_light.direction - expected_dir).length() < 0.001);
        }
        _ => panic!("Expected directional light"),
    }

    // Test point light
    let point = Light::point(Color::RED, 2.0, 10.0);
    
    assert_eq!(point.color, Color::RED);
    assert_eq!(point.intensity, 2.0);
    assert!(!point.cast_shadows); // Point lights don't cast shadows by default
    
    match point.light_type {
        LightType::Point(ref point_light) => {
            assert_eq!(point_light.range, 10.0);
        }
        _ => panic!("Expected point light"),
    }

    // Test spot light
    let spot = Light::spot(
        Vec3::new(0.0, -1.0, 0.0),
        Color::BLUE,
        1.5,
        15.0,
        30.0_f32.to_radians(),
        45.0_f32.to_radians()
    );
    
    assert_eq!(spot.color, Color::BLUE);
    assert_eq!(spot.intensity, 1.5);
    
    match spot.light_type {
        LightType::Spot(ref spot_light) => {
            assert_eq!(spot_light.range, 15.0);
            assert_eq!(spot_light.inner_cone_angle, 30.0_f32.to_radians());
            assert_eq!(spot_light.outer_cone_angle, 45.0_f32.to_radians());
        }
        _ => panic!("Expected spot light"),
    }
}

#[test]
fn test_light_contribution_calculations() {
    let light_position = Vec3::new(0.0, 5.0, 0.0);
    let point = Vec3::new(0.0, 0.0, 0.0);
    let normal = Vec3::new(0.0, 1.0, 0.0); // Pointing up

    // Test directional light
    let directional = Light::directional(
        Vec3::new(0.0, -1.0, 0.0), // Pointing down
        Color::WHITE,
        1.0
    );
    
    let contribution = directional.calculate_contribution(light_position, point, normal);
    assert!(contribution > 0.9); // Should be close to 1.0 for perfect alignment

    // Test point light
    let point_light = Light::point(Color::WHITE, 1.0, 10.0);
    let point_contribution = point_light.calculate_contribution(light_position, point, normal);
    
    // Point light contribution should be positive but attenuated by distance
    assert!(point_contribution > 0.0);
    assert!(point_contribution < 1.0);

    // Test light affecting a point
    assert!(directional.affects_point(light_position, point));
    assert!(point_light.affects_point(light_position, point));
    
    // Test point outside range
    let far_point = Vec3::new(0.0, 0.0, 20.0);
    assert!(!point_light.affects_point(light_position, far_point));
}

#[test]
fn test_scene_management() {
    let mut scene_manager = SceneManager::new();

    // Create scenes
    let scene1_handle = scene_manager.create_scene("MainScene");
    let scene2_handle = scene_manager.create_scene("LoadingScene");

    assert_eq!(scene_manager.scene_count(), 2);

    // Test scene access
    let scene1 = scene_manager.get_scene(scene1_handle).unwrap();
    assert_eq!(scene1.name, "MainScene");
    assert_eq!(scene1.handle, scene1_handle);

    // Test setting active scene
    assert!(scene_manager.set_active_scene(scene1_handle).is_ok());
    assert_eq!(scene_manager.active_scene_handle(), Some(scene1_handle));

    let active_scene = scene_manager.active_scene().unwrap();
    assert_eq!(active_scene.name, "MainScene");

    // Test scene listing
    let scene_list = scene_manager.list_scenes();
    assert_eq!(scene_list.len(), 2);
    
    let scene_names: Vec<_> = scene_list.iter().map(|(_, name)| *name).collect();
    assert!(scene_names.contains(&"MainScene"));
    assert!(scene_names.contains(&"LoadingScene"));

    // Test finding scene by name
    let found_handle = scene_manager.find_by_name("LoadingScene");
    assert_eq!(found_handle, Some(scene2_handle));

    // Test removing scene
    assert!(scene_manager.remove_scene(scene2_handle));
    assert_eq!(scene_manager.scene_count(), 1);
}

#[test]
fn test_scene_with_nodes() {
    let mut scene = Scene::new("TestScene");

    // Add nodes to scene
    let root_node = SceneNode::new("Root");
    let root_id = scene.add_node(root_node);

    let child_node = SceneNode::new("Child");
    let child_id = scene.add_node(child_node);

    // Set up hierarchy
    assert!(scene.set_parent(child_id, Some(root_id)).is_ok());

    // Test scene queries
    assert_eq!(scene.node_count(), 2);
    
    let root_nodes: Vec<_> = scene.root_nodes().collect();
    assert_eq!(root_nodes.len(), 1);
    assert_eq!(root_nodes[0].name, "Root");

    // Test finding nodes
    let found = scene.find_by_name("Child");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "Child");

    // Test component queries
    let renderables = scene.renderable_nodes();
    let cameras = scene.camera_nodes();
    let lights = scene.light_nodes();

    // Should all be empty since we haven't added components
    assert!(renderables.is_empty());
    assert!(cameras.is_empty());
    assert!(lights.is_empty());

    // Clear scene
    scene.clear();
    assert_eq!(scene.node_count(), 0);
}

#[test]
fn test_viewport_properties() {
    let viewport = Viewport::new(100.0, 200.0, 800.0, 600.0);
    
    assert_eq!(viewport.x, 100.0);
    assert_eq!(viewport.y, 200.0);
    assert_eq!(viewport.width, 800.0);
    assert_eq!(viewport.height, 600.0);

    // Test aspect ratio
    let aspect = viewport.aspect_ratio();
    assert_eq!(aspect, 800.0 / 600.0);

    // Test containment
    assert!(viewport.contains(150.0, 250.0)); // Inside
    assert!(!viewport.contains(50.0, 100.0)); // Outside

    // Test depth range
    let viewport_with_depth = viewport.with_depth_range(0.2, 0.8);
    assert_eq!(viewport_with_depth.min_depth, 0.2);
    assert_eq!(viewport_with_depth.max_depth, 0.8);

    // Test default viewport
    let default_viewport = Viewport::default();
    assert_eq!(default_viewport.x, 0.0);
    assert_eq!(default_viewport.y, 0.0);
    assert_eq!(default_viewport.width, 1.0);  // Default is 1.0, not 800
    assert_eq!(default_viewport.height, 1.0); // Default is 1.0, not 600
}