//! Input manager for coordinating all input systems

use crate::{InputResult, InputEvent, KeyCode, MouseButton, MouseState};
use winit::event::{WindowEvent, ElementState, MouseButton as WinitMouseButton, KeyEvent};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};
use std::collections::{HashMap, VecDeque};

/// Central input manager that processes winit events and maintains input state
#[derive(Debug)]
pub struct InputManager {
    /// Queue of input events for this frame
    events: VecDeque<InputEvent>,
    /// Current keyboard state
    keyboard_state: HashMap<KeyCode, bool>,
    /// Current mouse state
    mouse_state: MouseState,
    /// Events from previous frame (for detecting released keys)
    previous_keyboard_state: HashMap<KeyCode, bool>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> InputResult<Self> {
        Ok(Self {
            events: VecDeque::new(),
            keyboard_state: HashMap::new(),
            mouse_state: MouseState::default(),
            previous_keyboard_state: HashMap::new(),
        })
    }

    /// Process a winit window event and convert it to internal input events
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.process_keyboard_event(event);
            },
            WindowEvent::MouseInput { state, button, .. } => {
                self.process_mouse_button(*state, *button);
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.process_mouse_move(position.x as f32, position.y as f32);
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.process_mouse_wheel(delta);
            },
            _ => {}
        }
    }

    /// Update the input system - call this once per frame
    pub fn update(&mut self) {
        // Store current state as previous for next frame
        self.previous_keyboard_state = self.keyboard_state.clone();
        
        // Clear events from previous frame
        self.events.clear();
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keyboard_state.get(&key).copied().unwrap_or(false)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        let current = self.keyboard_state.get(&key).copied().unwrap_or(false);
        let previous = self.previous_keyboard_state.get(&key).copied().unwrap_or(false);
        current && !previous
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        let current = self.keyboard_state.get(&key).copied().unwrap_or(false);
        let previous = self.previous_keyboard_state.get(&key).copied().unwrap_or(false);
        !current && previous
    }

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.mouse_state.left_pressed,
            MouseButton::Right => self.mouse_state.right_pressed,
            MouseButton::Middle => self.mouse_state.middle_pressed,
            MouseButton::Other(_) => false, // Not implemented yet
        }
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_state.position
    }

    /// Get all input events from this frame
    pub fn events(&self) -> impl Iterator<Item = &InputEvent> {
        self.events.iter()
    }

    fn process_keyboard_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            if let Some(key) = winit_key_to_engine_key(keycode) {
                let pressed = event.state == ElementState::Pressed;
                self.keyboard_state.insert(key, pressed);
                
                self.events.push_back(InputEvent::Keyboard(crate::KeyboardEvent {
                    key,
                    pressed,
                }));
            }
        }
    }

    fn process_mouse_button(&mut self, state: ElementState, button: WinitMouseButton) {
        if let Some(engine_button) = winit_mouse_button_to_engine(button) {
            let pressed = state == ElementState::Pressed;
            
            match engine_button {
                MouseButton::Left => self.mouse_state.left_pressed = pressed,
                MouseButton::Right => self.mouse_state.right_pressed = pressed,
                MouseButton::Middle => self.mouse_state.middle_pressed = pressed,
                MouseButton::Other(_) => {} // Not implemented yet
            }

            self.events.push_back(InputEvent::Mouse(crate::MouseEvent::Button {
                button: engine_button,
                pressed,
            }));
        }
    }

    fn process_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_state.position = (x, y);
        self.events.push_back(InputEvent::Mouse(crate::MouseEvent::Move {
            position: (x, y),
        }));
    }

    fn process_mouse_wheel(&mut self, delta: &winit::event::MouseScrollDelta) {
        let delta = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
            winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
        };

        self.events.push_back(InputEvent::Mouse(crate::MouseEvent::Wheel { delta }));
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default InputManager")
    }
}

fn winit_key_to_engine_key(key: WinitKeyCode) -> Option<KeyCode> {
    match key {
        WinitKeyCode::KeyA => Some(KeyCode::A),
        WinitKeyCode::KeyB => Some(KeyCode::B),
        WinitKeyCode::KeyC => Some(KeyCode::C),
        WinitKeyCode::KeyD => Some(KeyCode::D),
        WinitKeyCode::KeyE => Some(KeyCode::E),
        WinitKeyCode::KeyF => Some(KeyCode::F),
        WinitKeyCode::KeyG => Some(KeyCode::G),
        WinitKeyCode::KeyH => Some(KeyCode::H),
        WinitKeyCode::KeyI => Some(KeyCode::I),
        WinitKeyCode::KeyJ => Some(KeyCode::J),
        WinitKeyCode::KeyK => Some(KeyCode::K),
        WinitKeyCode::KeyL => Some(KeyCode::L),
        WinitKeyCode::KeyM => Some(KeyCode::M),
        WinitKeyCode::KeyN => Some(KeyCode::N),
        WinitKeyCode::KeyO => Some(KeyCode::O),
        WinitKeyCode::KeyP => Some(KeyCode::P),
        WinitKeyCode::KeyQ => Some(KeyCode::Q),
        WinitKeyCode::KeyR => Some(KeyCode::R),
        WinitKeyCode::KeyS => Some(KeyCode::S),
        WinitKeyCode::KeyT => Some(KeyCode::T),
        WinitKeyCode::KeyU => Some(KeyCode::U),
        WinitKeyCode::KeyV => Some(KeyCode::V),
        WinitKeyCode::KeyW => Some(KeyCode::W),
        WinitKeyCode::KeyX => Some(KeyCode::X),
        WinitKeyCode::KeyY => Some(KeyCode::Y),
        WinitKeyCode::KeyZ => Some(KeyCode::Z),
        WinitKeyCode::Space => Some(KeyCode::Space),
        WinitKeyCode::Enter => Some(KeyCode::Enter),
        WinitKeyCode::Escape => Some(KeyCode::Escape),
        WinitKeyCode::ShiftLeft => Some(KeyCode::LeftShift),
        WinitKeyCode::ShiftRight => Some(KeyCode::RightShift),
        WinitKeyCode::ControlLeft => Some(KeyCode::LeftControl),
        WinitKeyCode::ControlRight => Some(KeyCode::RightControl),
        WinitKeyCode::AltLeft => Some(KeyCode::LeftAlt),
        WinitKeyCode::AltRight => Some(KeyCode::RightAlt),
        _ => None,
    }
}

fn winit_mouse_button_to_engine(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left => Some(MouseButton::Left),
        WinitMouseButton::Right => Some(MouseButton::Right),
        WinitMouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::{MouseButton as WinitMouseButton};

    #[test]
    fn test_input_manager_creation() {
        let manager = InputManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert_eq!(manager.events.len(), 0);
        assert_eq!(manager.keyboard_state.len(), 0);
    }

    #[test]
    fn test_keyboard_state_management() {
        let mut manager = InputManager::new().unwrap();
        
        // Initially no key should be pressed
        assert!(!manager.is_key_pressed(KeyCode::A));
        assert!(!manager.is_key_just_pressed(KeyCode::A));
        assert!(!manager.is_key_just_released(KeyCode::A));
        
        // Manually set key state (simulating key press processing)
        manager.keyboard_state.insert(KeyCode::A, true);
        
        // Key should be just pressed
        assert!(manager.is_key_pressed(KeyCode::A));
        assert!(manager.is_key_just_pressed(KeyCode::A));
        assert!(!manager.is_key_just_released(KeyCode::A));
        
        // Update to next frame
        manager.update();
        
        // Key should still be pressed but not just pressed
        assert!(manager.is_key_pressed(KeyCode::A));
        assert!(!manager.is_key_just_pressed(KeyCode::A));
        assert!(!manager.is_key_just_released(KeyCode::A));
        
        // Release key A
        manager.keyboard_state.insert(KeyCode::A, false);
        
        // Key should be just released
        assert!(!manager.is_key_pressed(KeyCode::A));
        assert!(!manager.is_key_just_pressed(KeyCode::A));
        assert!(manager.is_key_just_released(KeyCode::A));
    }

    #[test]
    fn test_mouse_state_management() {
        let mut manager = InputManager::new().unwrap();
        
        // Initially no mouse button should be pressed
        assert!(!manager.is_mouse_button_pressed(MouseButton::Left));
        assert!(!manager.is_mouse_button_pressed(MouseButton::Right));
        
        // Manually set mouse state
        manager.mouse_state.left_pressed = true;
        manager.mouse_state.position = (100.0, 200.0);
        
        // Check mouse state
        assert!(manager.is_mouse_button_pressed(MouseButton::Left));
        assert!(!manager.is_mouse_button_pressed(MouseButton::Right));
        assert_eq!(manager.mouse_position(), (100.0, 200.0));
    }

    #[test]
    fn test_update_clears_events() {
        let mut manager = InputManager::new().unwrap();
        
        // Manually add an event
        manager.events.push_back(InputEvent::Keyboard(crate::KeyboardEvent {
            key: KeyCode::A,
            pressed: true,
        }));
        
        assert_eq!(manager.events.len(), 1);
        
        // Update should clear events
        manager.update();
        assert_eq!(manager.events.len(), 0);
    }

    #[test]
    fn test_winit_key_conversion() {
        use winit::keyboard::KeyCode as WinitKeyCode;
        
        // Test key conversion
        assert_eq!(winit_key_to_engine_key(WinitKeyCode::KeyA), Some(KeyCode::A));
        assert_eq!(winit_key_to_engine_key(WinitKeyCode::Space), Some(KeyCode::Space));
        assert_eq!(winit_key_to_engine_key(WinitKeyCode::Enter), Some(KeyCode::Enter));
        assert_eq!(winit_key_to_engine_key(WinitKeyCode::ShiftLeft), Some(KeyCode::LeftShift));
    }

    #[test]
    fn test_winit_mouse_button_conversion() {
        // Test mouse button conversion
        assert_eq!(winit_mouse_button_to_engine(WinitMouseButton::Left), Some(MouseButton::Left));
        assert_eq!(winit_mouse_button_to_engine(WinitMouseButton::Right), Some(MouseButton::Right));
        assert_eq!(winit_mouse_button_to_engine(WinitMouseButton::Middle), Some(MouseButton::Middle));
    }

    #[test]
    fn test_mouse_button_other() {
        let manager = InputManager::new().unwrap();
        
        // Test that Other mouse buttons return false
        assert!(!manager.is_mouse_button_pressed(MouseButton::Other(4)));
    }
}