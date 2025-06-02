//! 2D rendering components for the mobile game engine
//! 
//! This crate provides 2D-specific components including Sprite and SpriteRenderer.

use serde::{Serialize, Deserialize};
use engine_ecs_core::{Component, ComponentV2};

// 2D Sprite Component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture_handle: Option<u64>,    // Asset handle for texture
    pub uv_rect: [f32; 4],             // [x, y, width, height] in texture space (0.0-1.0)
    pub color: [f32; 4],               // RGBA tint multiplier (1.0 = no tint)
    pub flip_x: bool,                  // Horizontal flip
    pub flip_y: bool,                  // Vertical flip
    pub pivot: [f32; 2],               // Local pivot point [0.0-1.0, 0.0-1.0]
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture_handle: None,
            uv_rect: [0.0, 0.0, 1.0, 1.0],    // Full texture
            color: [1.0, 1.0, 1.0, 1.0],      // White, no tint
            flip_x: false,
            flip_y: false,
            pivot: [0.5, 0.5],                // Center pivot
        }
    }
}

impl Sprite {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_texture(mut self, handle: u64) -> Self {
        self.texture_handle = Some(handle);
        self
    }
    
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
    
    pub fn with_uv_rect(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.uv_rect = [x, y, width, height];
        self
    }
}

// Component trait implementations
impl Component for Sprite {}
impl ComponentV2 for Sprite {}

// Sprite Renderer Component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteRenderer {
    pub sprite: Sprite,                 // Sprite data
    pub layer: i32,                     // Z-order for sorting (-32768 to 32767)
    pub material_override: Option<u64>,  // Custom material handle (optional)
    pub enabled: bool,                  // Whether to render this sprite
}

impl Default for SpriteRenderer {
    fn default() -> Self {
        Self {
            sprite: Sprite::default(),
            layer: 0,                   // Default layer
            material_override: None,
            enabled: true,
        }
    }
}

impl SpriteRenderer {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            layer: 0,
            material_override: None,
            enabled: true,
        }
    }
    
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
    
    pub fn with_material(mut self, material_handle: u64) -> Self {
        self.material_override = Some(material_handle);
        self
    }
}

// Component trait implementations
impl Component for SpriteRenderer {}
impl ComponentV2 for SpriteRenderer {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_default() {
        let sprite = Sprite::default();
        assert_eq!(sprite.texture_handle, None);
        assert_eq!(sprite.uv_rect, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]);
        assert!(!sprite.flip_x);
        assert!(!sprite.flip_y);
        assert_eq!(sprite.pivot, [0.5, 0.5]);
    }
    
    #[test]
    fn test_sprite_builder() {
        let sprite = Sprite::new()
            .with_texture(42)
            .with_color(1.0, 0.5, 0.0, 0.8)
            .with_uv_rect(0.1, 0.2, 0.3, 0.4);
        
        assert_eq!(sprite.texture_handle, Some(42));
        assert_eq!(sprite.color, [1.0, 0.5, 0.0, 0.8]);
        assert_eq!(sprite.uv_rect, [0.1, 0.2, 0.3, 0.4]);
    }
    
    #[test]
    fn test_sprite_renderer_default() {
        let renderer = SpriteRenderer::default();
        assert_eq!(renderer.layer, 0);
        assert!(renderer.enabled);
        assert_eq!(renderer.material_override, None);
        assert_eq!(renderer.sprite, Sprite::default());
    }
    
    #[test]
    fn test_sprite_renderer_builder() {
        let sprite = Sprite::new().with_texture(123);
        let renderer = SpriteRenderer::new(sprite.clone())
            .with_layer(5)
            .with_material(789);
        
        assert_eq!(renderer.layer, 5);
        assert_eq!(renderer.material_override, Some(789));
        assert_eq!(renderer.sprite, sprite);
    }
}