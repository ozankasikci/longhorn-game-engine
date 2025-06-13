//! Scene representation for the renderer
//! 
//! This module provides the intermediate representation between the ECS world
//! and the renderer, allowing for clean separation of concerns.

use glam::{Mat4, Vec3};
use crate::camera::Camera;

/// A complete scene ready for rendering
#[derive(Debug, Clone)]
pub struct RenderScene {
    pub camera: Camera,
    pub objects: Vec<RenderObject>,
    pub clear_color: [f32; 4],
}

impl RenderScene {
    /// Create a new empty render scene
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            objects: Vec::new(),
            clear_color: [0.1, 0.1, 0.2, 1.0], // Dark blue background
        }
    }
    
    /// Add a render object to the scene
    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }
    
    /// Clear all objects from the scene
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }
}

/// A single object to be rendered
#[derive(Debug, Clone)]
pub struct RenderObject {
    pub transform: Mat4,
    pub mesh_id: u32,
    pub material_id: u32,
}

impl RenderObject {
    /// Create a new render object
    pub fn new(transform: Mat4, mesh_id: u32, material_id: u32) -> Self {
        Self {
            transform,
            mesh_id,
            material_id,
        }
    }
}