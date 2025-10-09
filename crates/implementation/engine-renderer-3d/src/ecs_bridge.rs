//! ECS to Renderer Bridge - Trait-based implementation
//!
//! This module provides the bridge between the ECS world and the 3D renderer,
//! using the decoupled trait abstractions from engine-renderer-core.

use engine_components_3d::{Mesh, MeshType, Transform};
use engine_ecs_core::ecs_v2::{Entity, World};
use engine_renderer_core::{CameraProvider, Renderable, RenderableQuery};
use engine_renderer_ecs_bridge::EcsCameraProvider;
use glam::Vec3;
use std::collections::HashMap;

use crate::{Camera, RenderObject, RenderScene};

/// Bridge for converting ECS world to render scene using trait abstractions
pub struct EcsRenderBridge {
    /// Mapping from mesh names to renderer mesh IDs
    mesh_name_to_id: HashMap<String, u32>,
    /// Mapping from material names to renderer material IDs  
    material_name_to_id: HashMap<String, u32>,
    /// Default mesh ID for fallback
    default_mesh_id: u32,
    /// Default material ID for fallback
    default_material_id: u32,
}

impl EcsRenderBridge {
    /// Create a new ECS render bridge
    pub fn new(
        mesh_mappings: HashMap<String, u32>,
        material_mappings: HashMap<String, u32>,
        default_mesh_id: u32,
        default_material_id: u32,
    ) -> Self {
        Self {
            mesh_name_to_id: mesh_mappings,
            material_name_to_id: material_mappings,
            default_mesh_id,
            default_material_id,
        }
    }

    /// Add mesh name mapping
    pub fn add_mesh_mapping(&mut self, name: String, mesh_id: u32) {
        self.mesh_name_to_id.insert(name, mesh_id);
    }

    /// Add material name mapping
    pub fn add_material_mapping(&mut self, name: String, material_id: u32) {
        self.material_name_to_id.insert(name, material_id);
    }

    /// Convert ECS world to render scene using trait abstractions
    pub fn world_to_render_scene(
        &self,
        world: &World,
        camera: Camera,
        selected_entity: Option<Entity>,
    ) -> RenderScene {
        // Extract renderables using trait-based query
        let mut scene = RenderScene::new(camera);

        // Build custom query that includes our mesh mapping
        let renderables = self.extract_renderables_with_mapping(world, selected_entity);

        log::info!("Extracted {} renderables from ECS world", renderables.len());

        // Convert to render objects
        for renderable in renderables {
            scene.add_object(renderable);
        }

        log::info!(
            "Converted ECS world to render scene with {} objects",
            scene.objects.len()
        );

        scene
    }

    /// Extract renderables with proper mesh/material mapping
    fn extract_renderables_with_mapping(
        &self,
        world: &World,
        selected_entity: Option<Entity>,
    ) -> Vec<RenderObject> {
        let mut render_objects = Vec::new();

        // Query all entities with Transform components
        let transform_entities: HashMap<Entity, &Transform> =
            world.query_legacy::<Transform>().collect();

        // Query all entities with Mesh components
        let mesh_entities: HashMap<Entity, &Mesh> = world.query_legacy::<Mesh>().collect();
        

        // Find entities that have both transform and mesh
        for (entity, transform) in &transform_entities {
            if let Some(mesh) = mesh_entities.get(entity) {
                // Map mesh type to mesh ID using our mappings
                let mesh_id = self.map_mesh_to_id(&mesh.mesh_type);

                // For now, use default material
                let material_id = self.default_material_id;

                // Create render object
                let transform_matrix = transform.matrix();
                
                let mut render_object = RenderObject::new(transform_matrix, mesh_id, material_id);

                // Set selection state
                render_object.is_selected = selected_entity == Some(*entity);

                render_objects.push(render_object);
            }
        }

        render_objects
    }

    /// Map mesh type to renderer mesh ID
    fn map_mesh_to_id(&self, mesh_type: &MeshType) -> u32 {
        match mesh_type {
            MeshType::Cube => self
                .mesh_name_to_id
                .get("cube")
                .copied()
                .unwrap_or(self.default_mesh_id),
            MeshType::Sphere => self
                .mesh_name_to_id
                .get("sphere")
                .copied()
                .unwrap_or(self.default_mesh_id),
            MeshType::Plane => self
                .mesh_name_to_id
                .get("plane")
                .copied()
                .unwrap_or(self.default_mesh_id),
            MeshType::Custom(name) => {
                self.mesh_name_to_id.get(name).copied().unwrap_or_else(|| {
                    log::warn!("Custom mesh '{}' not found, using default", name);
                    self.default_mesh_id
                })
            }
        }
    }

    /// Update existing render scene from ECS world (more efficient than full rebuild)
    pub fn update_render_scene(&self, world: &World, scene: &mut RenderScene) {
        // Clear existing objects
        scene.clear_objects();

        // Extract and add new objects
        let renderables = self.extract_renderables_with_mapping(world, None);
        for renderable in renderables {
            scene.add_object(renderable);
        }

        log::info!("Updated render scene with {} objects", scene.objects.len());
    }

    /// Get statistics about the bridge mappings
    pub fn get_mapping_stats(&self) -> MappingStats {
        MappingStats {
            mesh_mappings: self.mesh_name_to_id.len(),
            material_mappings: self.material_name_to_id.len(),
            default_mesh_id: self.default_mesh_id,
            default_material_id: self.default_material_id,
        }
    }
}

/// Alternative implementation using pure trait abstractions
/// This demonstrates how the renderer could work with any RenderableQuery implementation
pub struct TraitBasedRenderBridge;

impl TraitBasedRenderBridge {
    /// Render using trait abstractions - completely decoupled from ECS
    pub fn render_with_traits<Q, C>(
        query: &Q,
        camera_provider: Option<&C>,
        default_camera: Camera,
    ) -> RenderScene
    where
        Q: RenderableQuery,
        C: CameraProvider,
    {
        // Use provided camera or default
        let camera = if let Some(provider) = camera_provider {
            // Convert trait-based camera to our Camera type
            Camera {
                position: provider.position(),
                target: provider.position() + provider.forward(),
                up: Vec3::Y,
                fov: 60.0, // Default FOV
                aspect: default_camera.aspect,
                near: 0.1,
                far: 1000.0,
                is_main: default_camera.is_main,
            }
        } else {
            default_camera
        };

        let mut scene = RenderScene::new(camera);

        // Convert renderables to render objects
        for renderable in query.iter() {
            if let (Some(mesh_id), Some(material_id)) =
                (renderable.mesh_handle(), renderable.material_handle())
            {
                let render_object =
                    RenderObject::new(renderable.world_matrix(), mesh_id, material_id);
                scene.add_object(render_object);
            }
        }

        scene
    }
}

/// Statistics about ECS bridge mappings
#[derive(Debug, Clone)]
pub struct MappingStats {
    pub mesh_mappings: usize,
    pub material_mappings: usize,
    pub default_mesh_id: u32,
    pub default_material_id: u32,
}

impl std::fmt::Display for MappingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "MappingStats {{ mesh_mappings: {}, material_mappings: {}, defaults: mesh={}, material={} }}",
            self.mesh_mappings,
            self.material_mappings,
            self.default_mesh_id,
            self.default_material_id
        )
    }
}

/// Camera extraction from ECS world
pub struct CameraExtractor;

impl CameraExtractor {
    /// Extract camera from ECS world (looks for camera entities)
    pub fn extract_camera(world: &World, aspect_ratio: f32) -> Camera {
        // Use trait-based extraction
        if let Some(camera_provider) = EcsCameraProvider::from_world(world) {
            Camera {
                position: camera_provider.position(),
                target: camera_provider.position() + camera_provider.forward(),
                up: Vec3::Y,
                fov: 60.0,
                aspect: aspect_ratio,
                near: 0.1,
                far: 1000.0,
                is_main: false,
            }
        } else {
            // Default camera fallback
            let mut camera = Camera::new(aspect_ratio);
            camera.position = Vec3::new(0.0, 2.0, 5.0);
            camera.target = Vec3::ZERO;

            log::debug!("Using default camera (no camera entity found in ECS world)");
            camera
        }
    }

    /// Extract camera with custom positioning
    pub fn extract_camera_with_position(
        world: &World,
        aspect_ratio: f32,
        position: Vec3,
        target: Vec3,
    ) -> Camera {
        let mut camera = Self::extract_camera(world, aspect_ratio);
        camera.position = position;
        camera.target = target;
        camera
    }
}

/// High-level ECS to renderer integration
pub struct EcsRendererIntegration {
    bridge: EcsRenderBridge,
}

impl EcsRendererIntegration {
    /// Create new integration with default mappings
    pub fn new(default_mesh_id: u32, default_material_id: u32) -> Self {
        let bridge = EcsRenderBridge::new(
            HashMap::new(),
            HashMap::new(),
            default_mesh_id,
            default_material_id,
        );

        Self { bridge }
    }

    /// Create integration with pre-populated mappings
    pub fn with_mappings(
        mesh_mappings: HashMap<String, u32>,
        material_mappings: HashMap<String, u32>,
        default_mesh_id: u32,
        default_material_id: u32,
    ) -> Self {
        let bridge = EcsRenderBridge::new(
            mesh_mappings,
            material_mappings,
            default_mesh_id,
            default_material_id,
        );

        Self { bridge }
    }

    /// Setup default mappings from renderer
    pub fn setup_default_mappings(&mut self, renderer: &crate::Renderer3D) {
        // Add default mesh mappings
        if let Some(triangle_id) = renderer.get_default_mesh_id("triangle") {
            self.bridge
                .add_mesh_mapping("triangle".to_string(), triangle_id);
        }
        if let Some(cube_id) = renderer.get_default_mesh_id("cube") {
            self.bridge.add_mesh_mapping("cube".to_string(), cube_id);
        }
        if let Some(sphere_id) = renderer.get_default_mesh_id("sphere") {
            self.bridge
                .add_mesh_mapping("sphere".to_string(), sphere_id);
        }
        if let Some(plane_id) = renderer.get_default_mesh_id("plane") {
            self.bridge.add_mesh_mapping("plane".to_string(), plane_id);
        }

        // Add default material mappings
        if let Some(default_id) = renderer.get_default_material_id("default") {
            self.bridge
                .add_material_mapping("default".to_string(), default_id);
        }
        if let Some(red_id) = renderer.get_default_material_id("red") {
            self.bridge.add_material_mapping("red".to_string(), red_id);
        }
        if let Some(green_id) = renderer.get_default_material_id("green") {
            self.bridge
                .add_material_mapping("green".to_string(), green_id);
        }
        if let Some(blue_id) = renderer.get_default_material_id("blue") {
            self.bridge
                .add_material_mapping("blue".to_string(), blue_id);
        }

        log::info!("Setup default ECS-to-renderer mappings");
    }

    /// Convert ECS world to renderable scene
    pub fn world_to_scene(&self, world: &World, aspect_ratio: f32) -> RenderScene {
        let camera = CameraExtractor::extract_camera(world, aspect_ratio);
        self.bridge.world_to_render_scene(world, camera, None)
    }

    /// Convert ECS world to scene with custom camera
    pub fn world_to_scene_with_camera(&self, world: &World, camera: Camera) -> RenderScene {
        self.bridge.world_to_render_scene(world, camera, None)
    }

    /// Update existing scene from ECS world
    pub fn update_scene(&self, world: &World, scene: &mut RenderScene) {
        self.bridge.update_render_scene(world, scene)
    }

    /// Add custom mesh mapping
    pub fn add_mesh_mapping(&mut self, name: String, mesh_id: u32) {
        self.bridge.add_mesh_mapping(name, mesh_id);
    }

    /// Add custom material mapping
    pub fn add_material_mapping(&mut self, name: String, material_id: u32) {
        self.bridge.add_material_mapping(name, material_id);
    }

    /// Get integration statistics
    pub fn get_stats(&self) -> MappingStats {
        self.bridge.get_mapping_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_mapping_stats_display() {
        let stats = MappingStats {
            mesh_mappings: 5,
            material_mappings: 3,
            default_mesh_id: 0,
            default_material_id: 1,
        };

        let display = format!("{}", stats);
        assert!(display.contains("mesh_mappings: 5"));
        assert!(display.contains("material_mappings: 3"));
    }

    #[test]
    fn test_trait_based_render_bridge() {
        use engine_renderer_core::Renderable;
        use glam::Mat4;

        // Create a simple mock renderable
        #[derive(Clone)]
        struct MockRenderable {
            transform: Mat4,
            mesh_id: u32,
            material_id: u32,
        }

        impl Renderable for MockRenderable {
            fn world_matrix(&self) -> Mat4 {
                self.transform
            }
            fn mesh_handle(&self) -> Option<u32> {
                Some(self.mesh_id)
            }
            fn material_handle(&self) -> Option<u32> {
                Some(self.material_id)
            }
        }

        // Create mock query
        struct MockQuery {
            items: Vec<MockRenderable>,
        }

        impl RenderableQuery for MockQuery {
            type Item = MockRenderable;
            type Iter = std::vec::IntoIter<MockRenderable>;

            fn iter(&self) -> Self::Iter {
                self.items.clone().into_iter()
            }

            fn len(&self) -> usize {
                self.items.len()
            }
        }

        // Test rendering with traits
        let query = MockQuery {
            items: vec![MockRenderable {
                transform: Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                mesh_id: 1,
                material_id: 10,
            }],
        };

        let default_camera = Camera::new(1.0);
        let scene = TraitBasedRenderBridge::render_with_traits(
            &query,
            None::<&EcsCameraProvider>,
            default_camera,
        );

        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].mesh_id, 1);
        assert_eq!(scene.objects[0].material_id, 10);
    }
}
