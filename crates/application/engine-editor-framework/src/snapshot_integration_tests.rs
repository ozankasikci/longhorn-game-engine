#[cfg(test)]
mod snapshot_integration_tests {
    use crate::unified_coordinator::UnifiedEditorCoordinator;
    use engine_ecs_core::World;
    use engine_components_3d::Transform;
    use engine_components_ui::Name;
    use engine_editor_scene_view::types::PlayState;
    
    #[test]
    fn test_end_to_end_snapshot_integration() {
        // Setup: Create world and coordinator
        let mut world = World::new();
        let mut coordinator = UnifiedEditorCoordinator::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create test entity with initial state
        let entity = world.spawn();
        let original_transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.1, 0.2, 0.3],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, original_transform).unwrap();
        world.add_component(entity, Name { name: "TestEntity".to_string() }).unwrap();
        
        // Verify initial state
        assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Editing);
        
        // Simulate Start Play with snapshot capture (mimicking toolbar action)
        coordinator.play_state_manager_mut().start_with_snapshot(&world);
        assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Playing);
        assert!(coordinator.play_state_manager().get_snapshot().is_some());
        
        // Simulate changes during play mode (e.g., from scripts or user interaction)
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [100.0, 200.0, 300.0];
            transform.scale = [5.0, 5.0, 5.0];
        }
        if let Some(mut name) = world.get_component_mut::<Name>(entity) {
            name.name = "ModifiedDuringPlay".to_string();
        }
        
        // Verify changes were applied
        let modified_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(modified_transform.position, [100.0, 200.0, 300.0]);
        assert_eq!(modified_transform.scale, [5.0, 5.0, 5.0]);
        
        let modified_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(modified_name.name, "ModifiedDuringPlay");
        
        // Simulate Stop Play with snapshot restoration (mimicking toolbar action)
        coordinator.play_state_manager_mut().stop_with_restore(&mut world).unwrap();
        assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Editing);
        assert!(coordinator.play_state_manager().get_snapshot().is_none());
        
        // Verify state was restored to original values
        let restored_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored_transform.position, [1.0, 2.0, 3.0]);
        assert_eq!(restored_transform.rotation, [0.1, 0.2, 0.3]);
        assert_eq!(restored_transform.scale, [1.0, 1.0, 1.0]);
        
        let restored_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(restored_name.name, "TestEntity");
    }
    
    #[test]
    fn test_pause_resume_preserves_snapshot() {
        let mut world = World::new();
        let mut coordinator = UnifiedEditorCoordinator::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        
        // Create test entity
        let entity = world.spawn();
        let transform = Transform {
            position: [10.0, 20.0, 30.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, transform).unwrap();
        
        // Start play mode with snapshot
        coordinator.play_state_manager_mut().start_with_snapshot(&world);
        assert!(coordinator.play_state_manager().get_snapshot().is_some());
        
        // Modify during play
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [50.0, 60.0, 70.0];
        }
        
        // Pause
        coordinator.play_state_manager_mut().pause();
        assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Paused);
        assert!(coordinator.play_state_manager().get_snapshot().is_some());
        
        // Resume
        coordinator.play_state_manager_mut().resume();
        assert_eq!(coordinator.play_state_manager().get_state(), PlayState::Playing);
        assert!(coordinator.play_state_manager().get_snapshot().is_some());
        
        // Stop and restore
        coordinator.play_state_manager_mut().stop_with_restore(&mut world).unwrap();
        
        // Verify restoration worked
        let restored = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored.position, [10.0, 20.0, 30.0]);
    }
    
    #[test]
    fn test_multiple_entities_snapshot_integration() {
        let mut world = World::new();
        let mut coordinator = UnifiedEditorCoordinator::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create multiple test entities
        let entity1 = world.spawn();
        let entity2 = world.spawn();
        let entity3 = world.spawn();
        
        world.add_component(entity1, Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        
        world.add_component(entity2, Transform {
            position: [4.0, 5.0, 6.0],
            rotation: [0.1, 0.2, 0.3],
            scale: [2.0, 2.0, 2.0],
        }).unwrap();
        
        world.add_component(entity3, Name { name: "Entity3".to_string() }).unwrap();
        
        // Start play with snapshot
        coordinator.play_state_manager_mut().start_with_snapshot(&world);
        
        // Modify all entities
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity1) {
            transform.position = [100.0, 100.0, 100.0];
        }
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity2) {
            transform.scale = [10.0, 10.0, 10.0];
        }
        if let Some(mut name) = world.get_component_mut::<Name>(entity3) {
            name.name = "ModifiedEntity3".to_string();
        }
        
        // Stop and restore
        coordinator.play_state_manager_mut().stop_with_restore(&mut world).unwrap();
        
        // Verify all entities were restored
        let restored1 = world.get_component::<Transform>(entity1).unwrap();
        assert_eq!(restored1.position, [1.0, 2.0, 3.0]);
        
        let restored2 = world.get_component::<Transform>(entity2).unwrap();
        assert_eq!(restored2.position, [4.0, 5.0, 6.0]);
        assert_eq!(restored2.scale, [2.0, 2.0, 2.0]);
        
        let restored3 = world.get_component::<Name>(entity3).unwrap();
        assert_eq!(restored3.name, "Entity3");
    }
}