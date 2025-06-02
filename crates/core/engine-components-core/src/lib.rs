//! Standard components for the mobile game engine
//! 
//! This crate provides the standard component library used across the engine.
//! Components are designed to work with both the legacy ECS (ecs.rs) and 
//! the new data-oriented ECS (ecs_v2.rs) systems.
//!
//! Note: Component trait implementations are provided directly in this crate.

use serde::{Serialize, Deserialize};
use engine_ecs_core::{Component, ComponentV2};

// Transform component - fundamental for all spatial objects
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3], 
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Transform {
    /// Create a new Transform with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a transform matrix from this transform
    pub fn matrix(&self) -> glam::Mat4 {
        let translation = glam::Vec3::from_array(self.position);
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            self.rotation[0],
            self.rotation[1], 
            self.rotation[2]
        );
        let scale = glam::Vec3::from_array(self.scale);
        
        glam::Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
    
    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = [x, y, z];
        self
    }
    
    /// Set rotation (in radians)
    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = [x, y, z];
        self
    }
    
    /// Set scale
    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = [x, y, z];
        self
    }
}

// Component trait implementations
impl Component for Transform {}
impl ComponentV2 for Transform {}

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

// Component trait implementations
impl Component for Mesh {}
impl ComponentV2 for Mesh {}

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

// Component trait implementations
impl Component for Material {}
impl ComponentV2 for Material {}

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

// Component trait implementations
impl Component for Name {}
impl ComponentV2 for Name {}

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

// Component trait implementations
impl Component for Visibility {}
impl ComponentV2 for Visibility {}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
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

// Component trait implementations
impl Component for Light {}
impl ComponentV2 for Light {}

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

// Component trait implementations
impl Component for Canvas {}
impl ComponentV2 for Canvas {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }
    
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
    fn test_name_creation() {
        let name = Name::new("Test Object");
        assert_eq!(name.name, "Test Object");
        
        let name2 = Name::new(String::from("Another Object"));
        assert_eq!(name2.name, "Another Object");
    }
    
    #[test]
    fn test_material_default() {
        let material = Material::default();
        assert_eq!(material.color, [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
        assert_eq!(material.emissive, [0.0, 0.0, 0.0]);
    }
    
    #[test]
    fn test_light_default() {
        let light = Light::default();
        assert_eq!(light.color, [1.0, 1.0, 1.0]);
        assert_eq!(light.intensity, 1.0);
        assert!(matches!(light.light_type, LightType::Directional));
    }
}