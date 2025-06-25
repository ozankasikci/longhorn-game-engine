use crate::{GraphicsBuffer, GraphicsTextureView, GraphicsSampler};

/// Binding resource types
pub enum BindingResource<'a> {
    /// Buffer binding
    Buffer(BufferBinding<'a>),
    /// Texture view binding
    TextureView(&'a dyn GraphicsTextureView),
    /// Sampler binding
    Sampler(&'a dyn GraphicsSampler),
}

/// Buffer binding information
pub struct BufferBinding<'a> {
    /// The buffer to bind
    pub buffer: &'a dyn GraphicsBuffer,
    /// Offset in bytes
    pub offset: u64,
    /// Size in bytes (None means whole buffer)
    pub size: Option<u64>,
}

/// Trait for bind groups (resource sets)
pub trait GraphicsBindGroup: Send + Sync {
    /// Get the layout this bind group was created with
    fn layout(&self) -> &dyn GraphicsBindGroupLayout;
}

/// Trait for bind group layouts
pub trait GraphicsBindGroupLayout: Send + Sync {
    /// Get the number of bindings in this layout
    fn binding_count(&self) -> u32;
}

/// Binding type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    /// Uniform buffer
    Uniform,
    /// Storage buffer (read-only)
    StorageReadOnly,
    /// Storage buffer (read-write)
    Storage,
    /// Texture binding
    Texture {
        /// Sample type (float, int, uint)
        sample_type: TextureSampleType,
        /// View dimension (2D, 3D, Cube, etc.)
        view_dimension: TextureViewDimension,
        /// Multisampled texture
        multisampled: bool,
    },
    /// Sampler binding
    Sampler {
        /// Filtering mode
        filtering: bool,
    },
}

/// Texture sample type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureSampleType {
    /// Float (filterable)
    Float { 
        /// Whether the texture can be filtered
        filterable: bool 
    },
    /// Signed integer
    Sint,
    /// Unsigned integer
    Uint,
    /// Depth
    Depth,
}

/// Texture view dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureViewDimension {
    /// 1D texture
    D1,
    /// 2D texture
    D2,
    /// 2D array texture
    D2Array,
    /// Cube texture
    Cube,
    /// Cube array texture
    CubeArray,
    /// 3D texture
    D3,
}

/// Shader stage visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderStages(u32);

impl ShaderStages {
    /// Visible in vertex stage
    pub const VERTEX: Self = Self(1 << 0);
    /// Visible in fragment stage
    pub const FRAGMENT: Self = Self(1 << 1);
    /// Visible in compute stage
    pub const COMPUTE: Self = Self(1 << 2);
    
    /// Visible in vertex and fragment stages
    pub const VERTEX_FRAGMENT: Self = Self(Self::VERTEX.0 | Self::FRAGMENT.0);
    
    /// Check if contains specific stages
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Bind group layout entry
#[derive(Debug, Clone)]
pub struct BindGroupLayoutEntry {
    /// Binding index
    pub binding: u32,
    /// Which shader stages can access this binding
    pub visibility: ShaderStages,
    /// Type of binding
    pub ty: BindingType,
    /// Binding count (for arrays)
    pub count: Option<u32>,
}

/// Bind group entry
pub struct BindGroupEntry<'a> {
    /// Binding index
    pub binding: u32,
    /// Resource to bind
    pub resource: BindingResource<'a>,
}

/// Bind group descriptor
pub struct BindGroupDescriptor<'a> {
    /// Layout for this bind group
    pub layout: &'a dyn GraphicsBindGroupLayout,
    /// List of entries
    pub entries: Vec<BindGroupEntry<'a>>,
}

/// Bind group layout descriptor
#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptor {
    /// List of entries
    pub entries: Vec<BindGroupLayoutEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock implementations
    struct MockBuffer {
        size: u64,
    }
    
    impl GraphicsBuffer for MockBuffer {
        fn write(&self, _offset: u64, _data: &[u8]) -> crate::Result<()> {
            Ok(())
        }
        
        fn read(&self) -> crate::Result<Vec<u8>> {
            Ok(vec![])
        }
        
        fn size(&self) -> u64 {
            self.size
        }
        
        fn map_write(&self) -> crate::Result<crate::BufferMappedRange> {
            unimplemented!()
        }
        
        fn unmap(&self) {}
    }
    
    struct MockTextureView;
    impl GraphicsTextureView for MockTextureView {
        fn texture(&self) -> &dyn crate::GraphicsTexture {
            unimplemented!()
        }
        
        fn format(&self) -> crate::TextureFormat {
            crate::TextureFormat::Rgba8Unorm
        }
    }
    
    struct MockSampler;
    impl GraphicsSampler for MockSampler {
        fn min_filter(&self) -> crate::FilterMode {
            crate::FilterMode::Linear
        }
        
        fn mag_filter(&self) -> crate::FilterMode {
            crate::FilterMode::Linear
        }
        
        fn address_mode_u(&self) -> crate::AddressMode {
            crate::AddressMode::ClampToEdge
        }
        
        fn address_mode_v(&self) -> crate::AddressMode {
            crate::AddressMode::ClampToEdge
        }
        
        fn address_mode_w(&self) -> crate::AddressMode {
            crate::AddressMode::ClampToEdge
        }
    }
    
    struct MockBindGroupLayout {
        binding_count: u32,
    }
    
    impl GraphicsBindGroupLayout for MockBindGroupLayout {
        fn binding_count(&self) -> u32 {
            self.binding_count
        }
    }
    
    struct MockBindGroup {
        layout: MockBindGroupLayout,
    }
    
    impl GraphicsBindGroup for MockBindGroup {
        fn layout(&self) -> &dyn GraphicsBindGroupLayout {
            &self.layout
        }
    }
    
    #[test]
    fn test_shader_stages() {
        let stages = ShaderStages::VERTEX_FRAGMENT;
        assert!(stages.contains(ShaderStages::VERTEX));
        assert!(stages.contains(ShaderStages::FRAGMENT));
        assert!(!stages.contains(ShaderStages::COMPUTE));
    }
    
    #[test]
    fn test_buffer_binding() {
        let buffer = MockBuffer { size: 256 };
        let binding = BufferBinding {
            buffer: &buffer,
            offset: 64,
            size: Some(128),
        };
        
        assert_eq!(binding.offset, 64);
        assert_eq!(binding.size, Some(128));
        assert_eq!(binding.buffer.size(), 256);
    }
    
    #[test]
    fn test_binding_resource_variants() {
        let buffer = MockBuffer { size: 1024 };
        let texture_view = MockTextureView;
        let sampler = MockSampler;
        
        // Test buffer binding
        let buffer_resource = BindingResource::Buffer(BufferBinding {
            buffer: &buffer,
            offset: 0,
            size: None,
        });
        
        match buffer_resource {
            BindingResource::Buffer(binding) => {
                assert_eq!(binding.buffer.size(), 1024);
            }
            _ => panic!("Wrong variant"),
        }
        
        // Test texture view binding
        let texture_resource = BindingResource::TextureView(&texture_view);
        match texture_resource {
            BindingResource::TextureView(_) => {
                // Success
            }
            _ => panic!("Wrong variant"),
        }
        
        // Test sampler binding
        let sampler_resource = BindingResource::Sampler(&sampler);
        match sampler_resource {
            BindingResource::Sampler(_) => {
                // Success
            }
            _ => panic!("Wrong variant"),
        }
    }
    
    #[test]
    fn test_bind_group_layout_descriptor() {
        let descriptor = BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Uniform,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler { filtering: true },
                    count: None,
                },
            ],
        };
        
        assert_eq!(descriptor.entries.len(), 3);
        assert_eq!(descriptor.entries[0].binding, 0);
        assert_eq!(descriptor.entries[1].binding, 1);
        assert_eq!(descriptor.entries[2].binding, 2);
    }
    
    #[test]
    fn test_bind_group_and_layout() {
        let _layout = MockBindGroupLayout { binding_count: 3 };
        let bind_group = MockBindGroup {
            layout: MockBindGroupLayout { binding_count: 3 },
        };
        
        assert_eq!(bind_group.layout().binding_count(), 3);
    }
    
    #[test]
    fn test_texture_sample_type() {
        let float_type = TextureSampleType::Float { filterable: true };
        let sint_type = TextureSampleType::Sint;
        let uint_type = TextureSampleType::Uint;
        let depth_type = TextureSampleType::Depth;
        
        // Just verify they can be created and compared
        assert_ne!(
            float_type,
            TextureSampleType::Float { filterable: false }
        );
        assert_eq!(sint_type, TextureSampleType::Sint);
        assert_eq!(uint_type, TextureSampleType::Uint);
        assert_eq!(depth_type, TextureSampleType::Depth);
    }
}