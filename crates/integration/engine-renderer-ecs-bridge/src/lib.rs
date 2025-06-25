/// ECS bridge for renderer integration
/// 
/// This crate provides implementations of renderer-core traits that work
/// with ECS components, completely decoupling the renderer from ECS.

use engine_renderer_core::{Renderable, TransformProvider, RenderableQuery, CameraProvider};
use engine_ecs_core::ecs_v2::{Entity, World};
use engine_components_3d::{Transform, Mesh, Camera, MeshType, ProjectionType};
use glam::{Mat4, Vec3, Quat};

/// ECS implementation of Renderable trait
#[derive(Clone)]
pub struct EcsRenderable {
    pub entity: Entity,
    pub transform: Transform,
    pub mesh: Option<Mesh>,
    pub material_handle: Option<u32>, // Simplified for now
}

impl EcsRenderable {
    pub fn new(entity: Entity, transform: Transform) -> Self {
        Self {
            entity,
            transform,
            mesh: None,
            material_handle: None,
        }
    }
    
    pub fn with_mesh(mut self, mesh: Mesh) -> Self {
        self.mesh = Some(mesh);
        self
    }
    
    pub fn with_material(mut self, material_handle: u32) -> Self {
        self.material_handle = Some(material_handle);
        self
    }
}

impl Renderable for EcsRenderable {
    fn world_matrix(&self) -> Mat4 {
        self.transform.matrix()
    }
    
    fn mesh_handle(&self) -> Option<u32> {
        self.mesh.as_ref().map(|m| match m.mesh_type {
            MeshType::Cube => 1,
            MeshType::Sphere => 2, 
            MeshType::Plane => 3,
            MeshType::Custom(_) => 100, // Custom meshes get a base handle
        })
    }
    
    fn material_handle(&self) -> Option<u32> {
        self.material_handle
    }
    
    fn is_visible(&self) -> bool {
        true // For now, assume all objects are visible
    }
}

/// ECS implementation of TransformProvider trait
pub struct EcsTransformProvider {
    transform: Transform,
}

impl EcsTransformProvider {
    pub fn new(transform: Transform) -> Self {
        Self { transform }
    }
}

impl TransformProvider for EcsTransformProvider {
    fn world_matrix(&self) -> Mat4 {
        self.transform.matrix()
    }
    
    fn position(&self) -> Vec3 {
        Vec3::from_array(self.transform.position)
    }
    
    fn rotation(&self) -> Quat {
        Quat::from_euler(
            glam::EulerRot::XYZ,
            self.transform.rotation[0],
            self.transform.rotation[1], 
            self.transform.rotation[2]
        )
    }
    
    fn scale(&self) -> Vec3 {
        Vec3::from_array(self.transform.scale)
    }
}

/// ECS implementation of RenderableQuery trait
pub struct EcsRenderableQuery {
    renderables: Vec<EcsRenderable>,
}

impl EcsRenderableQuery {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
        }
    }
    
    /// Extract renderables from ECS world
    pub fn from_world(world: &World) -> Self {
        let mut renderables = Vec::new();
        
        // Query all entities with Transform components
        for (entity, transform) in world.query_legacy::<Transform>() {
            let mut ecs_renderable = EcsRenderable::new(entity, transform.clone());
            
            // Try to get mesh component
            if let Some(mesh) = world.get_component::<Mesh>(entity) {
                ecs_renderable = ecs_renderable.with_mesh(mesh.clone());
            }
            
            // TODO: Add material component lookup when available
            
            renderables.push(ecs_renderable);
        }
        
        Self { renderables }
    }
}

impl Default for EcsRenderableQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderableQuery for EcsRenderableQuery {
    type Item = EcsRenderable;
    type Iter = std::vec::IntoIter<EcsRenderable>;
    
    fn iter(&self) -> Self::Iter {
        self.renderables.clone().into_iter()
    }
    
    fn len(&self) -> usize {
        self.renderables.len()
    }
}

/// ECS implementation of CameraProvider trait
pub struct EcsCameraProvider {
    transform: Transform,
    camera: Camera,
}

impl EcsCameraProvider {
    pub fn new(transform: Transform, camera: Camera) -> Self {
        Self { transform, camera }
    }
    
    /// Extract camera from ECS world
    pub fn from_world(world: &World) -> Option<Self> {
        // Find first entity with both Transform and Camera components
        for (entity, transform) in world.query_legacy::<Transform>() {
            if let Some(camera) = world.get_component::<Camera>(entity) {
                return Some(Self::new(transform.clone(), camera.clone()));
            }
        }
        None
    }
}

impl CameraProvider for EcsCameraProvider {
    fn view_matrix(&self) -> Mat4 {
        // Create view matrix from transform
        let position = Vec3::from_array(self.transform.position);
        let rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            self.transform.rotation[0],
            self.transform.rotation[1],
            self.transform.rotation[2]
        );
        let forward = rotation * Vec3::NEG_Z;
        let target = position + forward;
        
        Mat4::look_at_rh(position, target, Vec3::Y)
    }
    
    fn projection_matrix(&self) -> Mat4 {
        // Use camera's projection matrix
        let aspect = self.camera.calculate_aspect_ratio(1920, 1080); // Default aspect ratio
        let fov_radians = self.camera.fov_degrees.to_radians();
        
        match self.camera.projection_type {
            ProjectionType::Perspective => {
                Mat4::perspective_rh(fov_radians, aspect, self.camera.near_plane, self.camera.far_plane)
            }
            ProjectionType::Orthographic => {
                let half_height = self.camera.orthographic_size * 0.5;
                let half_width = half_height * aspect;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, 
                                     self.camera.near_plane, self.camera.far_plane)
            }
        }
    }
    
    fn position(&self) -> Vec3 {
        Vec3::from_array(self.transform.position)
    }
    
    fn forward(&self) -> Vec3 {
        let rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            self.transform.rotation[0],
            self.transform.rotation[1],
            self.transform.rotation[2]
        );
        rotation * Vec3::NEG_Z
    }
}

/// High-level bridge for rendering ECS worlds
pub struct EcsRenderBridge;

impl EcsRenderBridge {
    /// Extract all render data from ECS world
    pub fn extract_render_data(world: &World) -> (EcsRenderableQuery, Option<EcsCameraProvider>) {
        let renderables = EcsRenderableQuery::from_world(world);
        let camera = EcsCameraProvider::from_world(world);
        (renderables, camera)
    }
    
    /// Render ECS world using renderer-core traits
    pub fn render_world<R>(world: &World, renderer: &mut R) 
    where
        R: FnMut(&EcsRenderableQuery, Option<&dyn CameraProvider>)
    {
        let (renderables, camera) = Self::extract_render_data(world);
        renderer(&renderables, camera.as_ref().map(|c| c as &dyn CameraProvider));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_ecs_core::ecs_v2::World;
    use engine_components_3d::{Transform, Mesh, MeshType, Camera, ProjectionType};
    use glam::{Vec3, Quat};
    
    fn create_test_world() -> World {
        let mut world = World::new();
        
        // Add some test entities
        let entity1 = world.spawn();
        world.add_component(entity1, Transform::new().with_position(1.0, 0.0, 0.0)).unwrap();
        world.add_component(entity1, Mesh { mesh_type: MeshType::Cube }).unwrap();
        
        let entity2 = world.spawn();
        world.add_component(entity2, Transform::new().with_position(2.0, 0.0, 0.0)).unwrap();
        world.add_component(entity2, Mesh { mesh_type: MeshType::Sphere }).unwrap();
        
        // Add a camera entity
        let camera_entity = world.spawn();
        world.add_component(camera_entity, Transform::new().with_position(0.0, 0.0, 5.0)).unwrap();
        world.add_component(camera_entity, Camera::perspective(60.0, 0.1, 1000.0)).unwrap();
        
        world
    }
    
    #[test]
    fn test_ecs_renderable_creation() {
        let entity = Entity::new(42, 0);
        let transform = Transform::new().with_position(1.0, 2.0, 3.0);
        let mesh = Mesh { mesh_type: MeshType::Cube };
        
        let renderable = EcsRenderable::new(entity, transform)
            .with_mesh(mesh)
            .with_material(100);
        
        assert_eq!(renderable.entity, entity);
        assert_eq!(renderable.world_matrix(), Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)));
        assert_eq!(renderable.mesh_handle(), Some(1)); // Cube = 1
        assert_eq!(renderable.material_handle(), Some(100));
        assert!(renderable.is_visible());
    }
    
    #[test]
    fn test_ecs_transform_provider() {
        let transform = Transform::new()
            .with_position(1.0, 2.0, 3.0)
            .with_rotation(0.0, std::f32::consts::PI / 4.0, 0.0)
            .with_scale(2.0, 2.0, 2.0);
        
        let provider = EcsTransformProvider::new(transform);
        
        assert_eq!(provider.position(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(provider.scale(), Vec3::new(2.0, 2.0, 2.0));
        
        // Test that matrix calculation works
        let matrix = provider.world_matrix();
        assert!(!matrix.is_nan());
        assert_ne!(matrix, Mat4::ZERO);
    }
    
    #[test]
    fn test_ecs_renderable_query_from_world() {
        let world = create_test_world();
        let query = EcsRenderableQuery::from_world(&world);
        
        // Should have 3 entities (2 with meshes + 1 camera), but camera might not have mesh
        assert!(query.len() >= 2);
        
        let renderables: Vec<_> = query.iter().collect();
        let with_meshes: Vec<_> = renderables.iter()
            .filter(|r| r.mesh_handle().is_some())
            .collect();
        
        assert_eq!(with_meshes.len(), 2); // 2 entities with meshes
    }
    
    #[test]
    fn test_ecs_camera_provider_from_world() {
        let world = create_test_world();
        let camera_provider = EcsCameraProvider::from_world(&world);
        
        assert!(camera_provider.is_some());
        
        let camera = camera_provider.unwrap();
        assert_eq!(camera.position(), Vec3::new(0.0, 0.0, 5.0));
        
        // Verify matrices are valid
        assert!(!camera.view_matrix().is_nan());
        assert!(!camera.projection_matrix().is_nan());
        assert!(!camera.forward().is_nan());
    }
    
    #[test]
    fn test_ecs_camera_provider_no_camera() {
        let mut world = World::new();
        
        // Add entity without camera
        let entity = world.spawn();
        world.add_component(entity, Transform::new()).unwrap();
        
        let camera_provider = EcsCameraProvider::from_world(&world);
        assert!(camera_provider.is_none());
    }
    
    #[test]
    fn test_ecs_render_bridge_extract() {
        let world = create_test_world();
        let (renderables, camera) = EcsRenderBridge::extract_render_data(&world);
        
        assert!(renderables.len() >= 2);
        assert!(camera.is_some());
        
        // Test that extracted data works with renderer traits
        let _renderable_count = renderables.len();
        let _camera_position = camera.as_ref().unwrap().position();
    }
    
    #[test]
    fn test_ecs_render_bridge_render_world() {
        let world = create_test_world();
        let mut render_calls = 0;
        let mut last_renderable_count = 0;
        let mut camera_received = false;
        
        EcsRenderBridge::render_world(&world, &mut |renderables, camera| {
            render_calls += 1;
            last_renderable_count = renderables.len();
            camera_received = camera.is_some();
        });
        
        assert_eq!(render_calls, 1);
        assert!(last_renderable_count >= 2);
        assert!(camera_received);
    }
    
    #[test]
    fn test_trait_compatibility() {
        let world = create_test_world();
        let (renderables, camera) = EcsRenderBridge::extract_render_data(&world);
        
        // Test that our ECS implementations work as trait objects
        let _renderable_count = renderables.len();
        
        if let Some(camera) = camera {
            let _camera_provider: &dyn CameraProvider = &camera;
            
            // Verify camera trait methods work
            let _view = _camera_provider.view_matrix();
            let _proj = _camera_provider.projection_matrix();
            let _pos = _camera_provider.position();
            let _forward = _camera_provider.forward();
        }
    }
    
    #[test]
    fn test_empty_world() {
        let world = World::new();
        let (renderables, camera) = EcsRenderBridge::extract_render_data(&world);
        
        assert_eq!(renderables.len(), 0);
        assert!(camera.is_none());
    }
    
    #[test]
    fn test_world_with_transforms_only() {
        let mut world = World::new();
        
        // Add entities with only transforms (no meshes or cameras)
        for i in 0..3 {
            let entity = world.spawn();
            world.add_component(entity, Transform::new().with_position(i as f32, 0.0, 0.0)).unwrap();
        }
        
        let (renderables, camera) = EcsRenderBridge::extract_render_data(&world);
        
        assert_eq!(renderables.len(), 3);
        assert!(camera.is_none());
        
        // All renderables should have no mesh handles
        for renderable in renderables.iter() {
            assert!(renderable.mesh_handle().is_none());
        }
    }
}