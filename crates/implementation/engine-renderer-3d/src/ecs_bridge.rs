//! ECS to Renderer Bridge
//! 
//! This module provides the bridge between the ECS world and the 3D renderer,
//! converting ECS entities with 3D components into render objects.

use std::collections::HashMap;
use glam::{Mat4, Vec3};
use engine_ecs_core::ecs_v2::{World, Entity};
use engine_components_3d::{Transform, Material as EcsMaterial, MeshFilter, MeshRenderer, Mesh, MeshType};

use crate::{RenderScene, RenderObject, Camera};

/// Bridge for converting ECS world to render scene
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
    
    /// Convert ECS world to render scene
    pub fn world_to_render_scene(&self, world: &World, camera: Camera) -> RenderScene {
        let mut scene = RenderScene::new(camera);
        
        // Get all entities with Transform components
        let transform_entities: HashMap<Entity, &Transform> = world.query_legacy::<Transform>().collect();
        
        log::info!("Found {} entities with Transform", transform_entities.len());
        
        // Get all entities with Mesh components  
        let mesh_entities: HashMap<Entity, &Mesh> = world.query_legacy::<Mesh>().collect();
        
        log::info!("Found {} entities with Mesh", mesh_entities.len());
        
        // Find entities that have transform and mesh components
        for (entity, transform) in &transform_entities {
            if let Some(mesh) = mesh_entities.get(entity) {
                if let Some(render_object) = self.convert_to_render_object(*entity, transform, mesh) {
                    scene.add_object(render_object);
                }
            }
        }
        
        log::info!("Converted ECS world to render scene with {} objects", scene.objects.len());
        
        // Log details about objects
        for (i, obj) in scene.objects.iter().enumerate() {
            log::info!("Object {}: mesh_id={}, material_id={}, transform={:?}", 
                i, obj.mesh_id, obj.material_id, obj.transform);
        }
        
        scene
    }
    
    /// Convert individual ECS entity to render object
    fn convert_to_render_object(
        &self,
        _entity: Entity,
        transform: &Transform,
        mesh: &Mesh,
    ) -> Option<RenderObject> {
        // Map mesh type to mesh ID
        let mesh_id = match &mesh.mesh_type {
            MeshType::Cube => self.mesh_name_to_id.get("cube").copied().unwrap_or(self.default_mesh_id),
            MeshType::Sphere => self.mesh_name_to_id.get("sphere").copied().unwrap_or(self.default_mesh_id),
            MeshType::Plane => self.mesh_name_to_id.get("plane").copied().unwrap_or(self.default_mesh_id),
            MeshType::Custom(name) => self.mesh_name_to_id.get(name).copied().unwrap_or_else(|| {
                log::warn!("Custom mesh '{}' not found, using default", name);
                self.default_mesh_id
            }),
        };
        
        // For now, use a default material mapping based on color
        // In a full implementation, this would be more sophisticated
        let material_id = self.default_material_id;
        
        // Convert transform to matrix
        let transform_matrix = self.transform_to_matrix(transform);
        
        Some(RenderObject::new(transform_matrix, mesh_id, material_id))
    }
    
    /// Convert Transform component to transformation matrix
    fn transform_to_matrix(&self, transform: &Transform) -> Mat4 {
        // Log transform values for debugging
        log::debug!("Converting transform to matrix: pos={:?}, rot={:?}, scale={:?}", 
            transform.position, transform.rotation, transform.scale);
        
        // Use the built-in matrix method from the Transform component
        let matrix = transform.matrix();
        log::debug!("Resulting matrix: {:?}", matrix);
        matrix
    }
    
    /// Update existing render scene from ECS world (more efficient than full rebuild)
    pub fn update_render_scene(&self, world: &World, scene: &mut RenderScene) {
        // Clear existing objects
        scene.clear_objects();
        
        // Get all entities with required components
        let transform_entities: HashMap<Entity, &Transform> = world.query_legacy::<Transform>().collect();
        let mesh_entities: HashMap<Entity, &Mesh> = world.query_legacy::<Mesh>().collect();
        let material_entities: HashMap<Entity, &EcsMaterial> = world.query_legacy::<EcsMaterial>().collect();
        
        // Find entities that have transform and mesh components and add to scene
        for (entity, transform) in &transform_entities {
            if let Some(mesh) = mesh_entities.get(entity) {
                if let Some(render_object) = self.convert_to_render_object(*entity, transform, mesh) {
                    scene.add_object(render_object);
                }
            }
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
        // Find the main camera or highest priority active camera
        let camera_entity = engine_camera_impl::find_main_camera(world)
            .or_else(|| engine_camera_impl::find_active_camera(world));
        
        if let Some(entity) = camera_entity {
            // Get transform and camera components
            if let (Some(transform), Some(cam)) = (
                world.get_component::<engine_components_3d::Transform>(entity),
                world.get_component::<engine_components_3d::Camera>(entity)
            ) {
                log::info!("Found ECS camera at position: {:?}, rotation: {:?}", 
                    transform.position, transform.rotation);
                // Convert ECS components to renderer Camera
                let camera = Camera::from_position_rotation(
                    transform.position,
                    transform.rotation,
                    cam.calculate_aspect_ratio(800, 600), // TODO: get actual viewport size
                );
                log::info!("Camera target: {:?}", camera.target);
                return camera;
            }
        }
        
        // Default camera fallback
        let mut camera = Camera::new(aspect_ratio);
        camera.position = Vec3::new(0.0, 2.0, 5.0);
        camera.target = Vec3::ZERO;
        
        log::debug!("Using default camera (no camera entity found in ECS world)");
        camera
    }
    
    /// Extract camera with custom positioning
    pub fn extract_camera_with_position(
        world: &World,
        aspect_ratio: f32,
        position: Vec3,
        target: Vec3,
    ) -> Camera {
        // TODO: In the future, respect ECS camera entities but override position
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
            self.bridge.add_mesh_mapping("triangle".to_string(), triangle_id);
        }
        if let Some(cube_id) = renderer.get_default_mesh_id("cube") {
            self.bridge.add_mesh_mapping("cube".to_string(), cube_id);
        }
        
        // Add default material mappings
        if let Some(default_id) = renderer.get_default_material_id("default") {
            self.bridge.add_material_mapping("default".to_string(), default_id);
        }
        if let Some(red_id) = renderer.get_default_material_id("red") {
            self.bridge.add_material_mapping("red".to_string(), red_id);
        }
        if let Some(green_id) = renderer.get_default_material_id("green") {
            self.bridge.add_material_mapping("green".to_string(), green_id);
        }
        if let Some(blue_id) = renderer.get_default_material_id("blue") {
            self.bridge.add_material_mapping("blue".to_string(), blue_id);
        }
        
        log::info!("Setup default ECS-to-renderer mappings");
    }
    
    /// Convert ECS world to renderable scene
    pub fn world_to_scene(&self, world: &World, aspect_ratio: f32) -> RenderScene {
        let camera = CameraExtractor::extract_camera(world, aspect_ratio);
        self.bridge.world_to_render_scene(world, camera)
    }
    
    /// Convert ECS world to scene with custom camera
    pub fn world_to_scene_with_camera(&self, world: &World, camera: Camera) -> RenderScene {
        self.bridge.world_to_render_scene(world, camera)
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
    
    #[test]
    fn test_transform_to_matrix() {
        let bridge = EcsRenderBridge::new(HashMap::new(), HashMap::new(), 0, 0);
        
        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        
        let matrix = bridge.transform_to_matrix(&transform);
        
        // The matrix should have the translation component
        assert_eq!(matrix.col(3).xyz(), Vec3::new(1.0, 2.0, 3.0));
    }
    
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
}