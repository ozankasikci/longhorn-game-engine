use longhorn_assets::TextureData;
use longhorn_core::AssetId;
use std::collections::HashMap;
use wgpu;

/// GPU texture with associated view, sampler, and bind group
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

impl GpuTexture {
    /// Create a GPU texture from texture data
    pub fn from_texture_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        texture_data: &TextureData,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: texture_data.width,
            height: texture_data.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_data.width),
                rows_per_image: Some(texture_data.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
            width: texture_data.width,
            height: texture_data.height,
        }
    }
}

/// Cache for GPU textures
pub struct TextureCache {
    textures: HashMap<AssetId, GpuTexture>,
}

impl TextureCache {
    /// Create a new empty texture cache
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// Insert a texture into the cache
    pub fn insert(&mut self, asset_id: AssetId, texture: GpuTexture) {
        self.textures.insert(asset_id, texture);
    }

    /// Get a texture from the cache
    pub fn get(&self, asset_id: AssetId) -> Option<&GpuTexture> {
        self.textures.get(&asset_id)
    }

    /// Check if a texture is cached
    pub fn contains(&self, asset_id: AssetId) -> bool {
        self.textures.contains_key(&asset_id)
    }

    /// Clear all cached textures
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}
