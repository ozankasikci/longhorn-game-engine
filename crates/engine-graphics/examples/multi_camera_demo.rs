//! Multi-Camera Demo - Phase 4 Implementation
//! 
//! This example demonstrates the new multi-camera rendering system with:
//! - Multiple cameras with different priorities
//! - ECS v2 integration with engine-camera components
//! - Advanced camera features (different projection types, render orders)
//! - Real-time camera switching and property updates

use engine_core::{WorldV2, Transform, Mesh, MeshType, Name};
use engine_camera::{CameraComponent, CameraType, Viewport};
use engine_graphics::MultiCameraRenderer;
use winit::{
    event::{Event, WindowEvent, ElementState, KeyEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🎮 Multi-Camera Demo - Phase 4 Mobile Game Engine");
    println!("📷 Features: Multiple cameras, ECS v2, advanced camera system");
    println!("🎯 Controls: 1-4 keys to switch main camera, Esc to exit");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Multi-Camera Demo - Phase 4")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)?;

    // Initialize multi-camera renderer
    let mut renderer = MultiCameraRenderer::new(&window).await?;
    
    // Create ECS v2 world and populate with test scene
    let mut world = WorldV2::new();
    
    println!("🏗️ Creating multi-camera scene...");
    
    // Create multiple cameras with different configurations
    let viewport = Viewport::new(1200, 800);
    
    // Camera 1: Main perspective camera
    let camera1_entity = world.spawn();
    let camera1 = engine_camera::Camera::perspective_3d(60.0, viewport.clone());
    let mut camera1_comp = CameraComponent::main_camera(camera1);
    camera1_comp.camera.set_render_order(0); // Renders first
    camera1_comp.camera.set_clear_color([0.1, 0.2, 0.3, 1.0]); // Dark blue
    world.add_component(camera1_entity, camera1_comp)?;
    world.add_component(camera1_entity, Transform {
        position: [0.0, 2.0, 8.0],
        rotation: [-15.0, 0.0, 0.0], // Look down slightly
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera1_entity, Name::new("Main Camera (Perspective)"))?;
    
    // Camera 2: Orthographic 2D camera
    let camera2_entity = world.spawn();
    let camera2 = engine_camera::Camera::orthographic_2d(10.0, viewport.clone());
    let mut camera2_comp = CameraComponent::new(camera2);
    camera2_comp.camera.set_render_order(1); // Secondary camera
    camera2_comp.camera.set_clear_color([0.2, 0.1, 0.2, 1.0]); // Dark purple
    camera2_comp.camera.set_enabled(false); // Start disabled
    world.add_component(camera2_entity, camera2_comp)?;
    world.add_component(camera2_entity, Transform {
        position: [0.0, 0.0, 5.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera2_entity, Name::new("2D Camera (Orthographic)"))?;
    
    // Camera 3: Close-up perspective camera
    let camera3_entity = world.spawn();
    let camera3 = engine_camera::Camera::perspective_3d(90.0, viewport.clone()); // Wide FOV
    let mut camera3_comp = CameraComponent::new(camera3);
    camera3_comp.camera.set_render_order(2);
    camera3_comp.camera.set_clear_color([0.1, 0.3, 0.1, 1.0]); // Dark green
    camera3_comp.camera.set_enabled(false); // Start disabled
    world.add_component(camera3_entity, camera3_comp)?;
    world.add_component(camera3_entity, Transform {
        position: [2.0, 1.0, 3.0],
        rotation: [0.0, -30.0, 0.0], // Look at center from side
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera3_entity, Name::new("Close-up Camera (Wide FOV)"))?;
    
    // Camera 4: Top-down camera
    let camera4_entity = world.spawn();
    let camera4 = engine_camera::Camera::orthographic_2d(8.0, viewport);
    let mut camera4_comp = CameraComponent::new(camera4);
    camera4_comp.camera.set_render_order(3);
    camera4_comp.camera.set_clear_color([0.3, 0.2, 0.1, 1.0]); // Dark orange
    camera4_comp.camera.set_enabled(false); // Start disabled
    world.add_component(camera4_entity, camera4_comp)?;
    world.add_component(camera4_entity, Transform {
        position: [0.0, 10.0, 0.0],
        rotation: [-90.0, 0.0, 0.0], // Look straight down
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(camera4_entity, Name::new("Top-down Camera"))?;
    
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
    
    // Sphere to the right (green)
    let sphere_entity = world.spawn();
    world.add_component(sphere_entity, Mesh { mesh_type: MeshType::Sphere })?;
    world.add_component(sphere_entity, Transform {
        position: [3.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    })?;
    world.add_component(sphere_entity, Name::new("Right Sphere"))?;
    
    // Cube to the left (blue)
    let cube2_entity = world.spawn();
    world.add_component(cube2_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(cube2_entity, Transform {
        position: [-3.0, 0.0, 0.0],
        rotation: [45.0, 45.0, 0.0],
        scale: [0.8, 0.8, 0.8],
    })?;
    world.add_component(cube2_entity, Name::new("Left Rotated Cube"))?;
    
    // Elevated sphere (yellow)
    let sphere2_entity = world.spawn();
    world.add_component(sphere2_entity, Mesh { mesh_type: MeshType::Sphere })?;
    world.add_component(sphere2_entity, Transform {
        position: [0.0, 3.0, -2.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [0.6, 0.6, 0.6],
    })?;
    world.add_component(sphere2_entity, Name::new("Elevated Sphere"))?;
    
    // Ground plane (large flat cube)
    let ground_entity = world.spawn();
    world.add_component(ground_entity, Mesh { mesh_type: MeshType::Cube })?;
    world.add_component(ground_entity, Transform {
        position: [0.0, -2.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [10.0, 0.1, 10.0],
    })?;
    world.add_component(ground_entity, Name::new("Ground Plane"))?;
    
    println!("✅ Created ECS world with {} entities", world.entity_count());
    println!("📷 Active cameras: 4 (1 enabled, 3 disabled)");
    println!("📦 Scene objects: 5 meshes");
    
    // Print scene information
    for (entity, name) in world.query::<engine_core::Read<Name>>().iter() {
        if let Some(camera_comp) = world.get_component::<CameraComponent>(entity) {
            println!("  🎥 Camera {}: {} (enabled: {}, order: {})", 
                entity.id(), name.name, 
                camera_comp.camera.enabled(),
                camera_comp.camera.render_order()
            );
        } else if world.get_component::<Mesh>(entity).is_some() {
            println!("  📦 Mesh {}: {}", entity.id(), name.name);
        }
    }
    
    let mut rotation_time = 0.0f32;
    let mut current_camera = 1; // Track which camera is active
    
    let cameras = [camera1_entity, camera2_entity, camera3_entity, camera4_entity];
    let camera_names = ["Main Camera", "2D Camera", "Close-up Camera", "Top-down Camera"];
    
    println!("\n🎮 Demo Controls:");
    println!("  1-4: Switch to camera 1-4");
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
                    println!("👋 Closing multi-camera demo");
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
                                    // Disable all cameras
                                    for &camera_entity in &cameras {
                                        if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(camera_entity) {
                                            camera_comp.camera.set_enabled(false);
                                            camera_comp.is_main = false;
                                        }
                                    }
                                    // Enable camera 1
                                    if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(cameras[0]) {
                                        camera_comp.camera.set_enabled(true);
                                        camera_comp.is_main = true;
                                    }
                                    current_camera = 1;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Digit2) => {
                                if current_camera != 2 {
                                    // Disable all cameras
                                    for &camera_entity in &cameras {
                                        if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(camera_entity) {
                                            camera_comp.camera.set_enabled(false);
                                            camera_comp.is_main = false;
                                        }
                                    }
                                    // Enable camera 2
                                    if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(cameras[1]) {
                                        camera_comp.camera.set_enabled(true);
                                        camera_comp.is_main = true;
                                    }
                                    current_camera = 2;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Digit3) => {
                                if current_camera != 3 {
                                    // Disable all cameras
                                    for &camera_entity in &cameras {
                                        if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(camera_entity) {
                                            camera_comp.camera.set_enabled(false);
                                            camera_comp.is_main = false;
                                        }
                                    }
                                    // Enable camera 3
                                    if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(cameras[2]) {
                                        camera_comp.camera.set_enabled(true);
                                        camera_comp.is_main = true;
                                    }
                                    current_camera = 3;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Digit4) => {
                                if current_camera != 4 {
                                    // Disable all cameras
                                    for &camera_entity in &cameras {
                                        if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(camera_entity) {
                                            camera_comp.camera.set_enabled(false);
                                            camera_comp.is_main = false;
                                        }
                                    }
                                    // Enable camera 4
                                    if let Some(mut camera_comp) = world.get_component_mut::<CameraComponent>(cameras[3]) {
                                        camera_comp.camera.set_enabled(true);
                                        camera_comp.is_main = true;
                                    }
                                    current_camera = 4;
                                    println!("📷 Switched to: {} ({})", current_camera, camera_names[current_camera - 1]);
                                }
                            }
                            PhysicalKey::Code(KeyCode::Escape) => {
                                println!("👋 Exiting multi-camera demo");
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
                    
                    // Update camera matrices for the current frame
                    let frame = rotation_time as u64;
                    for &camera_entity in &cameras {
                        if let Some(transform) = world.get_component::<Transform>(camera_entity).cloned() {
                            if let Some(camera_comp) = world.get_component_mut::<CameraComponent>(camera_entity) {
                                let _ = camera_comp.update(&transform, frame);
                            }
                        }
                    }
                    
                    // Render the world with multi-camera system
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