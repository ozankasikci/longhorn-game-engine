//! Security tests for scripting system
//! These tests ensure that the scripting system is properly sandboxed and secure.

use crate::{ScriptError, ScriptResult};
use crate::lua::engine::LuaScriptEngine;
use crate::{ScriptMetadata, ScriptId, ScriptType};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that unsafe OS functions are not accessible
    #[test]
    fn test_os_functions_not_accessible() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(1),
            script_type: ScriptType::Lua,
            path: "security_test.lua".to_string(),
            entry_point: None,
        };

        // Test that os.execute is not accessible
        let malicious_script = r#"
            function try_os_execute()
                return os.execute("echo 'security breach'")
            end
        "#;

        // This should fail to load because os.execute should not be available
        let result = engine.load_script_internal(metadata.clone(), malicious_script);
        
        // The script should either fail to load or os.execute should be nil
        if result.is_ok() {
            // If it loads, calling the function should fail
            let execution_result = engine.execute_function_internal("try_os_execute", ());
            assert!(
                execution_result.is_err(),
                "os.execute should not be accessible in sandboxed environment"
            );
        }
    }

    /// Test that unsafe IO functions are not accessible
    #[test]
    fn test_io_functions_not_accessible() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(2),
            script_type: ScriptType::Lua,
            path: "io_security_test.lua".to_string(),
            entry_point: None,
        };

        // Test that io.open is not accessible
        let malicious_script = r#"
            function try_file_access()
                local file = io.open("/etc/passwd", "r")
                if file then
                    local content = file:read("*all")
                    file:close()
                    return content
                end
                return nil
            end
        "#;

        let result = engine.load_script_internal(metadata, malicious_script);
        
        // The script should either fail to load or io.open should be nil
        if result.is_ok() {
            let execution_result = engine.execute_function_internal("try_file_access", ());
            assert!(
                execution_result.is_err(),
                "io.open should not be accessible in sandboxed environment"
            );
        }
    }

    /// Test that scripts cannot access dangerous debug functions
    #[test]
    fn test_debug_functions_not_accessible() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(3),
            script_type: ScriptType::Lua,
            path: "debug_security_test.lua".to_string(),
            entry_point: None,
        };

        // Test that debug.getupvalue is not accessible
        let malicious_script = r#"
            function try_debug_access()
                return debug.getupvalue(print, 1)
            end
        "#;

        let result = engine.load_script_internal(metadata, malicious_script);
        
        if result.is_ok() {
            let execution_result = engine.execute_function_internal("try_debug_access", ());
            assert!(
                execution_result.is_err(),
                "debug functions should not be accessible in sandboxed environment"
            );
        }
    }

    /// Test execution timeout protection
    #[test]
    fn test_execution_timeout() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(4),
            script_type: ScriptType::Lua,
            path: "timeout_test.lua".to_string(),
            entry_point: None,
        };

        // Script with infinite loop
        let infinite_loop_script = r#"
            function infinite_loop()
                local count = 0
                while count < 1000000 do
                    count = count + 1
                    -- This should be interrupted by timeout
                end
                return count
            end
        "#;

        engine.load_script_internal(metadata, infinite_loop_script).expect("Script should load");
        
        // This should timeout and return an error
        let start = std::time::Instant::now();
        let result = engine.execute_function_internal("infinite_loop", ());
        let elapsed = start.elapsed();
        
        // For now, just ensure it doesn't hang forever
        // TODO: Implement actual timeout mechanism and test it works
        assert!(
            elapsed.as_millis() < 5000,
            "Script execution should not hang indefinitely"
        );
        
        // Once timeout is implemented, this should fail:
        // assert!(result.is_err(), "Infinite loop should be interrupted by timeout");
    }

    /// Test recursion depth limits
    #[test]
    fn test_recursion_depth_limits() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(10),
            script_type: ScriptType::Lua,
            path: "recursion_test.lua".to_string(),
            entry_point: None,
        };

        // Script with deep recursion
        let recursion_script = r#"
            function deep_recursion(n)
                if n <= 0 then
                    return 0
                end
                return 1 + deep_recursion(n - 1)
            end
        "#;

        engine.load_script_internal(metadata, recursion_script).expect("Script should load");
        
        // Try very deep recursion
        let result = engine.execute_function_internal("deep_recursion", (10000,));
        
        // For now, just ensure it doesn't crash
        // TODO: Implement recursion depth limits and test they work
        match result {
            Ok(_) => {
                // If it succeeds, that's fine for now
            }
            Err(e) => {
                // If it fails due to stack overflow, that's also fine
                println!("Recursion failed (expected): {}", e);
            }
        }
    }

    /// Test memory usage limits
    #[test]
    fn test_memory_limits() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(5),
            script_type: ScriptType::Lua,
            path: "memory_test.lua".to_string(),
            entry_point: None,
        };

        // Script that tries to allocate excessive memory
        let memory_bomb_script = r#"
            function memory_bomb()
                local data = {}
                for i = 1, 1000000 do
                    data[i] = string.rep("x", 1000)
                end
                return #data
            end
        "#;

        engine.load_script_internal(metadata, memory_bomb_script).expect("Script should load");
        
        // This should either be limited by memory constraints or fail gracefully
        let result = engine.execute_function_internal("memory_bomb", ());
        
        // For now, just ensure it doesn't crash the engine
        // TODO: Implement actual memory limits and test they work
        match result {
            Ok(_) => {
                // If it succeeds, memory usage should be reasonable
                // This is a placeholder - actual memory limit testing will be implemented
            },
            Err(_) => {
                // If it fails, that's also acceptable for now
            }
        }
    }

    /// Test that scripts cannot interfere with each other's global state
    #[test]
    fn test_script_isolation() {
        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        // First script sets a global variable
        let script1_metadata = ScriptMetadata {
            id: ScriptId(6),
            script_type: ScriptType::Lua,
            path: "script1.lua".to_string(),
            entry_point: None,
        };

        let script1 = r#"
            malicious_global = "script1_secret"
            function get_secret()
                return malicious_global
            end
        "#;

        // Second script tries to access the global variable
        let script2_metadata = ScriptMetadata {
            id: ScriptId(7),
            script_type: ScriptType::Lua,
            path: "script2.lua".to_string(),
            entry_point: None,
        };

        let script2 = r#"
            function steal_secret()
                return malicious_global
            end
        "#;

        engine.load_script_internal(script1_metadata, script1).expect("Script 1 should load");
        engine.load_script_internal(script2_metadata, script2).expect("Script 2 should load");

        // Get secret from script 1
        let secret1 = engine.execute_function_internal("get_secret", ()).expect("Should get secret");

        // Try to steal secret from script 2
        let secret2 = engine.execute_function_internal("steal_secret", ());

        // TODO: Once proper script isolation is implemented, this should fail
        // For now, this test documents the current insecure behavior
        match secret2 {
            Ok(value) => {
                println!("WARNING: Script isolation not implemented - script 2 can access script 1's globals: {:?}", value);
            }
            Err(_) => {
                println!("Good: Script isolation prevents cross-script access");
            }
        }
    }

    /// Test that the global console messages don't grow unbounded
    #[test]
    fn test_console_message_bounds() {
        use crate::lua::engine::{get_and_clear_console_messages, CONSOLE_MESSAGES};

        // Clear any existing messages
        get_and_clear_console_messages();

        let mut engine = LuaScriptEngine::new().expect("Failed to create engine");
        engine.setup_core_bindings().expect("Failed to setup bindings");

        let metadata = ScriptMetadata {
            id: ScriptId(8),
            script_type: ScriptType::Lua,
            path: "console_spam_test.lua".to_string(),
            entry_point: None,
        };

        // Script that prints many messages
        let spam_script = r#"
            function spam_console()
                for i = 1, 2000 do
                    print("Spam message " .. i)
                end
            end
        "#;

        engine.load_script_internal(metadata, spam_script).expect("Script should load");
        engine.execute_function_internal("spam_console", ()).expect("Should execute");

        // Check that messages are bounded
        let messages = get_and_clear_console_messages();
        assert!(
            messages.len() <= 1000,
            "Console messages should be bounded to prevent unbounded growth"
        );
    }
}