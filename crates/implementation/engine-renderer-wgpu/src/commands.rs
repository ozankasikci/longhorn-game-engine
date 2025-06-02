//! Render commands and draw calls

use crate::{Handle, Viewport};
use glam::Mat4;
use serde::{Serialize, Deserialize};

/// Render command for batched rendering
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Begin a render pass
    BeginRenderPass {
        clear_color: Option<[f32; 4]>,
        clear_depth: Option<f32>,
        clear_stencil: Option<u32>,
    },
    
    /// End the current render pass
    EndRenderPass,
    
    /// Set viewport
    SetViewport {
        viewport: Viewport,
    },
    
    /// Set scissor test rectangle
    SetScissor {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    
    /// Bind a render pipeline
    BindPipeline {
        pipeline: Handle,
    },
    
    /// Bind vertex buffer
    BindVertexBuffer {
        slot: u32,
        buffer: Handle,
        offset: u64,
    },
    
    /// Bind index buffer
    BindIndexBuffer {
        buffer: Handle,
        format: IndexFormat,
        offset: u64,
    },
    
    /// Bind uniform buffer
    BindUniformBuffer {
        binding: u32,
        buffer: Handle,
        offset: u64,
        size: u64,
    },
    
    /// Bind texture
    BindTexture {
        binding: u32,
        texture: Handle,
        sampler: Option<Handle>,
    },
    
    /// Draw indexed primitives
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    },
    
    /// Draw non-indexed primitives
    Draw {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    
    /// Draw with indirect parameters
    DrawIndirect {
        buffer: Handle,
        offset: u64,
        draw_count: u32,
        stride: u32,
    },
    
    /// Dispatch compute shader
    Dispatch {
        workgroup_count_x: u32,
        workgroup_count_y: u32,
        workgroup_count_z: u32,
    },
    
    /// Insert debug marker
    DebugMarker {
        label: String,
    },
    
    /// Push debug group
    PushDebugGroup {
        label: String,
    },
    
    /// Pop debug group
    PopDebugGroup,
}

/// High-level draw call representation
#[derive(Debug, Clone)]
pub struct DrawCall {
    /// Mesh to draw
    pub mesh: Handle,
    /// Material to use
    pub material: Handle,
    /// World transform matrix
    pub transform: Mat4,
    /// Instance data buffer (for instanced rendering)
    pub instance_buffer: Option<Handle>,
    /// Number of instances
    pub instance_count: u32,
    /// Render layer/priority
    pub layer: u32,
    /// Distance from camera (for sorting)
    pub distance: f32,
}

/// Index buffer format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

/// Render queue for sorting draw calls
#[derive(Debug, Clone, Default)]
pub struct RenderQueue {
    /// Opaque objects (front-to-back)
    pub opaque: Vec<DrawCall>,
    /// Transparent objects (back-to-front)
    pub transparent: Vec<DrawCall>,
    /// UI elements (in order)
    pub ui: Vec<DrawCall>,
    /// Debug/gizmo objects (last)
    pub debug: Vec<DrawCall>,
}

/// Render layers for organizing draw calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderLayer {
    Background = 0,
    Opaque = 100,
    AlphaTest = 200,
    Transparent = 300,
    UI = 400,
    Overlay = 500,
    Debug = 600,
}

impl RenderCommand {
    /// Check if this command requires a specific render pass
    pub fn requires_render_pass(&self) -> bool {
        matches!(
            self,
            RenderCommand::DrawIndexed { .. }
                | RenderCommand::Draw { .. }
                | RenderCommand::DrawIndirect { .. }
        )
    }
    
    /// Check if this command changes render state
    pub fn changes_state(&self) -> bool {
        matches!(
            self,
            RenderCommand::SetViewport { .. }
                | RenderCommand::SetScissor { .. }
                | RenderCommand::BindPipeline { .. }
                | RenderCommand::BindVertexBuffer { .. }
                | RenderCommand::BindIndexBuffer { .. }
                | RenderCommand::BindUniformBuffer { .. }
                | RenderCommand::BindTexture { .. }
        )
    }
    
    /// Get debug label if available
    pub fn debug_label(&self) -> Option<&str> {
        match self {
            RenderCommand::DebugMarker { label } | RenderCommand::PushDebugGroup { label } => {
                Some(label)
            }
            _ => None,
        }
    }
}

impl DrawCall {
    /// Create a new draw call
    pub fn new(mesh: Handle, material: Handle, transform: Mat4) -> Self {
        Self {
            mesh,
            material,
            transform,
            instance_buffer: None,
            instance_count: 1,
            layer: RenderLayer::Opaque as u32,
            distance: 0.0,
        }
    }
    
    /// Set the render layer
    pub fn with_layer(mut self, layer: RenderLayer) -> Self {
        self.layer = layer as u32;
        self
    }
    
    /// Set the distance from camera
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = distance;
        self
    }
    
    /// Set instance data
    pub fn with_instances(mut self, buffer: Handle, count: u32) -> Self {
        self.instance_buffer = Some(buffer);
        self.instance_count = count;
        self
    }
    
    /// Check if this is an instanced draw call
    pub fn is_instanced(&self) -> bool {
        self.instance_count > 1 || self.instance_buffer.is_some()
    }
}

impl RenderQueue {
    /// Create a new empty render queue
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a draw call to the appropriate queue
    pub fn add_draw_call(&mut self, draw_call: DrawCall) {
        match draw_call.layer {
            layer if layer < RenderLayer::AlphaTest as u32 => {
                self.opaque.push(draw_call);
            }
            layer if layer < RenderLayer::Transparent as u32 => {
                self.opaque.push(draw_call); // Alpha test is still opaque-like
            }
            layer if layer < RenderLayer::UI as u32 => {
                self.transparent.push(draw_call);
            }
            layer if layer < RenderLayer::Debug as u32 => {
                self.ui.push(draw_call);
            }
            _ => {
                self.debug.push(draw_call);
            }
        }
    }
    
    /// Sort all queues appropriately
    pub fn sort(&mut self) {
        // Sort opaque front-to-back (by distance ascending)
        self.opaque.sort_by(|a, b| {
            a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Sort transparent back-to-front (by distance descending)
        self.transparent.sort_by(|a, b| {
            b.distance.partial_cmp(&a.distance).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // UI and debug maintain insertion order
    }
    
    /// Get total number of draw calls
    pub fn total_draw_calls(&self) -> usize {
        self.opaque.len() + self.transparent.len() + self.ui.len() + self.debug.len()
    }
    
    /// Clear all queues
    pub fn clear(&mut self) {
        self.opaque.clear();
        self.transparent.clear();
        self.ui.clear();
        self.debug.clear();
    }
    
    /// Get all draw calls in render order
    pub fn iter_in_order(&self) -> impl Iterator<Item = &DrawCall> {
        self.opaque
            .iter()
            .chain(self.transparent.iter())
            .chain(self.ui.iter())
            .chain(self.debug.iter())
    }
}

impl Default for IndexFormat {
    fn default() -> Self {
        Self::Uint32
    }
}

impl IndexFormat {
    /// Get the size in bytes of this index format
    pub fn size(&self) -> u32 {
        match self {
            Self::Uint16 => 2,
            Self::Uint32 => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    #[test]
    fn test_render_command_requires_render_pass() {
        let draw_cmd = RenderCommand::DrawIndexed {
            index_count: 36,
            instance_count: 1,
            first_index: 0,
            vertex_offset: 0,
            first_instance: 0,
        };
        assert!(draw_cmd.requires_render_pass());

        let viewport_cmd = RenderCommand::SetViewport {
            viewport: Viewport::new(0, 0, 800, 600),
        };
        assert!(!viewport_cmd.requires_render_pass());
    }

    #[test]
    fn test_render_command_changes_state() {
        let bind_cmd = RenderCommand::BindPipeline { pipeline: 123 };
        assert!(bind_cmd.changes_state());

        let debug_cmd = RenderCommand::DebugMarker { 
            label: "test".to_string() 
        };
        assert!(!debug_cmd.changes_state());
    }

    #[test]
    fn test_render_command_debug_label() {
        let debug_cmd = RenderCommand::DebugMarker { 
            label: "test marker".to_string() 
        };
        assert_eq!(debug_cmd.debug_label(), Some("test marker"));

        let group_cmd = RenderCommand::PushDebugGroup { 
            label: "test group".to_string() 
        };
        assert_eq!(group_cmd.debug_label(), Some("test group"));

        let draw_cmd = RenderCommand::Draw {
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
        };
        assert_eq!(draw_cmd.debug_label(), None);
    }

    #[test]
    fn test_draw_call_creation() {
        let transform = Mat4::IDENTITY;
        let draw_call = DrawCall::new(1, 2, transform);

        assert_eq!(draw_call.mesh, 1);
        assert_eq!(draw_call.material, 2);
        assert_eq!(draw_call.transform, transform);
        assert_eq!(draw_call.instance_count, 1);
        assert_eq!(draw_call.layer, RenderLayer::Opaque as u32);
        assert!(!draw_call.is_instanced());
    }

    #[test]
    fn test_draw_call_builder_pattern() {
        let transform = Mat4::IDENTITY;
        let draw_call = DrawCall::new(1, 2, transform)
            .with_layer(RenderLayer::Transparent)
            .with_distance(10.5)
            .with_instances(42, 100);

        assert_eq!(draw_call.layer, RenderLayer::Transparent as u32);
        assert_eq!(draw_call.distance, 10.5);
        assert_eq!(draw_call.instance_buffer, Some(42));
        assert_eq!(draw_call.instance_count, 100);
        assert!(draw_call.is_instanced());
    }

    #[test]
    fn test_render_queue_add_draw_calls() {
        let mut queue = RenderQueue::new();
        let transform = Mat4::IDENTITY;

        let opaque = DrawCall::new(1, 1, transform)
            .with_layer(RenderLayer::Opaque);
        let transparent = DrawCall::new(2, 2, transform)
            .with_layer(RenderLayer::Transparent);
        let ui = DrawCall::new(3, 3, transform)
            .with_layer(RenderLayer::UI);
        let debug = DrawCall::new(4, 4, transform)
            .with_layer(RenderLayer::Debug);

        queue.add_draw_call(opaque);
        queue.add_draw_call(transparent);
        queue.add_draw_call(ui);
        queue.add_draw_call(debug);

        assert_eq!(queue.opaque.len(), 1);
        assert_eq!(queue.transparent.len(), 1);
        assert_eq!(queue.ui.len(), 1);
        assert_eq!(queue.debug.len(), 1);
        assert_eq!(queue.total_draw_calls(), 4);
    }

    #[test]
    fn test_render_queue_sorting() {
        let mut queue = RenderQueue::new();
        let transform = Mat4::IDENTITY;

        // Add opaque objects with different distances
        queue.add_draw_call(
            DrawCall::new(1, 1, transform)
                .with_layer(RenderLayer::Opaque)
                .with_distance(10.0)
        );
        queue.add_draw_call(
            DrawCall::new(2, 2, transform)
                .with_layer(RenderLayer::Opaque)
                .with_distance(5.0)
        );

        // Add transparent objects with different distances
        queue.add_draw_call(
            DrawCall::new(3, 3, transform)
                .with_layer(RenderLayer::Transparent)
                .with_distance(8.0)
        );
        queue.add_draw_call(
            DrawCall::new(4, 4, transform)
                .with_layer(RenderLayer::Transparent)
                .with_distance(12.0)
        );

        queue.sort();

        // Opaque should be sorted front-to-back (ascending distance)
        assert_eq!(queue.opaque[0].distance, 5.0);
        assert_eq!(queue.opaque[1].distance, 10.0);

        // Transparent should be sorted back-to-front (descending distance)
        assert_eq!(queue.transparent[0].distance, 12.0);
        assert_eq!(queue.transparent[1].distance, 8.0);
    }

    #[test]
    fn test_render_queue_clear() {
        let mut queue = RenderQueue::new();
        let transform = Mat4::IDENTITY;

        queue.add_draw_call(DrawCall::new(1, 1, transform));
        assert_eq!(queue.total_draw_calls(), 1);

        queue.clear();
        assert_eq!(queue.total_draw_calls(), 0);
        assert!(queue.opaque.is_empty());
        assert!(queue.transparent.is_empty());
        assert!(queue.ui.is_empty());
        assert!(queue.debug.is_empty());
    }

    #[test]
    fn test_render_queue_iter_in_order() {
        let mut queue = RenderQueue::new();
        let transform = Mat4::IDENTITY;

        // Add one of each type
        queue.add_draw_call(DrawCall::new(1, 1, transform).with_layer(RenderLayer::Opaque));
        queue.add_draw_call(DrawCall::new(2, 2, transform).with_layer(RenderLayer::Transparent));
        queue.add_draw_call(DrawCall::new(3, 3, transform).with_layer(RenderLayer::UI));
        queue.add_draw_call(DrawCall::new(4, 4, transform).with_layer(RenderLayer::Debug));

        let order: Vec<_> = queue.iter_in_order().map(|dc| dc.mesh).collect();
        assert_eq!(order, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_index_format() {
        assert_eq!(IndexFormat::Uint16.size(), 2);
        assert_eq!(IndexFormat::Uint32.size(), 4);
        assert_eq!(IndexFormat::default(), IndexFormat::Uint32);
    }

    #[test]
    fn test_render_layer_ordering() {
        assert!(RenderLayer::Background < RenderLayer::Opaque);
        assert!(RenderLayer::Opaque < RenderLayer::Transparent);
        assert!(RenderLayer::Transparent < RenderLayer::UI);
        assert!(RenderLayer::UI < RenderLayer::Debug);
    }
}