//! 3D rendering components for the mobile game engine
//!
//! This crate provides 3D-specific components including Transform, Mesh,
//! Material, Light, and Visibility components.

use engine_component_traits::Component;
use engine_geometry_core::MeshData;
use engine_resource_core::ResourceHandle;
use serde::{Deserialize, Serialize};

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
            self.rotation[2],
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

// Component trait implementation
impl Component for Transform {}

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

// Component trait implementation
impl Component for Mesh {}

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

// Component trait implementation
impl Component for Material {}

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

// Component trait implementation
impl Component for Light {}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        }
    }
}

// Visibility component - whether the object should be rendered
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visibility {
    pub visible: bool,
}

// Component trait implementation
impl Component for Visibility {}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

// ============================================================================
// CAMERA COMPONENTS
// ============================================================================

/// Camera component - defines camera properties for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Type of projection
    pub projection_type: ProjectionType,

    /// Vertical field of view in degrees (for perspective projection)
    pub fov_degrees: f32,

    /// Near clipping plane distance
    pub near_plane: f32,

    /// Far clipping plane distance  
    pub far_plane: f32,

    /// Orthographic projection size (height in world units)
    pub orthographic_size: f32,

    /// Viewport rectangle (normalized 0-1)
    pub viewport: Viewport,

    /// Render target (None = main framebuffer)
    pub render_target: Option<u32>,

    /// Rendering priority (higher = rendered later)
    pub priority: i32,

    /// Whether this camera is active
    pub active: bool,

    /// Clear flags
    pub clear_flags: ClearFlags,

    /// Background color (if clearing color buffer)
    pub clear_color: [f32; 4],
}

/// Projection type for cameras
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

/// Viewport defines the screen area to render to
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// X position (0-1, where 0 is left)
    pub x: f32,
    /// Y position (0-1, where 0 is top)
    pub y: f32,
    /// Width (0-1)
    pub width: f32,
    /// Height (0-1)
    pub height: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

/// Clear flags for camera rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClearFlags {
    pub color: bool,
    pub depth: bool,
    pub stencil: bool,
}

impl Default for ClearFlags {
    fn default() -> Self {
        Self {
            color: true,
            depth: true,
            stencil: false,
        }
    }
}

impl Component for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection_type: ProjectionType::Perspective,
            fov_degrees: 60.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            orthographic_size: 10.0,
            viewport: Viewport::default(),
            render_target: None,
            priority: 0,
            active: true,
            clear_flags: ClearFlags::default(),
            clear_color: [0.1, 0.1, 0.1, 1.0],
        }
    }
}

impl Camera {
    /// Create a perspective camera
    pub fn perspective(fov_degrees: f32, near: f32, far: f32) -> Self {
        Self {
            projection_type: ProjectionType::Perspective,
            fov_degrees,
            near_plane: near,
            far_plane: far,
            ..Default::default()
        }
    }

    /// Create an orthographic camera
    pub fn orthographic(size: f32, near: f32, far: f32) -> Self {
        Self {
            projection_type: ProjectionType::Orthographic,
            orthographic_size: size,
            near_plane: near,
            far_plane: far,
            ..Default::default()
        }
    }

    /// Set the viewport
    pub fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    /// Set the render priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set the clear color
    pub fn with_clear_color(mut self, color: [f32; 4]) -> Self {
        self.clear_color = color;
        self
    }

    /// Calculate aspect ratio from viewport and screen dimensions
    pub fn calculate_aspect_ratio(&self, screen_width: u32, screen_height: u32) -> f32 {
        let viewport_width = self.viewport.width * screen_width as f32;
        let viewport_height = self.viewport.height * screen_height as f32;
        viewport_width / viewport_height
    }
}

/// Tag component for the main camera
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MainCamera;

impl Component for MainCamera {}

/// Cached camera matrices (computed by camera system)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraMatrices {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub view_projection: glam::Mat4,
}

impl Component for CameraMatrices {}

// ============================================================================
// NEW MESH COMPONENTS - Separate filter and renderer components
// ============================================================================

/// MeshFilter component - holds reference to mesh data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshFilter {
    /// Handle to the mesh resource
    pub mesh: ResourceHandle<MeshData>,
}

impl Component for MeshFilter {}

impl MeshFilter {
    /// Create a new MeshFilter with the given mesh handle
    pub fn new(mesh: ResourceHandle<MeshData>) -> Self {
        Self { mesh }
    }
}

/// MeshRenderer component - handles rendering properties and materials
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshRenderer {
    /// Materials for each submesh (or single material for entire mesh)
    pub materials: Vec<engine_materials_core::MaterialHandle>,

    /// Whether this renderer casts shadows
    pub cast_shadows: bool,

    /// Whether this renderer receives shadows
    pub receive_shadows: bool,

    /// Layer mask for culling and rendering layers
    pub layer_mask: u32,

    /// Whether this renderer is enabled
    pub enabled: bool,
}

impl Component for MeshRenderer {}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            materials: vec![0], // Default material handle (0 = default material)
            cast_shadows: true,
            receive_shadows: true,
            layer_mask: 0xFFFFFFFF, // Render on all layers by default
            enabled: true,
        }
    }
}

impl MeshRenderer {
    /// Create a new MeshRenderer with a single material
    pub fn new(material: engine_materials_core::MaterialHandle) -> Self {
        Self {
            materials: vec![material],
            ..Default::default()
        }
    }

    /// Create a new MeshRenderer with multiple materials for submeshes
    pub fn with_materials(materials: Vec<engine_materials_core::MaterialHandle>) -> Self {
        Self {
            materials,
            ..Default::default()
        }
    }

    /// Set shadow casting
    pub fn with_shadows(mut self, cast: bool, receive: bool) -> Self {
        self.cast_shadows = cast;
        self.receive_shadows = receive;
        self
    }

    /// Set layer mask
    pub fn with_layer_mask(mut self, mask: u32) -> Self {
        self.layer_mask = mask;
        self
    }
}

// ============================================================================
// COMPONENT BUNDLES - Quick solution for multi-component entities
// ============================================================================

use engine_component_traits::Bundle;

/// Bundle for standard 3D game objects
#[derive(Default)]
pub struct GameObject3DBundle {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,
    pub visibility: Visibility,
}

impl Bundle for GameObject3DBundle {
    fn component_ids() -> Vec<std::any::TypeId>
    where
        Self: Sized,
    {
        vec![
            std::any::TypeId::of::<Transform>(),
            std::any::TypeId::of::<Mesh>(),
            std::any::TypeId::of::<Material>(),
            std::any::TypeId::of::<Visibility>(),
        ]
    }

    fn into_components(
        self,
    ) -> Vec<(
        std::any::TypeId,
        Box<dyn engine_component_traits::ComponentClone>,
    )> {
        vec![
            (
                std::any::TypeId::of::<Transform>(),
                Box::new(self.transform),
            ),
            (std::any::TypeId::of::<Mesh>(), Box::new(self.mesh)),
            (std::any::TypeId::of::<Material>(), Box::new(self.material)),
            (
                std::any::TypeId::of::<Visibility>(),
                Box::new(self.visibility),
            ),
        ]
    }
}

/// Bundle for camera entities
#[derive(Default)]
pub struct CameraBundle {
    pub transform: Transform,
    pub camera: Camera,
}

impl Bundle for CameraBundle {
    fn component_ids() -> Vec<std::any::TypeId>
    where
        Self: Sized,
    {
        vec![
            std::any::TypeId::of::<Transform>(),
            std::any::TypeId::of::<Camera>(),
        ]
    }

    fn into_components(
        self,
    ) -> Vec<(
        std::any::TypeId,
        Box<dyn engine_component_traits::ComponentClone>,
    )> {
        vec![
            (
                std::any::TypeId::of::<Transform>(),
                Box::new(self.transform),
            ),
            (std::any::TypeId::of::<Camera>(), Box::new(self.camera)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_resource_core::ResourceId;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
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

    #[test]
    fn test_mesh_filter_creation() {
        let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(123));
        let mesh_filter = MeshFilter::new(mesh_handle.clone());

        assert_eq!(mesh_filter.mesh.id(), mesh_handle.id());
    }

    #[test]
    fn test_mesh_filter_component_trait() {
        // Verify MeshFilter implements Component trait
        let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(456));
        let mesh_filter = MeshFilter::new(mesh_handle);

        // This will only compile if MeshFilter implements Component
        fn assert_is_component<T: Component>(_: &T) {}
        assert_is_component(&mesh_filter);
    }

    #[test]
    fn test_mesh_renderer_default() {
        let renderer = MeshRenderer::default();

        assert_eq!(renderer.materials.len(), 1);
        assert_eq!(renderer.materials[0], 0); // Default material handle
        assert!(renderer.cast_shadows);
        assert!(renderer.receive_shadows);
        assert_eq!(renderer.layer_mask, 0xFFFFFFFF);
        assert!(renderer.enabled);
    }

    #[test]
    fn test_mesh_renderer_with_single_material() {
        let material_handle = 42;
        let renderer = MeshRenderer::new(material_handle);

        assert_eq!(renderer.materials.len(), 1);
        assert_eq!(renderer.materials[0], material_handle);
        assert!(renderer.cast_shadows);
        assert!(renderer.receive_shadows);
        assert!(renderer.enabled);
    }

    #[test]
    fn test_mesh_renderer_with_multiple_materials() {
        let materials = vec![1, 2, 3, 4];
        let renderer = MeshRenderer::with_materials(materials.clone());

        assert_eq!(renderer.materials, materials);
    }

    #[test]
    fn test_mesh_renderer_builder_pattern() {
        let renderer = MeshRenderer::new(5)
            .with_shadows(false, true)
            .with_layer_mask(0x00FF00FF);

        assert_eq!(renderer.materials[0], 5);
        assert!(!renderer.cast_shadows);
        assert!(renderer.receive_shadows);
        assert_eq!(renderer.layer_mask, 0x00FF00FF);
    }

    #[test]
    fn test_mesh_renderer_component_trait() {
        let renderer = MeshRenderer::default();

        // This will only compile if MeshRenderer implements Component
        fn assert_is_component<T: Component>(_: &T) {}
        assert_is_component(&renderer);
    }

    #[test]
    fn test_mesh_filter_serialization() {
        let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(789));
        let mesh_filter = MeshFilter::new(mesh_handle);

        // Serialize
        let serialized = serde_json::to_string(&mesh_filter).unwrap();

        // Deserialize
        let deserialized: MeshFilter = serde_json::from_str(&serialized).unwrap();

        assert_eq!(mesh_filter.mesh.id(), deserialized.mesh.id());
    }

    #[test]
    fn test_mesh_renderer_serialization() {
        let renderer = MeshRenderer::with_materials(vec![10, 20, 30])
            .with_shadows(false, false)
            .with_layer_mask(0x12345678);

        // Serialize
        let serialized = serde_json::to_string(&renderer).unwrap();

        // Deserialize
        let deserialized: MeshRenderer = serde_json::from_str(&serialized).unwrap();

        assert_eq!(renderer.materials, deserialized.materials);
        assert_eq!(renderer.cast_shadows, deserialized.cast_shadows);
        assert_eq!(renderer.receive_shadows, deserialized.receive_shadows);
        assert_eq!(renderer.layer_mask, deserialized.layer_mask);
        assert_eq!(renderer.enabled, deserialized.enabled);
    }

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert_eq!(camera.projection_type, ProjectionType::Perspective);
        assert_eq!(camera.fov_degrees, 60.0);
        assert_eq!(camera.near_plane, 0.1);
        assert_eq!(camera.far_plane, 1000.0);
        assert!(camera.active);
        assert_eq!(camera.priority, 0);
    }

    #[test]
    fn test_camera_perspective() {
        let camera = Camera::perspective(90.0, 0.01, 500.0);
        assert_eq!(camera.projection_type, ProjectionType::Perspective);
        assert_eq!(camera.fov_degrees, 90.0);
        assert_eq!(camera.near_plane, 0.01);
        assert_eq!(camera.far_plane, 500.0);
    }

    #[test]
    fn test_camera_orthographic() {
        let camera = Camera::orthographic(20.0, -10.0, 10.0);
        assert_eq!(camera.projection_type, ProjectionType::Orthographic);
        assert_eq!(camera.orthographic_size, 20.0);
        assert_eq!(camera.near_plane, -10.0);
        assert_eq!(camera.far_plane, 10.0);
    }

    #[test]
    fn test_viewport_default() {
        let viewport = Viewport::default();
        assert_eq!(viewport.x, 0.0);
        assert_eq!(viewport.y, 0.0);
        assert_eq!(viewport.width, 1.0);
        assert_eq!(viewport.height, 1.0);
    }

    #[test]
    fn test_camera_aspect_ratio() {
        let camera = Camera::default();
        let aspect = camera.calculate_aspect_ratio(1920, 1080);
        assert!((aspect - 16.0 / 9.0).abs() < 0.001);

        // Test with custom viewport
        let camera = Camera::default().with_viewport(Viewport {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 0.5,
        });
        let aspect = camera.calculate_aspect_ratio(1920, 1080);
        assert!((aspect - 16.0 / 9.0).abs() < 0.001);
    }
}
