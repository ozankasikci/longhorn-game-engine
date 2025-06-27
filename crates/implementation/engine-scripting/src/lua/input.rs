//! Input system integration for Lua scripting

use crate::ScriptError;
use mlua::{Lua, Function as LuaFunction, Table};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

/// Key codes for keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    Space, Enter, Tab, Escape, Backspace, Delete,
    Left, Right, Up, Down,
    Shift, Ctrl, Alt, Cmd,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

impl KeyCode {
    /// Convert key code to string representation
    pub fn to_string(&self) -> String {
        match self {
            KeyCode::A => "A".to_string(),
            KeyCode::B => "B".to_string(),
            KeyCode::C => "C".to_string(),
            KeyCode::D => "D".to_string(),
            KeyCode::E => "E".to_string(),
            KeyCode::F => "F".to_string(),
            KeyCode::G => "G".to_string(),
            KeyCode::H => "H".to_string(),
            KeyCode::I => "I".to_string(),
            KeyCode::J => "J".to_string(),
            KeyCode::K => "K".to_string(),
            KeyCode::L => "L".to_string(),
            KeyCode::M => "M".to_string(),
            KeyCode::N => "N".to_string(),
            KeyCode::O => "O".to_string(),
            KeyCode::P => "P".to_string(),
            KeyCode::Q => "Q".to_string(),
            KeyCode::R => "R".to_string(),
            KeyCode::S => "S".to_string(),
            KeyCode::T => "T".to_string(),
            KeyCode::U => "U".to_string(),
            KeyCode::V => "V".to_string(),
            KeyCode::W => "W".to_string(),
            KeyCode::X => "X".to_string(),
            KeyCode::Y => "Y".to_string(),
            KeyCode::Z => "Z".to_string(),
            KeyCode::Space => "Space".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Escape => "Escape".to_string(),
            KeyCode::Shift => "Shift".to_string(),
            KeyCode::Ctrl => "Ctrl".to_string(),
            KeyCode::Alt => "Alt".to_string(),
            _ => format!("{:?}", self),
        }
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input events that can be processed
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseMoved { x: f32, y: f32 },
}

/// Current input state
#[derive(Debug, Clone)]
pub struct InputState {
    pressed_keys: HashSet<String>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_x: f32,
    mouse_y: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }
    
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.mouse_x = 0.0;
        self.mouse_y = 0.0;
    }
}

/// Unique identifier for input bindings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingId(u64);

static NEXT_BINDING_ID: AtomicU64 = AtomicU64::new(1);

impl BindingId {
    fn new() -> Self {
        BindingId(NEXT_BINDING_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Callback trigger event
struct CallbackTrigger {
    callback_name: String,
    trigger_type: String,
}

/// Lua input manager for handling input in scripts
pub struct LuaInputManager {
    input_state: InputState,
    key_bindings: HashMap<String, Vec<(BindingId, String)>>, // key -> [(id, callback_name)]
    mouse_bindings: HashMap<MouseButton, Vec<(BindingId, String)>>,
    binding_map: HashMap<BindingId, String>, // For unbinding
    callback_queue: VecDeque<CallbackTrigger>,
}

impl LuaInputManager {
    /// Create a new input manager
    pub fn new(_lua: &Lua) -> Result<Self, ScriptError> {
        Ok(Self {
            input_state: InputState::new(),
            key_bindings: HashMap::new(),
            mouse_bindings: HashMap::new(),
            binding_map: HashMap::new(),
            callback_queue: VecDeque::new(),
        })
    }
    
    /// Update input state with new events
    pub fn update(&mut self, events: &[InputEvent]) {
        for event in events {
            match event {
                InputEvent::KeyPressed(key) => {
                    let key_str = key.to_string();
                    self.input_state.pressed_keys.insert(key_str.clone());
                    
                    // Queue callbacks
                    if let Some(bindings) = self.key_bindings.get(&key_str) {
                        for (_, callback_name) in bindings {
                            self.callback_queue.push_back(CallbackTrigger {
                                callback_name: callback_name.clone(),
                                trigger_type: "key_press".to_string(),
                            });
                        }
                    }
                }
                InputEvent::KeyReleased(key) => {
                    self.input_state.pressed_keys.remove(&key.to_string());
                }
                InputEvent::MousePressed(button) => {
                    self.input_state.pressed_mouse_buttons.insert(*button);
                    
                    // Queue callbacks
                    if let Some(bindings) = self.mouse_bindings.get(button) {
                        for (_, callback_name) in bindings {
                            self.callback_queue.push_back(CallbackTrigger {
                                callback_name: callback_name.clone(),
                                trigger_type: "mouse_press".to_string(),
                            });
                        }
                    }
                }
                InputEvent::MouseReleased(button) => {
                    self.input_state.pressed_mouse_buttons.remove(button);
                }
                InputEvent::MouseMoved { x, y } => {
                    self.input_state.mouse_x = *x;
                    self.input_state.mouse_y = *y;
                }
            }
        }
    }
    
    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: &str) -> bool {
        self.input_state.pressed_keys.contains(key)
    }
    
    /// Check if a key combination is pressed
    pub fn is_key_combo_pressed(&self, keys: &[&str]) -> bool {
        keys.iter().all(|key| self.is_key_pressed(key))
    }
    
    /// Get current mouse position
    pub fn mouse_position(&self) -> (f32, f32) {
        (self.input_state.mouse_x, self.input_state.mouse_y)
    }
    
    /// Check if a mouse button is pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.input_state.pressed_mouse_buttons.contains(&button)
    }
    
    /// Bind a key to a callback (simplified - stores callback name)
    pub fn bind_key(&mut self, key: String, _callback: LuaFunction) -> Result<BindingId, ScriptError> {
        let id = BindingId::new();
        let callback_name = format!("key_callback_{}", id.0);
        
        self.key_bindings
            .entry(key.clone())
            .or_default()
            .push((id, callback_name.clone()));
        
        self.binding_map.insert(id, key);
        
        Ok(id)
    }
    
    /// Bind a mouse button to a callback
    pub fn bind_mouse_button(&mut self, button: MouseButton, _callback: LuaFunction) -> Result<BindingId, ScriptError> {
        let id = BindingId::new();
        let callback_name = format!("mouse_callback_{}", id.0);
        
        self.mouse_bindings
            .entry(button)
            .or_default()
            .push((id, callback_name.clone()));
        
        self.binding_map.insert(id, format!("{:?}", button));
        
        Ok(id)
    }
    
    /// Unbind a key or mouse button
    pub fn unbind_key(&mut self, binding_id: BindingId) -> Result<(), ScriptError> {
        if let Some(key) = self.binding_map.remove(&binding_id) {
            // Remove from key bindings
            if let Some(bindings) = self.key_bindings.get_mut(&key) {
                bindings.retain(|(id, _)| *id != binding_id);
                if bindings.is_empty() {
                    self.key_bindings.remove(&key);
                }
            }
            
            // Remove from mouse bindings
            for (_, bindings) in self.mouse_bindings.iter_mut() {
                bindings.retain(|(id, _)| *id != binding_id);
            }
        }
        Ok(())
    }
    
    /// Process queued callbacks (simplified for testing)
    pub fn process_callbacks(&mut self, _lua: &Lua) -> Result<(), ScriptError> {
        // In a real implementation, this would execute the Lua callbacks
        // For now, just clear the queue
        self.callback_queue.clear();
        Ok(())
    }
    
    /// Clear all input state
    pub fn clear(&mut self) {
        self.input_state.clear();
        self.key_bindings.clear();
        self.mouse_bindings.clear();
        self.binding_map.clear();
        self.callback_queue.clear();
    }
    
    /// Register the input API in Lua globals
    pub fn register_api(&self, lua: &Lua) -> Result<(), ScriptError> {
        let globals = lua.globals();
        
        // is_key_pressed function
        let is_key_pressed = lua.create_function(|_, _key: String| {
            // TODO: Get input manager instance and check key
            Ok(false)
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create is_key_pressed function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("is_key_pressed", is_key_pressed).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set is_key_pressed function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // get_mouse_position function
        let get_mouse_position = lua.create_function(|_, ()| {
            // TODO: Get input manager instance and return position
            Ok((0.0, 0.0))
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create get_mouse_position function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("get_mouse_position", get_mouse_position).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set get_mouse_position function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // bind_key function
        let bind_key = lua.create_function(|_, (_key, _callback): (String, LuaFunction)| {
            // TODO: Get input manager instance and bind key
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create bind_key function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("bind_key", bind_key).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set bind_key function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // unbind_key function
        let unbind_key = lua.create_function(|_, _binding_id: u64| {
            // TODO: Get input manager instance and unbind
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create unbind_key function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("unbind_key", unbind_key).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set unbind_key function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // is_mouse_button_pressed function
        let is_mouse_button_pressed = lua.create_function(|_, _button: String| {
            // TODO: Get input manager instance and check button
            Ok(false)
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create is_mouse_button_pressed function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("is_mouse_button_pressed", is_mouse_button_pressed).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set is_mouse_button_pressed function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        Ok(())
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;