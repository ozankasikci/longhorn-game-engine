//! Material system for graphics rendering

/// Material properties for rendering
pub struct Material {
    pub name: String,
    pub albedo: (f32, f32, f32, f32),
    pub metallic: f32,
    pub roughness: f32,
    pub emission: (f32, f32, f32),
}

impl Material {
    /// Create a new material
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            albedo: (1.0, 1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            emission: (0.0, 0.0, 0.0),
        }
    }
    
    /// Set albedo color
    pub fn with_albedo(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.albedo = (r, g, b, a);
        self
    }
    
    /// Set metallic value
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }
    
    /// Set roughness value
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }
}