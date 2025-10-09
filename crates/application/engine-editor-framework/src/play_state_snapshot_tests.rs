#[cfg(test)]
mod play_state_snapshot_tests {
    use crate::play_state::PlayStateManager;
    use engine_ecs_core::World;
    use engine_components_3d::Transform;
    use engine_components_ui::Name;
    use engine_editor_scene_view::types::PlayState;
    
    #[test]
    fn test_play_state_manager_capture_snapshot_on_start() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create test entities
        let entity = world.spawn();
        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, transform).unwrap();
        world.add_component(entity, Name { name: "TestEntity".to_string() }).unwrap();
        
        let mut play_state_manager = PlayStateManager::new();
        
        // Verify initial state
        assert_eq!(play_state_manager.get_state(), PlayState::Editing);
        assert!(play_state_manager.get_snapshot().is_none());
        
        // Start play mode with snapshot capture
        play_state_manager.start_with_snapshot(&world);
        
        // Verify state changed and snapshot was captured
        assert_eq!(play_state_manager.get_state(), PlayState::Playing);
        assert!(play_state_manager.get_snapshot().is_some());
        
        let snapshot = play_state_manager.get_snapshot().unwrap();
        assert_eq!(snapshot.entities.len(), 1);
        assert!(snapshot.entities.contains_key(&entity.id()));
    }
    
    #[test]
    fn test_play_state_manager_restore_snapshot_on_stop() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        // Create test entity
        let entity = world.spawn();
        let original_transform = Transform {
            position: [5.0, 10.0, 15.0],
            rotation: [0.0, 1.57, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, original_transform).unwrap();
        world.add_component(entity, Name { name: "OriginalName".to_string() }).unwrap();
        
        let mut play_state_manager = PlayStateManager::new();
        
        // Start play mode and capture snapshot
        play_state_manager.start_with_snapshot(&world);
        
        // Modify the world state during play mode
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [100.0, 200.0, 300.0];
            transform.scale = [5.0, 5.0, 5.0];
        }
        if let Some(mut name) = world.get_component_mut::<Name>(entity) {
            name.name = "ModifiedName".to_string();
        }
        
        // Verify modifications
        let modified_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(modified_transform.position, [100.0, 200.0, 300.0]);
        
        let modified_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(modified_name.name, "ModifiedName");
        
        // Stop play mode and restore snapshot
        play_state_manager.stop_with_restore(&mut world).unwrap();
        
        // Verify state changed back to editing
        assert_eq!(play_state_manager.get_state(), PlayState::Editing);
        assert!(play_state_manager.get_snapshot().is_none());
        
        // Verify world state was restored
        let restored_transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored_transform.position, [5.0, 10.0, 15.0]);
        assert_eq!(restored_transform.rotation, [0.0, 1.57, 0.0]);
        assert_eq!(restored_transform.scale, [1.0, 1.0, 1.0]);
        
        let restored_name = world.get_component::<Name>(entity).unwrap();
        assert_eq!(restored_name.name, "OriginalName");
    }
    
    #[test]
    fn test_play_state_manager_pause_resume_preserves_snapshot() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        
        // Create test entity
        let entity = world.spawn();
        let transform = Transform {
            position: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, transform).unwrap();
        
        let mut play_state_manager = PlayStateManager::new();
        
        // Start play mode
        play_state_manager.start_with_snapshot(&world);
        assert_eq!(play_state_manager.get_state(), PlayState::Playing);
        assert!(play_state_manager.get_snapshot().is_some());
        
        // Pause play mode
        play_state_manager.pause();
        assert_eq!(play_state_manager.get_state(), PlayState::Paused);
        assert!(play_state_manager.get_snapshot().is_some()); // Snapshot should be preserved
        
        // Resume play mode
        play_state_manager.resume();
        assert_eq!(play_state_manager.get_state(), PlayState::Playing);
        assert!(play_state_manager.get_snapshot().is_some()); // Snapshot should still be preserved
        
        // Stop play mode
        play_state_manager.stop_with_restore(&mut world).unwrap();
        assert_eq!(play_state_manager.get_state(), PlayState::Editing);
        assert!(play_state_manager.get_snapshot().is_none()); // Snapshot should be cleared
    }
    
    #[test]
    fn test_play_state_manager_stop_without_start_does_nothing() {
        let mut world = World::new();
        let mut play_state_manager = PlayStateManager::new();
        
        // Try to stop without starting
        let result = play_state_manager.stop_with_restore(&mut world);
        
        // Should succeed but do nothing
        assert!(result.is_ok());
        assert_eq!(play_state_manager.get_state(), PlayState::Editing);
        assert!(play_state_manager.get_snapshot().is_none());
    }
    
    #[test]
    fn test_play_state_manager_multiple_start_stop_cycles() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        
        // Create test entity
        let entity = world.spawn();
        let original_transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, original_transform).unwrap();
        
        let mut play_state_manager = PlayStateManager::new();
        
        // First cycle
        play_state_manager.start_with_snapshot(&world);
        
        // Modify world
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [10.0, 20.0, 30.0];
        }
        
        // Stop and restore
        play_state_manager.stop_with_restore(&mut world).unwrap();
        
        // Verify restoration
        let restored = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored.position, [1.0, 2.0, 3.0]);
        
        // Second cycle with new modifications to original state
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [5.0, 6.0, 7.0]; // New "original" state
        }
        
        play_state_manager.start_with_snapshot(&world);
        
        // Modify again during play
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [50.0, 60.0, 70.0];
        }
        
        // Stop and restore
        play_state_manager.stop_with_restore(&mut world).unwrap();
        
        // Should restore to the new "original" state
        let final_restored = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(final_restored.position, [5.0, 6.0, 7.0]);
    }
    
    #[test] 
    fn test_play_state_manager_backwards_compatibility() {
        let mut play_state_manager = PlayStateManager::new();
        
        // Test that old methods still work
        play_state_manager.start();
        assert_eq!(play_state_manager.get_state(), PlayState::Playing);
        
        play_state_manager.pause();
        assert_eq!(play_state_manager.get_state(), PlayState::Paused);
        
        play_state_manager.resume();
        assert_eq!(play_state_manager.get_state(), PlayState::Playing);
        
        play_state_manager.stop();
        assert_eq!(play_state_manager.get_state(), PlayState::Editing);
        
        // No snapshot functionality should be used with old methods
        assert!(play_state_manager.get_snapshot().is_none());
    }
}