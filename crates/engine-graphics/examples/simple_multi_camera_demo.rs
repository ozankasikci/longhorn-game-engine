//! Simple Multi-Camera Demo - Working Version
//! Uses basic ECS (World) instead of ECS v2 to ensure functionality

use engine_core::{World, Transform, Mesh, MeshType, Camera, Name};
use engine_graphics::Renderer;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🎮 Simple Multi-Camera Demo - Working Version");
    println!("📷 Features: Multiple cameras with basic ECS system");
    println!("🎯 Controls: 1-2 keys to switch main camera, Esc to exit");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Simple Multi-Camera Demo - Working")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?;

    // Initialize basic renderer (known to work)
    let mut renderer = Renderer::new(&window).await?;
    
    // Create basic ECS world
    let mut world = World::new();
    
    println!("🏗️ Creating multi-camera scene...");
    
    // Create Camera 1: Main perspective camera
    let camera1_entity = world.spawn();
    world.add_component(camera1_entity, Camera { 
        fov: 60.0, 
        near: 0.1, 
        far: 100.0, 
        is_main: true 
    })?;
    world.add_component(camera1_entity, Transform {
        position: [0.0, 2.0, 8.0],
        rotation: [-15.0, 0.0, 0.0], // Look down slightly
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera1_entity, Name::new("Main Camera (Perspective)"))?;
    
    // Create Camera 2: Wide-angle camera
    let camera2_entity = world.spawn();
    world.add_component(camera2_entity, Camera { 
        fov: 90.0,  // Wide angle
        near: 0.1, 
        far: 100.0, 
        is_main: false  // Start disabled
    })?;
    world.add_component(camera2_entity, Transform {
        position: [5.0, 1.0, 5.0],
        rotation: [0.0, -45.0, 0.0], // Look at center from side
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera2_entity, Name::new("Wide-angle Camera"))?;
    
    // Create test objects for the scene
    println!("📦 Creating scene objects...");
    
    // Central cube (red)
    let cube1_entity = world.spawn();
    world.add_component(cube1_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(cube1_entity, Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(cube1_entity, Name::new("Central Cube"))?;
    
    // Sphere to the right
    let sphere_entity = world.spawn();
    world.add_component(sphere_entity, Mesh { mesh_type: MeshType::Sphere })?;
    world.add_component(sphere_entity, Transform {
        position: [3.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(sphere_entity, Name::new("Right Sphere"))?;
    
    // Cube to the left
    let cube2_entity = world.spawn();
    world.add_component(cube2_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(cube2_entity, Transform {
        position: [-3.0, 0.0, 0.0],
        rotation: [45.0, 45.0, 0.0],
        scale: [0.8, 0.8, 0.8],
    })?;
    world.add_component(cube2_entity, Name::new("Left Rotated Cube"))?;
    
    // Elevated sphere
    let sphere2_entity = world.spawn();
    world.add_component(sphere2_entity, Mesh { mesh_type: MeshType::Sphere })?;
    world.add_component(sphere2_entity, Transform {
        position: [0.0, 3.0, -2.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [0.6, 0.6, 0.6],
    })?;
    world.add_component(sphere2_entity, Name::new("Elevated Sphere"))?;
    
    println!("✅ Created ECS world with {} entities", world.entity_count());
    
    // Print scene information
    for (entity, name) in world.query::<Name>() {
        if let Some(camera) = world.get_component::<Camera>(entity) {
            println!("  🎥 Camera {}: {} (main: {}, FOV: {}°)", 
                entity.id(), name.name, camera.is_main, camera.fov
            );
        } else if world.get_component::<Mesh>(entity).is_some() {
            println!("  📦 Mesh {}: {}", entity.id(), name.name);
        }
    }
    
    let mut rotation_time = 0.0f32;
    let mut current_camera = 1; // Track which camera is active
    
    let cameras = [camera1_entity, camera2_entity];
    let camera_names = ["Main Camera", "Wide-angle Camera"];
    
    println!("\n🎮 Demo Controls:");
    println!("  1-2: Switch to camera 1-2");
    println!("  ESC: Exit demo");
    println!("  Current camera: {} ({})", current_camera, camera_names[current_camera - 1]);
    
    let window_id = window.id();
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: event_window_id,
            } if event_window_id == window_id => match event {
                WindowEvent::CloseRequested => {
                    println!("👋 Closing simple multi-camera demo");
                    target.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Digit1) => {
                                if current_camera != 1 {
                                    // Disable camera 2, enable camera 1
                                    if let Some(camera2) = world.get_component_mut::<Camera>(cameras[1]) {
                                        camera2.is_main = false;
                                    }
                                    if let Some(camera1) = world.get_component_mut::<Camera>(cameras[0]) {
                                        camera1.is_main = true;
                                    }
                                    current_camera = 1;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Digit2) => {
                                if current_camera != 2 {
                                    // Disable camera 1, enable camera 2
                                    if let Some(camera1) = world.get_component_mut::<Camera>(cameras[0]) {
                                        camera1.is_main = false;
                                    }
                                    if let Some(camera2) = world.get_component_mut::<Camera>(cameras[1]) {
                                        camera2.is_main = true;
                                    }
                                    current_camera = 2;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Escape) => {
                                println!("👋 Exiting simple multi-camera demo");
                                target.exit();
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    // Animate the scene
                    rotation_time += 0.016; // ~60 FPS
                    
                    // Rotate the central cube
                    if let Some(transform) = world.get_component_mut::<Transform>(cube1_entity) {
                        transform.rotation[1] = rotation_time * 30.0; // 30 degrees per second
                    }
                    
                    // Animate the right sphere's Y position
                    if let Some(transform) = world.get_component_mut::<Transform>(sphere_entity) {
                        transform.position[1] = (rotation_time * 2.0).sin() * 0.5;
                    }
                    
                    // Rotate the left cube on multiple axes
                    if let Some(transform) = world.get_component_mut::<Transform>(cube2_entity) {
                        transform.rotation[0] = 45.0 + (rotation_time * 40.0).sin() * 15.0;
                        transform.rotation[1] = 45.0 + rotation_time * 25.0;
                    }
                    
                    // Animate the elevated sphere in a circular motion
                    if let Some(transform) = world.get_component_mut::<Transform>(sphere2_entity) {
                        transform.position[0] = (rotation_time * 1.5).cos() * 1.5;
                        transform.position[2] = -2.0 + (rotation_time * 1.5).sin() * 1.5;
                    }
                    
                    // Render the world with basic renderer (known to work)
                    match renderer.render(&world) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            eprintln!("⚠️ Surface lost - would need to resize");
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("💥 Out of memory!");
                            target.exit();
                        }
                        Err(e) => eprintln!("❌ Render error: {:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
            _ => {}
        }
    })?;
    
    Ok(())
}