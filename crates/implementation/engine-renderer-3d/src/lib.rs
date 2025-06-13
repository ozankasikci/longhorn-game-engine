//! Standalone 3D Renderer for Longhorn Game Engine
//! 
//! This crate provides a dedicated 3D renderer built on wgpu that renders to texture
//! for integration with egui-based editors. It follows patterns from successful
//! Rust game engines like Bevy and rend3.

pub mod renderer;
pub mod mesh;
pub mod material;
pub mod camera;
pub mod camera_advanced;
pub mod scene;
pub mod resources;
pub mod texture;
pub mod ecs_bridge;
pub mod integration;
pub mod render_queue;
pub mod culling;

// Re-export main types
pub use renderer::{Renderer3D, Vertex, CameraUniform};
pub use scene::{RenderScene, RenderObject};
pub use integration::egui::EguiRenderWidget;
pub use camera::Camera;
pub use mesh::Mesh;
pub use material::Material;
pub use resources::{ResourceManager, ResourceStats, MeshResource, MaterialResource};
pub use texture::{TextureManager, TextureResource, TextureDescriptor, create_test_pattern};
pub use ecs_bridge::{EcsRenderBridge, EcsRendererIntegration, CameraExtractor, MappingStats};
pub use render_queue::{RenderQueue, RenderItem, MaterialGroup, SortMode, RenderQueueStats};
pub use culling::{FrustumCuller, Frustum, Plane, BoundingVolume, CullingStats};
pub use camera_advanced::{CameraController, CameraPresets, CameraInfo, Ray};