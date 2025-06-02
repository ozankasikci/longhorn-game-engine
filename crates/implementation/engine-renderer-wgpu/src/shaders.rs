//! Shader management and compilation

/// Shader type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

/// Shader representation
pub struct Shader {
    pub name: String,
    pub shader_type: ShaderType,
    pub source: String,
}

/// Shader program combining vertex and fragment shaders
pub struct ShaderProgram {
    pub name: String,
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
}

impl Shader {
    /// Create a new shader
    pub fn new(name: &str, shader_type: ShaderType, source: &str) -> Self {
        Self {
            name: name.to_string(),
            shader_type,
            source: source.to_string(),
        }
    }
}

impl ShaderProgram {
    /// Create a new shader program
    pub fn new(name: &str, vertex_shader: Shader, fragment_shader: Shader) -> Self {
        Self {
            name: name.to_string(),
            vertex_shader,
            fragment_shader,
        }
    }
}