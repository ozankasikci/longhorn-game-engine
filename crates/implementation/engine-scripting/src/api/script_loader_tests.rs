//! Tests for the standardized script loading interface

#[cfg(test)]
mod tests {
    use super::super::{ScriptLoader, ScriptLoadRequest, ScriptSource, ScriptHandle, ExecutionContext};
    use crate::{ScriptError, manager::ScriptManager};
    use std::path::PathBuf;
    use engine_ecs_core::Entity;

    /// Mock implementation for testing
    struct TestScriptLoader {
        manager: ScriptManager,
        loaded_scripts: std::collections::HashMap<ScriptHandle, ScriptLoadRequest>,
        next_handle: u64,
    }

    impl TestScriptLoader {
        fn new() -> Self {
            Self {
                manager: ScriptManager::new().unwrap(),
                loaded_scripts: std::collections::HashMap::new(),
                next_handle: 1,
            }
        }
    }

    impl ScriptLoader for TestScriptLoader {
        fn load_script(&mut self, request: ScriptLoadRequest) -> Result<ScriptHandle, ScriptError> {
            let handle = ScriptHandle(self.next_handle);
            self.next_handle += 1;
            
            // Validate request
            match &request.source {
                ScriptSource::File(path) => {
                    if !path.exists() {
                        return Err(ScriptError::NotFound(format!("File not found: {:?}", path)));
                    }
                }
                ScriptSource::String { content, name } => {
                    if content.is_empty() {
                        return Err(ScriptError::InvalidScript {
                            script_name: name.clone(),
                            reason: "Empty script content".to_string(),
                        });
                    }
                }
                ScriptSource::Bytecode(data) => {
                    if data.is_empty() {
                        return Err(ScriptError::InvalidScript {
                            script_name: "bytecode".to_string(),
                            reason: "Empty bytecode".to_string(),
                        });
                    }
                }
            }
            
            self.loaded_scripts.insert(handle, request);
            Ok(handle)
        }

        fn unload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError> {
            self.loaded_scripts.remove(&handle)
                .ok_or_else(|| ScriptError::InvalidHandle { handle })?;
            Ok(())
        }

        fn reload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError> {
            if !self.loaded_scripts.contains_key(&handle) {
                return Err(ScriptError::InvalidHandle { handle });
            }
            // In a real implementation, this would reload from source
            Ok(())
        }
    }

    #[test]
    fn test_load_script_from_file() {
        let mut loader = TestScriptLoader::new();
        
        // Create a temporary test file
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test_script.lua");
        std::fs::write(&script_path, "print('Hello from test')").unwrap();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::File(script_path.clone()),
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        let handle = loader.load_script(request.clone()).unwrap();
        assert_eq!(loader.loaded_scripts.get(&handle).unwrap().source, request.source);
    }

    #[test]
    fn test_load_script_from_string() {
        let mut loader = TestScriptLoader::new();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "function update() end".to_string(),
                name: "inline_script".to_string(),
            },
            entity_binding: Some(Entity::new(42, 1)),
            execution_context: ExecutionContext::with_permissions(vec!["entity_read".to_string()]),
        };
        
        let handle = loader.load_script(request.clone()).unwrap();
        assert!(loader.loaded_scripts.contains_key(&handle));
        
        // Verify entity binding
        let loaded = loader.loaded_scripts.get(&handle).unwrap();
        assert_eq!(loaded.entity_binding, Some(Entity::new(42, 1)));
    }

    #[test]
    fn test_load_script_from_bytecode() {
        let mut loader = TestScriptLoader::new();
        
        let bytecode = vec![0x1B, 0x4C, 0x75, 0x61]; // Lua bytecode header
        let request = ScriptLoadRequest {
            source: ScriptSource::Bytecode(bytecode),
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        let handle = loader.load_script(request).unwrap();
        assert!(loader.loaded_scripts.contains_key(&handle));
    }

    #[test]
    fn test_load_script_file_not_found() {
        let mut loader = TestScriptLoader::new();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::File(PathBuf::from("/nonexistent/script.lua")),
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        match loader.load_script(request) {
            Err(ScriptError::NotFound { path, .. }) => {
                assert!(path.contains("/nonexistent/script.lua"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_load_script_empty_content() {
        let mut loader = TestScriptLoader::new();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "".to_string(),
                name: "empty_script".to_string(),
            },
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        match loader.load_script(request) {
            Err(ScriptError::InvalidScript { script_name, reason }) => {
                assert_eq!(script_name, "empty_script");
                assert!(reason.contains("Empty"));
            }
            _ => panic!("Expected InvalidScript error"),
        }
    }

    #[test]
    fn test_unload_script() {
        let mut loader = TestScriptLoader::new();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "print('test')".to_string(),
                name: "test".to_string(),
            },
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        let handle = loader.load_script(request).unwrap();
        assert!(loader.loaded_scripts.contains_key(&handle));
        
        loader.unload_script(handle).unwrap();
        assert!(!loader.loaded_scripts.contains_key(&handle));
    }

    #[test]
    fn test_unload_invalid_handle() {
        let mut loader = TestScriptLoader::new();
        let invalid_handle = ScriptHandle(999);
        
        match loader.unload_script(invalid_handle) {
            Err(ScriptError::InvalidHandle { handle }) => {
                assert_eq!(handle, invalid_handle);
            }
            _ => panic!("Expected InvalidHandle error"),
        }
    }

    #[test]
    fn test_reload_script() {
        let mut loader = TestScriptLoader::new();
        
        let request = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "print('original')".to_string(),
                name: "reload_test".to_string(),
            },
            entity_binding: None,
            execution_context: ExecutionContext::default(),
        };
        
        let handle = loader.load_script(request).unwrap();
        loader.reload_script(handle).unwrap();
        
        // Script should still be loaded
        assert!(loader.loaded_scripts.contains_key(&handle));
    }

    #[test]
    fn test_reload_invalid_handle() {
        let mut loader = TestScriptLoader::new();
        let invalid_handle = ScriptHandle(999);
        
        match loader.reload_script(invalid_handle) {
            Err(ScriptError::InvalidHandle { handle }) => {
                assert_eq!(handle, invalid_handle);
            }
            _ => panic!("Expected InvalidHandle error"),
        }
    }

    #[test]
    fn test_execution_context_permissions() {
        let mut loader = TestScriptLoader::new();
        
        let mut context = ExecutionContext::default();
        context.add_permission("file_read");
        context.add_permission("entity_write");
        
        let request = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "-- test".to_string(),
                name: "permission_test".to_string(),
            },
            entity_binding: None,
            execution_context: context,
        };
        
        let handle = loader.load_script(request).unwrap();
        let loaded = loader.loaded_scripts.get(&handle).unwrap();
        
        assert!(loaded.execution_context.has_permission("file_read"));
        assert!(loaded.execution_context.has_permission("entity_write"));
        assert!(!loaded.execution_context.has_permission("network_access"));
    }

    #[test]
    fn test_multiple_scripts_same_entity() {
        let mut loader = TestScriptLoader::new();
        let entity_id = Entity::new(123, 1);
        
        // Load first script
        let request1 = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "-- script 1".to_string(),
                name: "script1".to_string(),
            },
            entity_binding: Some(entity_id),
            execution_context: ExecutionContext::default(),
        };
        
        // Load second script
        let request2 = ScriptLoadRequest {
            source: ScriptSource::String {
                content: "-- script 2".to_string(),
                name: "script2".to_string(),
            },
            entity_binding: Some(entity_id),
            execution_context: ExecutionContext::default(),
        };
        
        let handle1 = loader.load_script(request1).unwrap();
        let handle2 = loader.load_script(request2).unwrap();
        
        assert_ne!(handle1, handle2);
        assert_eq!(loader.loaded_scripts.len(), 2);
    }
}