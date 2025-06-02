use engine_materials_core::{MaterialHandle, ShaderHandle};
use engine_geometry_core::MeshHandle;

pub trait RenderPipeline {
    type PipelineId;
    
    fn create_pipeline(&mut self, descriptor: &PipelineDescriptor) -> Self::PipelineId;
    fn bind_pipeline(&mut self, id: &Self::PipelineId);
}

#[derive(Debug, Clone)]
pub struct PipelineDescriptor {
    pub vertex_shader: ShaderHandle,
    pub fragment_shader: Option<ShaderHandle>,
    pub vertex_layout: VertexLayout,
    pub primitive_topology: PrimitiveTopology,
    pub cull_mode: Option<CullMode>,
    pub depth_test: bool,
    pub blend_state: Option<BlendState>,
}

#[derive(Debug, Clone)]
pub struct VertexLayout {
    pub attributes: Vec<VertexAttribute>,
    pub stride: u32,
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub location: u32,
    pub format: VertexFormat,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleStrip,
    LineList,
    PointList,
}

#[derive(Debug, Clone, Copy)]
pub enum CullMode {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub struct BlendState {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
}