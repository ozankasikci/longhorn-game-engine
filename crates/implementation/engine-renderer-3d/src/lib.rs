//! Standalone 3D Renderer for Longhorn Game Engine
//!
//! This crate provides a dedicated 3D renderer built on wgpu that renders to texture
//! for integration with egui-based editors. It follows patterns from successful
//! Rust game engines like Bevy and rend3.

pub mod camera;
pub mod camera_advanced;
pub mod culling;
pub mod ecs_bridge;
pub mod gizmo_3d;
pub mod grid;
pub mod integration;
pub mod material;
pub mod mesh;
pub mod render_queue;
pub mod renderer;
pub mod resources;
pub mod scene;
pub mod texture;

// Re-export main types
pub use camera::Camera;
pub use camera_advanced::{CameraController, CameraInfo, CameraPresets, Ray};
pub use culling::{BoundingVolume, CullingStats, Frustum, FrustumCuller, Plane};
pub use ecs_bridge::{CameraExtractor, EcsRenderBridge, EcsRendererIntegration, MappingStats};
pub use gizmo_3d::{GizmoComponent, GizmoMode, GizmoRenderer3D};
pub use grid::{GridConfig, GridRenderer};
pub use integration::egui::EguiRenderWidget;
pub use material::Material;
pub use mesh::Mesh;
pub use render_queue::{MaterialGroup, RenderItem, RenderQueue, RenderQueueStats, SortMode};
pub use renderer::{CameraUniform, Renderer3D, Vertex};
pub use resources::{MaterialResource, MeshResource, ResourceManager, ResourceStats};
pub use scene::{RenderObject, RenderScene};
pub use texture::{create_test_pattern, TextureDescriptor, TextureManager, TextureResource};
