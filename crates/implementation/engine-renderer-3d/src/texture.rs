//! Texture management for the 3D renderer
//! 
//! This module handles texture loading, GPU upload, and management
//! for efficient texture-based rendering.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wgpu::{Device, Queue, Texture, TextureView, Sampler};

/// Texture resource on GPU
#[derive(Debug)]
pub struct TextureResource {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub name: String,
}

/// Texture manager for loading and managing GPU textures
pub struct TextureManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    
    textures: RwLock<HashMap<u32, TextureResource>>,
    next_texture_id: std::sync::atomic::AtomicU32,
    
    // Default samplers
    linear_sampler: Sampler,
    nearest_sampler: Sampler,
}

/// Texture creation parameters
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub label: Option<String>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub data: Vec<u8>,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        // Create default samplers
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            device,
            queue,
            textures: RwLock::new(HashMap::new()),
            next_texture_id: std::sync::atomic::AtomicU32::new(0),
            linear_sampler,
            nearest_sampler,
        }
    }
    
    /// Create a texture from raw data and return its ID
    pub fn create_texture(&self, desc: TextureDescriptor) -> Result<u32, anyhow::Error> {
        let texture_id = self.next_texture_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let label_binding = desc.label.clone();
        let label = label_binding.as_deref().unwrap_or("Texture");
        
        // Create GPU texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: desc.usage | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Upload texture data
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &desc.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * desc.width), // Assuming RGBA8
                rows_per_image: Some(desc.height),
            },
            wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
        );
        
        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create a new linear sampler for this texture
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("Sampler for {}", label)),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        let texture_resource = TextureResource {
            texture,
            view,
            sampler,
            width: desc.width,
            height: desc.height,
            format: desc.format,
            name: desc.label.unwrap_or_else(|| format!("Texture {}", texture_id)),
        };
        
        // Store the texture resource
        self.textures.write().unwrap().insert(texture_id, texture_resource);
        
        log::info!("Created texture '{}' with ID {} ({}x{})", 
                   label, texture_id, desc.width, desc.height);
        
        Ok(texture_id)
    }
    
    /// Create a solid color texture
    pub fn create_solid_color_texture(&self, color: [u8; 4], size: u32) -> Result<u32, anyhow::Error> {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        for _ in 0..(size * size) {
            data.extend_from_slice(&color);
        }
        
        let desc = TextureDescriptor {
            label: Some(format!("Solid Color {:?}", color)),
            width: size,
            height: size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            data,
        };
        
        self.create_texture(desc)
    }
    
    /// Create a checkerboard test texture
    pub fn create_checkerboard_texture(&self, size: u32, square_size: u32) -> Result<u32, anyhow::Error> {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        
        for y in 0..size {
            for x in 0..size {
                let checker_x = (x / square_size) % 2;
                let checker_y = (y / square_size) % 2;
                let is_white = (checker_x + checker_y) % 2 == 0;
                
                let color = if is_white {
                    [255, 255, 255, 255] // White
                } else {
                    [128, 128, 128, 255] // Gray
                };
                
                data.extend_from_slice(&color);
            }
        }
        
        let desc = TextureDescriptor {
            label: Some("Checkerboard".to_string()),
            width: size,
            height: size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            data,
        };
        
        self.create_texture(desc)
    }
    
    /// Get texture resource by ID
    pub fn get_texture(&self, texture_id: u32) -> Option<std::sync::RwLockReadGuard<HashMap<u32, TextureResource>>> {
        let textures = self.textures.read().unwrap();
        if textures.contains_key(&texture_id) {
            Some(textures)
        } else {
            None
        }
    }
    
    /// Load default textures
    pub fn load_default_textures(&self) -> Result<HashMap<String, u32>, anyhow::Error> {
        let mut default_textures = HashMap::new();
        
        // White texture (1x1)
        let white_id = self.create_solid_color_texture([255, 255, 255, 255], 1)?;
        default_textures.insert("white".to_string(), white_id);
        
        // Black texture (1x1)
        let black_id = self.create_solid_color_texture([0, 0, 0, 255], 1)?;
        default_textures.insert("black".to_string(), black_id);
        
        // Red texture (1x1)
        let red_id = self.create_solid_color_texture([255, 0, 0, 255], 1)?;
        default_textures.insert("red".to_string(), red_id);
        
        // Checkerboard texture (64x64)
        let checkerboard_id = self.create_checkerboard_texture(64, 8)?;
        default_textures.insert("checkerboard".to_string(), checkerboard_id);
        
        log::info!("Loaded {} default textures", default_textures.len());
        Ok(default_textures)
    }
    
    /// Get texture count
    pub fn texture_count(&self) -> usize {
        self.textures.read().unwrap().len()
    }
    
    /// Get linear sampler reference
    pub fn linear_sampler(&self) -> &Sampler {
        &self.linear_sampler
    }
    
    /// Get nearest sampler reference
    pub fn nearest_sampler(&self) -> &Sampler {
        &self.nearest_sampler
    }
}

/// Helper function to create a test pattern texture
pub fn create_test_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = ((x + y) % 255) as u8;
            let a = 255u8;
            
            data.extend_from_slice(&[r, g, b, a]);
        }
    }
    
    data
}