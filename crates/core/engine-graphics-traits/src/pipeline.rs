use crate::types::VertexFormat;

/// Shader stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    /// Vertex shader stage
    Vertex,
    /// Fragment/pixel shader stage
    Fragment,
    /// Compute shader stage
    Compute,
}

/// Shader source formats
#[derive(Debug, Clone)]
pub enum ShaderSource {
    /// WebGPU Shading Language
    Wgsl(String),
    /// SPIR-V bytecode
    SpirV(Vec<u32>),
    /// HLSL source code
    Hlsl(String),
    /// GLSL source code
    Glsl(String),
}

/// Trait for compiled shaders
pub trait GraphicsShader: Send + Sync {
    /// Get the shader stage
    fn stage(&self) -> ShaderStage;

    /// Get the entry point name
    fn entry_point(&self) -> &str;
}

/// Trait for graphics pipelines
pub trait GraphicsPipeline: Send + Sync {
    /// Get the pipeline layout
    fn layout(&self) -> &dyn GraphicsPipelineLayout;
}

/// Trait for compute pipelines
pub trait ComputePipeline: Send + Sync {
    /// Get the pipeline layout
    fn layout(&self) -> &dyn GraphicsPipelineLayout;
}

/// Trait for pipeline layouts
pub trait GraphicsPipelineLayout: Send + Sync {
    /// Get the number of bind group layouts
    fn bind_group_layout_count(&self) -> usize;
}

/// Vertex attribute description
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    /// Shader location
    pub location: u32,
    /// Offset in bytes
    pub offset: u64,
    /// Format of the attribute
    pub format: VertexFormat,
}

/// Vertex buffer layout
#[derive(Debug, Clone)]
pub struct VertexBufferLayout {
    /// Size of each vertex in bytes
    pub stride: u64,
    /// Step mode (per vertex or per instance)
    pub step_mode: VertexStepMode,
    /// List of attributes
    pub attributes: Vec<VertexAttribute>,
}

/// Vertex step mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexStepMode {
    /// Advance per vertex
    Vertex,
    /// Advance per instance
    Instance,
}

/// Primitive topology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTopology {
    /// Points
    PointList,
    /// Lines
    LineList,
    /// Line strip
    LineStrip,
    /// Triangles
    TriangleList,
    /// Triangle strip
    TriangleStrip,
}

/// Face culling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    /// No culling
    None,
    /// Cull front faces
    Front,
    /// Cull back faces
    Back,
}

/// Polygon front face orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontFace {
    /// Counter-clockwise
    Ccw,
    /// Clockwise
    Cw,
}

/// Primitive state configuration
#[derive(Debug, Clone)]
pub struct PrimitiveState {
    /// Primitive topology
    pub topology: PrimitiveTopology,
    /// Face culling
    pub cull_mode: CullMode,
    /// Front face orientation
    pub front_face: FrontFace,
}

impl Default for PrimitiveState {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: CullMode::Back,
            front_face: FrontFace::Ccw,
        }
    }
}

/// Render pipeline descriptor
pub struct RenderPipelineDescriptor<'a> {
    /// Pipeline layout
    pub layout: &'a dyn GraphicsPipelineLayout,
    /// Vertex shader
    pub vertex: &'a dyn GraphicsShader,
    /// Fragment shader (optional for depth-only pipelines)
    pub fragment: Option<&'a dyn GraphicsShader>,
    /// Vertex buffer layouts
    pub vertex_buffers: Vec<VertexBufferLayout>,
    /// Primitive state
    pub primitive: PrimitiveState,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations
    struct MockShader {
        stage: ShaderStage,
        entry_point: String,
    }

    impl GraphicsShader for MockShader {
        fn stage(&self) -> ShaderStage {
            self.stage
        }

        fn entry_point(&self) -> &str {
            &self.entry_point
        }
    }

    struct MockPipelineLayout {
        bind_group_count: usize,
    }

    impl GraphicsPipelineLayout for MockPipelineLayout {
        fn bind_group_layout_count(&self) -> usize {
            self.bind_group_count
        }
    }

    struct MockPipeline {
        layout: MockPipelineLayout,
    }

    impl GraphicsPipeline for MockPipeline {
        fn layout(&self) -> &dyn GraphicsPipelineLayout {
            &self.layout
        }
    }

    #[test]
    fn test_shader_properties() {
        let shader = MockShader {
            stage: ShaderStage::Vertex,
            entry_point: "main".to_string(),
        };

        assert_eq!(shader.stage(), ShaderStage::Vertex);
        assert_eq!(shader.entry_point(), "main");
    }

    #[test]
    fn test_graphics_pipeline() {
        let layout = MockPipelineLayout {
            bind_group_count: 2,
        };
        let pipeline = MockPipeline { layout };

        assert_eq!(pipeline.layout().bind_group_layout_count(), 2);
    }

    #[test]
    fn test_vertex_buffer_layout() {
        let layout = VertexBufferLayout {
            stride: 32,
            step_mode: VertexStepMode::Vertex,
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    offset: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    location: 1,
                    offset: 12,
                    format: VertexFormat::Float32x2,
                },
            ],
        };

        assert_eq!(layout.stride, 32);
        assert_eq!(layout.step_mode, VertexStepMode::Vertex);
        assert_eq!(layout.attributes.len(), 2);
        assert_eq!(layout.attributes[0].location, 0);
        assert_eq!(layout.attributes[1].offset, 12);
    }

    #[test]
    fn test_pipeline_layout() {
        let layout = MockPipelineLayout {
            bind_group_count: 3,
        };

        assert_eq!(layout.bind_group_layout_count(), 3);
    }

    #[test]
    fn test_primitive_state_default() {
        let state = PrimitiveState::default();

        assert_eq!(state.topology, PrimitiveTopology::TriangleList);
        assert_eq!(state.cull_mode, CullMode::Back);
        assert_eq!(state.front_face, FrontFace::Ccw);
    }

    #[test]
    fn test_render_pipeline_descriptor() {
        let layout = MockPipelineLayout {
            bind_group_count: 2,
        };

        let vertex_shader = MockShader {
            stage: ShaderStage::Vertex,
            entry_point: "vs_main".to_string(),
        };

        let fragment_shader = MockShader {
            stage: ShaderStage::Fragment,
            entry_point: "fs_main".to_string(),
        };

        let descriptor = RenderPipelineDescriptor {
            layout: &layout,
            vertex: &vertex_shader,
            fragment: Some(&fragment_shader),
            vertex_buffers: vec![],
            primitive: PrimitiveState::default(),
        };

        assert_eq!(descriptor.layout.bind_group_layout_count(), 2);
        assert_eq!(descriptor.vertex.stage(), ShaderStage::Vertex);
        assert_eq!(descriptor.fragment.unwrap().stage(), ShaderStage::Fragment);
    }
}
