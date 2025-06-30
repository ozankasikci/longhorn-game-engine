#[cfg(test)]
mod world_snapshot_tests {
    use crate::world_snapshot::WorldSnapshot;
    use engine_ecs_core::World;
    use engine_components_3d::Transform;
    use engine_components_ui::Name;
    
    #[test]
    fn test_world_snapshot_creation() {
        let mut world = World::new();
        
        // Register components first
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create test entities with Transform components
        let entity1 = world.spawn();
        let entity2 = world.spawn();
        
        let transform1 = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        let transform2 = Transform {
            position: [4.0, 5.0, 6.0],
            rotation: [0.0, 1.57, 0.0],
            scale: [2.0, 2.0, 2.0],
        };
        
        world.add_component(entity1, transform1).unwrap();
        world.add_component(entity2, transform2).unwrap();
        world.add_component(entity1, Name { name: "Entity1".to_string() }).unwrap();
        
        // Create snapshot
        let snapshot = WorldSnapshot::capture(&world);
        
        // Verify snapshot contains both entities
        assert_eq!(snapshot.entities.len(), 2);
        assert!(snapshot.entities.contains_key(&entity1.id()));
        assert!(snapshot.entities.contains_key(&entity2.id()));
        
        // Verify snapshot has a valid timestamp and ID
        assert!(snapshot.timestamp.elapsed().as_secs() < 1);
        assert!(!snapshot.snapshot_id.is_nil());
    }
    
    #[test]
    fn test_world_snapshot_restoration() {
        let mut world = World::new();
        
        // Register components first
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create initial state
        let entity = world.spawn();
        let original_transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, original_transform).unwrap();
        world.add_component(entity, Name { name: "TestEntity".to_string() }).unwrap();
        
        // Capture snapshot
        let snapshot = WorldSnapshot::capture(&world);
        
        // Modify the world state
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [10.0, 20.0, 30.0];
            transform.scale = [5.0, 5.0, 5.0];
        }
        
        if let Some(mut name) = world.get_component_mut::<Name>(entity) {
            name.name = "ModifiedEntity".to_string();
        }
        
        // Verify state was modified
        let modified_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(modified_transform.position, [10.0, 20.0, 30.0]);
        
        let modified_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(modified_name.name, "ModifiedEntity");
        
        // Restore from snapshot
        snapshot.restore(&mut world).unwrap();
        
        // Verify restoration
        let restored_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored_transform.position, [1.0, 2.0, 3.0]);
        assert_eq!(restored_transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(restored_transform.scale, [1.0, 1.0, 1.0]);
        
        let restored_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(restored_name.name, "TestEntity");
    }
    
    #[test]
    fn test_empty_world_snapshot() {
        let world = World::new();
        let snapshot = WorldSnapshot::capture(&world);
        
        assert_eq!(snapshot.entities.len(), 0);
        assert!(!snapshot.snapshot_id.is_nil());
    }
    
    #[test]
    fn test_snapshot_with_entities_without_supported_components() {
        let mut world = World::new();
        
        // Create entity but don't add any components we support for snapshots
        let _entity = world.spawn();
        
        let snapshot = WorldSnapshot::capture(&world);
        
        // Should not capture entities without supported components
        assert_eq!(snapshot.entities.len(), 0);
    }
}