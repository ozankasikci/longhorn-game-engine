use crate::Color;
use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use longhorn_core::AssetId;

/// Vertex data for sprite rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    /// Get the vertex buffer layout descriptor
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Texture coordinates
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Instance data for a single sprite
#[derive(Debug, Clone)]
pub struct SpriteInstance {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub texture: AssetId,
    pub z_index: i32,
}

impl SpriteInstance {
    pub fn new(position: Vec2, size: Vec2, texture: AssetId) -> Self {
        Self {
            position,
            size,
            color: Color::WHITE,
            texture,
            z_index: 0,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

/// Batch of sprites for rendering
pub struct SpriteBatch {
    sprites: Vec<SpriteInstance>,
}

impl SpriteBatch {
    /// Create a new empty sprite batch
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    /// Add a sprite to the batch
    pub fn add(&mut self, sprite: SpriteInstance) {
        self.sprites.push(sprite);
    }

    /// Clear all sprites from the batch
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Get the number of sprites in the batch
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Sort sprites by texture (for batching) then z-index (for layering)
    pub fn sort(&mut self) {
        // Sort by texture first to group sprites for efficient batching,
        // then by z_index for proper layering within each texture group
        self.sprites.sort_by_key(|s| (s.texture.0, s.z_index));
    }

    /// Get an iterator over the sprites
    pub fn iter(&self) -> impl Iterator<Item = &SpriteInstance> {
        self.sprites.iter()
    }

    /// Generate vertices for a sprite (2 triangles = 6 vertices)
    pub fn generate_vertices(sprite: &SpriteInstance) -> [SpriteVertex; 6] {
        let half_width = sprite.size.x / 2.0;
        let half_height = sprite.size.y / 2.0;

        let x0 = sprite.position.x - half_width;
        let y0 = sprite.position.y - half_height;
        let x1 = sprite.position.x + half_width;
        let y1 = sprite.position.y + half_height;

        let color = sprite.color.to_array();

        // Two triangles forming a quad
        // Triangle 1: top-left, bottom-left, bottom-right
        // Triangle 2: top-left, bottom-right, top-right
        [
            // Triangle 1
            SpriteVertex {
                position: [x0, y1], // top-left
                tex_coords: [0.0, 0.0],
                color,
            },
            SpriteVertex {
                position: [x0, y0], // bottom-left
                tex_coords: [0.0, 1.0],
                color,
            },
            SpriteVertex {
                position: [x1, y0], // bottom-right
                tex_coords: [1.0, 1.0],
                color,
            },
            // Triangle 2
            SpriteVertex {
                position: [x0, y1], // top-left
                tex_coords: [0.0, 0.0],
                color,
            },
            SpriteVertex {
                position: [x1, y0], // bottom-right
                tex_coords: [1.0, 1.0],
                color,
            },
            SpriteVertex {
                position: [x1, y1], // top-right
                tex_coords: [1.0, 0.0],
                color,
            },
        ]
    }
}

impl Default for SpriteBatch {
    fn default() -> Self {
        Self::new()
    }
}
