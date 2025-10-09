//! Integration tests for Phase 27.2 Core Loop Implementation
//! 
//! This test validates that all components work together correctly:
//! - SystemScheduler, GameContext, and InterpolationManager integration
//! - Fixed and variable timestep system execution
//! - Resource management and interpolation state handling

use engine_runtime_core::{
    SystemScheduler, System, SystemError, GameContext, 
    InterpolationManager, Position3D, Scale3D
};

#[derive(Debug)]
struct TestPhysicsSystem {
    update_count: u32,
}

impl TestPhysicsSystem {
    fn new() -> Self {
        Self { update_count: 0 }
    }
}

impl System for TestPhysicsSystem {
    fn execute(&mut self, context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        self.update_count += 1;
        
        // Update position based on physics
        if let Some(position) = context.get_resource_mut::<Position3D>() {
            position.x += 10.0 * delta_time; // Move 10 units per second
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "TestPhysicsSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        true
    }
}

#[derive(Debug)]
struct TestRenderSystem {
    render_count: u32,
    last_interpolated_x: f32,
}

impl TestRenderSystem {
    fn new() -> Self {
        Self { 
            render_count: 0,
            last_interpolated_x: 0.0,
        }
    }
    
    fn get_last_interpolated_x(&self) -> f32 {
        self.last_interpolated_x
    }
}

impl System for TestRenderSystem {
    fn execute(&mut self, context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
        self.render_count += 1;
        
        // Get interpolated position for rendering
        if let Some(interp_manager) = context.get_resource::<InterpolationManager>() {
            if let Ok(interpolated_pos) = interp_manager.get_interpolated_state::<Position3D>(1, 0.5) {
                self.last_interpolated_x = interpolated_pos.x;
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "TestRenderSystem"
    }
    
    fn is_fixed_timestep(&self) -> bool {
        false
    }
}

#[test]
fn test_complete_game_loop_integration() {
    // Set up game context with resources
    let mut context = GameContext::with_target_fps(60.0);
    context.insert_resource(Position3D::new(0.0, 0.0, 0.0));
    context.insert_resource(Scale3D::uniform(1.0));
    
    // Set up interpolation manager
    let mut interp_manager = InterpolationManager::new();
    interp_manager.register_component_type::<Position3D>();
    interp_manager.register_component_type::<Scale3D>();
    context.insert_resource(interp_manager);
    
    // Set up system scheduler
    let mut scheduler = SystemScheduler::new();
    scheduler.add_system(Box::new(TestPhysicsSystem::new()));
    scheduler.add_system(Box::new(TestRenderSystem::new()));
    
    // Verify system separation
    assert_eq!(scheduler.fixed_system_count(), 1);
    assert_eq!(scheduler.variable_system_count(), 1);
    
    // Resolve dependencies
    scheduler.resolve_dependencies().unwrap();
    assert!(scheduler.are_dependencies_resolved());
    
    // Simulate multiple frames
    for frame in 0..5 {
        // Update interpolation state before physics
        let position_clone = context.get_resource::<Position3D>().cloned();
        if let (Some(interp_manager), Some(position)) = (
            context.get_resource_mut::<InterpolationManager>(),
            position_clone
        ) {
            interp_manager.update_current_state(1, position).unwrap();
        }
        
        // Execute fixed timestep systems
        let fixed_timestep = 1.0 / 60.0; // 60 FPS
        scheduler.execute_fixed_systems(&mut context, fixed_timestep).unwrap();
        
        // Advance interpolation frame
        if let Some(interp_manager) = context.get_resource_mut::<InterpolationManager>() {
            interp_manager.advance_frame();
        }
        
        // Execute variable timestep systems
        let variable_timestep = 1.0 / 120.0; // 120 FPS
        scheduler.execute_variable_systems(&mut context, variable_timestep).unwrap();
        
        // Update context
        context.update(variable_timestep).unwrap();
        
        // Verify position has been updated by physics
        let position = context.get_resource::<Position3D>().unwrap();
        let expected_x = 10.0 * fixed_timestep * (frame + 1) as f32;
        assert!((position.x - expected_x).abs() < 0.001, 
                "Frame {}: Expected x={:.3}, got x={:.3}", frame, expected_x, position.x);
    }
    
    // Verify frame count
    assert_eq!(context.frame_count(), 5);
    
    // Verify that context delta time is being tracked
    assert!(context.delta_time() > 0.0);
}

#[test]
fn test_system_dependency_resolution() {
    let mut scheduler = SystemScheduler::new();
    
    #[derive(Debug)]
    struct SystemA;
    #[derive(Debug)]
    struct SystemB;
    #[derive(Debug)]
    struct SystemC;
    
    impl System for SystemA {
        fn execute(&mut self, _: &mut GameContext, _: f32) -> Result<(), SystemError> { Ok(()) }
        fn name(&self) -> &str { "SystemA" }
        fn is_fixed_timestep(&self) -> bool { true }
    }
    
    impl System for SystemB {
        fn execute(&mut self, _: &mut GameContext, _: f32) -> Result<(), SystemError> { Ok(()) }
        fn name(&self) -> &str { "SystemB" }
        fn dependencies(&self) -> Vec<&str> { vec!["SystemA"] }
        fn is_fixed_timestep(&self) -> bool { true }
    }
    
    impl System for SystemC {
        fn execute(&mut self, _: &mut GameContext, _: f32) -> Result<(), SystemError> { Ok(()) }
        fn name(&self) -> &str { "SystemC" }
        fn dependencies(&self) -> Vec<&str> { vec!["SystemB"] }
        fn is_fixed_timestep(&self) -> bool { true }
    }
    
    // Add systems in random order
    scheduler.add_system(Box::new(SystemC));
    scheduler.add_system(Box::new(SystemA));
    scheduler.add_system(Box::new(SystemB));
    
    // Resolve dependencies
    scheduler.resolve_dependencies().unwrap();
    
    // Verify correct execution order
    let execution_order = scheduler.fixed_execution_order();
    assert_eq!(execution_order.len(), 3);
    assert_eq!(execution_order[0], "SystemA");
    assert_eq!(execution_order[1], "SystemB");
    assert_eq!(execution_order[2], "SystemC");
}

#[test]
fn test_interpolation_manager_integration() {
    let mut context = GameContext::new();
    
    // Set up interpolation manager
    let mut interp_manager = InterpolationManager::new();
    interp_manager.register_component_type::<Position3D>();
    
    // Test initial state
    let initial_pos = Position3D::new(0.0, 0.0, 0.0);
    interp_manager.update_current_state(1, initial_pos.clone()).unwrap();
    
    // Without previous state, should return current state
    let result = interp_manager.get_interpolated_state::<Position3D>(1, 0.5).unwrap();
    assert_eq!(result, initial_pos);
    
    // Advance frame and set new state
    interp_manager.advance_frame();
    let new_pos = Position3D::new(10.0, 20.0, 30.0);
    interp_manager.update_current_state(1, new_pos).unwrap();
    
    // Test interpolation
    let interpolated = interp_manager.get_interpolated_state::<Position3D>(1, 0.5).unwrap();
    assert_eq!(interpolated, Position3D::new(5.0, 10.0, 15.0));
    
    // Add to context and verify resource management
    context.insert_resource(interp_manager);
    assert!(context.has_resource::<InterpolationManager>());
    
    // Verify we can retrieve and use it
    let interp_ref = context.get_resource::<InterpolationManager>().unwrap();
    assert!(interp_ref.is_component_registered::<Position3D>());
}

#[test]
fn test_resource_management() {
    let mut context = GameContext::new();
    
    // Test inserting resources
    context.insert_resource(Position3D::new(1.0, 2.0, 3.0));
    context.insert_resource(Scale3D::new(2.0, 2.0, 2.0));
    
    // Test resource retrieval
    assert!(context.has_resource::<Position3D>());
    assert!(context.has_resource::<Scale3D>());
    
    let pos = context.get_resource::<Position3D>().unwrap();
    assert_eq!(pos.x, 1.0);
    assert_eq!(pos.y, 2.0);
    assert_eq!(pos.z, 3.0);
    
    // Test mutable access
    {
        let pos_mut = context.get_resource_mut::<Position3D>().unwrap();
        pos_mut.x = 10.0;
    }
    
    let updated_pos = context.get_resource::<Position3D>().unwrap();
    assert_eq!(updated_pos.x, 10.0);
    
    // Test resource removal
    let removed_pos = context.remove_resource::<Position3D>().unwrap();
    assert_eq!(removed_pos.x, 10.0);
    assert!(!context.has_resource::<Position3D>());
    assert!(context.has_resource::<Scale3D>()); // Other resources unaffected
}

#[test]
fn test_time_manager_integration() {
    let mut context = GameContext::with_target_fps(120.0);
    
    // Verify target FPS setting
    let target_fps = context.time.target_fps();
    assert!((target_fps - 120.0).abs() < 0.01);
    
    // Test time manager updates
    let initial_frame_count = context.frame_count();
    context.update(1.0 / 120.0).unwrap();
    
    assert_eq!(context.frame_count(), initial_frame_count + 1);
    assert!(context.delta_time() >= 0.0);
}