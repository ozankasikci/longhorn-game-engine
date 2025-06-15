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
        let mut renderer = self.renderer.lock().unwrap();
        
        // Check if we need to resize
        let width = size.x as u32;
        let height = size.y as u32;
        
        if width == 0 || height == 0 {
            return Ok(());
        }
        
        log::debug!("Updating texture with size {}x{}", width, height);
        
        if let Some(last_size) = self.last_size {
            if (last_size.x - size.x).abs() > 1.0 || (last_size.y - size.y).abs() > 1.0 {
                // Need to resize the renderer
                renderer.resize(width, height)?;
                self.last_size = Some(size);
            }
        } else {
            self.last_size = Some(size);
            renderer.resize(width, height)?;
        }
        
        // Get the render output from the renderer
        log::info!("Getting texture data from renderer");
        let pixels = renderer.get_texture_data()?;
        
        // Convert from RGBA bytes to egui Color32
        let width_usize = width as usize;
        let height_usize = height as usize;
        
        log::info!("Got {} bytes of pixel data, expected {}", pixels.len(), width_usize * height_usize * 4);
        
        // Debug: Check if we're getting valid pixel data
        if !pixels.is_empty() {
            // Sample a few pixels to see what we're getting
            log::debug!("First pixel: R={}, G={}, B={}, A={}", pixels[0], pixels[1], pixels[2], pixels[3]);
            let mid = pixels.len() / 2;
            if mid + 3 < pixels.len() {
                log::debug!("Middle pixel: R={}, G={}, B={}, A={}", pixels[mid], pixels[mid+1], pixels[mid+2], pixels[mid+3]);
            }
        }
        let mut egui_pixels = Vec::with_capacity(width_usize * height_usize);
        
        for chunk in pixels.chunks_exact(4) {
            egui_pixels.push(egui::Color32::from_rgba_premultiplied(
                chunk[0], chunk[1], chunk[2], chunk[3]
            ));
        }
        
        let color_image = ColorImage {
            size: [width_usize, height_usize],
            pixels: egui_pixels,
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
        
        log::info!("EguiRenderWidget::ui called with size {:?}", size);
        
        // Update texture if needed
        if let Err(e) = self.update_texture(ui, size) {
            log::error!("Failed to update render texture: {}", e);
            return ui.label("Render Error");
        }
        
        // Display the texture
        if let Some(texture_id) = self.texture_id {
            log::info!("Drawing texture with id {:?} at rect {:?}", texture_id, rect);
            let response = ui.allocate_response(size, egui::Sense::hover());
            
            // Draw the texture
            ui.painter().image(
                texture_id,
                response.rect,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            
            // Debug: Draw a border around the texture area
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::RED),
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