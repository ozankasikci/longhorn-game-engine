//! Scene management and hierarchical structures

pub mod node;
pub mod scene;
pub mod transform;
pub mod camera;
pub mod light;

pub use node::{SceneNode, NodeId, NodeHierarchy};
pub use scene::{Scene, SceneHandle, SceneManager, SceneMetadata};
pub use transform::{Transform, TransformMatrix};
pub use camera::{Camera, CameraProjection, CameraView, Viewport, Ray};
pub use light::{Light, LightType, DirectionalLight, PointLight, SpotLight, AreaLight};

use thiserror::Error;

/// Scene system errors
#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),
    
    #[error("Invalid parent-child relationship")]
    InvalidHierarchy,
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Transform calculation failed: {0}")]
    TransformError(String),
    
    #[error("Scene not loaded: {0:?}")]
    SceneNotLoaded(SceneHandle),
}