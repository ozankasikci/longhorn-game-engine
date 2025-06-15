//! Camera systems for ECS integration
//!
//! This module provides systems that process camera components and update matrices.

use engine_components_3d::{Camera, Transform, CameraMatrices, ProjectionType, MainCamera};
use engine_ecs_core::World;
use engine_camera_core::matrices;

/// System that updates camera matrices from Camera and Transform components
pub fn camera_update_system(world: &mut World) {
    // Get all entities with Camera component
    let camera_entities: Vec<_> = world.query_legacy::<Camera>()
        .map(|(entity, _)| entity)
        .collect();
    
    for entity in camera_entities {
        // Get components
        let (camera, transform) = {
            let camera = world.get_component::<Camera>(entity).unwrap();
            let transform = world.get_component::<Transform>(entity).unwrap();
            (camera.clone(), transform.clone())
        };
        
        // Skip inactive cameras
        if !camera.active {
            continue;
        }
        
        // Calculate view matrix from transform
        let rotation = matrices::euler_to_quaternion(transform.rotation.into());
        let view_matrix = matrices::calculate_view_matrix(transform.position.into(), rotation);
        
        // Calculate projection matrix based on type
        // Note: Aspect ratio should be calculated from viewport and screen size
        // For now, we'll use a default aspect ratio
        let aspect_ratio = 16.0 / 9.0; // TODO: Get from screen/viewport
        
        let projection_matrix = match camera.projection_type {
            ProjectionType::Perspective => {
                matrices::calculate_perspective_matrix(
                    camera.fov_degrees.to_radians(),
                    aspect_ratio,
                    camera.near_plane,
                    camera.far_plane,
                )
            }
            ProjectionType::Orthographic => {
                matrices::calculate_orthographic_matrix_from_size(
                    camera.orthographic_size,
                    aspect_ratio,
                    camera.near_plane,
                    camera.far_plane,
                )
            }
        };
        
        // Calculate combined view-projection matrix
        let view_projection = projection_matrix * view_matrix;
        
        // Create or update CameraMatrices component
        let matrices = CameraMatrices {
            view: view_matrix,
            projection: projection_matrix,
            view_projection,
        };
        
        // Add or update the component
        if world.has_component::<CameraMatrices>(entity) {
            if let Some(matrices_comp) = world.get_component_mut::<CameraMatrices>(entity) {
                *matrices_comp = matrices;
            }
        } else {
            world.add_component(entity, matrices);
        }
    }
}

/// Find the main camera entity
pub fn find_main_camera(world: &World) -> Option<engine_ecs_core::Entity> {
    world.query_legacy::<MainCamera>()
        .map(|(entity, _)| entity)
        .next()
}

/// Find the highest priority active camera
pub fn find_active_camera(world: &World) -> Option<engine_ecs_core::Entity> {
    let mut cameras: Vec<_> = world.query_legacy::<Camera>()
        .filter(|(_, camera)| camera.active)
        .collect();
    
    // Sort by priority (highest first)
    cameras.sort_by_key(|(_, camera)| -camera.priority);
    
    cameras.first().map(|(entity, _)| *entity)
}

/// Get camera matrices for rendering
pub fn get_camera_matrices(world: &World, entity: engine_ecs_core::Entity) -> Option<CameraMatrices> {
    world.get_component::<CameraMatrices>(entity).cloned()
}

/// Update aspect ratio for a camera
pub fn update_camera_aspect_ratio(
    world: &mut World,
    entity: engine_ecs_core::Entity,
    screen_width: u32,
    screen_height: u32,
) {
    if let Some(camera) = world.get_component::<Camera>(entity) {
        let _aspect_ratio = camera.calculate_aspect_ratio(screen_width, screen_height);
        
        // Force recalculation of matrices by marking as dirty
        // For now, we'll just run the system again
        // In a more advanced system, we'd have a dirty flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_components_3d::{Camera, Transform, CameraBundle};
    
    #[test]
    fn test_camera_update_system() {
        let mut world = World::new();
        
        // Create a camera entity
        let camera_entity = world.spawn((
            Transform::default().with_position(0.0, 5.0, 10.0),
            Camera::perspective(60.0, 0.1, 1000.0),
        ));
        
        // Run the system
        camera_update_system(&mut world);
        
        // Check that matrices were created
        let matrices = world.get_component::<CameraMatrices>(camera_entity);
        assert!(matrices.is_some());
        
        let matrices = matrices.unwrap();
        assert_ne!(matrices.view, Mat4::IDENTITY);
        assert_ne!(matrices.projection, Mat4::IDENTITY);
    }
    
    #[test]
    fn test_find_main_camera() {
        let mut world = World::new();
        
        // Create regular camera
        world.spawn((Transform::default(), Camera::default()));
        
        // Create main camera
        let main_entity = world.spawn((
            Transform::default(),
            Camera::default(),
            MainCamera,
        ));
        
        let found = find_main_camera(&world);
        assert_eq!(found, Some(main_entity));
    }
    
    #[test]
    fn test_camera_priority() {
        let mut world = World::new();
        
        // Create low priority camera
        world.spawn((
            Transform::default(),
            Camera::default().with_priority(0),
        ));
        
        // Create high priority camera
        let high_priority = world.spawn((
            Transform::default(),
            Camera::default().with_priority(10),
        ));
        
        let found = find_active_camera(&world);
        assert_eq!(found, Some(high_priority));
    }
}