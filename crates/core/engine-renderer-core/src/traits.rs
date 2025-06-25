use glam::{Mat4, Quat, Vec3};

/// Trait for objects that can be rendered
pub trait Renderable {
    /// Get the world transform matrix
    fn world_matrix(&self) -> Mat4;

    /// Get the mesh handle if available
    fn mesh_handle(&self) -> Option<u32>; // Using u32 instead of MeshHandle for now

    /// Get the material handle if available  
    fn material_handle(&self) -> Option<u32>; // Using u32 instead of MaterialHandle for now

    /// Check if the object is visible
    fn is_visible(&self) -> bool {
        true // Default implementation
    }
}

/// Trait for providing transform data without ECS coupling
pub trait TransformProvider {
    /// Get the world transformation matrix
    fn world_matrix(&self) -> Mat4;

    /// Get the position component
    fn position(&self) -> Vec3;

    /// Get the rotation component
    fn rotation(&self) -> Quat;

    /// Get the scale component
    fn scale(&self) -> Vec3;
}

/// Trait for querying renderable objects without ECS coupling
pub trait RenderableQuery {
    type Item: Renderable;
    type Iter: Iterator<Item = Self::Item>;

    /// Get an iterator over all renderable objects
    fn iter(&self) -> Self::Iter;

    /// Get the count of renderable objects
    fn len(&self) -> usize;

    /// Check if the query is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Trait for camera providers without ECS coupling
pub trait CameraProvider {
    /// Get the view matrix
    fn view_matrix(&self) -> Mat4;

    /// Get the projection matrix
    fn projection_matrix(&self) -> Mat4;

    /// Get the camera position
    fn position(&self) -> Vec3;

    /// Get the camera forward direction
    fn forward(&self) -> Vec3;
}

/// Simple implementation of Renderable for testing
#[derive(Debug, Clone)]
pub struct SimpleRenderable {
    pub transform: Mat4,
    pub mesh_handle: Option<u32>,
    pub material_handle: Option<u32>,
    pub visible: bool,
}

impl SimpleRenderable {
    pub fn new(transform: Mat4) -> Self {
        Self {
            transform,
            mesh_handle: None,
            material_handle: None,
            visible: true,
        }
    }

    pub fn with_mesh(mut self, handle: u32) -> Self {
        self.mesh_handle = Some(handle);
        self
    }

    pub fn with_material(mut self, handle: u32) -> Self {
        self.material_handle = Some(handle);
        self
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl Renderable for SimpleRenderable {
    fn world_matrix(&self) -> Mat4 {
        self.transform
    }

    fn mesh_handle(&self) -> Option<u32> {
        self.mesh_handle
    }

    fn material_handle(&self) -> Option<u32> {
        self.material_handle
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

/// Simple implementation of TransformProvider for testing
#[derive(Debug, Clone)]
pub struct SimpleTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl SimpleTransform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl TransformProvider for SimpleTransform {
    fn world_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn rotation(&self) -> Quat {
        self.rotation
    }

    fn scale(&self) -> Vec3 {
        self.scale
    }
}

/// Simple implementation of RenderableQuery for testing
#[derive(Debug, Clone)]
pub struct SimpleRenderableQuery {
    items: Vec<SimpleRenderable>,
}

impl SimpleRenderableQuery {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(items: Vec<SimpleRenderable>) -> Self {
        Self { items }
    }

    pub fn add_item(&mut self, item: SimpleRenderable) {
        self.items.push(item);
    }
}

impl Default for SimpleRenderableQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderableQuery for SimpleRenderableQuery {
    type Item = SimpleRenderable;
    type Iter = std::vec::IntoIter<SimpleRenderable>;

    fn iter(&self) -> Self::Iter {
        self.items.clone().into_iter()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

/// Simple implementation of CameraProvider for testing
#[derive(Debug, Clone)]
pub struct SimpleCamera {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub position: Vec3,
    pub forward: Vec3,
}

impl SimpleCamera {
    pub fn new(position: Vec3, target: Vec3, projection: Mat4) -> Self {
        let direction = target - position;
        let forward = if direction.length() > 0.0001 {
            direction.normalize()
        } else {
            Vec3::NEG_Z // Default forward direction
        };

        let view_matrix = if direction.length() > 0.0001 {
            Mat4::look_at_rh(position, target, Vec3::Y)
        } else {
            // If position equals target, create identity view matrix
            Mat4::IDENTITY
        };

        Self {
            view_matrix,
            projection_matrix: projection,
            position,
            forward,
        }
    }

    pub fn perspective(
        position: Vec3,
        target: Vec3,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let projection = Mat4::perspective_rh(fov, aspect, near, far);
        Self::new(position, target, projection)
    }
}

impl CameraProvider for SimpleCamera {
    fn view_matrix(&self) -> Mat4 {
        self.view_matrix
    }

    fn projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn forward(&self) -> Vec3 {
        self.forward
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Quat, Vec3};

    #[test]
    fn test_simple_renderable() {
        let transform = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let renderable = SimpleRenderable::new(transform)
            .with_mesh(42)
            .with_material(100)
            .with_visibility(true);

        assert_eq!(renderable.world_matrix(), transform);
        assert_eq!(renderable.mesh_handle(), Some(42));
        assert_eq!(renderable.material_handle(), Some(100));
        assert!(renderable.is_visible());
    }

    #[test]
    fn test_simple_transform() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let rotation = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
        let scale = Vec3::new(2.0, 2.0, 2.0);

        let transform = SimpleTransform::new(position, rotation, scale);

        assert_eq!(transform.position(), position);
        assert_eq!(transform.rotation(), rotation);
        assert_eq!(transform.scale(), scale);

        let expected_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);
        assert_eq!(transform.world_matrix(), expected_matrix);
    }

    #[test]
    fn test_simple_transform_identity() {
        let transform = SimpleTransform::identity();

        assert_eq!(transform.position(), Vec3::ZERO);
        assert_eq!(transform.rotation(), Quat::IDENTITY);
        assert_eq!(transform.scale(), Vec3::ONE);
        assert_eq!(transform.world_matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn test_simple_renderable_query() {
        let mut query = SimpleRenderableQuery::new();
        assert_eq!(query.len(), 0);
        assert!(query.is_empty());

        let renderable1 = SimpleRenderable::new(Mat4::IDENTITY).with_mesh(1);
        let renderable2 = SimpleRenderable::new(Mat4::from_translation(Vec3::X)).with_mesh(2);

        query.add_item(renderable1.clone());
        query.add_item(renderable2.clone());

        assert_eq!(query.len(), 2);
        assert!(!query.is_empty());

        let items: Vec<_> = query.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].mesh_handle(), Some(1));
        assert_eq!(items[1].mesh_handle(), Some(2));
    }

    #[test]
    fn test_simple_renderable_query_with_items() {
        let items = vec![
            SimpleRenderable::new(Mat4::IDENTITY).with_mesh(10),
            SimpleRenderable::new(Mat4::from_translation(Vec3::Y)).with_mesh(20),
            SimpleRenderable::new(Mat4::from_translation(Vec3::Z)).with_mesh(30),
        ];

        let query = SimpleRenderableQuery::with_items(items.clone());
        assert_eq!(query.len(), 3);

        let collected: Vec<_> = query.iter().collect();
        assert_eq!(collected.len(), 3);
        for (i, item) in collected.iter().enumerate() {
            assert_eq!(item.mesh_handle(), Some((i as u32 + 1) * 10));
        }
    }

    #[test]
    fn test_simple_camera() {
        let position = Vec3::new(0.0, 0.0, 5.0);
        let target = Vec3::ZERO;
        let fov = std::f32::consts::PI / 4.0;
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        let camera = SimpleCamera::perspective(position, target, fov, aspect, near, far);

        assert_eq!(camera.position(), position);
        assert!((camera.forward() - Vec3::NEG_Z).length() < 0.001); // Looking towards -Z

        // Check that matrices are properly set
        let view = camera.view_matrix();
        let proj = camera.projection_matrix();

        // Basic sanity checks - matrices should not be zero or identity
        assert_ne!(view, Mat4::ZERO);
        assert_ne!(view, Mat4::IDENTITY);
        assert_ne!(proj, Mat4::ZERO);
        assert_ne!(proj, Mat4::IDENTITY);
    }

    #[test]
    fn test_camera_provider_trait() {
        let camera = SimpleCamera::perspective(
            Vec3::new(0.0, 1.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            std::f32::consts::PI / 3.0,
            1.0,
            0.1,
            50.0,
        );

        // Test trait methods
        let _view = camera.view_matrix();
        let _proj = camera.projection_matrix();
        let _pos = camera.position();
        let _forward = camera.forward();

        // Just verify the trait methods are callable
        assert_eq!(camera.position(), Vec3::new(0.0, 1.0, 5.0));
    }

    #[test]
    fn test_renderable_visibility() {
        let visible_renderable = SimpleRenderable::new(Mat4::IDENTITY).with_visibility(true);
        let hidden_renderable = SimpleRenderable::new(Mat4::IDENTITY).with_visibility(false);

        assert!(visible_renderable.is_visible());
        assert!(!hidden_renderable.is_visible());
    }

    #[test]
    fn test_trait_object_compatibility() {
        // Test that our traits can be used as trait objects
        let renderable = SimpleRenderable::new(Mat4::IDENTITY).with_mesh(42);
        let transform = SimpleTransform::identity();
        let camera = SimpleCamera::perspective(Vec3::Z, Vec3::ZERO, 1.0, 1.0, 0.1, 10.0);

        // Test trait object usage
        let _renderable_ref: &dyn Renderable = &renderable;
        let _transform_ref: &dyn TransformProvider = &transform;
        let _camera_ref: &dyn CameraProvider = &camera;

        // Verify they work through trait objects
        assert_eq!(_renderable_ref.mesh_handle(), Some(42));
        assert_eq!(_transform_ref.position(), Vec3::ZERO);
        assert_eq!(_camera_ref.position(), Vec3::Z);
    }
}
