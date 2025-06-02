//! Core camera implementation with ECS v2 integration

use crate::{Viewport, Frustum, Result, OrthographicProjection, PerspectiveProjection};
use engine_components_core::Transform;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};
use bitflags::bitflags;

/// Camera types supported by the engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CameraType {
    /// 2D orthographic camera for 2D games
    Orthographic2D {
        size: f32,
        near: f32,
        far: f32,
    },
    /// 3D perspective camera for 3D games
    Perspective3D {
        fov_degrees: f32,
        near: f32,
        far: f32,
    },
    /// Custom projection matrix
    Custom {
        projection_matrix: [[f32; 4]; 4],
    },
}

impl Default for CameraType {
    fn default() -> Self {
        Self::Orthographic2D {
            size: 5.0,
            near: -10.0,
            far: 10.0,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct CameraDirtyFlags: u8 {
        const VIEW_MATRIX = 0b0001;
        const PROJECTION_MATRIX = 0b0010;
        const FRUSTUM = 0b0100;
        const ALL = 0b0111;
    }
}

/// Core camera with view and projection matrix management
#[derive(Debug, Clone)]
pub struct Camera {
    camera_type: CameraType,
    viewport: Viewport,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    view_projection_matrix: Mat4,
    frustum: Frustum,
    
    // Rendering properties
    clear_color: [f32; 4],
    clear_depth: f32,
    render_order: i32,
    enabled: bool,
    
    // Performance tracking
    last_update_frame: u64,
    dirty_flags: CameraDirtyFlags,
}

impl Camera {
    /// Create a new camera with specified type and viewport
    pub fn new(camera_type: CameraType, viewport: Viewport) -> Self {
        let mut camera = Self {
            camera_type,
            viewport,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            frustum: Frustum::default(),
            clear_color: [0.2, 0.2, 0.3, 1.0],
            clear_depth: 1.0,
            render_order: 0,
            enabled: true,
            last_update_frame: 0,
            dirty_flags: CameraDirtyFlags::ALL,
        };
        
        camera.update_projection_matrix().ok();
        camera
    }
    
    /// Create a 2D orthographic camera
    pub fn orthographic_2d(size: f32, viewport: Viewport) -> Self {
        Self::new(
            CameraType::Orthographic2D {
                size,
                near: -10.0,
                far: 10.0,
            },
            viewport,
        )
    }
    
    /// Create a 3D perspective camera
    pub fn perspective_3d(fov_degrees: f32, viewport: Viewport) -> Self {
        Self::new(
            CameraType::Perspective3D {
                fov_degrees,
                near: 0.1,
                far: 1000.0,
            },
            viewport,
        )
    }
    
    /// Update view matrix from transform
    pub fn update_view_matrix(&mut self, transform: &Transform) -> Result<()> {
        let position = Vec3::from_array(transform.position);
        let rotation = Vec3::from_array(transform.rotation);
        
        // Convert Euler angles to direction vector
        let yaw = rotation.y.to_radians();
        let pitch = rotation.x.to_radians();
        
        let direction = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        ).normalize();
        
        let up = Vec3::Y;
        let target = position + direction;
        
        self.view_matrix = Mat4::look_at_rh(position, target, up);
        self.dirty_flags.insert(CameraDirtyFlags::VIEW_MATRIX);
        
        Ok(())
    }
    
    /// Update projection matrix based on camera type
    pub fn update_projection_matrix(&mut self) -> Result<()> {
        let aspect_ratio = self.viewport.aspect_ratio();
        
        self.projection_matrix = match &self.camera_type {
            CameraType::Orthographic2D { size, near, far } => {
                let projection = OrthographicProjection::from_size_aspect(*size, aspect_ratio, *near, *far)?;
                projection.to_matrix()?
            }
            CameraType::Perspective3D { fov_degrees, near, far } => {
                let projection = PerspectiveProjection::new(*fov_degrees, aspect_ratio, *near, *far)?;
                projection.to_matrix()?
            }
            CameraType::Custom { projection_matrix } => {
                Mat4::from_cols_array_2d(projection_matrix)
            }
        };
        
        self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
        Ok(())
    }
    
    /// Update combined view-projection matrix and frustum
    pub fn update_derived_data(&mut self) -> Result<()> {
        if self.dirty_flags.intersects(CameraDirtyFlags::VIEW_MATRIX | CameraDirtyFlags::PROJECTION_MATRIX) {
            self.view_projection_matrix = self.projection_matrix * self.view_matrix;
            self.dirty_flags.insert(CameraDirtyFlags::FRUSTUM);
        }
        
        if self.dirty_flags.contains(CameraDirtyFlags::FRUSTUM) {
            self.frustum = Frustum::from_matrix(self.view_projection_matrix)?;
        }
        
        self.dirty_flags = CameraDirtyFlags::empty();
        Ok(())
    }
    
    /// Get world-to-screen projection
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec3> {
        let clip_pos = self.view_projection_matrix * world_pos.extend(1.0);
        
        if clip_pos.w <= 0.0 {
            return None; // Behind camera
        }
        
        let ndc = clip_pos.xyz() / clip_pos.w;
        
        // Check if point is within NDC bounds
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
            return None;
        }
        
        // Convert to screen coordinates
        let screen_x = (ndc.x + 1.0) * 0.5 * self.viewport.width as f32;
        let screen_y = (1.0 - ndc.y) * 0.5 * self.viewport.height as f32;
        
        Some(Vec3::new(screen_x, screen_y, ndc.z))
    }
    
    /// Get screen-to-world ray
    pub fn screen_to_world_ray(&self, screen_pos: Vec3) -> Option<(Vec3, Vec3)> {
        let x = (screen_pos.x / self.viewport.width as f32) * 2.0 - 1.0;
        let y = 1.0 - (screen_pos.y / self.viewport.height as f32) * 2.0;
        
        let inv_view_proj = self.view_projection_matrix.inverse();
        
        let near_point = inv_view_proj * Vec4::new(x, y, -1.0, 1.0);
        let far_point = inv_view_proj * Vec4::new(x, y, 1.0, 1.0);
        
        if near_point.w == 0.0 || far_point.w == 0.0 {
            return None;
        }
        
        let near_world = near_point.xyz() / near_point.w;
        let far_world = far_point.xyz() / far_point.w;
        
        let origin = near_world;
        let direction = (far_world - near_world).normalize();
        
        Some((origin, direction))
    }
    
    // Getters
    pub fn camera_type(&self) -> &CameraType { &self.camera_type }
    pub fn viewport(&self) -> &Viewport { &self.viewport }
    pub fn view_matrix(&self) -> Mat4 { self.view_matrix }
    pub fn projection_matrix(&self) -> Mat4 { self.projection_matrix }
    pub fn view_projection_matrix(&self) -> Mat4 { self.view_projection_matrix }
    pub fn frustum(&self) -> &Frustum { &self.frustum }
    pub fn clear_color(&self) -> [f32; 4] { self.clear_color }
    pub fn clear_depth(&self) -> f32 { self.clear_depth }
    pub fn render_order(&self) -> i32 { self.render_order }
    pub fn enabled(&self) -> bool { self.enabled }
    
    // Setters
    pub fn set_camera_type(&mut self, camera_type: CameraType) {
        self.camera_type = camera_type;
        self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
    }
    
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.dirty_flags.insert(CameraDirtyFlags::PROJECTION_MATRIX);
    }
    
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }
    
    pub fn set_render_order(&mut self, order: i32) {
        self.render_order = order;
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Camera component for ECS integration
#[derive(Debug, Clone)]
pub struct CameraComponent {
    pub camera: Camera,
    pub is_main: bool,
    pub target_texture: Option<u64>, // Handle to render target texture
}

impl CameraComponent {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            is_main: false,
            target_texture: None,
        }
    }
    
    pub fn main_camera(camera: Camera) -> Self {
        Self {
            camera,
            is_main: true,
            target_texture: None,
        }
    }
    
    pub fn with_render_target(mut self, texture_handle: u64) -> Self {
        self.target_texture = Some(texture_handle);
        self
    }
    
    /// Update camera matrices from transform
    pub fn update(&mut self, transform: &Transform, frame: u64) -> Result<()> {
        if self.camera.last_update_frame != frame {
            self.camera.update_view_matrix(transform)?;
            self.camera.update_derived_data()?;
            self.camera.last_update_frame = frame;
        }
        Ok(())
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self::new(Camera::orthographic_2d(5.0, Viewport::new(800, 600)))
    }
}

// ECS v2 integration
impl engine_ecs_core::Component for CameraComponent {}
impl engine_ecs_core::ecs_v2::Component for CameraComponent {}

/// Camera uniform data for GPU shaders
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
    pub view_projection_matrix: [[f32; 4]; 4],
    pub camera_position: [f32; 3],
    pub _padding1: f32,
    pub viewport_size: [f32; 2],
    pub near_far: [f32; 2], // [near, far]
    pub clear_color: [f32; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera, camera_position: Vec3) -> Self {
        let (near, far) = match &camera.camera_type {
            CameraType::Orthographic2D { near, far, .. } => (*near, *far),
            CameraType::Perspective3D { near, far, .. } => (*near, *far),
            CameraType::Custom { .. } => (0.1, 1000.0), // Default values
        };
        
        Self {
            view_matrix: camera.view_matrix.to_cols_array_2d(),
            projection_matrix: camera.projection_matrix.to_cols_array_2d(),
            view_projection_matrix: camera.view_projection_matrix.to_cols_array_2d(),
            camera_position: camera_position.to_array(),
            _padding1: 0.0,
            viewport_size: [camera.viewport.width as f32, camera.viewport.height as f32],
            near_far: [near, far],
            clear_color: camera.clear_color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_creation() {
        let viewport = Viewport::new(800, 600);
        let camera = Camera::orthographic_2d(10.0, viewport);
        
        assert!(camera.enabled);
        assert_eq!(camera.render_order, 0);
        assert_eq!(camera.clear_color, [0.2, 0.2, 0.3, 1.0]);
    }
    
    #[test]
    fn test_camera_component() {
        let viewport = Viewport::new(800, 600);
        let camera = Camera::perspective_3d(60.0, viewport);
        let component = CameraComponent::main_camera(camera);
        
        assert!(component.is_main);
        assert_eq!(component.target_texture, None);
    }
    
    #[test]
    fn test_camera_matrix_update() {
        let viewport = Viewport::new(800, 600);
        let mut camera = Camera::orthographic_2d(5.0, viewport);
        let transform = Transform::default();
        
        assert!(camera.update_view_matrix(&transform).is_ok());
        assert!(camera.update_derived_data().is_ok());
    }
    
    #[test]
    fn test_world_to_screen_conversion() {
        let viewport = Viewport::new(800, 600);
        let mut camera = Camera::orthographic_2d(5.0, viewport);
        let transform = Transform::default();
        
        camera.update_view_matrix(&transform).unwrap();
        camera.update_derived_data().unwrap();
        
        // Origin should map to center of screen
        let screen_pos = camera.world_to_screen(Vec3::ZERO);
        assert!(screen_pos.is_some());
        
        if let Some(pos) = screen_pos {
            assert!((pos.x - 400.0).abs() < 1.0); // Center X
            assert!((pos.y - 300.0).abs() < 1.0); // Center Y
        }
    }
    
    #[test]
    fn test_camera_uniform() {
        let viewport = Viewport::new(800, 600);
        let camera = Camera::orthographic_2d(5.0, viewport);
        let position = Vec3::new(1.0, 2.0, 3.0);
        
        let uniform = CameraUniform::from_camera(&camera, position);
        assert_eq!(uniform.camera_position, [1.0, 2.0, 3.0]);
        assert_eq!(uniform.viewport_size, [800.0, 600.0]);
        assert_eq!(uniform.clear_color, camera.clear_color);
    }
}