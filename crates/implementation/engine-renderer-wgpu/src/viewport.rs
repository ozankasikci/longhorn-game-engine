//! Viewport and render target abstractions

use serde::{Serialize, Deserialize};

/// Viewport configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Render target specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTarget {
    pub label: Option<String>,
    pub color_attachments: Vec<RenderTargetAttachment>,
    pub depth_stencil_attachment: Option<RenderTargetAttachment>,
    pub sample_count: u32,
}

/// Render target attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTargetAttachment {
    pub texture: crate::TextureHandle,
    pub mip_level: u32,
    pub array_layer: Option<u32>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

impl Viewport {
    /// Create a new viewport
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
    
    /// Set depth range
    pub fn with_depth_range(mut self, min_depth: f32, max_depth: f32) -> Self {
        self.min_depth = min_depth;
        self.max_depth = max_depth;
        self
    }
    
    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    
    /// Check if the viewport contains a point
    pub fn contains_point(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
    
    /// Get the center point of the viewport
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
    
    /// Convert local coordinates to viewport coordinates
    pub fn local_to_viewport(&self, local_x: f32, local_y: f32) -> (u32, u32) {
        let x = self.x + (local_x * self.width as f32) as u32;
        let y = self.y + (local_y * self.height as f32) as u32;
        (x, y)
    }
    
    /// Convert viewport coordinates to local coordinates (0.0 to 1.0)
    pub fn viewport_to_local(&self, viewport_x: u32, viewport_y: u32) -> (f32, f32) {
        let x = (viewport_x - self.x) as f32 / self.width as f32;
        let y = (viewport_y - self.y) as f32 / self.height as f32;
        (x, y)
    }
}

impl RenderTarget {
    /// Create a simple color-only render target
    pub fn color_only(texture: crate::TextureHandle) -> Self {
        Self {
            label: None,
            color_attachments: vec![RenderTargetAttachment {
                texture,
                mip_level: 0,
                array_layer: None,
            }],
            depth_stencil_attachment: None,
            sample_count: 1,
        }
    }
    
    /// Create a render target with depth
    pub fn with_depth(
        color_texture: crate::TextureHandle,
        depth_texture: crate::TextureHandle,
    ) -> Self {
        Self {
            label: None,
            color_attachments: vec![RenderTargetAttachment {
                texture: color_texture,
                mip_level: 0,
                array_layer: None,
            }],
            depth_stencil_attachment: Some(RenderTargetAttachment {
                texture: depth_texture,
                mip_level: 0,
                array_layer: None,
            }),
            sample_count: 1,
        }
    }
    
    /// Set a debug label
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
    
    /// Set sample count for multisampling
    pub fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }
    
    /// Check if this render target has depth attachment
    pub fn has_depth(&self) -> bool {
        self.depth_stencil_attachment.is_some()
    }
    
    /// Check if this render target uses multisampling
    pub fn is_multisampled(&self) -> bool {
        self.sample_count > 1
    }
    
    /// Get the number of color attachments
    pub fn color_attachment_count(&self) -> usize {
        self.color_attachments.len()
    }
}