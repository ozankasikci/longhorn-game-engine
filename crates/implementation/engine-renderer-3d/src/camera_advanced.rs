//! Advanced Camera System with MVP Matrix Management
//! 
//! This module provides an enhanced camera system with:
//! - Proper MVP matrix calculations
//! - View frustum extraction
//! - Camera controller utilities
//! - Screen-to-world ray casting

use crate::{Camera, Frustum};
use glam::{Mat4, Vec3, Vec4, Quat};

/// Advanced camera controller with enhanced matrix management
#[derive(Debug, Clone)]
pub struct CameraController {
    /// Base camera data
    pub camera: Camera,
    /// Cached view matrix (updated when camera moves)
    view_matrix: Mat4,
    /// Cached projection matrix (updated when projection params change)
    projection_matrix: Mat4,
    /// Cached view-projection matrix
    view_projection_matrix: Mat4,
    /// Cached inverse view matrix (for world-to-camera transforms)
    inverse_view_matrix: Mat4,
    /// Cached inverse projection matrix (for screen-to-world)
    inverse_projection_matrix: Mat4,
    /// Whether matrices need updating
    matrices_dirty: bool,
}

impl CameraController {
    /// Create a new camera controller
    pub fn new(camera: Camera) -> Self {
        let mut controller = Self {
            camera,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            inverse_view_matrix: Mat4::IDENTITY,
            inverse_projection_matrix: Mat4::IDENTITY,
            matrices_dirty: true,
        };
        controller.update_matrices();
        controller
    }
    
    /// Update all cached matrices
    pub fn update_matrices(&mut self) {
        if !self.matrices_dirty {
            return;
        }
        
        // Calculate view matrix
        self.view_matrix = Mat4::look_at_rh(
            self.camera.position,
            self.camera.target,
            self.camera.up
        );
        
        // Calculate projection matrix
        self.projection_matrix = Mat4::perspective_rh(
            self.camera.fov,
            self.camera.aspect,
            self.camera.near,
            self.camera.far
        );
        
        // Calculate combined view-projection matrix
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        
        // Calculate inverse matrices
        self.inverse_view_matrix = self.view_matrix.inverse();
        self.inverse_projection_matrix = self.projection_matrix.inverse();
        
        self.matrices_dirty = false;
    }
    
    /// Mark matrices as needing update
    fn mark_dirty(&mut self) {
        self.matrices_dirty = true;
    }
    
    /// Get the view matrix (cached)
    pub fn view_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.view_matrix
    }
    
    /// Get the projection matrix (cached)
    pub fn projection_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.projection_matrix
    }
    
    /// Get the view-projection matrix (cached)
    pub fn view_projection_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.view_projection_matrix
    }
    
    /// Get the inverse view matrix (cached)
    pub fn inverse_view_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.inverse_view_matrix
    }
    
    /// Get the inverse projection matrix (cached)
    pub fn inverse_projection_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.inverse_projection_matrix
    }
    
    /// Move the camera to a new position
    pub fn set_position(&mut self, position: Vec3) {
        self.camera.position = position;
        self.mark_dirty();
    }
    
    /// Set the camera target
    pub fn set_target(&mut self, target: Vec3) {
        self.camera.target = target;
        self.mark_dirty();
    }
    
    /// Set both position and target
    pub fn set_position_target(&mut self, position: Vec3, target: Vec3) {
        self.camera.position = position;
        self.camera.target = target;
        self.mark_dirty();
    }
    
    /// Set the field of view
    pub fn set_fov(&mut self, fov_radians: f32) {
        self.camera.fov = fov_radians;
        self.mark_dirty();
    }
    
    /// Set the aspect ratio
    pub fn set_aspect(&mut self, aspect: f32) {
        self.camera.aspect = aspect;
        self.mark_dirty();
    }
    
    /// Set near and far clip planes
    pub fn set_clip_planes(&mut self, near: f32, far: f32) {
        self.camera.near = near;
        self.camera.far = far;
        self.mark_dirty();
    }
    
    /// Get the camera's forward direction
    pub fn forward(&self) -> Vec3 {
        (self.camera.target - self.camera.position).normalize()
    }
    
    /// Get the camera's right direction
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.camera.up).normalize()
    }
    
    /// Get the camera's actual up direction (may differ from input up)
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }
    
    /// Move the camera forward/backward
    pub fn move_forward(&mut self, distance: f32) {
        let forward = self.forward();
        self.camera.position += forward * distance;
        self.camera.target += forward * distance;
        self.mark_dirty();
    }
    
    /// Move the camera right/left
    pub fn move_right(&mut self, distance: f32) {
        let right = self.right();
        self.camera.position += right * distance;
        self.camera.target += right * distance;
        self.mark_dirty();
    }
    
    /// Move the camera up/down
    pub fn move_up(&mut self, distance: f32) {
        let up = self.camera.up;
        self.camera.position += up * distance;
        self.camera.target += up * distance;
        self.mark_dirty();
    }
    
    /// Rotate the camera around its position (first-person style)
    pub fn rotate_fps(&mut self, yaw_delta: f32, pitch_delta: f32) {
        let forward = self.forward();
        let right = self.right();
        let up = self.up();
        
        // Create rotation quaternions
        let yaw_rotation = Quat::from_axis_angle(up, yaw_delta);
        let pitch_rotation = Quat::from_axis_angle(right, pitch_delta);
        
        // Apply rotations to forward vector
        let new_forward = yaw_rotation * (pitch_rotation * forward);
        
        // Update target based on new forward direction
        self.camera.target = self.camera.position + new_forward;
        self.mark_dirty();
    }
    
    /// Orbit the camera around the target point
    pub fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) {
        let distance = self.camera.position.distance(self.camera.target);
        let direction = (self.camera.position - self.camera.target).normalize();
        
        // Convert to spherical coordinates
        let radius = distance;
        let current_yaw = direction.z.atan2(direction.x);
        let current_pitch = direction.y.asin();
        
        // Apply deltas
        let new_yaw = current_yaw + yaw_delta;
        let new_pitch = (current_pitch + pitch_delta).clamp(-1.5, 1.5); // Limit pitch
        
        // Convert back to cartesian
        let new_direction = Vec3::new(
            new_pitch.cos() * new_yaw.cos(),
            new_pitch.sin(),
            new_pitch.cos() * new_yaw.sin(),
        );
        
        self.camera.position = self.camera.target + new_direction * radius;
        self.mark_dirty();
    }
    
    /// Zoom in/out by moving closer/farther from target
    pub fn zoom(&mut self, zoom_delta: f32) {
        let direction = (self.camera.position - self.camera.target).normalize();
        let distance = self.camera.position.distance(self.camera.target);
        let new_distance = (distance + zoom_delta).max(0.1); // Minimum distance
        
        self.camera.position = self.camera.target + direction * new_distance;
        self.mark_dirty();
    }
    
    /// Convert screen coordinates to world ray
    pub fn screen_to_world_ray(&mut self, screen_x: f32, screen_y: f32, screen_width: f32, screen_height: f32) -> Ray {
        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = (2.0 * screen_x / screen_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / screen_height); // Flip Y
        
        // Create NDC points at near and far planes
        let near_point = Vec4::new(ndc_x, ndc_y, -1.0, 1.0); // Near plane
        let far_point = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);   // Far plane
        
        // Transform to world space
        let inv_view_proj = self.view_projection_matrix().inverse();
        let world_near = inv_view_proj * near_point;
        let world_far = inv_view_proj * far_point;
        
        // Perspective divide
        let world_near = world_near.truncate() / world_near.w;
        let world_far = world_far.truncate() / world_far.w;
        
        // Create ray
        Ray {
            origin: world_near,
            direction: (world_far - world_near).normalize(),
        }
    }
    
    /// Convert world position to screen coordinates
    pub fn world_to_screen(&mut self, world_pos: Vec3, screen_width: f32, screen_height: f32) -> Option<Vec3> {
        let world_pos_4 = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        let clip_pos = self.view_projection_matrix() * world_pos_4;
        
        // Check if point is behind camera
        if clip_pos.w <= 0.0 {
            return None;
        }
        
        // Perspective divide
        let ndc = clip_pos.truncate() / clip_pos.w;
        
        // Convert to screen coordinates
        let screen_x = (ndc.x + 1.0) * 0.5 * screen_width;
        let screen_y = (1.0 - ndc.y) * 0.5 * screen_height; // Flip Y
        let depth = ndc.z; // Depth value (0 = near, 1 = far)
        
        Some(Vec3::new(screen_x, screen_y, depth))
    }
    
    /// Get the view frustum for culling
    pub fn get_frustum(&mut self) -> Frustum {
        Frustum::from_view_projection_matrix(self.view_projection_matrix())
    }
    
    /// Check if a point is visible in the camera's view
    pub fn is_point_visible(&mut self, point: Vec3) -> bool {
        let frustum = self.get_frustum();
        frustum.is_point_inside(point)
    }
    
    /// Get camera info for debugging
    pub fn get_info(&self) -> CameraInfo {
        CameraInfo {
            position: self.camera.position,
            target: self.camera.target,
            forward: self.forward(),
            right: self.right(),
            up: self.up(),
            fov_degrees: self.camera.fov.to_degrees(),
            aspect: self.camera.aspect,
            near: self.camera.near,
            far: self.camera.far,
            distance_to_target: self.camera.position.distance(self.camera.target),
        }
    }
}

/// A ray in 3D space for mouse picking and ray casting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin point of the ray
    pub origin: Vec3,
    /// Direction of the ray (should be normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    /// Get a point along the ray at the given distance
    pub fn at(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
    
    /// Test ray intersection with a sphere
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                Some(t)
            } else {
                None
            }
        }
    }
    
    /// Test ray intersection with a plane
    pub fn intersect_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
        let denom = plane_normal.dot(self.direction);
        if denom.abs() < 1e-6 {
            None // Ray is parallel to plane
        } else {
            let t = (plane_point - self.origin).dot(plane_normal) / denom;
            if t >= 0.0 {
                Some(t)
            } else {
                None
            }
        }
    }
}

/// Camera information for debugging and UI display
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub position: Vec3,
    pub target: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub fov_degrees: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub distance_to_target: f32,
}

impl std::fmt::Display for CameraInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Camera {{ pos: {:?}, target: {:?}, fov: {:.1}°, aspect: {:.2}, near: {:.2}, far: {:.1}, distance: {:.1} }}",
            self.position,
            self.target,
            self.fov_degrees,
            self.aspect,
            self.near,
            self.far,
            self.distance_to_target
        )
    }
}

/// Camera controller presets for common use cases
pub struct CameraPresets;

impl CameraPresets {
    /// Create a first-person camera controller
    pub fn first_person(position: Vec3, target: Vec3, aspect: f32) -> CameraController {
        let mut camera = Camera::new(aspect);
        camera.position = position;
        camera.target = target;
        camera.fov = 75.0_f32.to_radians(); // Typical FPS FOV
        camera.near = 0.01;
        camera.far = 1000.0;
        
        CameraController::new(camera)
    }
    
    /// Create an orbital camera controller (good for 3D viewers)
    pub fn orbital(target: Vec3, distance: f32, aspect: f32) -> CameraController {
        let mut camera = Camera::new(aspect);
        camera.target = target;
        camera.position = target + Vec3::new(0.0, 0.0, distance);
        camera.fov = 60.0_f32.to_radians();
        camera.near = 0.1;
        camera.far = distance * 10.0;
        
        CameraController::new(camera)
    }
    
    /// Create an orthographic-style camera for 2D/UI work
    pub fn orthographic(center: Vec3, _zoom: f32, aspect: f32) -> CameraController {
        let mut camera = Camera::new(aspect);
        camera.position = center + Vec3::new(0.0, 0.0, 10.0);
        camera.target = center;
        camera.fov = 1.0_f32.to_radians(); // Very small FOV for "orthographic" effect
        camera.near = 0.1;
        camera.far = 100.0;
        
        CameraController::new(camera)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_controller_creation() {
        let camera = Camera::new(16.0 / 9.0);
        let mut controller = CameraController::new(camera);
        
        // Matrices should be calculated
        let _view = controller.view_matrix();
        let _proj = controller.projection_matrix();
        let _vp = controller.view_projection_matrix();
    }
    
    #[test]
    fn test_camera_movement() {
        let camera = Camera::new(16.0 / 9.0);
        let mut controller = CameraController::new(camera);
        
        let initial_pos = controller.camera.position;
        controller.move_forward(5.0);
        
        // Should have moved forward
        assert!((controller.camera.position - initial_pos).length() > 0.0);
    }
    
    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let intersection = ray.intersect_sphere(Vec3::new(0.0, 0.0, 5.0), 1.0);
        
        assert!(intersection.is_some());
        assert!((intersection.unwrap() - 4.0).abs() < 0.1);
    }
    
    #[test]
    fn test_camera_presets() {
        let fps_cam = CameraPresets::first_person(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            16.0 / 9.0
        );
        
        assert_eq!(fps_cam.camera.position, Vec3::new(0.0, 0.0, 0.0));
        
        let orbital_cam = CameraPresets::orbital(Vec3::ZERO, 10.0, 16.0 / 9.0);
        assert!(orbital_cam.camera.position.distance(Vec3::ZERO) > 9.0);
    }
}