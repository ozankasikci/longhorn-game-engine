//! Viewport management and coordinate transformations

use crate::{CameraError, Result};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Viewport configuration for camera rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    /// Create a new viewport
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Create a viewport with offset
    pub fn with_offset(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Get viewport center
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            self.x as f32 + self.width as f32 * 0.5,
            self.y as f32 + self.height as f32 * 0.5,
        )
    }

    /// Check if point is inside viewport
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.x as f32
            && point.x < (self.x + self.width) as f32
            && point.y >= self.y as f32
            && point.y < (self.y + self.height) as f32
    }

    /// Resize viewport maintaining aspect ratio
    pub fn resize_maintaining_aspect(&mut self, new_width: u32, new_height: u32) {
        let current_aspect = self.aspect_ratio();
        let new_aspect = new_width as f32 / new_height as f32;

        if new_aspect > current_aspect {
            // Wider - letterbox horizontally
            self.height = new_height;
            self.width = (new_height as f32 * current_aspect) as u32;
            self.x = (new_width - self.width) / 2;
            self.y = 0;
        } else {
            // Taller - letterbox vertically
            self.width = new_width;
            self.height = (new_width as f32 / current_aspect) as u32;
            self.x = 0;
            self.y = (new_height - self.height) / 2;
        }
    }

    /// Convert screen coordinates to normalized device coordinates (-1 to 1)
    pub fn screen_to_ndc(&self, screen_pos: Vec2) -> Vec2 {
        Vec2::new(
            (screen_pos.x - self.x as f32) / self.width as f32 * 2.0 - 1.0,
            1.0 - (screen_pos.y - self.y as f32) / self.height as f32 * 2.0,
        )
    }

    /// Convert normalized device coordinates to screen coordinates
    pub fn ndc_to_screen(&self, ndc: Vec2) -> Vec2 {
        Vec2::new(
            (ndc.x + 1.0) * 0.5 * self.width as f32 + self.x as f32,
            (1.0 - ndc.y) * 0.5 * self.height as f32 + self.y as f32,
        )
    }

    /// Validate viewport dimensions
    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(CameraError::InvalidViewport(self.width, self.height));
        }
        Ok(())
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

/// Utility for viewport coordinate transformations
#[derive(Debug, Clone)]
pub struct ViewportTransform {
    viewport: Viewport,
}

impl ViewportTransform {
    pub fn new(viewport: Viewport) -> Self {
        Self { viewport }
    }

    /// Transform screen coordinates to viewport-relative coordinates
    pub fn screen_to_viewport(&self, screen_pos: Vec2) -> Vec2 {
        Vec2::new(
            screen_pos.x - self.viewport.x as f32,
            screen_pos.y - self.viewport.y as f32,
        )
    }

    /// Transform viewport-relative coordinates to screen coordinates
    pub fn viewport_to_screen(&self, viewport_pos: Vec2) -> Vec2 {
        Vec2::new(
            viewport_pos.x + self.viewport.x as f32,
            viewport_pos.y + self.viewport.y as f32,
        )
    }

    /// Get viewport
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Update viewport
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_aspect_ratio() {
        let viewport = Viewport::new(800, 600);
        assert!((viewport.aspect_ratio() - 4.0 / 3.0).abs() < f32::EPSILON);

        let square = Viewport::new(500, 500);
        assert!((square.aspect_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_viewport_contains_point() {
        let viewport = Viewport::with_offset(100, 100, 400, 300);

        assert!(viewport.contains_point(Vec2::new(200.0, 200.0)));
        assert!(viewport.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!viewport.contains_point(Vec2::new(50.0, 50.0)));
        assert!(!viewport.contains_point(Vec2::new(600.0, 500.0)));
    }

    #[test]
    fn test_ndc_conversion() {
        let viewport = Viewport::new(800, 600);

        // Center should map to origin
        let center = Vec2::new(400.0, 300.0);
        let ndc = viewport.screen_to_ndc(center);
        assert!((ndc.x - 0.0).abs() < f32::EPSILON);
        assert!((ndc.y - 0.0).abs() < f32::EPSILON);

        // Top-left corner
        let top_left = Vec2::new(0.0, 0.0);
        let ndc = viewport.screen_to_ndc(top_left);
        assert!((ndc.x - (-1.0)).abs() < f32::EPSILON);
        assert!((ndc.y - 1.0).abs() < f32::EPSILON);

        // Bottom-right corner
        let bottom_right = Vec2::new(800.0, 600.0);
        let ndc = viewport.screen_to_ndc(bottom_right);
        assert!((ndc.x - 1.0).abs() < f32::EPSILON);
        assert!((ndc.y - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_viewport_validation() {
        let valid = Viewport::new(800, 600);
        assert!(valid.validate().is_ok());

        let invalid_width = Viewport::new(0, 600);
        assert!(invalid_width.validate().is_err());

        let invalid_height = Viewport::new(800, 0);
        assert!(invalid_height.validate().is_err());
    }
}
