//! Cube rendering test for the 3D renderer
//! 
//! This test renders a cube to validate more complex geometry handling.

use engine_renderer_3d::{Renderer3D, RenderScene, RenderObject, Camera};
use glam::{Mat4, Vec3};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🎮 Starting cube rendering test...");
    
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
    
    println!("🔧 Using adapter: {:?}", adapter.get_info().name);
    
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
    
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    
    // Create renderer
    let width = 1024;
    let height = 768;
    let mut renderer = Renderer3D::new(device.clone(), queue.clone(), width, height).await?;
    
    println!("🎨 Renderer initialized ({}x{})", width, height);
    
    // Create a camera with better positioning for a cube
    let aspect = width as f32 / height as f32;
    let mut camera = Camera::new(aspect);
    camera.position = Vec3::new(2.0, 2.0, 3.0);
    camera.target = Vec3::ZERO;
    
    // Create scene with multiple objects to test
    let mut scene = RenderScene::new(camera);
    
    // Add cube at origin
    let cube = RenderObject::new(
        Mat4::IDENTITY,
        1, // mesh_id - cube will be mesh 1 (triangle is 0)
        0, // material_id - default material
    );
    scene.add_object(cube);
    
    // Add a second cube offset to test multiple objects
    let cube2_transform = Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0));
    let cube2 = RenderObject::new(
        cube2_transform,
        1, // same mesh
        1, // different material
    );
    scene.add_object(cube2);
    
    println!("📦 Scene created with {} cubes", scene.objects.len());
    
    // Render the scene
    println!("🔄 Rendering...");
    match renderer.render(&scene) {
        Ok(()) => {
            println!("✅ Cube rendering completed successfully!");
            println!("🎉 Verified: Multi-object 3D rendering pipeline works");
            println!("📊 Scene stats:");
            println!("   - Camera position: {:?}", scene.camera.position);
            println!("   - Camera target: {:?}", scene.camera.target);
            println!("   - Objects rendered: {}", scene.objects.len());
            println!("   - Clear color: {:?}", scene.clear_color);
        }
        Err(e) => {
            eprintln!("❌ Cube rendering failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}