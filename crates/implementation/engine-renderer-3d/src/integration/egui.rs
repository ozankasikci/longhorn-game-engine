//! egui integration for the 3D renderer
//! 
//! This module provides widgets and utilities for displaying the rendered
//! 3D scene in egui-based user interfaces.

use std::sync::{Arc, Mutex};
use egui::{Widget, Response, Ui, TextureId, ColorImage, TextureOptions};
use egui::epaint::ImageDelta;
use crate::{Renderer3D, RenderScene};

/// Widget for displaying 3D rendered content in egui
pub struct EguiRenderWidget {
    renderer: Arc<Mutex<Renderer3D>>,
    texture_id: Option<TextureId>,
    last_size: Option<egui::Vec2>,
}

impl EguiRenderWidget {
    /// Create a new egui render widget
    pub fn new(renderer: Arc<Mutex<Renderer3D>>) -> Self {
        Self {
            renderer,
            texture_id: None,
            last_size: None,
        }
    }
    
    /// Render a scene using the internal renderer
    pub fn render_scene(&mut self, scene: &RenderScene) -> Result<(), anyhow::Error> {
        let mut renderer = self.renderer.lock().unwrap();
        renderer.render(scene)
    }
    
    /// Update the texture in egui's texture manager
    fn update_texture(&mut self, ui: &mut Ui, size: egui::Vec2) -> Result<(), anyhow::Error> {
        let renderer = self.renderer.lock().unwrap();
        
        // Check if we need to resize
        if let Some(last_size) = self.last_size {
            if (last_size.x - size.x).abs() > 1.0 || (last_size.y - size.y).abs() > 1.0 {
                // Need to resize - this would require recreation of the renderer
                // For now, we'll just update the size tracking
                self.last_size = Some(size);
            }
        } else {
            self.last_size = Some(size);
        }
        
        // For now, we'll create a placeholder texture
        // In a full implementation, we would copy from the render texture
        let width = size.x as usize;
        let height = size.y as usize;
        
        if width == 0 || height == 0 {
            return Ok(());
        }
        
        // Create a placeholder gradient image
        let mut pixels = vec![egui::Color32::BLACK; width * height];
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let r = (x as f32 / width as f32 * 255.0) as u8;
                let g = (y as f32 / height as f32 * 255.0) as u8;
                let b = 128;
                pixels[idx] = egui::Color32::from_rgb(r, g, b);
            }
        }
        
        let color_image = ColorImage {
            size: [width, height],
            pixels,
        };
        
        // Update or create texture
        let image_data = egui::ImageData::Color(Arc::new(color_image));
        
        if let Some(texture_id) = self.texture_id {
            let image_delta = ImageDelta::full(image_data, TextureOptions::default());
            ui.ctx().tex_manager().write().set(texture_id, image_delta);
        } else {
            let texture_id = ui.ctx().tex_manager().write().alloc(
                "render_texture".into(),
                image_data,
                TextureOptions::default(),
            );
            self.texture_id = Some(texture_id);
        }
        
        Ok(())
    }
}

impl Widget for &mut EguiRenderWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();
        let size = rect.size();
        
        // Update texture if needed
        if let Err(e) = self.update_texture(ui, size) {
            log::error!("Failed to update render texture: {}", e);
            return ui.label("Render Error");
        }
        
        // Display the texture
        if let Some(texture_id) = self.texture_id {
            let response = ui.allocate_response(size, egui::Sense::hover());
            ui.painter().image(
                texture_id,
                response.rect,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            response
        } else {
            ui.label("Initializing renderer...")
        }
    }
}

/// Helper function to create a render widget from wgpu state
pub fn create_render_widget(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    width: u32,
    height: u32,
) -> Result<EguiRenderWidget, anyhow::Error> {
    // Create renderer (this would be async in real usage)
    // For now, we'll use a blocking approach
    let renderer = pollster::block_on(async {
        Renderer3D::new(device, queue, width, height).await
    })?;
    
    let renderer = Arc::new(Mutex::new(renderer));
    Ok(EguiRenderWidget::new(renderer))
}