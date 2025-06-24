use engine_geometry_core::MeshHandle;
use engine_materials_core::MaterialHandle;
use glam::{Mat4, Vec4};

pub trait CommandBuffer {
    fn begin_render_pass(&mut self, pass: &RenderPassDescriptor);
    fn end_render_pass(&mut self);
    fn draw_mesh(&mut self, mesh: &MeshHandle, material: &MaterialHandle, transform: &Mat4);
    fn set_viewport(&mut self, viewport: &Viewport);
    fn clear(&mut self, clear_value: &ClearValue);
    fn submit(self);
}

#[derive(Debug, Clone)]
pub struct RenderPassDescriptor {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub target: RenderTarget,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: Vec4,
}

#[derive(Debug, Clone)]
pub struct DepthAttachment {
    pub target: RenderTarget,
    pub depth_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub clear_depth: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Clear,
    Load,
    DontCare,
}

#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Store,
    DontCare,
}

#[derive(Debug, Clone)]
pub enum RenderTarget {
    Surface,
    Texture { width: u32, height: u32 },
}

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Debug, Clone)]
pub struct ClearValue {
    pub color: Vec4,
    pub depth: f32,
}
