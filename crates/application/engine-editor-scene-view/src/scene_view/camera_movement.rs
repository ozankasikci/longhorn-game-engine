// Camera movement calculation - transforms input from camera space to world space

use engine_components_3d::Transform;

/// Transform movement vector from camera space to world space based on camera rotation
pub fn transform_movement_by_camera(camera_transform: &Transform, movement: [f32; 3]) -> [f32; 3] {
    let yaw = camera_transform.rotation[1];
    let pitch = camera_transform.rotation[0];

    // Debug output disabled - uncomment for debugging
    // if movement.iter().any(|&m| m.abs() > 0.001) {
    //     eprintln!("\n=== Movement Debug ===");
    //     eprintln!("Camera yaw: {:.1}°, pitch: {:.1}°", yaw.to_degrees(), pitch.to_degrees());
    //     eprintln!("Input: right={:.1}, up={:.1}, forward={:.1}", movement[0], movement[1], movement[2]);
    // }

    // Based on our coordinate system test:
    // - At yaw=0, camera looks at +Z
    // - Positive yaw rotates LEFT (towards -X)
    // - At yaw=90°, camera looks at +X

    // Calculate forward direction including pitch for full 3D movement
    // When pitch is 0, forward is horizontal
    // When pitch is positive (looking up), forward has positive Y
    // When pitch is negative (looking down), forward has negative Y

    let forward_x = yaw.sin() * pitch.cos();
    let forward_y = pitch.sin();
    let forward_z = yaw.cos() * pitch.cos();

    // Right direction is always horizontal (no pitch component)
    // Right is perpendicular to forward in the XZ plane
    let right_x = -yaw.cos();
    let right_y = 0.0;
    let right_z = yaw.sin();

    // Up direction for Q/E movement (world up)
    let up_x = 0.0;
    let up_y = 1.0;
    let up_z = 0.0;

    // Transform movement
    let world_x = movement[0] * right_x + movement[1] * up_x + movement[2] * forward_x;
    let world_y = movement[0] * right_y + movement[1] * up_y + movement[2] * forward_y;
    let world_z = movement[0] * right_z + movement[1] * up_z + movement[2] * forward_z;

    // Debug output disabled - uncomment for debugging
    // if movement.iter().any(|&m| m.abs() > 0.001) {
    //     eprintln!("Forward: [{:.2}, {:.2}, {:.2}]", forward_x, forward_y, forward_z);
    //     eprintln!("World movement: [{:.2}, {:.2}, {:.2}]", world_x, world_y, world_z);
    // }

    [world_x, world_y, world_z]
}
