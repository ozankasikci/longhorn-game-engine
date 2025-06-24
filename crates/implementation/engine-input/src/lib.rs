//! Input system for the mobile game engine
//!
//! This crate handles keyboard, mouse, touch, and gamepad input
//! across different platforms.

pub mod events;
pub mod gamepad;
pub mod keyboard;
pub mod manager;
pub mod mouse;
pub mod touch;

pub use events::{InputEvent, KeyboardEvent, MouseEvent, TouchEvent};
pub use keyboard::KeyCode;
pub use manager::InputManager;
pub use mouse::{MouseButton, MouseState};
pub use touch::{TouchInput, TouchPhase};

/// Input system errors
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("Failed to initialize input system")]
    Initialization,
    #[error("Unsupported input device: {0}")]
    UnsupportedDevice(String),
    #[error("Input processing error: {0}")]
    ProcessingError(String),
}

/// Input system result type
pub type InputResult<T> = Result<T, InputError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_manager_creation() {
        // Placeholder test
        assert!(true);
    }
}
