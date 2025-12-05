use longhorn_renderer::Color;

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Viewport width in pixels
    pub viewport_width: u32,
    /// Viewport height in pixels
    pub viewport_height: u32,
    /// Target frames per second
    pub target_fps: u32,
    /// Clear color (stored as normalized RGBA)
    clear_color: [f32; 4],
}

impl EngineConfig {
    /// Create a new engine config
    pub fn new(viewport_width: u32, viewport_height: u32, target_fps: u32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            target_fps,
            clear_color: [0.2, 0.2, 0.2, 1.0], // Dark gray
        }
    }

    /// Set the clear color from normalized RGBA values
    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.clear_color = [r, g, b, a];
        self
    }

    /// Get the clear color as a renderer Color
    pub fn clear_color(&self) -> Color {
        Color::new(
            self.clear_color[0],
            self.clear_color[1],
            self.clear_color[2],
            self.clear_color[3],
        )
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::new(1280, 720, 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
        assert_eq!(config.target_fps, 60);
    }

    #[test]
    fn test_custom_config() {
        let config = EngineConfig::new(800, 600, 30);
        assert_eq!(config.viewport_width, 800);
        assert_eq!(config.viewport_height, 600);
        assert_eq!(config.target_fps, 30);
    }

    #[test]
    fn test_clear_color() {
        let config = EngineConfig::default().with_clear_color(1.0, 0.0, 0.0, 1.0);
        let color = config.clear_color();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }
}
