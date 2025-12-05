use glam::{Mat4, Vec2, Vec3};

/// 2D camera with position, zoom, and viewport size
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec2,
    /// Camera zoom level (1.0 = normal, >1.0 = zoomed in, <1.0 = zoomed out)
    pub zoom: f32,
    /// Viewport size in pixels
    pub viewport_size: Vec2,
}

impl Camera {
    /// Create a new camera with the given viewport dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(width, height),
        }
    }

    /// Get the view-projection matrix for this camera
    pub fn view_projection(&self) -> Mat4 {
        // Calculate the orthographic projection based on viewport size and zoom
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        // Create orthographic projection
        let projection = Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -1.0,
            1.0,
        );

        // Create view matrix (translate to camera position)
        let view = Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));

        projection * view
    }

    /// Get the visible bounds in world space [min_x, min_y, max_x, max_y]
    pub fn visible_bounds(&self) -> [f32; 4] {
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        [
            self.position.x - half_width,
            self.position.y - half_height,
            self.position.x + half_width,
            self.position.y + half_height,
        ]
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // Convert from screen space (0,0 at top-left) to NDC space (-1 to 1)
        let ndc_x = (screen_pos.x / self.viewport_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / self.viewport_size.y) * 2.0;

        // Apply zoom and camera position
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        Vec2::new(
            self.position.x + ndc_x * half_width,
            self.position.y + ndc_y * half_height,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;

        // Convert from world space to NDC space
        let ndc_x = (world_pos.x - self.position.x) / half_width;
        let ndc_y = (world_pos.y - self.position.y) / half_height;

        // Convert from NDC space to screen space
        Vec2::new(
            (ndc_x + 1.0) * self.viewport_size.x / 2.0,
            (1.0 - ndc_y) * self.viewport_size.y / 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_new() {
        let camera = Camera::new(800.0, 600.0);
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.viewport_size, Vec2::new(800.0, 600.0));
    }

    #[test]
    fn test_visible_bounds() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.position = Vec2::new(100.0, 50.0);
        camera.zoom = 1.0;

        let bounds = camera.visible_bounds();
        assert_eq!(bounds[0], 100.0 - 400.0); // min_x
        assert_eq!(bounds[1], 50.0 - 300.0); // min_y
        assert_eq!(bounds[2], 100.0 + 400.0); // max_x
        assert_eq!(bounds[3], 50.0 + 300.0); // max_y
    }

    #[test]
    fn test_visible_bounds_with_zoom() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.position = Vec2::ZERO;
        camera.zoom = 2.0;

        let bounds = camera.visible_bounds();
        assert_eq!(bounds[0], -200.0); // min_x (half the width due to 2x zoom)
        assert_eq!(bounds[1], -150.0); // min_y
        assert_eq!(bounds[2], 200.0); // max_x
        assert_eq!(bounds[3], 150.0); // max_y
    }

    #[test]
    fn test_screen_to_world() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.position = Vec2::ZERO;
        camera.zoom = 1.0;

        // Center of screen should map to camera position
        let center = camera.screen_to_world(Vec2::new(400.0, 300.0));
        assert!((center.x - 0.0).abs() < 0.1);
        assert!((center.y - 0.0).abs() < 0.1);

        // Top-left corner
        let top_left = camera.screen_to_world(Vec2::new(0.0, 0.0));
        assert!((top_left.x - (-400.0)).abs() < 0.1);
        assert!((top_left.y - 300.0).abs() < 0.1);

        // Bottom-right corner
        let bottom_right = camera.screen_to_world(Vec2::new(800.0, 600.0));
        assert!((bottom_right.x - 400.0).abs() < 0.1);
        assert!((bottom_right.y - (-300.0)).abs() < 0.1);
    }

    #[test]
    fn test_world_to_screen() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.position = Vec2::ZERO;
        camera.zoom = 1.0;

        // Camera position should map to center of screen
        let center = camera.world_to_screen(Vec2::ZERO);
        assert!((center.x - 400.0).abs() < 0.1);
        assert!((center.y - 300.0).abs() < 0.1);

        // Top-left of visible area
        let top_left = camera.world_to_screen(Vec2::new(-400.0, 300.0));
        assert!((top_left.x - 0.0).abs() < 0.1);
        assert!((top_left.y - 0.0).abs() < 0.1);

        // Bottom-right of visible area
        let bottom_right = camera.world_to_screen(Vec2::new(400.0, -300.0));
        assert!((bottom_right.x - 800.0).abs() < 0.1);
        assert!((bottom_right.y - 600.0).abs() < 0.1);
    }

    #[test]
    fn test_coordinate_conversion_roundtrip() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.position = Vec2::new(100.0, 50.0);
        camera.zoom = 1.5;

        let screen_pos = Vec2::new(250.0, 180.0);
        let world_pos = camera.screen_to_world(screen_pos);
        let back_to_screen = camera.world_to_screen(world_pos);

        assert!((back_to_screen.x - screen_pos.x).abs() < 0.1);
        assert!((back_to_screen.y - screen_pos.y).abs() < 0.1);
    }
}
