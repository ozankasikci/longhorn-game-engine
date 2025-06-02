//! Render pipeline abstractions

use crate::{Handle, ShaderHandle};
use serde::{Serialize, Deserialize};

/// Render pipeline handle
pub type RenderPipelineHandle = Handle;

/// Render pipeline descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDescriptor {
    pub label: Option<String>,
    pub vertex: VertexState,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub fragment: Option<FragmentState>,
}

/// Vertex processing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexState {
    pub shader: ShaderHandle,
    pub entry_point: String,
    pub buffers: Vec<VertexBufferLayout>,
}

/// Fragment processing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentState {
    pub shader: ShaderHandle,
    pub entry_point: String,
    pub targets: Vec<ColorTargetState>,
}

/// Primitive assembly and rasterization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub front_face: FrontFace,
    pub cull_mode: Option<Face>,
    pub unclipped_depth: bool,
    pub polygon_mode: PolygonMode,
    pub conservative: bool,
}

/// Depth and stencil test state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthStencilState {
    pub format: crate::resources::TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil: StencilState,
    pub bias: DepthBiasState,
}

/// Multisampling state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisampleState {
    pub count: u32,
    pub mask: u64,
    pub alpha_to_coverage_enabled: bool,
}

/// Color target state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTargetState {
    pub format: crate::resources::TextureFormat,
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrites,
}

/// Vertex buffer layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexBufferLayout {
    pub array_stride: u64,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

/// Vertex attribute descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u64,
    pub shader_location: u32,
}

/// Primitive topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

/// Index format for strip topologies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

/// Front face orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrontFace {
    Ccw,
    Cw,
}

/// Face for culling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Face {
    Front,
    Back,
}

/// Polygon rasterization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

/// Depth/stencil compare function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Stencil test state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StencilState {
    pub front: StencilFaceState,
    pub back: StencilFaceState,
    pub read_mask: u32,
    pub write_mask: u32,
}

/// Stencil test for one face
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StencilFaceState {
    pub compare: CompareFunction,
    pub fail_op: StencilOperation,
    pub depth_fail_op: StencilOperation,
    pub pass_op: StencilOperation,
}

/// Stencil operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    Invert,
    IncrementClamp,
    DecrementClamp,
    IncrementWrap,
    DecrementWrap,
}

/// Depth bias state
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DepthBiasState {
    pub constant: i32,
    pub slope_scale: f32,
    pub clamp: f32,
}

/// Blend state for color target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}

/// Blend component (color or alpha)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendComponent {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

/// Blend factor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    Src,
    OneMinusSrc,
    SrcAlpha,
    OneMinusSrcAlpha,
    Dst,
    OneMinusDst,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    Constant,
    OneMinusConstant,
}

/// Blend operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// Color write mask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorWrites {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub alpha: bool,
}

/// Vertex step mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexFormat {
    Uint8x2,
    Uint8x4,
    Sint8x2,
    Sint8x4,
    Unorm8x2,
    Unorm8x4,
    Snorm8x2,
    Snorm8x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
    Unorm16x2,
    Unorm16x4,
    Snorm16x2,
    Snorm16x4,
    Float16x2,
    Float16x4,
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Float64,
    Float64x2,
    Float64x3,
    Float64x4,
}

/// Render pipeline trait
pub trait RenderPipeline: Send + Sync {
    /// Get the pipeline descriptor used to create this pipeline
    fn descriptor(&self) -> &PipelineDescriptor;
    
    /// Check if the pipeline is compatible with the given vertex layout
    fn is_compatible_with_vertex_layout(&self, layout: &VertexBufferLayout) -> bool;
    
    /// Get the expected vertex stride
    fn vertex_stride(&self) -> u64;
}

impl Default for PrimitiveState {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        }
    }
}

impl Default for MultisampleState {
    fn default() -> Self {
        Self {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            front: StencilFaceState::default(),
            back: StencilFaceState::default(),
            read_mask: !0,
            write_mask: !0,
        }
    }
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            compare: CompareFunction::Always,
            fail_op: StencilOperation::Keep,
            depth_fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}

impl Default for ColorWrites {
    fn default() -> Self {
        Self {
            red: true,
            green: true,
            blue: true,
            alpha: true,
        }
    }
}

impl ColorWrites {
    pub const ALL: Self = Self {
        red: true,
        green: true,
        blue: true,
        alpha: true,
    };
    
    pub const NONE: Self = Self {
        red: false,
        green: false,
        blue: false,
        alpha: false,
    };
    
    pub const COLOR: Self = Self {
        red: true,
        green: true,
        blue: true,
        alpha: false,
    };
    
    pub const ALPHA: Self = Self {
        red: false,
        green: false,
        blue: false,
        alpha: true,
    };
}

impl BlendState {
    /// Alpha blending (source over destination)
    pub const ALPHA_BLENDING: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };
    
    /// Premultiplied alpha blending
    pub const PREMULTIPLIED_ALPHA_BLENDING: Self = Self {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    };
}

impl VertexFormat {
    /// Get the size in bytes of this vertex format
    pub fn size(&self) -> u64 {
        match self {
            Self::Uint8x2 | Self::Sint8x2 | Self::Unorm8x2 | Self::Snorm8x2 => 2,
            Self::Uint8x4 | Self::Sint8x4 | Self::Unorm8x4 | Self::Snorm8x4 
            | Self::Uint16x2 | Self::Sint16x2 | Self::Unorm16x2 | Self::Snorm16x2 
            | Self::Float16x2 | Self::Float32 | Self::Uint32 | Self::Sint32 => 4,
            Self::Uint16x4 | Self::Sint16x4 | Self::Unorm16x4 | Self::Snorm16x4 
            | Self::Float16x4 | Self::Float32x2 | Self::Uint32x2 | Self::Sint32x2 
            | Self::Float64 => 8,
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 12,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 | Self::Float64x2 => 16,
            Self::Float64x3 => 24,
            Self::Float64x4 => 32,
        }
    }
}