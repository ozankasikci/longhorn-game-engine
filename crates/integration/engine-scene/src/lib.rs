//! Scene management and hierarchical structures

pub mod light;
pub mod node;
pub mod scene;
pub mod transform;

pub use light::{AreaLight, DirectionalLight, Light, LightType, PointLight, SpotLight};
pub use node::{NodeHierarchy, NodeId, SceneNode};
pub use scene::{Scene, SceneHandle, SceneManager, SceneMetadata};
pub use transform::TransformMatrix;

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
