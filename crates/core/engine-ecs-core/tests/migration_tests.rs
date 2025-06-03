//! Tests for ECS component migration functionality

use engine_ecs_core::{World, Component, register_component};

#[derive(Debug, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {}

#[derive(Debug, Clone, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

impl Component for Velocity {}

#[derive(Debug, Clone, PartialEq)]
struct Health {
    current: i32,
    max: i32,
}

impl Component for Health {}

#[test]
fn test_add_component_to_existing_entity() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    
    let mut world = World::new();
    
    // Create entity with just Position
    let entity = world.spawn();
    world.add_component(entity, Position { x: 10.0, y: 20.0 }).unwrap();
    
    // Should be able to add Velocity component
    let result = world.add_component(entity, Velocity { dx: 1.0, dy: 2.0 });
    
    // This should succeed!
    assert!(result.is_ok(), "Failed to add component to existing entity: {:?}", result);
    
    // Verify both components exist
    assert!(world.get_component::<Position>(entity).is_some());
    assert!(world.get_component::<Velocity>(entity).is_some());
}

#[test]
fn test_add_multiple_components_sequentially() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    register_component::<Health>();
    
    let mut world = World::new();
    
    // Create empty entity
    let entity = world.spawn();
    
    // Add components one by one
    world.add_component(entity, Position { x: 0.0, y: 0.0 }).unwrap();
    world.add_component(entity, Velocity { dx: 5.0, dy: 5.0 }).unwrap();
    world.add_component(entity, Health { current: 100, max: 100 }).unwrap();
    
    // Verify all components
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 0.0);
    
    let vel = world.get_component::<Velocity>(entity).unwrap();
    assert_eq!(vel.dx, 5.0);
    
    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 100);
}

#[test]
fn test_component_data_preserved_during_migration() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    
    let mut world = World::new();
    
    // Create entity with Position
    let entity = world.spawn();
    world.add_component(entity, Position { x: 42.0, y: 84.0 }).unwrap();
    
    // Add another component (triggers migration)
    world.add_component(entity, Velocity { dx: -1.0, dy: -2.0 }).unwrap();
    
    // Original component data should be preserved
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 42.0);
    assert_eq!(pos.y, 84.0);
}

#[test]
fn test_multiple_entities_with_different_archetypes() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    register_component::<Health>();
    
    let mut world = World::new();
    
    // Entity 1: Position only
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 1.0, y: 1.0 }).unwrap();
    
    // Entity 2: Position + Velocity
    let e2 = world.spawn();
    world.add_component(e2, Position { x: 2.0, y: 2.0 }).unwrap();
    world.add_component(e2, Velocity { dx: 10.0, dy: 20.0 }).unwrap();
    
    // Entity 3: Position + Velocity + Health
    let e3 = world.spawn();
    world.add_component(e3, Position { x: 3.0, y: 3.0 }).unwrap();
    world.add_component(e3, Velocity { dx: 30.0, dy: 40.0 }).unwrap();
    world.add_component(e3, Health { current: 75, max: 100 }).unwrap();
    
    // Add component to e1 (migration test)
    world.add_component(e1, Health { current: 50, max: 50 }).unwrap();
    
    // Verify all entities have correct components
    assert_eq!(world.get_component::<Position>(e1).unwrap().x, 1.0);
    assert_eq!(world.get_component::<Health>(e1).unwrap().current, 50);
    assert!(world.get_component::<Velocity>(e1).is_none());
    
    assert_eq!(world.get_component::<Position>(e2).unwrap().x, 2.0);
    assert_eq!(world.get_component::<Velocity>(e2).unwrap().dx, 10.0);
    assert!(world.get_component::<Health>(e2).is_none());
    
    assert_eq!(world.get_component::<Position>(e3).unwrap().x, 3.0);
    assert_eq!(world.get_component::<Velocity>(e3).unwrap().dx, 30.0);
    assert_eq!(world.get_component::<Health>(e3).unwrap().current, 75);
}

#[test]
fn test_query_after_migration() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    
    let mut world = World::new();
    
    // Create multiple entities
    for i in 0..5 {
        let entity = world.spawn();
        world.add_component(entity, Position { x: i as f32, y: i as f32 }).unwrap();
        
        // Add Velocity to even entities
        if i % 2 == 0 {
            world.add_component(entity, Velocity { dx: i as f32 * 10.0, dy: 0.0 }).unwrap();
        }
    }
    
    // Query all Position components
    let positions: Vec<_> = world.query_legacy::<Position>()
        .map(|(_, p)| p.x)
        .collect();
    assert_eq!(positions.len(), 5);
    
    // Query all Velocity components
    let velocities: Vec<_> = world.query_legacy::<Velocity>()
        .map(|(_, v)| v.dx)
        .collect();
    assert_eq!(velocities.len(), 3); // Only even entities
}

#[test]
fn test_remove_component() {
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    register_component::<Health>();
    
    let mut world = World::new();
    
    // Create entity with multiple components
    let entity = world.spawn();
    world.add_component(entity, Position { x: 100.0, y: 200.0 }).unwrap();
    world.add_component(entity, Velocity { dx: 5.0, dy: 10.0 }).unwrap();
    world.add_component(entity, Health { current: 80, max: 100 }).unwrap();
    
    // Remove Velocity component
    let removed = world.remove_component::<Velocity>(entity);
    assert!(removed.is_ok());
    
    // Verify component is gone
    assert!(world.get_component::<Velocity>(entity).is_none());
    
    // Other components should still exist
    assert!(world.get_component::<Position>(entity).is_some());
    assert!(world.get_component::<Health>(entity).is_some());
}

#[test]
fn test_migration_performance() {
    use std::time::Instant;
    
    // Register component types
    register_component::<Position>();
    register_component::<Velocity>();
    
    let mut world = World::new();
    let entity_count = 1000;
    let mut entities = Vec::new();
    
    // Create entities with one component
    for i in 0..entity_count {
        let entity = world.spawn();
        world.add_component(entity, Position { x: i as f32, y: i as f32 }).unwrap();
        entities.push(entity);
    }
    
    // Time adding second component (triggers migration)
    let start = Instant::now();
    for (i, &entity) in entities.iter().enumerate() {
        world.add_component(entity, Velocity { dx: i as f32, dy: 0.0 }).unwrap();
    }
    let duration = start.elapsed();
    
    println!("Migration of {} entities took: {:?}", entity_count, duration);
    
    // Should complete in reasonable time (< 100ms for 1000 entities)
    assert!(duration.as_millis() < 100, "Migration too slow: {:?}", duration);
}