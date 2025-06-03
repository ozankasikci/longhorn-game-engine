//! Advanced camera system for mobile-first game engine
//! 
//! This crate provides sophisticated camera management with viewport control,
//! efficient culling, and mobile-optimized rendering capabilities.

pub mod camera;
pub mod viewport;
pub mod projection;
pub mod culling;
pub mod components;

// Core exports
pub use camera::{Camera as AdvancedCamera, CameraType, CameraComponent, CameraUniform};
pub use viewport::{Viewport, ViewportTransform};
pub use projection::{ProjectionMatrix, OrthographicProjection, PerspectiveProjection};
pub use culling::*;

// ECS Component exports
pub use components::{Camera, Camera2D};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("Invalid viewport dimensions: width={0}, height={1}")]
    InvalidViewport(u32, u32),
    
    #[error("Invalid projection parameters: {0}")]
    InvalidProjection(String),
    
    #[error("Matrix calculation failed: {0}")]
    MatrixCalculationFailed(String),
    
    #[error("Culling operation failed: {0}")]
    CullingFailed(String),
}

pub type Result<T> = std::result::Result<T, CameraError>;

// Bundle support
use engine_ecs_core::{Bundle, Entity, World, ArchetypeId, ComponentTicks};
use engine_components_3d::Transform;
use engine_components_ui::Name;

/// Bundle for camera entities
pub struct CameraBundle {
    pub transform: Transform,
    pub camera: Camera,
    pub name: Name,
}

impl Bundle for CameraBundle {
    fn insert(self, entity: Entity, world: &mut World) -> std::result::Result<(), &'static str> {
        // Create archetype ID with all components
        let archetype_id = ArchetypeId::new()
            .with_component::<Transform>()
            .with_component::<Camera>()
            .with_component::<Name>();
            
        // Add entity to archetype
        let _index = world.add_entity_to_archetype(entity, archetype_id.clone());
        
        // Get tick before borrowing archetype
        let tick = world.change_tick();
        
        // Get the archetype and add all components
        let archetype = world.archetypes_mut().get_mut(&archetype_id)
            .ok_or("Failed to get archetype")?;
            
        archetype.add_component(self.transform, ComponentTicks::new(tick));
        archetype.add_component(self.camera, ComponentTicks::new(tick));
        archetype.add_component(self.name, ComponentTicks::new(tick));
        
        Ok(())
    }
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            camera: Camera::default(),
            name: Name::new("Camera"),
        }
    }
}