//! Input event types

use crate::{KeyCode, MouseButton, TouchInput};

/// Generic input event
#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    Touch(TouchEvent),
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: KeyCode,
    pub pressed: bool,
}

/// Mouse event
#[derive(Debug, Clone)]
pub enum MouseEvent {
    Button { button: MouseButton, pressed: bool },
    Move { position: (f32, f32) },
    Wheel { delta: (f32, f32) },
}

/// Touch event
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub touch: TouchInput,
}