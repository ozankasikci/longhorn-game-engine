use crate::{MipmapLevel, TextureData, TextureError, TextureFormat};

pub struct MipmapGenerator;

impl MipmapGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_mipmaps(&self, texture: &mut TextureData) -> Result<(), TextureError> {
        let mut current_width = texture.width;
        let mut current_height = texture.height;
        let mut current_data = texture.data.clone();

        texture.mipmaps.clear();

        while current_width > 1 || current_height > 1 {
            current_width = (current_width / 2).max(1);
            current_height = (current_height / 2).max(1);

            let mip_data = self.downsample(
                &current_data,
                current_width * 2,
                current_height * 2,
                current_width,
                current_height,
                4, // Assuming RGBA8
            );

            texture.mipmaps.push(MipmapLevel {
                width: current_width,
                height: current_height,
                data: mip_data.clone(),
            });

            current_data = mip_data;
        }

        Ok(())
    }

    fn downsample(
        &self,
        src: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
        bytes_per_pixel: usize,
    ) -> Vec<u8> {
        let mut dst = vec![0u8; (dst_width * dst_height) as usize * bytes_per_pixel];

        for y in 0..dst_height {
            for x in 0..dst_width {
                let src_x = x * 2;
                let src_y = y * 2;

                for c in 0..bytes_per_pixel {
                    let mut sum = 0u32;
                    let mut count = 0u32;

                    // Sample 2x2 block
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = (src_x + dx).min(src_width - 1);
                            let sy = (src_y + dy).min(src_height - 1);
                            let src_idx = ((sy * src_width + sx) as usize * bytes_per_pixel) + c;

                            if src_idx < src.len() {
                                sum += src[src_idx] as u32;
                                count += 1;
                            }
                        }
                    }

                    let dst_idx = ((y * dst_width + x) as usize * bytes_per_pixel) + c;
                    dst[dst_idx] = (sum / count.max(1)) as u8;
                }
            }
        }

        dst
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResizeFilter {
    Nearest,
    Bilinear,
    Lanczos3,
}

pub struct ResizeOptions {
    pub max_width: u32,
    pub max_height: u32,
    pub maintain_aspect_ratio: bool,
    pub filter: ResizeFilter,
}

pub struct TextureResizer;

impl TextureResizer {
    pub fn new() -> Self {
        Self
    }

    pub fn resize(
        &self,
        texture: &mut TextureData,
        options: &ResizeOptions,
    ) -> Result<(), TextureError> {
        let (new_width, new_height) = if options.maintain_aspect_ratio {
            let aspect = texture.width as f32 / texture.height as f32;

            if texture.width > options.max_width {
                let w = options.max_width;
                let h = (w as f32 / aspect) as u32;
                (w, h.min(options.max_height))
            } else if texture.height > options.max_height {
                let h = options.max_height;
                let w = (h as f32 * aspect) as u32;
                (w.min(options.max_width), h)
            } else {
                (texture.width, texture.height)
            }
        } else {
            (
                texture.width.min(options.max_width),
                texture.height.min(options.max_height),
            )
        };

        if new_width == texture.width && new_height == texture.height {
            return Ok(());
        }

        // Simple bilinear resize for now
        let bytes_per_pixel = match texture.format {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Rgb8 => 3,
            TextureFormat::Rg8 => 2,
            TextureFormat::R8 => 1,
            _ => return Err(TextureError::UnsupportedFormat),
        };

        let mut new_data = vec![0u8; (new_width * new_height) as usize * bytes_per_pixel];

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * texture.width as f32 / new_width as f32) as u32;
                let src_y = (y as f32 * texture.height as f32 / new_height as f32) as u32;

                let src_idx = ((src_y * texture.width + src_x) as usize) * bytes_per_pixel;
                let dst_idx = ((y * new_width + x) as usize) * bytes_per_pixel;

                for c in 0..bytes_per_pixel {
                    new_data[dst_idx + c] = texture.data[src_idx + c];
                }
            }
        }

        texture.width = new_width;
        texture.height = new_height;
        texture.data = new_data;
        texture.mipmaps.clear();

        Ok(())
    }
}

pub struct NormalMapOptions {
    pub strength: f32,
    pub invert_y: bool,
}

pub struct NormalMapProcessor;

impl NormalMapProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn height_to_normal(
        &self,
        texture: &mut TextureData,
        options: &NormalMapOptions,
    ) -> Result<(), TextureError> {
        if texture.format != TextureFormat::R8 {
            return Err(TextureError::InvalidData(
                "Expected R8 height map".to_string(),
            ));
        }

        let width = texture.width as i32;
        let height = texture.height as i32;
        let mut normal_data = vec![0u8; (width * height * 4) as usize];

        for y in 0..height {
            for x in 0..width {
                // Sample neighboring heights
                let center = self.sample_height(&texture.data, x, y, width, height);
                let left = self.sample_height(&texture.data, x - 1, y, width, height);
                let right = self.sample_height(&texture.data, x + 1, y, width, height);
                let up = self.sample_height(&texture.data, x, y - 1, width, height);
                let down = self.sample_height(&texture.data, x, y + 1, width, height);

                // Calculate gradients
                let dx = (right - left) * options.strength;
                let dy = (down - up) * options.strength;

                // Create normal vector
                let mut nx = -dx;
                let mut ny = if options.invert_y { dy } else { -dy };
                let nz = 1.0;

                // Normalize
                let len = (nx * nx + ny * ny + nz * nz).sqrt();
                nx /= len;
                ny /= len;
                let nz = nz / len;

                // Convert to 0-255 range
                let idx = ((y * width + x) * 4) as usize;
                normal_data[idx] = ((nx * 0.5 + 0.5) * 255.0) as u8;
                normal_data[idx + 1] = ((ny * 0.5 + 0.5) * 255.0) as u8;
                normal_data[idx + 2] = ((nz * 0.5 + 0.5) * 255.0) as u8;
                normal_data[idx + 3] = 255;
            }
        }

        texture.format = TextureFormat::Rgba8;
        texture.data = normal_data;

        Ok(())
    }

    fn sample_height(&self, data: &[u8], x: i32, y: i32, width: i32, height: i32) -> f32 {
        let x = x.clamp(0, width - 1);
        let y = y.clamp(0, height - 1);
        let idx = (y * width + x) as usize;
        data[idx] as f32 / 255.0
    }
}

pub struct FormatConverter;

impl FormatConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(
        &self,
        texture: &TextureData,
        target_format: TextureFormat,
    ) -> Result<TextureData, TextureError> {
        if texture.format == target_format {
            return Ok(texture.clone());
        }

        // For now, only support RGB8 to RGBA8 conversion
        if texture.format == TextureFormat::Rgb8 && target_format == TextureFormat::Rgba8 {
            let mut rgba_data = Vec::with_capacity(texture.data.len() * 4 / 3);

            for chunk in texture.data.chunks_exact(3) {
                rgba_data.push(chunk[0]); // R
                rgba_data.push(chunk[1]); // G
                rgba_data.push(chunk[2]); // B
                rgba_data.push(255); // A
            }

            return Ok(TextureData {
                width: texture.width,
                height: texture.height,
                format: TextureFormat::Rgba8,
                data: rgba_data,
                mipmaps: vec![],
            });
        }

        Err(TextureError::UnsupportedFormat)
    }
}
