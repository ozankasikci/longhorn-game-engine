use crate::{ImportContext, TextureData, TextureError, TextureFormat};

pub struct TGAImporter;

impl Default for TGAImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TGAImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn import(
        &self,
        data: &[u8],
        _context: &ImportContext,
    ) -> Result<TextureData, TextureError> {
        if data.len() < 18 {
            return Err(TextureError::InvalidData(
                "TGA header too short".to_string(),
            ));
        }

        // Parse TGA header
        let id_length = data[0];
        let color_map_type = data[1];
        let image_type = data[2];

        // We only support uncompressed RGB/RGBA for now
        if color_map_type != 0 || (image_type != 2 && image_type != 3) {
            return Err(TextureError::UnsupportedFormat);
        }

        let width = u16::from_le_bytes([data[12], data[13]]) as u32;
        let height = u16::from_le_bytes([data[14], data[15]]) as u32;
        let bits_per_pixel = data[16];

        let header_size = 18 + id_length as usize;

        if data.len() < header_size {
            return Err(TextureError::InvalidData("TGA data too short".to_string()));
        }

        // Convert BGR(A) to RGBA
        let pixel_data = &data[header_size..];
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

        match bits_per_pixel {
            24 => {
                // BGR to RGBA
                for chunk in pixel_data.chunks_exact(3) {
                    rgba_data.push(chunk[2]); // R
                    rgba_data.push(chunk[1]); // G
                    rgba_data.push(chunk[0]); // B
                    rgba_data.push(255); // A
                }
            }
            32 => {
                // BGRA to RGBA
                for chunk in pixel_data.chunks_exact(4) {
                    rgba_data.push(chunk[2]); // R
                    rgba_data.push(chunk[1]); // G
                    rgba_data.push(chunk[0]); // B
                    rgba_data.push(chunk[3]); // A
                }
            }
            _ => return Err(TextureError::UnsupportedFormat),
        }

        Ok(TextureData {
            width,
            height,
            format: TextureFormat::Rgba8,
            data: rgba_data,
            mipmaps: vec![],
        })
    }
}
