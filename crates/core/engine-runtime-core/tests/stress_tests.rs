//! Stress tests for engine-runtime-core
//!
//! These tests push the system to its limits to identify breaking points,
//! memory leaks, and performance degradation under extreme conditions.

use engine_runtime_core::{
    SystemScheduler, GameContext, InterpolationManager, System, SystemError, Position3D
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Stress test system that simulates heavy workload
#[derive(Debug)]
struct StressTestSystem {
    name: String,
    work_multiplier: u32,
    execution_count: Arc<Mutex<u64>>,
    total_work_done: Arc<Mutex<u64>>,
}

impl StressTestSystem {
    fn new(name: &str, work_multiplier: u32) -> Self {
        Self {
            name: name.to_string(),
            work_multiplier,
            execution_count: Arc::new(Mutex::new(0)),
            total_work_done: Arc::new(Mutex::new(0)),
        }
    }
    
    fn execution_count(&self) -> u64 {
        *self.execution_count.lock().unwrap()
    }
    
    fn total_work_done(&self) -> u64 {
        *self.total_work_done.lock().unwrap()
    }
}

impl System for StressTestSystem {
    fn execute(&mut self, context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
        // Simulate heavy computational work
        let mut work_done = 0u64;
        for i in 0..(self.work_multiplier * 1000) {
            work_done = work_done.wrapping_add(i as u64);
            work_done = work_done.wrapping_mul(17);
            work_done = work_done.wrapping_add(13);
        }
        
        // Heavy resource access pattern
        for _ in 0..10 {
            if let Some(pos) = context.get_resource_mut::<Position3D>() {
                pos.x += work_done as f32 * 0.000001;
                pos.y = pos.x.sin();
                pos.z = pos.y.cos();
            }
        }
        
        *self.execution_count.lock().unwrap() += 1;
        *self.total_work_done.lock().unwrap() += work_done;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn is_fixed_timestep(&self) -> bool {
        true
    }
}

#[test]
fn stress_test_massive_system_count() {
    const SYSTEM_COUNT: usize = 1000;
    const EXECUTION_COUNT: usize = 100;
    
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    // Add initial resources
    context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
    
    println!("Creating {} systems for stress test...", SYSTEM_COUNT);
    
    // Add a massive number of systems
    let start_time = Instant::now();
    for i in 0..SYSTEM_COUNT {
        let system = StressTestSystem::new(&format!("StressSystem{}", i), 1);
        scheduler.add_system(Box::new(system));
    }
    
    scheduler.resolve_dependencies().unwrap();
    let setup_time = start_time.elapsed();
    
    println!("Setup completed in {:?}", setup_time);
    println!("Running {} executions with {} systems...", EXECUTION_COUNT, SYSTEM_COUNT);
    
    // Execute multiple times to stress test
    let execution_start = Instant::now();
    for frame in 0..EXECUTION_COUNT {
        context.update(1.0 / 60.0).unwrap();
        scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        
        if frame % 20 == 0 {
            println!("Completed frame {}/{}", frame + 1, EXECUTION_COUNT);
        }
    }
    let execution_time = execution_start.elapsed();
    
    println!("Execution completed in {:?}", execution_time);
    println!("Average time per frame: {:?}", execution_time / EXECUTION_COUNT as u32);
    
    // Verify all systems executed
    assert_eq!(scheduler.fixed_system_count(), SYSTEM_COUNT);
    
    // Memory should be released properly when dropping
    drop(scheduler);
    drop(context);
}

#[test]
fn stress_test_heavy_resource_churn() {
    const CHURN_CYCLES: usize = 10000;
    const RESOURCES_PER_CYCLE: usize = 100;
    
    let mut context = GameContext::with_target_fps(60.0);
    
    println!("Starting resource churn stress test: {} cycles, {} resources per cycle", 
             CHURN_CYCLES, RESOURCES_PER_CYCLE);
    
    let start_time = Instant::now();
    
    for cycle in 0..CHURN_CYCLES {
        // Add many resources
        for i in 0..RESOURCES_PER_CYCLE {
            let pos = Position3D { 
                x: (cycle * RESOURCES_PER_CYCLE + i) as f32, 
                y: (cycle as f32).sin(), 
                z: (i as f32).cos() 
            };
            context.insert_resource(pos);
        }
        
        // Access resources
        for _ in 0..10 {
            if let Some(pos) = context.get_resource_mut::<Position3D>() {
                pos.x += 1.0;
            }
        }
        
        // Remove and re-add resources
        context.remove_resource::<Position3D>();
        
        if cycle % 1000 == 0 {
            println!("Completed cycle {}/{}", cycle + 1, CHURN_CYCLES);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Resource churn completed in {:?}", elapsed);
    println!("Average time per cycle: {:?}", elapsed / CHURN_CYCLES as u32);
}

#[test]
fn stress_test_interpolation_with_many_entities() {
    const ENTITY_COUNT: u32 = 10000;
    const FRAME_COUNT: usize = 100;
    
    let mut interpolation_manager = InterpolationManager::new();
    interpolation_manager.register_component_type::<Position3D>();
    
    println!("Creating {} entities for interpolation stress test...", ENTITY_COUNT);
    
    // Create many entities with interpolation states
    let start_time = Instant::now();
    for entity_id in 0..ENTITY_COUNT {
        let pos = Position3D { 
            x: entity_id as f32, 
            y: (entity_id as f32 * 0.1).sin(), 
            z: (entity_id as f32 * 0.05).cos() 
        };
        interpolation_manager.update_current_state(entity_id, pos).unwrap();
    }
    let setup_time = start_time.elapsed();
    
    println!("Entity setup completed in {:?}", setup_time);
    println!("Running {} frames with interpolation...", FRAME_COUNT);
    
    // Simulate many frames with interpolation
    let execution_start = Instant::now();
    for frame in 0..FRAME_COUNT {
        interpolation_manager.advance_frame();
        
        // Update all entities
        for entity_id in 0..ENTITY_COUNT {
            let pos = Position3D { 
                x: entity_id as f32 + frame as f32 * 0.1, 
                y: ((entity_id as f32 * 0.1) + (frame as f32 * 0.01)).sin(), 
                z: ((entity_id as f32 * 0.05) + (frame as f32 * 0.005)).cos() 
            };
            interpolation_manager.update_current_state(entity_id, pos).unwrap();
        }
        
        // Perform interpolation for a subset of entities
        for entity_id in (0..ENTITY_COUNT).step_by(10) {
            let _interpolated = interpolation_manager
                .get_interpolated_state::<Position3D>(entity_id, 0.5);
        }
        
        if frame % 20 == 0 {
            println!("Completed frame {}/{}", frame + 1, FRAME_COUNT);
        }
    }
    let execution_time = execution_start.elapsed();
    
    println!("Interpolation stress test completed in {:?}", execution_time);
    println!("Average time per frame: {:?}", execution_time / FRAME_COUNT as u32);
}

#[test]
fn stress_test_deep_dependency_chain() {
    const CHAIN_LENGTH: usize = 100;
    const EXECUTION_COUNT: usize = 50;
    
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
    
    println!("Creating dependency chain of length {}...", CHAIN_LENGTH);
    
    // Create systems with dependency chain: System0 -> System1 -> System2 -> ...
    // Note: Due to current system architecture, we'll create independent systems
    // but track the logical dependency in our test
    let system_handles = Arc::new(Mutex::new(Vec::new()));
    
    for i in 0..CHAIN_LENGTH {
        let system = StressTestSystem::new(&format!("ChainSystem{}", i), 5);
        let handle = system.execution_count.clone();
        system_handles.lock().unwrap().push(handle);
        scheduler.add_system(Box::new(system));
    }
    
    scheduler.resolve_dependencies().unwrap();
    
    println!("Running {} executions with deep dependency chain...", EXECUTION_COUNT);
    
    let start_time = Instant::now();
    for frame in 0..EXECUTION_COUNT {
        context.update(1.0 / 60.0).unwrap();
        scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        
        if frame % 10 == 0 {
            println!("Completed frame {}/{}", frame + 1, EXECUTION_COUNT);
        }
    }
    let elapsed = start_time.elapsed();
    
    println!("Deep dependency chain test completed in {:?}", elapsed);
    
    // Verify all systems in the chain executed
    let handles = system_handles.lock().unwrap();
    for (i, handle) in handles.iter().enumerate() {
        let count = *handle.lock().unwrap();
        assert_eq!(count, EXECUTION_COUNT as u64, "System {} did not execute correctly", i);
    }
}

#[test] 
fn stress_test_concurrent_access() {
    const THREAD_COUNT: usize = 8;
    const ITERATIONS_PER_THREAD: usize = 1000;
    
    println!("Starting concurrent access stress test with {} threads...", THREAD_COUNT);
    
    let start_time = Instant::now();
    
    let handles: Vec<_> = (0..THREAD_COUNT).map(|thread_id| {
        thread::spawn(move || {
            let mut scheduler = SystemScheduler::new();
            let mut context = GameContext::with_target_fps(60.0);
            context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
            
            // Each thread creates its own systems
            for i in 0..10 {
                let system = StressTestSystem::new(&format!("Thread{}System{}", thread_id, i), 2);
                scheduler.add_system(Box::new(system));
            }
            
            scheduler.resolve_dependencies().unwrap();
            
            // Execute many iterations
            for iteration in 0..ITERATIONS_PER_THREAD {
                context.update(1.0 / 60.0).unwrap();
                scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
                
                if iteration % 200 == 0 {
                    println!("Thread {} completed iteration {}/{}", 
                             thread_id, iteration + 1, ITERATIONS_PER_THREAD);
                }
            }
            
            thread_id
        })
    }).collect();
    
    // Wait for all threads to complete
    let mut completed_threads = Vec::new();
    for handle in handles {
        completed_threads.push(handle.join().unwrap());
    }
    
    let elapsed = start_time.elapsed();
    
    println!("Concurrent access stress test completed in {:?}", elapsed);
    assert_eq!(completed_threads.len(), THREAD_COUNT);
    
    // Verify all threads completed successfully
    for i in 0..THREAD_COUNT {
        assert!(completed_threads.contains(&i));
    }
}

#[test]
fn stress_test_rapid_system_lifecycle() {
    const LIFECYCLE_CYCLES: usize = 100;
    const SYSTEMS_PER_CYCLE: usize = 50;
    
    println!("Starting rapid system lifecycle test: {} cycles, {} systems per cycle", 
             LIFECYCLE_CYCLES, SYSTEMS_PER_CYCLE);
    
    let start_time = Instant::now();
    
    for cycle in 0..LIFECYCLE_CYCLES {
        let mut scheduler = SystemScheduler::new();
        let mut context = GameContext::with_target_fps(60.0);
        context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
        
        // Create many systems
        for i in 0..SYSTEMS_PER_CYCLE {
            let system = StressTestSystem::new(&format!("Cycle{}System{}", cycle, i), 1);
            scheduler.add_system(Box::new(system));
        }
        
        scheduler.resolve_dependencies().unwrap();
        
        // Execute a few frames
        for _ in 0..5 {
            context.update(1.0 / 60.0).unwrap();
            scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        }
        
        // Scheduler and context will be dropped here, testing cleanup
        
        if cycle % 20 == 0 {
            println!("Completed lifecycle cycle {}/{}", cycle + 1, LIFECYCLE_CYCLES);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Rapid system lifecycle test completed in {:?}", elapsed);
    println!("Average time per cycle: {:?}", elapsed / LIFECYCLE_CYCLES as u32);
}

#[test]
fn stress_test_memory_pressure() {
    const PRESSURE_CYCLES: usize = 50;
    const MEMORY_ALLOCATIONS: usize = 1000;
    
    println!("Starting memory pressure stress test...");
    
    let start_time = Instant::now();
    
    for cycle in 0..PRESSURE_CYCLES {
        let mut scheduler = SystemScheduler::new();
        let mut context = GameContext::with_target_fps(60.0);
        let mut interpolation_manager = InterpolationManager::new();
        interpolation_manager.register_component_type::<Position3D>();
        
        // Create memory pressure with many allocations
        let mut _memory_pressure = Vec::new();
        for i in 0..MEMORY_ALLOCATIONS {
            _memory_pressure.push(vec![i; 1000]); // Allocate 1000 integers each
        }
        
        // Add resources and systems under memory pressure
        context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
        context.insert_resource(interpolation_manager);
        
        for i in 0..20 {
            let system = StressTestSystem::new(&format!("PressureSystem{}", i), 3);
            scheduler.add_system(Box::new(system));
        }
        
        scheduler.resolve_dependencies().unwrap();
        
        // Execute under memory pressure
        for _ in 0..10 {
            context.update(1.0 / 60.0).unwrap();
            scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        }
        
        // All allocations will be dropped here
        
        if cycle % 10 == 0 {
            println!("Completed memory pressure cycle {}/{}", cycle + 1, PRESSURE_CYCLES);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Memory pressure stress test completed in {:?}", elapsed);
}

#[test]
fn stress_test_error_recovery() {
    const ERROR_CYCLES: usize = 100;
    
    /// System that occasionally fails to test error handling
    #[derive(Debug)]
    struct FailingSystem {
        name: String,
        fail_frequency: u32,
        execution_count: u32,
    }
    
    impl FailingSystem {
        fn new(name: &str, fail_frequency: u32) -> Self {
            Self {
                name: name.to_string(),
                fail_frequency,
                execution_count: 0,
            }
        }
    }
    
    impl System for FailingSystem {
        fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
            self.execution_count += 1;
            
            if self.execution_count % self.fail_frequency == 0 {
                Err(SystemError::ExecutionFailed(format!("{} intentional failure", self.name)))
            } else {
                Ok(())
            }
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn is_fixed_timestep(&self) -> bool {
            true
        }
    }
    
    println!("Starting error recovery stress test with {} cycles...", ERROR_CYCLES);
    
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
    
    // Add systems with different failure rates
    scheduler.add_system(Box::new(FailingSystem::new("Fail5", 5)));
    scheduler.add_system(Box::new(FailingSystem::new("Fail10", 10)));
    scheduler.add_system(Box::new(FailingSystem::new("Fail20", 20)));
    scheduler.add_system(Box::new(StressTestSystem::new("Stable", 1)));
    
    scheduler.resolve_dependencies().unwrap();
    
    let start_time = Instant::now();
    let mut error_count = 0;
    let mut success_count = 0;
    
    for cycle in 0..ERROR_CYCLES {
        context.update(1.0 / 60.0).unwrap();
        
        match scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0) {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
        
        if cycle % 20 == 0 {
            println!("Completed error recovery cycle {}/{} (errors: {}, successes: {})", 
                     cycle + 1, ERROR_CYCLES, error_count, success_count);
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Error recovery stress test completed in {:?}", elapsed);
    println!("Total errors: {}, Total successes: {}", error_count, success_count);
    
    // We expect some errors due to failing systems
    assert!(error_count > 0, "Expected some errors from failing systems");
    assert!(success_count > 0, "Expected some successful executions");
}