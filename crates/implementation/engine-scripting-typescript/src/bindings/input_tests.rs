//! Tests for TypeScript Input API bindings
//! 
//! These tests define the expected behavior of the Input bindings for TypeScript scripts.
//! Following TDD principles, these tests are written before implementation.

use crate::initialize_v8_platform;
use crate::runtime::TypeScriptRuntime;
use engine_scripting::{
    runtime::ScriptRuntime,
    types::{ScriptId, ScriptMetadata, ScriptType},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_queries() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            // Test basic key state queries
            function testKeyPressed(): boolean {
                return engine.input.isKeyPressed("A");
            }
            
            function testMouseButtonPressed(): boolean {
                return engine.input.isMouseButtonPressed("left");
            }
            
            function getKeyCode(): number {
                return engine.input.keys.A;
            }
        "#;
        
        let script_id = ScriptId(1);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_input_queries.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test key state queries work (should return false when no input)
        let key_pressed = runtime.execute_function("testKeyPressed", vec![]).unwrap();
        assert_eq!(key_pressed, "false", "Key should not be pressed by default");
        
        let mouse_pressed = runtime.execute_function("testMouseButtonPressed", vec![]).unwrap();
        assert_eq!(mouse_pressed, "false", "Mouse button should not be pressed by default");
        
        // Test key code access
        let key_code = runtime.execute_function("getKeyCode", vec![]).unwrap();
        let code: u32 = key_code.parse().unwrap();
        assert!(code > 0, "Key code should be a valid number");
    }

    #[test]
    fn test_mouse_position() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function getMousePosition(): string {
                const pos = engine.input.getMousePosition();
                return JSON.stringify({ x: pos.x, y: pos.y });
            }
            
            function getMouseX(): number {
                const pos = engine.input.getMousePosition();
                return pos.x;
            }
            
            function getMouseY(): number {
                const pos = engine.input.getMousePosition();
                return pos.y;
            }
        "#;
        
        let script_id = ScriptId(2);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_mouse_position.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test mouse position returns valid coordinates
        let mouse_pos = runtime.execute_function("getMousePosition", vec![]).unwrap();
        assert!(mouse_pos.contains("x") && mouse_pos.contains("y"), 
                "Mouse position should contain x and y coordinates");
        
        let mouse_x = runtime.execute_function("getMouseX", vec![]).unwrap();
        let x: f64 = mouse_x.parse().unwrap();
        assert!(x >= 0.0, "Mouse X should be a valid coordinate");
        
        let mouse_y = runtime.execute_function("getMouseY", vec![]).unwrap();
        let y: f64 = mouse_y.parse().unwrap();
        assert!(y >= 0.0, "Mouse Y should be a valid coordinate");
    }

    #[test]
    fn test_key_bindings() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let bindingTriggered = false;
            let lastTriggeredKey = "";
            
            function onKeyPress(key: string): void {
                bindingTriggered = true;
                lastTriggeredKey = key;
            }
            
            function bindKey(): number {
                return engine.input.bindKey("Space", onKeyPress);
            }
            
            function unbindKey(bindingId: number): void {
                engine.input.unbindKey(bindingId);
            }
            
            function wasTriggered(): boolean {
                return bindingTriggered;
            }
            
            function getLastKey(): string {
                return lastTriggeredKey;
            }
            
            function resetTrigger(): void {
                bindingTriggered = false;
                lastTriggeredKey = "";
            }
        "#;
        
        let script_id = ScriptId(3);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_key_bindings.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test key binding returns valid ID
        let binding_id = runtime.execute_function("bindKey", vec![]).unwrap();
        let id: u32 = binding_id.parse().unwrap();
        assert!(id > 0, "Binding ID should be a valid positive number");
        
        // Test initial state
        let triggered = runtime.execute_function("wasTriggered", vec![]).unwrap();
        assert_eq!(triggered, "false", "Binding should not be triggered initially");
        
        // Test unbinding works without error
        runtime.execute_function("unbindKey", vec![binding_id]).unwrap();
    }

    #[test]
    fn test_input_events() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let events: Array<{type: string, key?: string, button?: string}> = [];
            
            function onKeyDown(key: string): void {
                events.push({ type: "keydown", key: key });
            }
            
            function onKeyUp(key: string): void {
                events.push({ type: "keyup", key: key });
            }
            
            function onMouseDown(button: string): void {
                events.push({ type: "mousedown", button: button });
            }
            
            function onMouseUp(button: string): void {
                events.push({ type: "mouseup", button: button });
            }
            
            function setupEventListeners(): void {
                engine.input.onKeyDown(onKeyDown);
                engine.input.onKeyUp(onKeyUp);
                engine.input.onMouseDown(onMouseDown);
                engine.input.onMouseUp(onMouseUp);
            }
            
            function getEventCount(): number {
                return events.length;
            }
            
            function getLastEvent(): string {
                if (events.length === 0) return "{}";
                return JSON.stringify(events[events.length - 1]);
            }
            
            function clearEvents(): void {
                events = [];
            }
        "#;
        
        let script_id = ScriptId(4);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_input_events.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test event listener setup works without error
        runtime.execute_function("setupEventListeners", vec![]).unwrap();
        
        // Test initial event count
        let event_count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(event_count, "0", "Should have no events initially");
        
        // Test clearing events works
        runtime.execute_function("clearEvents", vec![]).unwrap();
        let cleared_count = runtime.execute_function("getEventCount", vec![]).unwrap();
        assert_eq!(cleared_count, "0", "Event count should be 0 after clearing");
    }

    #[test]
    fn test_input_validation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function testInvalidKey(): boolean {
                try {
                    return engine.input.isKeyPressed("InvalidKey");
                } catch (e) {
                    return false; // Invalid key should return false or throw
                }
            }
            
            function testInvalidMouseButton(): boolean {
                try {
                    return engine.input.isMouseButtonPressed("invalid");
                } catch (e) {
                    return false;
                }
            }
            
            function testEmptyKeyBind(): number {
                try {
                    return engine.input.bindKey("", () => {});
                } catch (e) {
                    return -1; // Should handle empty key gracefully
                }
            }
        "#;
        
        let script_id = ScriptId(5);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_input_validation.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test invalid key handling
        let invalid_key = runtime.execute_function("testInvalidKey", vec![]).unwrap();
        assert_eq!(invalid_key, "false", "Invalid key should return false");
        
        // Test invalid mouse button handling
        let invalid_button = runtime.execute_function("testInvalidMouseButton", vec![]).unwrap();
        assert_eq!(invalid_button, "false", "Invalid mouse button should return false");
        
        // Test empty key binding handling
        let empty_bind = runtime.execute_function("testEmptyKeyBind", vec![]).unwrap();
        let bind_result: i32 = empty_bind.parse().unwrap();
        assert!(bind_result == -1 || bind_result == 0, "Empty key binding should be handled gracefully");
    }
}