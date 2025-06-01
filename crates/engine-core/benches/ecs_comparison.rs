// Benchmark comparing old vs new ECS performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use engine_core::{Transform, Component, ComponentV2};
use engine_core::ecs::{World as OldWorld, Entity as OldEntity};
use engine_core::ecs_v2::{World as NewWorld, Entity as NewEntity};
use std::time::Duration;

#[derive(Debug, Clone)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Velocity {}
impl ComponentV2 for Velocity {}

#[derive(Debug, Clone)]
struct Health {
    current: f32,
    max: f32,
}

impl Component for Health {}
impl ComponentV2 for Health {}

fn benchmark_entity_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_creation");
    
    for entity_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("old_ecs", entity_count),
            entity_count,
            |b, &entity_count| {
                b.iter(|| {
                    let mut world = OldWorld::new();
                    for _i in 0..entity_count {
                        let entity = world.spawn();
                        world.add_component(entity, Transform::default()).unwrap();
                    }
                    black_box(world);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("new_ecs", entity_count),
            entity_count,
            |b, &entity_count| {
                b.iter(|| {
                    let mut world = NewWorld::new();
                    for _i in 0..entity_count {
                        let entity = world.spawn();
                        world.add_component(entity, Transform::default()).unwrap();
                    }
                    black_box(world);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");
    
    for entity_count in [1000, 10000, 100000].iter() {
        // Setup old ECS
        let mut old_world = OldWorld::new();
        for _i in 0..*entity_count {
            let entity = old_world.spawn();
            old_world.add_component(entity, Transform::default()).unwrap();
        }
        
        // Setup new ECS
        let mut new_world = NewWorld::new();
        for _i in 0..*entity_count {
            let entity = new_world.spawn();
            new_world.add_component(entity, Transform::default()).unwrap();
        }
        
        group.bench_with_input(
            BenchmarkId::new("old_ecs_transform_query", entity_count),
            entity_count,
            |b, &_entity_count| {
                b.iter(|| {
                    let mut sum = 0.0f32;
                    for (_, transform) in old_world.query::<Transform>() {
                        sum += transform.position[0];
                    }
                    black_box(sum);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("new_ecs_transform_query", entity_count),
            entity_count,
            |b, &_entity_count| {
                b.iter(|| {
                    let mut sum = 0.0f32;
                    for (_, transform) in new_world.query::<Transform>() {
                        sum += transform.position[0];
                    }
                    black_box(sum);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("new_ecs_parallel_query", entity_count),
            entity_count,
            |b, &_entity_count| {
                b.iter(|| {
                    use rayon::prelude::*;
                    let sum: f32 = new_world.par_query::<Transform>()
                        .map(|(_, transform)| transform.position[0])
                        .sum();
                    black_box(sum);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_component_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_access");
    
    const ENTITY_COUNT: usize = 10000;
    
    // Setup old ECS
    let mut old_world = OldWorld::new();
    let mut old_entities = Vec::new();
    for _i in 0..ENTITY_COUNT {
        let entity = old_world.spawn();
        old_world.add_component(entity, Transform::default()).unwrap();
        old_entities.push(entity);
    }
    
    // Setup new ECS
    let mut new_world = NewWorld::new();
    let mut new_entities = Vec::new();
    for _i in 0..ENTITY_COUNT {
        let entity = new_world.spawn();
        new_world.add_component(entity, Transform::default()).unwrap();
        new_entities.push(entity);
    }
    
    group.bench_function("old_ecs_random_access", |b| {
        b.iter(|| {
            for &entity in &old_entities {
                if let Some(transform) = old_world.get_component::<Transform>(entity) {
                    black_box(transform.position[0]);
                }
            }
        });
    });
    
    group.bench_function("new_ecs_random_access", |b| {
        b.iter(|| {
            for &entity in &new_entities {
                if let Some(transform) = new_world.get_component::<Transform>(entity) {
                    black_box(transform.position[0]);
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_entity_creation,
    benchmark_iteration,
    benchmark_component_access
);
criterion_main!(benches);