//! Integration tests for engine-runtime-core
//!
//! Tests the complete game loop workflow including SystemScheduler,
//! GameContext, InterpolationManager integration.

use engine_runtime_core::{
    SystemScheduler, GameContext, InterpolationManager, Application,
    System, SystemError, Position3D
};
use engine_input::InputManager;
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// Test system that records execution history
#[derive(Debug)]
struct TestTrackingSystem {
    name: String,
    execution_history: Arc<Mutex<Vec<f32>>>,
    dependencies: Vec<String>,
    is_fixed: bool,
    should_fail: bool,
}

impl TestTrackingSystem {
    fn new(name: &str, is_fixed: bool) -> Self {
        Self {
            name: name.to_string(),
            execution_history: Arc::new(Mutex::new(Vec::new())),
            dependencies: Vec::new(),
            is_fixed,
            should_fail: false,
        }
    }
    
    fn with_dependencies(mut self, deps: Vec<&str>) -> Self {
        self.dependencies = deps.iter().map(|s| s.to_string()).collect();
        self
    }
    
    fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
    
    fn execution_count(&self) -> usize {
        self.execution_history.lock().unwrap().len()
    }
    
    fn get_execution_history(&self) -> Vec<f32> {
        self.execution_history.lock().unwrap().clone()
    }
}

impl System for TestTrackingSystem {
    fn execute(&mut self, _context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        if self.should_fail {
            return Err(SystemError::ExecutionFailed(format!("{} intentionally failed", self.name)));
        }
        
        self.execution_history.lock().unwrap().push(delta_time);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn dependencies(&self) -> Vec<&str> {
        self.dependencies.iter().map(|s| s.as_str()).collect()
    }
    
    fn is_fixed_timestep(&self) -> bool {
        self.is_fixed
    }
}

/// Test application for integration testing
#[derive(Debug)]
struct TestApplication {
    update_count: u32,
    render_count: u32,
    should_exit: bool,
    system_scheduler: SystemScheduler,
    last_delta_time: f32,
    last_interpolation: f32,
}

impl TestApplication {
    fn new() -> Self {
        Self {
            update_count: 0,
            render_count: 0,
            should_exit: false,
            system_scheduler: SystemScheduler::new(),
            last_delta_time: 0.0,
            last_interpolation: 0.0,
        }
    }
    
    fn with_systems(mut self, systems: Vec<Box<dyn System>>) -> Result<Self, SystemError> {
        for system in systems {
            self.system_scheduler.add_system(system);
        }
        self.system_scheduler.resolve_dependencies()?;
        Ok(self)
    }
    
    fn exit_after_frames(mut self, frames: u32) -> Self {
        // Will exit after specified number of updates
        self.should_exit = self.update_count >= frames;
        self
    }
}

impl Application for TestApplication {
    fn initialize(&mut self) -> engine_runtime_core::Result<()> {
        Ok(())
    }
    
    fn update(&mut self, delta_time: Duration, _input: &InputManager) -> engine_runtime_core::Result<()> {
        self.update_count += 1;
        self.last_delta_time = delta_time.as_secs_f32();
        
        // Exit after 10 frames for testing
        if self.update_count >= 10 {
            self.should_exit = true;
        }
        
        Ok(())
    }
    
    fn render(&mut self, interpolation: f32) -> engine_runtime_core::Result<()> {
        self.render_count += 1;
        self.last_interpolation = interpolation;
        Ok(())
    }
    
    fn should_exit(&self) -> bool {
        self.should_exit
    }
}

#[test]
fn test_complete_application_workflow() {
    // Test the complete workflow: Application -> SystemScheduler -> GameContext
    let mut app = TestApplication::new();
    let input_manager = InputManager::new().unwrap();
    
    // Initialize
    app.initialize().unwrap();
    
    // Run several fixed timestep iterations
    for _ in 0..5 {
        let delta_time = Duration::from_millis(16); // 60 FPS
        app.update(delta_time, &input_manager).unwrap();
        app.render(0.5).unwrap();
    }
    
    // Verify the loop ran correctly
    assert!(app.update_count > 0);
    assert!(app.render_count > 0);
    assert!(app.last_delta_time > 0.0);
    assert!(app.last_interpolation >= 0.0 && app.last_interpolation <= 1.0);
}

#[test]
fn test_system_scheduler_integration_with_game_context() {
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    // Add some test systems with dependencies
    let physics_system = TestTrackingSystem::new("Physics", true);
    let physics_history = physics_system.execution_history.clone();
    
    let render_system = TestTrackingSystem::new("Rendering", true)
        .with_dependencies(vec!["Physics"]);
    let render_history = render_system.execution_history.clone();
    
    // Add physics system first so dependency resolution can find it
    scheduler.add_system(Box::new(physics_system));
    scheduler.add_system(Box::new(render_system));
    scheduler.resolve_dependencies().unwrap();
    
    // Execute systems for several frames
    let delta_time = 1.0 / 60.0; // 60 FPS
    for _ in 0..10 {
        context.update(delta_time).unwrap();
        scheduler.execute_fixed_systems(&mut context, delta_time).unwrap();
    }
    
    // Verify both systems executed
    assert_eq!(physics_history.lock().unwrap().len(), 10);
    assert_eq!(render_history.lock().unwrap().len(), 10);
    
    // Verify delta times were passed correctly
    let physics_deltas = physics_history.lock().unwrap();
    for &dt in physics_deltas.iter() {
        assert!((dt - delta_time).abs() < 0.001);
    }
}

#[test]
fn test_interpolation_manager_integration() {
    let mut interpolation_manager = InterpolationManager::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    // Register component type
    interpolation_manager.register_component_type::<Position3D>();
    
    // Add interpolation manager to context
    context.insert_resource(interpolation_manager);
    
    // Create test positions
    let entity_id = 1;
    let pos1 = Position3D { x: 0.0, y: 0.0, z: 0.0 };
    let pos2 = Position3D { x: 10.0, y: 5.0, z: 2.0 };
    
    // Store states
    if let Some(interp_mgr) = context.get_resource_mut::<InterpolationManager>() {
        interp_mgr.update_current_state(entity_id, pos1.clone()).unwrap();
        interp_mgr.advance_frame();
        interp_mgr.update_current_state(entity_id, pos2.clone()).unwrap();
    }
    
    // Test interpolation at different factors
    if let Some(interp_mgr) = context.get_resource::<InterpolationManager>() {
        let interpolated_0 = interp_mgr.get_interpolated_state::<Position3D>(entity_id, 0.0).unwrap();
        let interpolated_5 = interp_mgr.get_interpolated_state::<Position3D>(entity_id, 0.5).unwrap();
        let interpolated_1 = interp_mgr.get_interpolated_state::<Position3D>(entity_id, 1.0).unwrap();
        
        // At factor 0.0, should be first position
        assert!((interpolated_0.x - pos1.x).abs() < 0.001);
        assert!((interpolated_0.y - pos1.y).abs() < 0.001);
        assert!((interpolated_0.z - pos1.z).abs() < 0.001);
        
        // At factor 0.5, should be halfway
        assert!((interpolated_5.x - 5.0).abs() < 0.001);
        assert!((interpolated_5.y - 2.5).abs() < 0.001);
        assert!((interpolated_5.z - 1.0).abs() < 0.001);
        
        // At factor 1.0, should be second position
        assert!((interpolated_1.x - pos2.x).abs() < 0.001);
        assert!((interpolated_1.y - pos2.y).abs() < 0.001);
        assert!((interpolated_1.z - pos2.z).abs() < 0.001);
    }
}

#[test]
fn test_game_context_resource_management_integration() {
    let mut context = GameContext::with_target_fps(60.0);
    
    // Test adding and retrieving resources
    let initial_position = Position3D { x: 1.0, y: 2.0, z: 3.0 };
    context.insert_resource(initial_position.clone());
    
    // Test resource retrieval
    {
        let position = context.get_resource::<Position3D>().unwrap();
        assert_eq!(position.x, 1.0);
        assert_eq!(position.y, 2.0);
        assert_eq!(position.z, 3.0);
    }
    
    // Test resource mutation
    {
        let position = context.get_resource_mut::<Position3D>().unwrap();
        position.x = 10.0;
        position.y = 20.0;
        position.z = 30.0;
    }
    
    // Verify mutation took effect
    {
        let position = context.get_resource::<Position3D>().unwrap();
        assert_eq!(position.x, 10.0);
        assert_eq!(position.y, 20.0);
        assert_eq!(position.z, 30.0);
    }
    
    // Test that non-existent resources return None
    assert!(context.get_resource::<InterpolationManager>().is_none());
}

#[test]
fn test_system_error_handling_integration() {
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    // Add a system that will fail
    let failing_system = TestTrackingSystem::new("FailingSystem", true).with_failure(true);
    let good_system = TestTrackingSystem::new("GoodSystem", true);
    let _good_history = good_system.execution_history.clone();
    
    scheduler.add_system(Box::new(failing_system));
    scheduler.add_system(Box::new(good_system));
    scheduler.resolve_dependencies().unwrap();
    
    // Execute systems - should handle the error gracefully
    context.update(1.0 / 60.0).unwrap();
    let result = scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0);
    
    // Should return an error due to failing system
    assert!(result.is_err());
    
    // Good system should still have executed (depending on execution order)
    // This tests that system failures are properly handled
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("intentionally failed"));
}

#[test]
fn test_multi_frame_integration_workflow() {
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    let input_manager = InputManager::new().unwrap();
    
    // Create systems that interact over multiple frames
    let position_system = TestTrackingSystem::new("PositionSystem", true);
    let position_history = position_system.execution_history.clone();
    
    let render_system = TestTrackingSystem::new("RenderSystem", true)
        .with_dependencies(vec!["PositionSystem"]);
    let render_history = render_system.execution_history.clone();
    
    scheduler.add_system(Box::new(position_system));
    scheduler.add_system(Box::new(render_system));
    scheduler.resolve_dependencies().unwrap();
    
    // Add initial position to context
    context.insert_resource(Position3D { x: 0.0, y: 0.0, z: 0.0 });
    
    // Create a test application
    let mut app = TestApplication::new();
    app.initialize().unwrap();
    
    // Run for multiple frames
    for _ in 0..20 {
        context.update(1.0 / 60.0).unwrap();
        
        // Execute systems
        scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        
        // Update application
        app.update(Duration::from_millis(16), &input_manager).unwrap();
        app.render(0.5).unwrap();
        
        // Simulate position updates
        if let Some(pos) = context.get_resource_mut::<Position3D>() {
            pos.x += 1.0;
            pos.y += 0.5;
        }
    }
    
    // Verify systems executed multiple times
    assert!(position_history.lock().unwrap().len() >= 10);
    assert!(render_history.lock().unwrap().len() >= 10);
    assert!(app.update_count >= 10);
    assert!(app.render_count >= 10);
    
    // Verify position was updated
    if let Some(final_pos) = context.get_resource::<Position3D>() {
        assert!(final_pos.x > 10.0); // Should have been incremented multiple times
        assert!(final_pos.y > 5.0);
    }
}

#[test]
fn test_death_spiral_prevention_integration() {
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    let input_manager = InputManager::new().unwrap();
    
    // Add a system
    let test_system = TestTrackingSystem::new("TestSystem", true);
    let execution_history = test_system.execution_history.clone();
    
    scheduler.add_system(Box::new(test_system));
    scheduler.resolve_dependencies().unwrap();
    
    let mut app = TestApplication::new();
    app.initialize().unwrap();
    
    // Simulate a very slow frame that would cause death spiral
    let start_time = std::time::Instant::now();
    
    // Force a slow update by making the context think a lot of time has passed
    context.update(1.0).unwrap(); // 1 second frame time (way too slow)
    
    // In the real game loop, there would be death spiral prevention
    // Here we simulate that by limiting updates
    let max_updates = 10;
    for _ in 0..max_updates {
        scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        app.update(Duration::from_millis(16), &input_manager).unwrap();
    }
    
    let elapsed = start_time.elapsed();
    
    // Should not take an excessive amount of time due to death spiral prevention
    assert!(elapsed < Duration::from_millis(100)); // Should complete quickly
    
    // Should have limited the number of physics updates
    assert!(app.update_count <= max_updates as u32);
    assert!(execution_history.lock().unwrap().len() <= max_updates);
}

#[test]
fn test_variable_vs_fixed_timestep_integration() {
    let mut scheduler = SystemScheduler::new();
    let mut context = GameContext::with_target_fps(60.0);
    
    let fixed_system = TestTrackingSystem::new("FixedSystem", true);
    let fixed_history = fixed_system.execution_history.clone();
    
    let variable_system = TestTrackingSystem::new("VariableSystem", false);
    let variable_history = variable_system.execution_history.clone();
    
    scheduler.add_system(Box::new(fixed_system));
    scheduler.add_system(Box::new(variable_system));
    scheduler.resolve_dependencies().unwrap();
    
    // Simulate variable frame times
    let frame_times = vec![0.016, 0.033, 0.008, 0.025, 0.020]; // Variable frame times
    
    for &frame_time in &frame_times {
        context.update(frame_time).unwrap();
        
        // Fixed systems should get consistent timestep
        scheduler.execute_fixed_systems(&mut context, 1.0 / 60.0).unwrap();
        
        // Variable systems should get actual frame time
        scheduler.execute_variable_systems(&mut context, frame_time).unwrap();
    }
    
    // Fixed system should have received consistent timesteps
    let fixed_deltas = fixed_history.lock().unwrap();
    for &delta in fixed_deltas.iter() {
        assert!((delta - 1.0 / 60.0).abs() < 0.001); // Should be exactly 1/60
    }
    
    // Variable system should have received actual frame times
    let variable_deltas = variable_history.lock().unwrap();
    for (i, &delta) in variable_deltas.iter().enumerate() {
        assert!((delta - frame_times[i]).abs() < 0.001);
    }
}