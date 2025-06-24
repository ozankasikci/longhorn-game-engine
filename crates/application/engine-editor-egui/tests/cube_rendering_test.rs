#[cfg(test)]
mod cube_rendering_tests {
    use engine_camera::Camera;
    use engine_components_3d::{Material, Mesh, MeshType, Transform, Visibility};
    use engine_components_ui::Name;
    use engine_ecs_core::World;

    /// Test that cube entities are created with correct components
    #[test]
    fn test_cube_entity_creation() {
        let mut world = World::new();

        // Create entity first
        let cube_entity = world.spawn();
        println!("Created entity: {:?}", cube_entity);

        // Add transform component
        let transform = Transform {
            position: [0.0, 0.0, -2.0], // In front of camera at origin
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };

        match world.add_component(cube_entity, transform) {
            Ok(_) => println!("Added Transform component successfully"),
            Err(e) => panic!("Failed to add Transform: {:?}", e),
        }

        // Try to retrieve the transform component
        match world.get_component::<Transform>(cube_entity) {
            Some(t) => println!("Retrieved Transform: {:?}", t),
            None => panic!("Could not retrieve Transform component"),
        }

        // Verify the entity has the component
        assert!(world.get_component::<Transform>(cube_entity).is_some());
    }

    /// Test that cube entities are visible in scene queries
    #[test]
    fn test_cube_entity_visibility_query() {
        let mut world = World::new();

        // Create camera
        let camera_entity = world.spawn_with(Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world
            .add_component(camera_entity, Camera::default())
            .unwrap();

        // Create cube in front of camera
        let cube_entity = world.spawn_with(Transform {
            position: [0.0, 0.0, -2.0], // In front of camera
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world
            .add_component(
                cube_entity,
                Mesh {
                    mesh_type: MeshType::Cube,
                },
            )
            .unwrap();
        world
            .add_component(
                cube_entity,
                Material {
                    color: [1.0, 0.0, 0.0, 1.0],
                    metallic: 0.0,
                    roughness: 0.5,
                    emissive: [0.0, 0.0, 0.0],
                },
            )
            .unwrap();
        world
            .add_component(cube_entity, Visibility::default())
            .unwrap();

        // Query all entities with mesh components
        let entities_with_meshes: Vec<_> = world
            .query_legacy::<Transform>()
            .filter(|(entity, _)| world.get_component::<Mesh>(*entity).is_some())
            .collect();

        // Should find our cube entity
        assert_eq!(entities_with_meshes.len(), 1);

        let (found_entity, transform) = &entities_with_meshes[0];
        assert_eq!(*found_entity, cube_entity);
        assert_eq!(transform.position, [0.0, 0.0, -2.0]);
    }

    /// Test that a cube should be visible to camera (failing test - represents the bug)
    #[test]
    #[should_panic(expected = "Cube should be rendered but isn't visible")]
    fn test_cube_should_be_rendered_but_isnt() {
        let mut world = World::new();

        // Create camera at origin looking down negative Z axis (standard)
        let camera_entity = world.spawn_with(Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world
            .add_component(camera_entity, Camera::default())
            .unwrap();

        // Create cube clearly in front of camera
        let cube_entity = world.spawn_with(Transform {
            position: [0.0, 0.0, -2.0], // Clearly in front
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world
            .add_component(
                cube_entity,
                Mesh {
                    mesh_type: MeshType::Cube,
                },
            )
            .unwrap();
        world
            .add_component(
                cube_entity,
                Material {
                    color: [1.0, 0.0, 0.0, 1.0],
                    metallic: 0.0,
                    roughness: 0.5,
                    emissive: [0.0, 0.0, 0.0],
                },
            )
            .unwrap();
        world
            .add_component(cube_entity, Visibility::default())
            .unwrap();

        // Simulate what the scene renderer should do
        let renderable_entities = collect_renderable_entities(&world, camera_entity);

        // THIS SHOULD FAIL because the current renderer doesn't properly connect to WGPU
        if renderable_entities.is_empty() {
            panic!("Cube should be rendered but isn't visible");
        }
    }

    /// Helper function to simulate scene rendering logic
    fn collect_renderable_entities(
        world: &World,
        camera_entity: engine_ecs_core::Entity,
    ) -> Vec<engine_ecs_core::Entity> {
        let camera_transform = world.get_component::<Transform>(camera_entity).unwrap();
        let camera = world.get_component::<Camera>(camera_entity).unwrap();

        // Find all entities with mesh components that should be visible
        world
            .query_legacy::<Transform>()
            .filter(|(entity, transform)| {
                // Has mesh component
                if world.get_component::<Mesh>(*entity).is_none() {
                    return false;
                }

                // Has visibility component and is visible
                if let Some(visibility) = world.get_component::<Visibility>(*entity) {
                    if !visibility.visible {
                        return false;
                    }
                }

                // Is within camera frustum (simplified check)
                is_in_camera_frustum(transform, camera_transform, camera)
            })
            .map(|(entity, _)| entity)
            .collect()
    }

    /// Simplified frustum check
    fn is_in_camera_frustum(
        object_transform: &Transform,
        camera_transform: &Transform,
        _camera: &Camera,
    ) -> bool {
        // For this test, just check if object is in front of camera
        let relative_z = object_transform.position[2] - camera_transform.position[2];
        relative_z < 0.0 // Object is in front of camera (negative Z in view space)
    }
}
