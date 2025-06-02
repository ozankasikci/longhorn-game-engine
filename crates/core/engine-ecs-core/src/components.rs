// Standard components for the game engine

use serde::{Serialize, Deserialize};
use crate::{Component, ecs_v2};

// Mesh component - defines what mesh to render
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_type: MeshType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MeshType {
    Cube,
    Sphere,
    Plane,
    Custom(String), // Asset path for custom meshes
}

impl Component for Mesh {}
impl ecs_v2::Component for Mesh {}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            mesh_type: MeshType::Cube,
        }
    }
}

// Material component - defines how the mesh should be rendered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub color: [f32; 4], // RGBA
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3], // RGB emissive color
}

impl Component for Material {}
impl ecs_v2::Component for Material {}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: [0.8, 0.8, 0.8, 1.0], // Light gray
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }
    }
}

// Name component - for identifying objects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

impl Component for Name {}
impl ecs_v2::Component for Name {}

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

// Visibility component - whether the object should be rendered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visibility {
    pub visible: bool,
}

impl Component for Visibility {}
impl ecs_v2::Component for Visibility {}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

// Camera component - defines a camera for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub fov: f32, // Field of view in degrees
    pub near: f32,
    pub far: f32,
    pub is_main: bool, // Is this the main camera?
}

impl Component for Camera {}
impl ecs_v2::Component for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            is_main: false,
        }
    }
}

// Light component - defines various light types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3], // RGB
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point { range: f32 },
    Spot { range: f32, angle: f32 },
}

impl Component for Light {}
impl ecs_v2::Component for Light {}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        }
    }
}

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

impl Component for Sprite {}
impl ecs_v2::Component for Sprite {}

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

impl Component for SpriteRenderer {}
impl ecs_v2::Component for SpriteRenderer {}

// Canvas Component for UI rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    pub render_mode: CanvasRenderMode,  // How the canvas is rendered
    pub sorting_layer: i32,             // Global sorting layer
    pub order_in_layer: i32,            // Order within the sorting layer
    pub pixel_perfect: bool,            // Snap to pixel boundaries
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanvasRenderMode {
    WorldSpace,                         // Rendered in 3D world space
    ScreenSpaceOverlay,                 // Rendered as overlay on top of everything
    ScreenSpaceCamera,                  // Rendered relative to a specific camera
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            render_mode: CanvasRenderMode::WorldSpace,
            sorting_layer: 0,
            order_in_layer: 0,
            pixel_perfect: true,
        }
    }
}

impl Component for Canvas {}
impl ecs_v2::Component for Canvas {}

// 2D Camera Component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    pub size: f32,                      // Orthographic size (world units from center to top)
    pub aspect_ratio: f32,              // Width/height ratio (auto-calculated if 0.0)
    pub near: f32,                      // Near clipping plane
    pub far: f32,                       // Far clipping plane
    pub background_color: [f32; 4],     // Clear color RGBA
    pub viewport_rect: [f32; 4],        // [x, y, width, height] normalized (0.0-1.0)
    pub is_main: bool,                  // Whether this is the main 2D camera
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            size: 5.0,                  // 5 world units from center to top
            aspect_ratio: 0.0,          // Auto-calculate from screen
            near: -10.0,                // Behind the camera
            far: 10.0,                  // In front of the camera
            background_color: [0.2, 0.2, 0.3, 1.0], // Dark blue-gray
            viewport_rect: [0.0, 0.0, 1.0, 1.0],    // Full screen
            is_main: false,
        }
    }
}

impl Camera2D {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn main_camera() -> Self {
        Self {
            is_main: true,
            ..Default::default()
        }
    }
    
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.background_color = [r, g, b, a];
        self
    }
}

impl Component for Camera2D {}
impl ecs_v2::Component for Camera2D {}

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
    
    #[test]
    fn test_canvas_default() {
        let canvas = Canvas::default();
        assert_eq!(canvas.render_mode, CanvasRenderMode::WorldSpace);
        assert_eq!(canvas.sorting_layer, 0);
        assert_eq!(canvas.order_in_layer, 0);
        assert!(canvas.pixel_perfect);
    }
    
    #[test]
    fn test_camera_2d_default() {
        let camera = Camera2D::default();
        assert_eq!(camera.size, 5.0);
        assert_eq!(camera.aspect_ratio, 0.0);
        assert_eq!(camera.near, -10.0);
        assert_eq!(camera.far, 10.0);
        assert_eq!(camera.background_color, [0.2, 0.2, 0.3, 1.0]);
        assert_eq!(camera.viewport_rect, [0.0, 0.0, 1.0, 1.0]);
        assert!(!camera.is_main);
    }
    
    #[test]
    fn test_camera_2d_main() {
        let camera = Camera2D::main_camera();
        assert!(camera.is_main);
        assert_eq!(camera.size, 5.0); // Should still have default size
    }
    
    #[test]
    fn test_camera_2d_builder() {
        let camera = Camera2D::new()
            .with_size(10.0)
            .with_background_color(1.0, 0.0, 0.0, 1.0);
        
        assert_eq!(camera.size, 10.0);
        assert_eq!(camera.background_color, [1.0, 0.0, 0.0, 1.0]);
    }
}