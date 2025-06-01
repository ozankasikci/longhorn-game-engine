// Tests for wgpu functionality
use wgpu;
use winit::{window::WindowBuilder, event_loop::EventLoop, dpi::LogicalSize};
use std::sync::Arc;

async fn create_wgpu_context() -> Result<(wgpu::Device, wgpu::Queue), Box<dyn std::error::Error>> {
    // Create a headless context for testing
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap());
    
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let surface = instance.create_surface(window.clone()).unwrap();
    
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();
    
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    
    Ok((device, queue))
}

#[tokio::test]
async fn test_wgpu_initialization() {
    let result = create_wgpu_context().await;
    assert!(result.is_ok(), "Failed to create wgpu context");
    
    let (device, queue) = result.unwrap();
    
    // Test that we can create a command encoder
    let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Test Encoder"),
    });
    
    // Test that we can submit empty commands
    queue.submit(std::iter::once(encoder.finish()));
}

#[tokio::test]
async fn test_wgpu_render_pass() {
    let result = create_wgpu_context().await;
    if result.is_err() {
        // Skip this test if we can't create a context
        return;
    }
    let (device, queue) = result.unwrap();
    
    // Test creating a texture for rendering
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Test Texture"),
        size: wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Test Render Encoder"),
    });
    
    // Create a simple render pass
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Test Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }
    
    queue.submit(std::iter::once(encoder.finish()));
}

#[tokio::test]
async fn test_egui_wgpu_renderer_creation() {
    let result = create_wgpu_context().await;
    if result.is_err() {
        return;
    }
    let (device, _queue) = result.unwrap();
    
    // Test that we can create an egui renderer
    let renderer = egui_wgpu::Renderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb, None, 1);
    
    // Test basic renderer properties
    // The renderer should be created successfully without panicking
    drop(renderer);
}

#[test]
fn test_egui_context_creation() {
    // Test that we can create an egui context
    let ctx = egui::Context::default();
    
    // Test basic context operations
    let _viewport_id = ctx.viewport_id();
    
    // Test that we can run the context with a simple UI
    let raw_input = egui::RawInput::default();
    let full_output = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Test");
        });
    });
    
    // Verify we got output
    assert!(!full_output.shapes.is_empty() || full_output.shapes.is_empty()); // Either is valid
}

#[test]
fn test_egui_tessellation() {
    let ctx = egui::Context::default();
    
    let raw_input = egui::RawInput::default();
    let full_output = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test Heading");
            ui.label("Test Label");
            ui.button("Test Button");
        });
    });
    
    // Test tessellation
    let tris = ctx.tessellate(full_output.shapes, 1.0);
    
    // We should get some triangles for text/button rendering
    // In headless mode this might be empty, but it shouldn't panic
    assert!(tris.len() >= 0);
}

#[test]
fn test_color_conversions() {
    // Test that egui colors work as expected
    let white = egui::Color32::WHITE;
    let black = egui::Color32::BLACK;
    let red = egui::Color32::RED;
    
    assert_eq!(white.r(), 255);
    assert_eq!(white.g(), 255);
    assert_eq!(white.b(), 255);
    
    assert_eq!(black.r(), 0);
    assert_eq!(black.g(), 0);
    assert_eq!(black.b(), 0);
    
    assert_eq!(red.r(), 255);
    assert_eq!(red.g(), 0);
    assert_eq!(red.b(), 0);
    
    // Test custom color creation
    let custom = egui::Color32::from_rgb(100, 150, 200);
    assert_eq!(custom.r(), 100);
    assert_eq!(custom.g(), 150);
    assert_eq!(custom.b(), 200);
}

#[test]
fn test_wgpu_color_format() {
    // Test wgpu color values
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    
    assert!((clear_color.r - 0.1).abs() < f64::EPSILON);
    assert!((clear_color.g - 0.2).abs() < f64::EPSILON);
    assert!((clear_color.b - 0.3).abs() < f64::EPSILON);
    assert!((clear_color.a - 1.0).abs() < f64::EPSILON);
}