/// RGBA color with floating-point components (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// White color
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Black color
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Red color
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Green color
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// Blue color
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Transparent color
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Create a new color from RGBA components (0.0-1.0)
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color from RGB components (0.0-1.0), with alpha = 1.0
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from 8-bit RGBA values (0-255)
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Convert to array [r, g, b, a]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert to wgpu::Color
    pub fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_constructors() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);

        let rgb_color = Color::rgb(0.1, 0.2, 0.3);
        assert_eq!(rgb_color.r, 0.1);
        assert_eq!(rgb_color.g, 0.2);
        assert_eq!(rgb_color.b, 0.3);
        assert_eq!(rgb_color.a, 1.0);
    }

    #[test]
    fn test_from_rgba8() {
        let color = Color::from_rgba8(255, 128, 64, 32);
        assert_eq!(color.r, 1.0);
        assert!((color.g - 0.502).abs() < 0.01); // 128/255 ≈ 0.502
        assert!((color.b - 0.251).abs() < 0.01); // 64/255 ≈ 0.251
        assert!((color.a - 0.125).abs() < 0.01); // 32/255 ≈ 0.125
    }

    #[test]
    fn test_to_array() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);
        let array = color.to_array();
        assert_eq!(array, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn test_to_wgpu() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        let wgpu_color = color.to_wgpu();
        assert_eq!(wgpu_color.r, 0.5);
        assert_eq!(wgpu_color.g, 0.6);
        assert_eq!(wgpu_color.b, 0.7);
        assert_eq!(wgpu_color.a, 0.8);
    }
}
