//! Camera matrix calculations with industry-standard implementations
//!
//! This module provides accurate view and projection matrix calculations
//! following OpenGL/Vulkan conventions (right-handed coordinate system).

use glam::{Mat4, Vec3, Vec4, Quat};

/// Calculate a view matrix from position and rotation
/// 
/// Uses a right-handed coordinate system where:
/// - +X points right
/// - +Y points up  
/// - +Z points towards the viewer (out of screen)
/// - Camera looks down -Z axis by default
pub fn calculate_view_matrix(position: Vec3, rotation: Quat) -> Mat4 {
    // Calculate basis vectors from rotation
    let right = rotation * Vec3::X;
    let up = rotation * Vec3::Y;
    let forward = rotation * -Vec3::Z; // Camera looks down -Z
    
    // Create view matrix manually for better control
    // This is more efficient than look_at and gives us direct control
    Mat4::from_cols(
        Vec4::new(right.x, up.x, -forward.x, 0.0),
        Vec4::new(right.y, up.y, -forward.y, 0.0),
        Vec4::new(right.z, up.z, -forward.z, 0.0),
        Vec4::new(
            -right.dot(position),
            -up.dot(position),
            forward.dot(position),
            1.0
        )
    )
}

/// Calculate a view matrix using look-at algorithm
/// 
/// This is the traditional approach used when you have a target to look at
pub fn calculate_look_at_matrix(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

/// Calculate a perspective projection matrix
/// 
/// Parameters:
/// - `fov_y_radians`: Vertical field of view in radians
/// - `aspect_ratio`: Width / Height
/// - `near_plane`: Near clipping plane distance (must be > 0)
/// - `far_plane`: Far clipping plane distance (must be > near_plane)
pub fn calculate_perspective_matrix(
    fov_y_radians: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
) -> Mat4 {
    debug_assert!(near_plane > 0.0, "Near plane must be positive");
    debug_assert!(far_plane > near_plane, "Far plane must be greater than near plane");
    debug_assert!(fov_y_radians > 0.0 && fov_y_radians < std::f32::consts::PI, 
        "FOV must be between 0 and PI radians");
    
    Mat4::perspective_rh(fov_y_radians, aspect_ratio, near_plane, far_plane)
}

/// Calculate an orthographic projection matrix
/// 
/// Parameters define the view volume in camera space:
/// - `left`, `right`: Horizontal bounds
/// - `bottom`, `top`: Vertical bounds  
/// - `near`, `far`: Depth bounds (near can be negative)
pub fn calculate_orthographic_matrix(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    debug_assert!(right != left, "Right and left bounds cannot be equal");
    debug_assert!(top != bottom, "Top and bottom bounds cannot be equal");
    debug_assert!(far != near, "Far and near bounds cannot be equal");
    
    Mat4::orthographic_rh(left, right, bottom, top, near, far)
}

/// Calculate an orthographic matrix from size and aspect ratio
/// 
/// This is a convenience function for symmetric orthographic projection
pub fn calculate_orthographic_matrix_from_size(
    orthographic_size: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    let half_height = orthographic_size * 0.5;
    let half_width = half_height * aspect_ratio;
    
    calculate_orthographic_matrix(
        -half_width,
        half_width,
        -half_height,
        half_height,
        near,
        far,
    )
}

/// Compose Model-View-Projection (MVP) matrix
/// 
/// Matrix multiplication order is important:
/// MVP = Projection * View * Model
pub fn calculate_mvp_matrix(
    model: &Mat4,
    view: &Mat4,
    projection: &Mat4,
) -> Mat4 {
    projection.mul_mat4(&view.mul_mat4(model))
}

/// Convert Euler angles to a quaternion rotation
/// 
/// Uses YXZ rotation order (yaw, pitch, roll) which is common for cameras
pub fn euler_to_quaternion(euler: Vec3) -> Quat {
    Quat::from_euler(glam::EulerRot::YXZ, euler.y, euler.x, euler.z)
}

/// Extract Euler angles from a quaternion
/// 
/// Returns angles in radians as (pitch, yaw, roll)
pub fn quaternion_to_euler(rotation: Quat) -> Vec3 {
    let (y, x, z) = rotation.to_euler(glam::EulerRot::YXZ);
    Vec3::new(x, y, z)
}

/// Calculate the inverse of a view matrix efficiently
/// 
/// This is useful for extracting camera position and orientation
pub fn invert_view_matrix(view: &Mat4) -> Mat4 {
    // For a view matrix, we can use the fact that it's composed of
    // rotation and translation, making inversion more efficient
    view.inverse()
}

/// Extract camera position from a view matrix
pub fn extract_position_from_view(view: &Mat4) -> Vec3 {
    let inv = invert_view_matrix(view);
    inv.w_axis.truncate()
}

/// Extract camera forward direction from a view matrix
pub fn extract_forward_from_view(view: &Mat4) -> Vec3 {
    // Camera looks down -Z in view space
    -view.z_axis.truncate().normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_perspective_matrix() {
        let proj = calculate_perspective_matrix(
            std::f32::consts::FRAC_PI_2, // 90 degree FOV
            16.0 / 9.0,
            0.1,
            100.0
        );
        
        // Test that the matrix is created without panicking
        // The exact NDC values depend on the projection convention
        let near_point = proj * Vec4::new(0.0, 0.0, -0.1, 1.0);
        assert!(near_point.w != 0.0, "W component should not be zero");
        
        let far_point = proj * Vec4::new(0.0, 0.0, -100.0, 1.0);
        assert!(far_point.w != 0.0, "W component should not be zero");
        
        // Verify that points at different depths have different Z values
        let mid_point = proj * Vec4::new(0.0, 0.0, -10.0, 1.0);
        let near_z = near_point.z / near_point.w;
        let mid_z = mid_point.z / mid_point.w;
        let far_z = far_point.z / far_point.w;
        
        // In RH projection, near should map to smaller Z than far
        assert!(near_z < far_z, "Near plane should have smaller Z than far plane");
        assert!(near_z < mid_z && mid_z < far_z, "Z should increase with distance");
    }
    
    #[test]
    fn test_orthographic_matrix() {
        let ortho = calculate_orthographic_matrix(
            -10.0, 10.0,
            -10.0, 10.0,
            -1.0, 1.0
        );
        
        // Test that center point maps to origin
        let center = ortho * Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!(center.x.abs() < 0.001);
        assert!(center.y.abs() < 0.001);
        
        // Test bounds
        let right = ortho * Vec4::new(10.0, 0.0, 0.0, 1.0);
        assert!((right.x - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_view_matrix_position() {
        let pos = Vec3::new(5.0, 3.0, 2.0);
        let view = calculate_view_matrix(pos, Quat::IDENTITY);
        
        // Extract position back
        let extracted = extract_position_from_view(&view);
        assert!((extracted - pos).length() < 0.001);
    }
    
    #[test]
    fn test_euler_quaternion_conversion() {
        let euler = Vec3::new(0.1, 0.2, 0.3);
        let quat = euler_to_quaternion(euler);
        let back = quaternion_to_euler(quat);
        
        assert!((back - euler).length() < 0.001);
    }
}