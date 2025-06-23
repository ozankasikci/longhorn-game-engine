#[cfg(test)]
mod coverage_tests {
    use crate::world::World;
    use crate::component::Component;
    
    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    
    impl Component for Position {}
    
    #[test]
    fn basic_ecs_coverage() {
        let mut world = World::new();
        
        // Create entity
        let entity = world.create_entity();
        assert!(world.is_alive(entity));
        
        // Add component
        world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();
        
        // Query component
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        
        // Remove entity
        world.destroy_entity(entity).unwrap();
        assert!(!world.is_alive(entity));
    }
}