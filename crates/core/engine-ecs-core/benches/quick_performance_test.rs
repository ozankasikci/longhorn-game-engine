// Quick performance comparison test
use std::time::Instant;
use engine_core::{Transform, Component};
use engine_core::ecs::{World as OldWorld};
use engine_core::ecs_v2::{World as NewWorld, Read};

fn main() {
    println!("=== ECS Performance Comparison ===\n");
    
    // Test 1: Entity creation with 10k entities
    println!("📊 Test 1: Entity Creation (10,000 entities)");
    
    let start = Instant::now();
    let mut old_world = OldWorld::new();
    for _i in 0..10000 {
        let entity = old_world.spawn();
        old_world.add_component(entity, Transform::default()).unwrap();
    }
    let old_time = start.elapsed();
    println!("   Old ECS: {:?}", old_time);
    
    let start = Instant::now();
    let mut new_world = NewWorld::new();
    for _i in 0..10000 {
        let entity = new_world.spawn();
        new_world.add_component(entity, Transform::default()).unwrap();
    }
    let new_time = start.elapsed();
    println!("   New ECS: {:?}", new_time);
    
    let speedup = old_time.as_nanos() as f64 / new_time.as_nanos() as f64;
    println!("   Speedup: {:.2}x {}", speedup, if speedup > 1.0 { "🚀" } else { "⚠️" });
    
    // Test 2: Query iteration
    println!("\n📊 Test 2: Query Iteration (10,000 entities)");
    
    let start = Instant::now();
    let mut sum = 0.0f32;
    for (_, transform) in old_world.query::<Transform>() {
        sum += transform.position[0];
    }
    let old_query_time = start.elapsed();
    println!("   Old ECS Query: {:?} (sum: {:.2})", old_query_time, sum);
    
    let start = Instant::now();
    let mut sum = 0.0f32;
    for (_, transform) in new_world.query::<Read<Transform>>().iter() {
        sum += transform.position[0];
    }
    let new_query_time = start.elapsed();
    println!("   New ECS Query: {:?} (sum: {:.2})", new_query_time, sum);
    
    let query_speedup = old_query_time.as_nanos() as f64 / new_query_time.as_nanos() as f64;
    println!("   Query Speedup: {:.2}x {}", query_speedup, if query_speedup > 1.0 { "🚀" } else { "⚠️" });
    
    // Test 3: Memory efficiency
    println!("\n📊 Test 3: Memory Layout");
    println!("   Old ECS entities: {}", old_world.entity_count());
    println!("   New ECS entities: {}", new_world.entity_count());
    println!("   New ECS archetypes: {}", new_world.archetype_count());
    
    // Summary
    println!("\n🎯 Summary:");
    println!("   Entity Creation: {:.2}x {}", speedup, if speedup > 1.0 { "faster" } else { "slower" });
    println!("   Query Performance: {:.2}x {}", query_speedup, if query_speedup > 1.0 { "faster" } else { "slower" });
    println!("   Cache Efficiency: {} archetype(s) vs scattered storage", new_world.archetype_count());
    
    if speedup > 1.0 && query_speedup > 1.0 {
        println!("   ✅ ECS v2 shows performance improvements!");
    } else {
        println!("   ⚠️  Performance results need analysis");
    }
}