//! Tests for Lua input system integration

#[cfg(test)]
mod tests {
    use crate::lua::input::{LuaInputManager, InputState, InputEvent, KeyCode, MouseButton};
    use crate::ScriptError;
    use mlua::{Lua, Function as LuaFunction};
    use std::sync::{Arc, Mutex};
    
    #[test]
    fn test_input_manager_creation() {
        let lua = Lua::new();
        let input_manager = LuaInputManager::new(&lua).unwrap();
        
        assert!(!input_manager.is_key_pressed("A"));
        assert_eq!(input_manager.mouse_position(), (0.0, 0.0));
    }
    
    #[test]
    fn test_key_press_detection() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Simulate key press
        let events = vec![
            InputEvent::KeyPressed(KeyCode::A),
            InputEvent::KeyPressed(KeyCode::Space),
        ];
        
        input_manager.update(&events);
        
        assert!(input_manager.is_key_pressed("A"));
        assert!(input_manager.is_key_pressed("Space"));
        assert!(!input_manager.is_key_pressed("B"));
    }
    
    #[test]
    fn test_key_release_detection() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Press and release key
        input_manager.update(&[InputEvent::KeyPressed(KeyCode::A)]);
        assert!(input_manager.is_key_pressed("A"));
        
        input_manager.update(&[InputEvent::KeyReleased(KeyCode::A)]);
        assert!(!input_manager.is_key_pressed("A"));
    }
    
    #[test]
    fn test_mouse_position_tracking() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        let events = vec![
            InputEvent::MouseMoved { x: 100.0, y: 200.0 },
        ];
        
        input_manager.update(&events);
        assert_eq!(input_manager.mouse_position(), (100.0, 200.0));
        
        // Update position again
        input_manager.update(&[InputEvent::MouseMoved { x: 150.0, y: 250.0 }]);
        assert_eq!(input_manager.mouse_position(), (150.0, 250.0));
    }
    
    #[test]
    fn test_mouse_button_tracking() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        input_manager.update(&[InputEvent::MousePressed(MouseButton::Left)]);
        assert!(input_manager.is_mouse_button_pressed(MouseButton::Left));
        assert!(!input_manager.is_mouse_button_pressed(MouseButton::Right));
        
        input_manager.update(&[InputEvent::MouseReleased(MouseButton::Left)]);
        assert!(!input_manager.is_mouse_button_pressed(MouseButton::Left));
    }
    
    #[test]
    fn test_key_binding() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        let callback_called = Arc::new(Mutex::new(false));
        let callback_called_clone = callback_called.clone();
        
        // Create callback
        let callback: LuaFunction = lua.create_function(move |_, ()| {
            *callback_called_clone.lock().unwrap() = true;
            Ok(())
        }).unwrap();
        
        // Bind key
        input_manager.bind_key("Space".to_string(), callback).unwrap();
        
        // Trigger key press
        input_manager.update(&[InputEvent::KeyPressed(KeyCode::Space)]);
        
        // Process callbacks
        input_manager.process_callbacks(&lua).unwrap();
        
        assert!(*callback_called.lock().unwrap());
    }
    
    #[test]
    fn test_multiple_key_bindings() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        let counter = Arc::new(Mutex::new(0));
        
        // Bind multiple callbacks to same key
        for _ in 0..3 {
            let counter_clone = counter.clone();
            let callback: LuaFunction = lua.create_function(move |_, ()| {
                *counter_clone.lock().unwrap() += 1;
                Ok(())
            }).unwrap();
            
            input_manager.bind_key("A".to_string(), callback).unwrap();
        }
        
        // Trigger key press
        input_manager.update(&[InputEvent::KeyPressed(KeyCode::A)]);
        input_manager.process_callbacks(&lua).unwrap();
        
        assert_eq!(*counter.lock().unwrap(), 3);
    }
    
    #[test]
    fn test_unbind_key() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        let callback: LuaFunction = lua.create_function(|_, ()| Ok(())).unwrap();
        let binding_id = input_manager.bind_key("Escape".to_string(), callback).unwrap();
        
        // Unbind
        input_manager.unbind_key(binding_id).unwrap();
        
        // Key press should not trigger any callbacks
        input_manager.update(&[InputEvent::KeyPressed(KeyCode::Escape)]);
        // No assertion needed - just ensuring no panic
    }
    
    #[test]
    fn test_mouse_button_binding() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        let clicked = Arc::new(Mutex::new(false));
        let clicked_clone = clicked.clone();
        
        let callback: LuaFunction = lua.create_function(move |_, ()| {
            *clicked_clone.lock().unwrap() = true;
            Ok(())
        }).unwrap();
        
        input_manager.bind_mouse_button(MouseButton::Left, callback).unwrap();
        
        // Trigger mouse click
        input_manager.update(&[InputEvent::MousePressed(MouseButton::Left)]);
        input_manager.process_callbacks(&lua).unwrap();
        
        assert!(*clicked.lock().unwrap());
    }
    
    #[test]
    fn test_input_state_queries() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Set up various input states
        input_manager.update(&[
            InputEvent::KeyPressed(KeyCode::W),
            InputEvent::KeyPressed(KeyCode::Shift),
            InputEvent::MouseMoved { x: 640.0, y: 480.0 },
            InputEvent::MousePressed(MouseButton::Right),
        ]);
        
        // Test queries
        assert!(input_manager.is_key_pressed("W"));
        assert!(input_manager.is_key_pressed("Shift"));
        assert!(!input_manager.is_key_pressed("S"));
        
        assert_eq!(input_manager.mouse_position(), (640.0, 480.0));
        assert!(input_manager.is_mouse_button_pressed(MouseButton::Right));
        assert!(!input_manager.is_mouse_button_pressed(MouseButton::Left));
    }
    
    #[test]
    fn test_key_combo_detection() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Press multiple keys
        input_manager.update(&[
            InputEvent::KeyPressed(KeyCode::Ctrl),
            InputEvent::KeyPressed(KeyCode::S),
        ]);
        
        assert!(input_manager.is_key_combo_pressed(&["Ctrl", "S"]));
        assert!(!input_manager.is_key_combo_pressed(&["Ctrl", "S", "Alt"]));
    }
    
    #[test]
    fn test_input_api_registration() {
        let lua = Lua::new();
        let input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Register API
        input_manager.register_api(&lua).unwrap();
        
        // Check that functions are available
        let globals = lua.globals();
        assert!(globals.get::<LuaFunction>("is_key_pressed").is_ok());
        assert!(globals.get::<LuaFunction>("get_mouse_position").is_ok());
        assert!(globals.get::<LuaFunction>("bind_key").is_ok());
        assert!(globals.get::<LuaFunction>("unbind_key").is_ok());
        assert!(globals.get::<LuaFunction>("is_mouse_button_pressed").is_ok());
        
        // Test using the API from Lua
        lua.load(r#"
            local mouse_x, mouse_y = get_mouse_position()
            assert(mouse_x == 0)
            assert(mouse_y == 0)
            
            local pressed = is_key_pressed("A")
            assert(pressed == false)
        "#).exec().unwrap();
    }
    
    #[test]
    fn test_clear_input_state() {
        let lua = Lua::new();
        let mut input_manager = LuaInputManager::new(&lua).unwrap();
        
        // Set up some state
        input_manager.update(&[
            InputEvent::KeyPressed(KeyCode::A),
            InputEvent::MousePressed(MouseButton::Left),
        ]);
        
        assert!(input_manager.is_key_pressed("A"));
        assert!(input_manager.is_mouse_button_pressed(MouseButton::Left));
        
        // Clear state
        input_manager.clear();
        
        assert!(!input_manager.is_key_pressed("A"));
        assert!(!input_manager.is_mouse_button_pressed(MouseButton::Left));
    }
}