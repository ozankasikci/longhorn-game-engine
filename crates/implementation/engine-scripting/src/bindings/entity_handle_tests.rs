//! Tests for entity handle ownership and lifetime management

#[cfg(test)]
mod tests {
    use super::super::EntityHandle;
    use crate::{ScriptError, api::AccessPermissions};
    use engine_ecs_core::{Entity, World, register_component};
    use crate::test_utils::{TestTransform as Transform, TestComponent};
    
    fn setup_test_components() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            register_component::<Transform>();
            register_component::<TestComponent>();
        });
    }

    #[test]
    fn test_entity_handle_validity() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        
        // Handle should be valid
        assert!(handle.is_valid(&world));
        
        // Handle should remain valid as long as entity exists
        // (version tracking not yet implemented)
        world.spawn(); // This doesn't affect validity
        assert!(handle.is_valid(&world));
    }

    #[test]
    fn test_entity_handle_after_despawn() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap(); // Add a component so entity is in world
        
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        assert!(handle.is_valid(&world));
        
        // Despawn entity
        assert!(world.despawn(entity));
        
        // Handle should be invalid
        assert!(!handle.is_valid(&world));
    }

    #[test]
    fn test_get_component_with_permissions() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        // Create handle with read permissions
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::read_only());
        
        // Should be able to read
        match handle.get_component::<Transform>(&world) {
            Ok(component_ref) => {
                assert_eq!(component_ref.position, [0.0, 0.0, 0.0]);
            }
            Err(_) => panic!("Should be able to read component"),
        }
    }

    #[test]
    fn test_get_component_without_permissions() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        // Create handle with no permissions
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::none());
        
        // Should not be able to read
        match handle.get_component::<Transform>(&world) {
            Err(ScriptError::AccessDenied { operation }) => {
                assert!(operation.contains("read"));
            }
            _ => panic!("Expected AccessDenied error"),
        }
    }

    #[test]
    fn test_get_component_mut_with_permissions() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        // Create handle with write permissions
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        
        // Should be able to write
        match handle.get_component_mut::<Transform>(&mut world) {
            Ok(mut component_mut) => {
                component_mut.position = [1.0, 2.0, 3.0];
            }
            Err(_) => panic!("Should be able to write component"),
        }
        
        // Verify the change
        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_get_component_mut_without_permissions() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        // Create handle with read-only permissions
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::read_only());
        
        // Should not be able to write
        match handle.get_component_mut::<Transform>(&mut world) {
            Err(ScriptError::AccessDenied { operation }) => {
                assert!(operation.contains("write"));
            }
            _ => panic!("Expected AccessDenied error"),
        }
    }

    #[test]
    fn test_invalid_handle_error() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        
        // Make handle invalid by changing world
        world.spawn();
        
        // Should get InvalidEntityHandle error
        match handle.get_component::<Transform>(&world) {
            Err(ScriptError::InvalidEntityHandle) => {},
            _ => panic!("Expected InvalidEntityHandle error"),
        }
    }

    #[test]
    fn test_component_not_found() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        // Add a different component so entity exists in world, but not Transform
        world.add_component(entity, TestComponent { value: 42 }).unwrap();
        
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        
        // Should get ComponentNotFound error
        match handle.get_component::<Transform>(&world) {
            Err(ScriptError::ComponentNotFound { entity: e, component }) => {
                assert_eq!(e, entity);
                assert!(component.contains("TestTransform") || component.contains("Transform"));
            }
            Err(e) => panic!("Got unexpected error: {:?}", e),
            Ok(_) => panic!("Expected ComponentNotFound error, but got Ok"),
        }
    }

    #[test]
    fn test_handle_cloning() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap(); // Add a component so entity is in world
        
        let handle1 = EntityHandle::new(entity, world.change_tick().get() as u64, AccessPermissions::all());
        let handle2 = handle1.clone();
        
        assert_eq!(handle1.entity_id(), handle2.entity_id());
        assert_eq!(handle1.world_version(), handle2.world_version());
        assert!(handle2.is_valid(&world));
    }

    #[test]
    fn test_specific_component_permissions() {
        setup_test_components();
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, Transform::default()).unwrap();
        
        // Create handle with custom permissions that only allow Transform access
        let mut permissions = AccessPermissions::none();
        permissions.can_read_components = true;
        permissions.add_component_permission::<Transform>("read");
        
        let handle = EntityHandle::new(entity, world.change_tick().get() as u64, permissions);
        
        // Should be able to read Transform
        handle.get_component::<Transform>(&world).unwrap();
        
        // But not other components (if we had them)
        // This would be tested with other component types
    }

    #[test]
    fn test_handle_debug_info() {
        setup_test_components();
        let _world = World::new();
        let entity = Entity::new(42, 1);
        
        let handle = EntityHandle::new(entity, 100, AccessPermissions::read_only());
        
        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("42")); // entity id
        assert!(debug_str.contains("100")); // world version
    }
}