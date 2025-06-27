//! Test to verify TypeScript console output integration with game engine console

#[cfg(test)]
mod tests {
    use crate::typescript_script_system::SimpleTypeScriptRuntime;
    use crate::lua::engine::get_and_clear_console_messages;

    #[test]
    fn test_typescript_console_log_appears_in_game_engine_console() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // Clear any existing console messages
        get_and_clear_console_messages();
        
        let test_script = r#"
            export class ConsoleTestScript {
                init(): void {
                    console.log("Hello from TypeScript!");
                    console.log("Multiple", "arguments", "test");
                    console.error("This is an error message");
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "console_test.ts", test_script).unwrap();
        runtime.call_init(1).unwrap();
        
        // Assert
        let messages = get_and_clear_console_messages();
        
        assert_eq!(messages.len(), 3, "Should have 3 console messages");
        
        assert_eq!(messages[0].message, "Hello from TypeScript!");
        assert_eq!(messages[1].message, "Multiple arguments test");
        assert_eq!(messages[2].message, "ERROR: This is an error message");
        
        // Verify timestamps are recent
        for message in &messages {
            let elapsed = message.timestamp.elapsed().unwrap();
            assert!(elapsed.as_secs() < 5, "Message timestamp should be recent");
        }
    }

    #[test]
    fn test_typescript_console_works_with_examples() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // Clear any existing console messages
        get_and_clear_console_messages();
        
        // Use the hello world example
        let hello_world_script = r#"
            export class HelloWorld {
                init(): void {
                    console.log("Hello, World!");
                    console.log("Welcome to Longhorn Game Engine TypeScript scripting!");
                }
                
                update(deltaTime: number): void {
                    // Update logic here
                }
                
                destroy(): void {
                    console.log("Goodbye from TypeScript!");
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "hello_world.ts", hello_world_script).unwrap();
        runtime.call_init(1).unwrap();
        runtime.call_destroy(1).unwrap();
        
        // Assert
        let messages = get_and_clear_console_messages();
        
        assert_eq!(messages.len(), 3, "Should have 3 console messages from hello world example");
        
        assert_eq!(messages[0].message, "Hello, World!");
        assert_eq!(messages[1].message, "Welcome to Longhorn Game Engine TypeScript scripting!");
        assert_eq!(messages[2].message, "Goodbye from TypeScript!");
    }

    #[test]
    fn test_typescript_console_mixed_with_lua_console() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // Clear any existing console messages
        get_and_clear_console_messages();
        
        let typescript_script = r#"
            export class TypeScriptLogger {
                init(): void {
                    console.log("TypeScript message");
                }
            }
        "#;
        
        // Act - TypeScript console output
        runtime.load_and_compile_script(1, "ts_logger.ts", typescript_script).unwrap();
        runtime.call_init(1).unwrap();
        
        // Simulate Lua console output by adding message directly
        crate::lua::engine::CONSOLE_MESSAGES.lock().unwrap().push(
            crate::lua::engine::ConsoleMessage {
                message: "Lua message".to_string(),
                timestamp: std::time::SystemTime::now(),
            }
        );
        
        // Assert - Both TypeScript and Lua messages should be in the same console
        let messages = get_and_clear_console_messages();
        
        assert_eq!(messages.len(), 2, "Should have messages from both TypeScript and Lua");
        
        // Messages are in the order they were added
        assert_eq!(messages[0].message, "TypeScript message");
        assert_eq!(messages[1].message, "Lua message");
    }
}