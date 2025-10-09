#[cfg(test)]
mod entity_memento_tests {
    use crate::world_snapshot::EntityMemento;
    use engine_ecs_core::World;
    use engine_components_3d::Transform;
    use engine_components_ui::Name;
    use engine_scripting::components::TypeScriptScript;
    
    #[test]
    fn test_entity_memento_capture_transform_only() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        engine_ecs_core::register_component::<TypeScriptScript>();
        
        let entity = world.spawn();
        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.1, 0.2, 0.3],
            scale: [2.0, 2.0, 2.0],
        };
        
        world.add_component(entity, transform).unwrap();
        
        // Capture memento
        let memento = EntityMemento::capture(&world, entity);
        
        // Verify memento contains Transform data
        assert_eq!(memento.entity_id, entity.id());
        assert!(memento.components.contains_key("Transform"));
        assert!(!memento.components.contains_key("Name"));
        assert!(!memento.components.contains_key("TypeScriptScript"));
        
        // Verify Transform data can be deserialized
        let transform_data = memento.components.get("Transform").unwrap();
        let deserialized: Transform = bincode::deserialize(transform_data).unwrap();
        assert_eq!(deserialized.position, [1.0, 2.0, 3.0]);
        assert_eq!(deserialized.rotation, [0.1, 0.2, 0.3]);
        assert_eq!(deserialized.scale, [2.0, 2.0, 2.0]);
    }
    
    #[test]
    fn test_entity_memento_capture_all_components() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        engine_ecs_core::register_component::<TypeScriptScript>();
        
        let entity = world.spawn();
        
        // Add all supported components
        let transform = Transform {
            position: [5.0, 10.0, 15.0],
            rotation: [0.0, 1.57, 0.0],
            scale: [1.5, 1.5, 1.5],
        };
        let name = Name { name: "TestEntity".to_string() };
        let script = TypeScriptScript::new("test_script.ts".to_string());
        
        world.add_component(entity, transform).unwrap();
        world.add_component(entity, name).unwrap();
        world.add_component(entity, script).unwrap();
        
        // Capture memento
        let memento = EntityMemento::capture(&world, entity);
        
        // Verify all components are captured
        assert_eq!(memento.entity_id, entity.id());
        assert!(memento.components.contains_key("Transform"));
        assert!(memento.components.contains_key("Name"));
        assert!(memento.components.contains_key("TypeScriptScript"));
        
        // Verify component data integrity
        let transform_data = memento.components.get("Transform").unwrap();
        let deserialized_transform: Transform = bincode::deserialize(transform_data).unwrap();
        assert_eq!(deserialized_transform.position, [5.0, 10.0, 15.0]);
        
        let name_data = memento.components.get("Name").unwrap();
        let deserialized_name: Name = bincode::deserialize(name_data).unwrap();
        assert_eq!(deserialized_name.name, "TestEntity");
        
        let script_data = memento.components.get("TypeScriptScript").unwrap();
        let deserialized_script: TypeScriptScript = bincode::deserialize(script_data).unwrap();
        assert_eq!(deserialized_script.script_path, "test_script.ts");
    }
    
    #[test]
    fn test_entity_memento_restore_transform() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        let entity = world.spawn();
        
        // Add initial component
        let original_transform = Transform {
            position: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, original_transform).unwrap();
        
        // Capture memento
        let memento = EntityMemento::capture(&world, entity);
        
        // Modify the component
        if let Some(mut transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = [100.0, 200.0, 300.0];
            transform.scale = [5.0, 5.0, 5.0];
        }
        
        // Verify modification
        let modified = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(modified.position, [100.0, 200.0, 300.0]);
        
        // Restore from memento
        memento.restore(&mut world, entity).unwrap();
        
        // Verify restoration
        let restored = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored.position, [1.0, 1.0, 1.0]);
        assert_eq!(restored.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(restored.scale, [1.0, 1.0, 1.0]);
    }
    
    #[test] 
    fn test_entity_memento_restore_missing_component() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        engine_ecs_core::register_component::<Name>();
        
        let entity = world.spawn();
        
        // Add Transform component
        let transform = Transform {
            position: [2.0, 3.0, 4.0],
            rotation: [0.1, 0.2, 0.3],
            scale: [1.0, 1.0, 1.0],
        };
        world.add_component(entity, transform).unwrap();
        
        // Capture memento
        let memento = EntityMemento::capture(&world, entity);
        
        // Remove the component
        world.remove_component::<Transform>(entity).unwrap();
        
        // Verify component is gone
        assert!(world.get_component::<Transform>(entity).is_none());
        
        // Restore from memento (should re-add the component)
        memento.restore(&mut world, entity).unwrap();
        
        // Verify component is restored
        let restored = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(restored.position, [2.0, 3.0, 4.0]);
        assert_eq!(restored.rotation, [0.1, 0.2, 0.3]);
        assert_eq!(restored.scale, [1.0, 1.0, 1.0]);
    }
    
    #[test]
    fn test_entity_memento_empty_entity() {
        let mut world = World::new();
        
        // Register components
        engine_ecs_core::register_component::<Transform>();
        
        let entity = world.spawn();
        // Don't add any components
        
        // Capture memento
        let memento = EntityMemento::capture(&world, entity);
        
        // Verify memento is empty but valid
        assert_eq!(memento.entity_id, entity.id());
        assert!(memento.components.is_empty());
        
        // Restore should succeed (no-op)
        let result = memento.restore(&mut world, entity);
        assert!(result.is_ok());
    }
}