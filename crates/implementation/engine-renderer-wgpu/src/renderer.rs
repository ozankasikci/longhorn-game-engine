//! Core renderer trait and abstractions

use crate::{RenderCommand, RendererCapabilities, RendererError, Result, Viewport};
use glam::{Mat4, Vec3};
use serde::{Serialize, Deserialize};

/// Core renderer trait that all graphics implementations must implement
pub trait Renderer: Send + Sync {
    /// Initialize the renderer
    fn initialize(&mut self) -> Result<()>;
    
    /// Shutdown the renderer
    fn shutdown(&mut self) -> Result<()>;
    
    /// Begin a new frame
    fn begin_frame(&mut self) -> Result<()>;
    
    /// End the current frame and present
    fn end_frame(&mut self) -> Result<()>;
    
    /// Execute a single render command
    fn execute_command(&mut self, command: &RenderCommand) -> Result<()>;
    
    /// Execute multiple render commands
    fn execute_commands(&mut self, commands: &[RenderCommand]) -> Result<()> {
        for command in commands {
            self.execute_command(command)?;
        }
        Ok(())
    }
    
    /// Resize the render target
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;
    
    /// Set the viewport
    fn set_viewport(&mut self, viewport: Viewport) -> Result<()>;
    
    /// Clear the render target
    fn clear(&mut self, color: Option<[f32; 4]>, depth: Option<f32>, stencil: Option<u32>) -> Result<()>;
    
    /// Get renderer capabilities
    fn capabilities(&self) -> RendererCapabilities;
    
    /// Get current render statistics
    fn statistics(&self) -> RenderStatistics;
}

/// Batch renderer for efficient rendering of multiple objects
pub trait BatchRenderer: Renderer {
    /// Begin a new render batch
    fn begin_batch(&mut self) -> Result<()>;
    
    /// Add a render command to the current batch
    fn add_to_batch(&mut self, command: RenderCommand) -> Result<()>;
    
    /// Execute all commands in the current batch
    fn execute_batch(&mut self) -> Result<()>;
    
    /// Get the maximum number of commands that can be batched
    fn max_batch_size(&self) -> usize;
    
    /// Get the current batch size
    fn current_batch_size(&self) -> usize;
}

/// Current render state
#[derive(Debug, Clone)]
pub struct RenderState {
    /// Current view matrix
    pub view_matrix: Mat4,
    /// Current projection matrix
    pub projection_matrix: Mat4,
    /// Combined view-projection matrix
    pub view_projection_matrix: Mat4,
    /// Camera position in world space
    pub camera_position: Vec3,
    /// Camera forward direction
    pub camera_forward: Vec3,
    /// Current viewport
    pub viewport: Viewport,
    /// Current render target dimensions
    pub render_target_size: (u32, u32),
    /// Near and far clip planes
    pub clip_planes: (f32, f32),
    /// Current frame number
    pub frame_number: u64,
}

/// Render statistics for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderStatistics {
    /// Current frame number
    pub frame_number: u64,
    /// Time to render last frame in milliseconds
    pub frame_time_ms: f32,
    /// Number of draw calls in last frame
    pub draw_calls: u32,
    /// Number of triangles rendered in last frame
    pub triangles: u32,
    /// Number of vertices processed in last frame
    pub vertices: u32,
    /// GPU memory usage in bytes
    pub gpu_memory_used: u64,
    /// GPU memory total in bytes
    pub gpu_memory_total: u64,
    /// Number of texture bindings
    pub texture_bindings: u32,
    /// Number of buffer bindings
    pub buffer_bindings: u32,
    /// Number of state changes
    pub state_changes: u32,
}

/// Render pass descriptor
#[derive(Debug, Clone)]
pub struct RenderPassDescriptor {
    /// Name for debugging
    pub label: Option<String>,
    /// Color attachments
    pub color_attachments: Vec<ColorAttachment>,
    /// Depth stencil attachment
    pub depth_stencil_attachment: Option<DepthStencilAttachment>,
}

/// Color attachment for render pass
#[derive(Debug, Clone)]
pub struct ColorAttachment {
    /// Texture to render to
    pub texture: crate::TextureHandle,
    /// Load operation
    pub load_op: LoadOp,
    /// Store operation
    pub store_op: StoreOp,
    /// Clear color if load_op is Clear
    pub clear_color: [f32; 4],
}

/// Depth stencil attachment for render pass
#[derive(Debug, Clone)]
pub struct DepthStencilAttachment {
    /// Depth texture
    pub texture: crate::TextureHandle,
    /// Depth load operation
    pub depth_load_op: LoadOp,
    /// Depth store operation
    pub depth_store_op: StoreOp,
    /// Clear depth value if depth_load_op is Clear
    pub clear_depth: f32,
    /// Stencil load operation
    pub stencil_load_op: LoadOp,
    /// Stencil store operation
    pub stencil_store_op: StoreOp,
    /// Clear stencil value if stencil_load_op is Clear
    pub clear_stencil: u32,
}

/// Load operations for render pass attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadOp {
    /// Clear the attachment
    Clear,
    /// Load existing contents
    Load,
    /// Don't care about existing contents
    DontCare,
}

/// Store operations for render pass attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreOp {
    /// Store the results
    Store,
    /// Don't store the results
    DontCare,
}

impl Default for RenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderState {
    /// Create a new render state
    pub fn new() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            camera_position: Vec3::ZERO,
            camera_forward: Vec3::NEG_Z,
            viewport: Viewport::default(),
            render_target_size: (800, 600),
            clip_planes: (0.1, 1000.0),
            frame_number: 0,
        }
    }
    
    /// Update view matrix and recalculate combined matrix
    pub fn set_view_matrix(&mut self, view: Mat4) {
        self.view_matrix = view;
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
    }
    
    /// Update projection matrix and recalculate combined matrix
    pub fn set_projection_matrix(&mut self, projection: Mat4) {
        self.projection_matrix = projection;
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
    }
    
    /// Set camera position and forward direction
    pub fn set_camera(&mut self, position: Vec3, forward: Vec3) {
        self.camera_position = position;
        self.camera_forward = forward.normalize();
    }
    
    /// Set viewport
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }
    
    /// Set render target size
    pub fn set_render_target_size(&mut self, width: u32, height: u32) {
        self.render_target_size = (width, height);
    }
    
    /// Increment frame number
    pub fn next_frame(&mut self) {
        self.frame_number += 1;
    }
    
    /// Convert screen coordinates to normalized device coordinates
    pub fn screen_to_ndc(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let ndc_x = (screen_x / self.viewport.width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / self.viewport.height as f32) * 2.0;
        (ndc_x, ndc_y)
    }
    
    /// Convert normalized device coordinates to screen coordinates
    pub fn ndc_to_screen(&self, ndc_x: f32, ndc_y: f32) -> (f32, f32) {
        let screen_x = (ndc_x + 1.0) * 0.5 * self.viewport.width as f32;
        let screen_y = (1.0 - ndc_y) * 0.5 * self.viewport.height as f32;
        (screen_x, screen_y)
    }
}

impl Default for RenderStatistics {
    fn default() -> Self {
        Self {
            frame_number: 0,
            frame_time_ms: 0.0,
            draw_calls: 0,
            triangles: 0,
            vertices: 0,
            gpu_memory_used: 0,
            gpu_memory_total: 0,
            texture_bindings: 0,
            buffer_bindings: 0,
            state_changes: 0,
        }
    }
}

impl RenderStatistics {
    /// Calculate frames per second from frame time
    pub fn fps(&self) -> f32 {
        if self.frame_time_ms > 0.0 {
            1000.0 / self.frame_time_ms
        } else {
            0.0
        }
    }
    
    /// Calculate GPU memory usage percentage
    pub fn gpu_memory_usage_percent(&self) -> f32 {
        if self.gpu_memory_total > 0 {
            (self.gpu_memory_used as f32 / self.gpu_memory_total as f32) * 100.0
        } else {
            0.0
        }
    }
    
    /// Check if performance is good (>= 60 FPS)
    pub fn is_performance_good(&self) -> bool {
        self.fps() >= 60.0
    }
    
    /// Check if GPU memory usage is concerning (>= 80%)
    pub fn is_memory_usage_high(&self) -> bool {
        self.gpu_memory_usage_percent() >= 80.0
    }
}