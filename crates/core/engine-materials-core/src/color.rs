//! Color management and utilities

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Color representation with RGBA components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Color space enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    Linear,
    Srgb,
    Hsl,
    Hsv,
}

impl Color {
    /// Predefined colors
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

    /// Create a new color
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB values (alpha = 1.0)
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Create a color from a single grayscale value
    pub const fn gray(value: f32) -> Self {
        Self::new(value, value, value, 1.0)
    }

    /// Create a color from HSL values
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(r + m, g + m, b + m, 1.0)
    }

    /// Create a color from hexadecimal value
    pub fn hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    /// Convert to linear color space (assuming input is sRGB)
    pub fn to_linear(&self) -> Self {
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        Self::new(
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            self.a,
        )
    }

    /// Convert to sRGB color space (assuming input is linear)
    pub fn to_srgb(&self) -> Self {
        fn linear_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }

        Self::new(
            linear_to_srgb(self.r),
            linear_to_srgb(self.g),
            linear_to_srgb(self.b),
            self.a,
        )
    }

    /// Convert to HSL color space
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let max = self.r.max(self.g.max(self.b));
        let min = self.r.min(self.g.min(self.b));
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return (0.0, 0.0, l);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };

        (h, s, l)
    }

    /// Multiply color by a scalar
    pub fn scale(&self, factor: f32) -> Self {
        Self::new(self.r * factor, self.g * factor, self.b * factor, self.a)
    }

    /// Multiply two colors component-wise
    pub fn multiply(&self, other: &Self) -> Self {
        Self::new(
            self.r * other.r,
            self.g * other.g,
            self.b * other.b,
            self.a * other.a,
        )
    }

    /// Add two colors component-wise
    pub fn add(&self, other: &Self) -> Self {
        Self::new(
            self.r + other.r,
            self.g + other.g,
            self.b + other.b,
            self.a + other.a,
        )
    }

    /// Linearly interpolate between two colors
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    /// Get the luminance of the color
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Convert to Vec3 (RGB only)
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }

    /// Convert to array [r, g, b, a]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert to array [r, g, b]
    pub fn to_rgb_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<[f32; 4]> for Color {
    fn from(array: [f32; 4]) -> Self {
        Self::new(array[0], array[1], array[2], array[3])
    }
}

impl From<[f32; 3]> for Color {
    fn from(array: [f32; 3]) -> Self {
        Self::rgb(array[0], array[1], array[2])
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self::new(r, g, b, a)
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<Vec3> for Color {
    fn from(vec: Vec3) -> Self {
        Self::rgb(vec.x, vec.y, vec.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(0.1, 0.2, 0.3);
        assert_eq!(color.r, 0.1);
        assert_eq!(color.g, 0.2);
        assert_eq!(color.b, 0.3);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_gray() {
        let color = Color::gray(0.5);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_predefined_colors() {
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::YELLOW, Color::new(1.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::CYAN, Color::new(0.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::MAGENTA, Color::new(1.0, 0.0, 1.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::hex(0xFF0000); // Red
        assert!((color.r - 1.0).abs() < 0.001);
        assert!((color.g - 0.0).abs() < 0.001);
        assert!((color.b - 0.0).abs() < 0.001);

        let color = Color::hex(0x00FF00); // Green
        assert!((color.r - 0.0).abs() < 0.001);
        assert!((color.g - 1.0).abs() < 0.001);
        assert!((color.b - 0.0).abs() < 0.001);

        let color = Color::hex(0x0000FF); // Blue
        assert!((color.r - 0.0).abs() < 0.001);
        assert!((color.g - 0.0).abs() < 0.001);
        assert!((color.b - 1.0).abs() < 0.001);

        let color = Color::hex(0x808080); // Gray
        assert!((color.r - 0.502).abs() < 0.01);
        assert!((color.g - 0.502).abs() < 0.01);
        assert!((color.b - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_color_hsl() {
        // Test red
        let red = Color::hsl(0.0, 1.0, 0.5);
        assert!((red.r - 1.0).abs() < 0.001);
        assert!((red.g - 0.0).abs() < 0.001);
        assert!((red.b - 0.0).abs() < 0.001);

        // Test green
        let green = Color::hsl(120.0, 1.0, 0.5);
        assert!((green.r - 0.0).abs() < 0.001);
        assert!((green.g - 1.0).abs() < 0.001);
        assert!((green.b - 0.0).abs() < 0.001);

        // Test blue
        let blue = Color::hsl(240.0, 1.0, 0.5);
        assert!((blue.r - 0.0).abs() < 0.001);
        assert!((blue.g - 0.0).abs() < 0.001);
        assert!((blue.b - 1.0).abs() < 0.001);

        // Test gray (no saturation)
        let gray = Color::hsl(0.0, 0.0, 0.5);
        assert!((gray.r - 0.5).abs() < 0.001);
        assert!((gray.g - 0.5).abs() < 0.001);
        assert!((gray.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_color_to_hsl() {
        // Test red
        let (h, s, l) = Color::RED.to_hsl();
        assert!((h - 0.0).abs() < 0.001);
        assert!((s - 1.0).abs() < 0.001);
        assert!((l - 0.5).abs() < 0.001);

        // Test gray
        let gray = Color::gray(0.5);
        let (h, s, l) = gray.to_hsl();
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert!((l - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_srgb_linear_conversion() {
        // Test conversion at boundary values
        let color = Color::rgb(0.04045, 0.5, 1.0);
        let linear = color.to_linear();
        assert!((linear.r - 0.04045 / 12.92).abs() < 0.0001);

        let srgb = linear.to_srgb();
        assert!((srgb.r - color.r).abs() < 0.0001);
        assert!((srgb.g - color.g).abs() < 0.0001);
        assert!((srgb.b - color.b).abs() < 0.0001);
    }

    #[test]
    fn test_color_operations() {
        let c1 = Color::rgb(0.5, 0.5, 0.5);
        let c2 = Color::rgb(0.2, 0.3, 0.4);

        // Test scale
        let scaled = c1.scale(2.0);
        assert_eq!(scaled.r, 1.0);
        assert_eq!(scaled.g, 1.0);
        assert_eq!(scaled.b, 1.0);

        // Test multiply
        let multiplied = c1.multiply(&c2);
        assert!((multiplied.r - 0.1).abs() < 0.001);
        assert!((multiplied.g - 0.15).abs() < 0.001);
        assert!((multiplied.b - 0.2).abs() < 0.001);

        // Test add
        let added = c1.add(&c2);
        assert!((added.r - 0.7).abs() < 0.001);
        assert!((added.g - 0.8).abs() < 0.001);
        assert!((added.b - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_color_lerp() {
        let c1 = Color::BLACK;
        let c2 = Color::WHITE;

        let mid = c1.lerp(&c2, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);

        let same = c1.lerp(&c2, 0.0);
        assert_eq!(same, c1);

        let other = c1.lerp(&c2, 1.0);
        assert_eq!(other, c2);

        // Test clamping
        let clamped = c1.lerp(&c2, 1.5);
        assert_eq!(clamped, c2);
    }

    #[test]
    fn test_luminance() {
        assert!((Color::WHITE.luminance() - 1.0).abs() < 0.001);
        assert!((Color::BLACK.luminance() - 0.0).abs() < 0.001);

        // Standard luminance calculation
        let color = Color::rgb(0.5, 0.5, 0.5);
        let expected = 0.2126 * 0.5 + 0.7152 * 0.5 + 0.0722 * 0.5;
        assert!((color.luminance() - expected).abs() < 0.001);
    }

    #[test]
    fn test_conversions() {
        let color = Color::rgb(0.1, 0.2, 0.3);

        // Test to_vec3
        let vec = color.to_vec3();
        assert_eq!(vec.x, 0.1);
        assert_eq!(vec.y, 0.2);
        assert_eq!(vec.z, 0.3);

        // Test to_array
        let arr = color.to_array();
        assert_eq!(arr, [0.1, 0.2, 0.3, 1.0]);

        // Test to_rgb_array
        let rgb_arr = color.to_rgb_array();
        assert_eq!(rgb_arr, [0.1, 0.2, 0.3]);

        // Test from conversions
        assert_eq!(
            Color::from([0.1, 0.2, 0.3, 0.4]),
            Color::new(0.1, 0.2, 0.3, 0.4)
        );
        assert_eq!(Color::from([0.1, 0.2, 0.3]), Color::rgb(0.1, 0.2, 0.3));
        assert_eq!(
            Color::from((0.1, 0.2, 0.3, 0.4)),
            Color::new(0.1, 0.2, 0.3, 0.4)
        );
        assert_eq!(Color::from((0.1, 0.2, 0.3)), Color::rgb(0.1, 0.2, 0.3));
        assert_eq!(
            Color::from(Vec3::new(0.1, 0.2, 0.3)),
            Color::rgb(0.1, 0.2, 0.3)
        );
    }

    #[test]
    fn test_color_space_enum() {
        assert_eq!(ColorSpace::Linear, ColorSpace::Linear);
        assert_ne!(ColorSpace::Linear, ColorSpace::Srgb);
        assert_ne!(ColorSpace::Hsl, ColorSpace::Hsv);
    }
}
