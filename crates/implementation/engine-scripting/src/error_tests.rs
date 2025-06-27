//! Tests for comprehensive error handling with helpful messages

#[cfg(test)]
mod tests {
    use crate::{ScriptError, ScriptId};
    use engine_ecs_core::Entity;

    #[test]
    fn test_runtime_error_with_context() {
        let error = ScriptError::RuntimeError {
            message: "undefined variable 'player'".to_string(),
            script_id: Some(ScriptId(42)),
            line: Some(15),
            column: Some(8),
            source: None,
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("undefined variable 'player'"));
        
        // Should have context
        assert!(error.has_context());
        assert_eq!(error.script_id(), Some(ScriptId(42)));
    }

    #[test]
    fn test_component_not_found_error() {
        let entity = Entity::new(123, 1);
        let error = ScriptError::ComponentNotFound {
            entity,
            component: "Transform".to_string(),
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("Component 'Transform' not found"));
        assert!(error_str.contains("123")); // entity id
    }

    #[test]
    fn test_access_denied_error() {
        let error = ScriptError::AccessDenied {
            operation: "write component Transform".to_string(),
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("Access denied"));
        assert!(error_str.contains("write component Transform"));
    }

    #[test]
    fn test_invalid_script_error() {
        let error = ScriptError::InvalidScript {
            script_name: "player_controller.lua".to_string(),
            reason: "syntax error: unexpected end of file".to_string(),
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("Invalid script: player_controller.lua"));
        assert!(error_str.contains("syntax error"));
    }

    #[test]
    fn test_error_with_location() {
        let mut error = ScriptError::runtime("division by zero");
        error = error.with_location(42, 15);
        
        if let ScriptError::RuntimeError { line, column, .. } = error {
            assert_eq!(line, Some(42));
            assert_eq!(column, Some(15));
        } else {
            panic!("Expected RuntimeError variant");
        }
    }

    #[test]
    fn test_error_with_script_id() {
        let mut error = ScriptError::compilation("invalid syntax");
        error = error.with_script_id(ScriptId(99));
        
        assert_eq!(error.script_id(), Some(ScriptId(99)));
    }

    #[test]
    fn test_multiple_errors() {
        let errors = vec![
            ScriptError::runtime("error 1"),
            ScriptError::compilation("error 2"),
            ScriptError::NotFound { 
                path: "missing.lua".to_string(), 
                script_id: None 
            },
        ];
        
        let multi_error = ScriptError::Multiple(errors);
        let error_str = multi_error.to_string();
        assert_eq!(error_str, "Multiple errors occurred");
        
        // Check that all errors have context
        assert!(multi_error.has_context());
    }

    #[test]
    fn test_error_message_quality() {
        // Test that error messages are helpful and specific
        let test_cases = vec![
            (
                ScriptError::RuntimeError {
                    message: "attempt to index nil value 'player'".to_string(),
                    script_id: Some(ScriptId(1)),
                    line: Some(25),
                    column: Some(12),
                    source: None,
                },
                vec!["attempt to index nil value 'player'"],
            ),
            (
                ScriptError::ComponentNotFound {
                    entity: Entity::new(456, 1),
                    component: "RigidBody".to_string(),
                },
                vec!["Component 'RigidBody' not found", "456"],
            ),
            (
                ScriptError::PermissionDenied {
                    script_id: ScriptId(2),
                    resource: "filesystem".to_string(),
                    action: "write_file".to_string(),
                    required_permission: "file_write".to_string(),
                },
                vec!["Permission denied", "filesystem", "write_file"],
            ),
        ];
        
        for (error, expected_parts) in test_cases {
            let error_str = error.to_string();
            for part in expected_parts {
                assert!(
                    error_str.contains(part),
                    "Error message '{}' should contain '{}'",
                    error_str,
                    part
                );
            }
        }
    }

    #[test]
    fn test_backward_compatibility() {
        // Test old-style constructors still work
        let error1 = ScriptError::CompilationError("test error".to_string());
        assert!(error1.to_string().contains("test error"));
        
        let error2 = ScriptError::RuntimeError("runtime issue".to_string());
        assert!(error2.to_string().contains("runtime issue"));
        
        let error3 = ScriptError::NotFound("/path/to/script.lua".to_string());
        assert!(error3.to_string().contains("/path/to/script.lua"));
        
        let error4 = ScriptError::InvalidApiCall("bad call".to_string());
        assert!(error4.to_string().contains("bad call"));
    }

    #[test]
    fn test_from_string_conversion() {
        let test_cases = vec![
            ("CompilationError", "syntax error"),
            ("RuntimeError", "undefined variable"),
            ("NotFound", "/missing/file.lua"),
            ("InvalidApiCall", "not allowed"),
            ("UnknownType", "some error"), // Should default to runtime
        ];
        
        for (error_type, message) in test_cases {
            let error = ScriptError::from_string(error_type, message.to_string());
            let error_str = error.to_string();
            assert!(error_str.contains(message));
        }
    }
}