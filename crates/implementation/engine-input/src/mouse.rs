//! Mouse input handling

/// Mouse button enumeration
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Mouse state
#[derive(Debug, Clone)]
pub struct MouseState {
    pub position: (f32, f32),
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub middle_pressed: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseState {
    /// Create a new mouse state
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            left_pressed: false,
            right_pressed: false,
            middle_pressed: false,
        }
    }

    /// Check if a mouse button is pressed
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.left_pressed,
            MouseButton::Right => self.right_pressed,
            MouseButton::Middle => self.middle_pressed,
            MouseButton::Other(_) => false, // Not implemented yet
        }
    }

    /// Get mouse position
    pub fn position(&self) -> (f32, f32) {
        self.position
    }
}
