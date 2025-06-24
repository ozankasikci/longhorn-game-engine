//! Simple triangle rendering test for the 3D renderer
//!
//! This example demonstrates basic functionality of the engine-renderer-3d crate
//! by rendering a simple triangle to verify WGPU initialization works.

use engine_renderer_3d::{Camera, RenderObject, RenderScene, Renderer3D};
use glam::Mat4;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Initializing WGPU...");

    // Create WGPU instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Request adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or("Failed to find an appropriate adapter")?;

    println!("Found adapter: {:?}", adapter.get_info());

    // Request device and queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await?;

    println!("Device created successfully");

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // Create renderer
    let width = 800;
    let height = 600;
    let mut renderer = Renderer3D::new(device.clone(), queue.clone(), width, height).await?;

    println!("Renderer created successfully");

    // Create a camera
    let aspect = width as f32 / height as f32;
    let camera = Camera::new(aspect);

    // Create a simple scene with a triangle
    let mut scene = RenderScene::new(camera);

    // Add a simple colored object (this will be a triangle in our basic renderer)
    // For now, using placeholder mesh/material IDs - these should come from the renderer's resource management
    let test_object = RenderObject::new(
        Mat4::IDENTITY,
        0, // mesh_id - triangle will be mesh 0
        0, // material_id - default material will be 0
    );

    scene.add_object(test_object);

    println!("Scene created with test triangle");

    // Render the scene
    match renderer.render(&scene) {
        Ok(()) => {
            println!("✅ Triangle rendered successfully!");
            println!("🎨 Renderer initialization and basic rendering completed");
        }
        Err(e) => {
            eprintln!("❌ Rendering failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
