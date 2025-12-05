use std::io;

/// Texture data loaded from an image file
#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA format
}

impl TextureData {
    /// Load texture data from bytes (PNG/JPEG)
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self::from_image(img))
    }

    /// Create texture data from an image
    pub fn from_image(img: image::DynamicImage) -> Self {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Self {
            width,
            height,
            pixels: rgba.into_raw(),
        }
    }

    /// Get the number of bytes per row
    pub fn bytes_per_row(&self) -> u32 {
        self.width * 4 // 4 bytes per pixel (RGBA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_from_image() {
        // Create a simple 2x2 RGBA image
        let img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(2, 2, vec![
                255, 0, 0, 255,   // Red
                0, 255, 0, 255,   // Green
                0, 0, 255, 255,   // Blue
                255, 255, 255, 255, // White
            ]).unwrap()
        );

        let texture = TextureData::from_image(img);

        assert_eq!(texture.width, 2);
        assert_eq!(texture.height, 2);
        assert_eq!(texture.pixels.len(), 16); // 2x2 * 4 bytes per pixel
        assert_eq!(texture.bytes_per_row(), 8);
    }

    #[test]
    fn test_texture_from_bytes_png() {
        // Create a small PNG in memory
        let mut png_bytes = Vec::new();
        let img = image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();

        let texture = TextureData::from_bytes(&png_bytes).unwrap();

        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.pixels.len(), 4);
    }

    #[test]
    fn test_texture_from_invalid_bytes() {
        let invalid_bytes = vec![0, 1, 2, 3];
        let result = TextureData::from_bytes(&invalid_bytes);
        assert!(result.is_err());
    }
}
