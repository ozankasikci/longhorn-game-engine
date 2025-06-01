// Test application demonstrating ECS + Renderer integration
// This shows actual 3D objects (cubes/spheres) being rendered from ECS entities

use engine_core::{World, Transform, Mesh, MeshType, Camera, Name};
use engine_graphics::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("ECS + Renderer Test - Mobile Game Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // Initialize renderer
    let mut renderer = Renderer::new(&window).await?;
    
    // Create ECS world and populate with test objects
    let mut world = World::new();
    
    // Create main camera
    let camera_entity = world.spawn();
    world.add_component(camera_entity, Camera { 
        fov: 60.0, 
        near: 0.1, 
        far: 100.0, 
        is_main: true 
    })?;
    world.add_component(camera_entity, Transform {
        position: [0.0, 0.0, 5.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera_entity, Name::new("Main Camera"))?;
    
    // Create a red cube at origin
    let cube_entity = world.spawn();
    world.add_component(cube_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(cube_entity, Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(cube_entity, Name::new("Red Cube"))?;
    
    // Create a sphere to the right
    let sphere_entity = world.spawn();
    world.add_component(sphere_entity, Mesh { mesh_type: MeshType::Sphere })?;
    world.add_component(sphere_entity, Transform {
        position: [3.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(sphere_entity, Name::new("Sphere"))?;
    
    // Create another cube to the left
    let cube2_entity = world.spawn();
    world.add_component(cube2_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(cube2_entity, Transform {
        position: [-3.0, 0.0, 0.0],
        rotation: [45.0, 0.0, 0.0],
        scale: [0.8, 0.8, 0.8],
    })?;
    world.add_component(cube2_entity, Name::new("Rotated Cube"))?;
    
    println!("Created ECS world with {} entities:", world.entity_count());
    for (entity, name) in world.query::<Name>() {
        println!("  Entity {}: {}", entity.id(), name.name);
    }
    
    let mut rotation_time = 0.0f32;
    
    let window_id = window.id();
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: event_window_id,
            } if event_window_id == window_id => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Animate the objects by rotating them
                    rotation_time += 0.016; // ~60 FPS
                    
                    // Rotate the first cube
                    if let Some(transform) = world.get_component_mut::<Transform>(cube_entity) {
                        transform.rotation[1] = rotation_time * 30.0; // 30 degrees per second
                    }
                    
                    // Animate the sphere's Y position
                    if let Some(transform) = world.get_component_mut::<Transform>(sphere_entity) {
                        transform.position[1] = (rotation_time * 2.0).sin() * 0.5;
                    }
                    
                    // Render the world
                    match renderer.render(&world) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            // We can't access window.inner_size() here since window is borrowed by renderer
                            // For now, just log the error - in a real app you'd track the size separately
                            eprintln!("Surface lost - would need to resize");
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                // We need to request redraw through the window, but it's borrowed by renderer
                // For this example, we'll use a different approach - just trigger redraws in a loop
                target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
            _ => {}
        }
    })?;
    
    Ok(())
}