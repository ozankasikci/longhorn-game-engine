use glam::Vec2;
use longhorn_engine::Transform;

#[derive(Debug, Clone)]
pub struct EditorCamera {
    pub transform: Transform,
    pub zoom: f32,
}

#[derive(Debug, Default)]
pub struct CameraInput {
    pub mmb_held: bool,
    pub rmb_held: bool,
    pub mouse_delta: Vec2,
    pub scroll_delta: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            transform: Transform {
                position: Vec2::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
            zoom: 1.0,
        }
    }
}

impl EditorCamera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_input(&mut self, input: &CameraInput) {
        // Middle mouse button - Pan
        if input.mmb_held {
            let pan_speed = self.pan_speed();
            self.transform.position.x -= input.mouse_delta.x * pan_speed;
            self.transform.position.y += input.mouse_delta.y * pan_speed;
        }

        // Scroll - Zoom
        if input.scroll_delta != 0.0 {
            self.zoom *= 1.0 + input.scroll_delta * 0.1;
            self.zoom = self.zoom.clamp(0.1, 10.0);
        }
    }

    fn pan_speed(&self) -> f32 {
        // Pan speed scales with zoom level
        self.zoom * 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_camera_default() {
        let camera = EditorCamera::default();
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.transform.position, Vec2::ZERO);
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = EditorCamera::default();
        let input = CameraInput {
            mmb_held: true,
            mouse_delta: Vec2::new(10.0, 5.0),
            ..Default::default()
        };

        camera.handle_input(&input);

        // Camera should move opposite to mouse delta
        assert!(camera.transform.position.x < 0.0);
        assert!(camera.transform.position.y > 0.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = EditorCamera::default();
        let input = CameraInput {
            scroll_delta: 1.0,
            ..Default::default()
        };

        camera.handle_input(&input);

        assert!(camera.zoom > 1.0);
        assert!(camera.zoom <= 10.0); // Clamped
    }
}
