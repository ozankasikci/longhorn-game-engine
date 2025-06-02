// Simple test for archetype migration in editor pattern
use engine_core::{Transform, WorldV2, EntityV2, Read, Name};

fn main() {
    println!("Testing archetype migration like in editor...");
    
    let mut world = WorldV2::new();
    
    // Create entities like in the editor
    let entity1 = world.spawn();
    world.add_component(entity1, Transform::default()).unwrap();
    
    let entity2 = world.spawn();
    world.add_component(entity2, Transform::default()).unwrap();
    
    let entity3 = world.spawn();
    world.add_component(entity3, Transform::default()).unwrap();
    
    println!("Created 3 entities, archetypes: {}", world.archetype_count());
    
    // Query them like in the editor
    for (entity, _transform) in world.query::<Read<Transform>>().iter() {
        println!("Entity {} has Transform", entity.id());
    }
    
    // Now add components like in the editor
    println!("Adding Name to entity 1...");
    world.add_component(entity1, Name::new("Test Entity")).unwrap();
    println!("Success! Archetypes: {}", world.archetype_count());
    
    // Query again
    for (entity, _transform) in world.query::<Read<Transform>>().iter() {
        println!("Entity {} still has Transform", entity.id());
        if let Some(name) = world.get_component::<Name>(entity) {
            println!("  - Also has Name: {}", name.name);
        }
    }
    
    println!("Test completed successfully!");
}