//! TDD Tests for TypeScript Script Hot Reload Functionality
//! 
//! These tests follow TDD methodology to define and implement TypeScript script
//! hot reload capabilities that preserve script state and handle runtime updates.

#[cfg(test)]
mod tests {
    use crate::typescript_script_system::SimpleTypeScriptRuntime;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // TDD RED PHASE: Define expected hot reload behavior with failing tests

    #[test]
    fn test_hot_reload_preserves_script_state() {
        // Arrange - Create a temporary script file and runtime
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("stateful_script.ts");
        
        let initial_script = r#"
            export class StatefulScript {
                private counter: number = 0;
                
                init(): void {
                    this.counter = 5;
                    console.log("Initial counter:", this.counter);
                }
                
                update(deltaTime: number): void {
                    this.counter++;
                    console.log("Counter:", this.counter);
                }
                
                getCounter(): number {
                    return this.counter;
                }
            }
        "#;
        
        fs::write(&script_path, initial_script).expect("Failed to write initial script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        runtime.call_init(1).unwrap();
        runtime.call_update(1, 0.016).unwrap(); // counter should be 6
        
        // Act - Modify script and hot reload
        let updated_script = r#"
            export class StatefulScript {
                private counter: number = 0;
                
                init(): void {
                    this.counter = 5;
                    console.log("Initial counter:", this.counter);
                }
                
                update(deltaTime: number): void {
                    this.counter += 2; // Changed increment logic
                    console.log("Updated counter:", this.counter);
                }
                
                getCounter(): number {
                    return this.counter;
                }
                
                // New method added
                resetCounter(): void {
                    this.counter = 0;
                }
            }
        "#;
        
        fs::write(&script_path, updated_script).expect("Failed to write updated script");
        
        // This should preserve state (counter = 6) and reload with new logic
        let reload_result = runtime.hot_reload_script(1, script_path.as_path());
        
        // Assert - State should be preserved and new functionality available
        assert!(reload_result.is_ok(), "Hot reload should succeed: {:?}", reload_result);
        
        // State should be preserved - counter should still be 6
        let preserved_state = runtime.call_script_method(1, "getCounter", &[]);
        assert!(preserved_state.is_ok(), "Should be able to call preserved method");
        
        // New functionality should be available
        let reset_result = runtime.call_script_method(1, "resetCounter", &[]);
        assert!(reset_result.is_ok(), "Should be able to call new method: {:?}", reset_result);
    }

    #[test]
    fn test_hot_reload_handles_compilation_errors_gracefully() {
        // Arrange - Create valid script first
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("error_test_script.ts");
        
        let valid_script = r#"
            export class TestScript {
                init(): void {
                    console.log("Valid script loaded");
                }
            }
        "#;
        
        fs::write(&script_path, valid_script).expect("Failed to write valid script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        runtime.call_init(1).unwrap();
        
        // Act - Write invalid script and attempt hot reload
        let invalid_script = r#"
            export class TestScript {
                init(): void {
                    console.log("Invalid script with syntax error"
                    // Missing closing parenthesis and semicolon
                }
            }
        "#;
        
        fs::write(&script_path, invalid_script).expect("Failed to write invalid script");
        let reload_result = runtime.hot_reload_script(1, script_path.as_path());
        
        // Assert - Hot reload should fail but not crash the runtime
        assert!(reload_result.is_err(), "Hot reload should fail for invalid script");
        
        // Original script should still be functional
        let init_result = runtime.call_init(1);
        assert!(init_result.is_ok(), "Original script should still work after failed reload");
    }

    #[test]
    fn test_hot_reload_updates_multiple_scripts_independently() {
        // Arrange - Create multiple script files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        let script1_path = temp_dir.path().join("script1.ts");
        let script2_path = temp_dir.path().join("script2.ts");
        
        let script1_content = r#"
            export class Script1 {
                private value: number = 1;
                
                init(): void {
                    console.log("Script 1 initialized");
                }
                
                getValue(): number {
                    return this.value;
                }
            }
        "#;
        
        let script2_content = r#"
            export class Script2 {
                private value: number = 2;
                
                init(): void {
                    console.log("Script 2 initialized");
                }
                
                getValue(): number {
                    return this.value;
                }
            }
        "#;
        
        fs::write(&script1_path, script1_content).expect("Failed to write script 1");
        fs::write(&script2_path, script2_content).expect("Failed to write script 2");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, &script1_path).unwrap();
        runtime.load_and_compile_script_from_file(2, &script2_path).unwrap();
        
        // Act - Hot reload only script 1
        let updated_script1 = r#"
            export class Script1 {
                private value: number = 10; // Changed value
                
                init(): void {
                    console.log("Script 1 updated and reloaded");
                }
                
                getValue(): number {
                    return this.value;
                }
            }
        "#;
        
        fs::write(&script1_path, updated_script1).expect("Failed to update script 1");
        let reload_result = runtime.hot_reload_script(1, &script1_path);
        
        // Assert - Script 1 should be updated, Script 2 unchanged
        assert!(reload_result.is_ok(), "Hot reload of script 1 should succeed");
        
        // Both scripts should still be functional
        let script1_result = runtime.call_init(1);
        let script2_result = runtime.call_init(2);
        
        assert!(script1_result.is_ok(), "Updated script 1 should work");
        assert!(script2_result.is_ok(), "Script 2 should remain unchanged");
    }

    #[test]
    fn test_hot_reload_maintains_engine_api_injection() {
        // Arrange - Create script that uses Engine APIs
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("api_test_script.ts");
        
        let initial_script = r#"
            export class ApiTestScript {
                init(): void {
                    const entityId = globalThis.World.createEntity();
                    console.log("Created entity:", entityId);
                }
                
                update(deltaTime: number): void {
                    if (globalThis.Input.isKeyPressed("Space")) {
                        console.log("Space pressed");
                    }
                }
            }
        "#;
        
        fs::write(&script_path, initial_script).expect("Failed to write script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        runtime.call_init(1).unwrap();
        
        // Act - Hot reload with updated API usage
        let updated_script = r#"
            export class ApiTestScript {
                init(): void {
                    const entityId = globalThis.World.createEntity();
                    globalThis.World.addComponent(entityId, 'Transform', {
                        position: { x: 1.0, y: 2.0, z: 3.0 },
                        rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                        scale: { x: 1.0, y: 1.0, z: 1.0 }
                    });
                    console.log("Created entity with transform:", entityId);
                }
                
                update(deltaTime: number): void {
                    if (globalThis.Input.isKeyPressed("Space")) {
                        console.log("Space pressed in updated script");
                    }
                    
                    // New physics API usage
                    globalThis.Physics.applyForce(123, { x: 10.0, y: 0.0, z: 0.0 });
                }
            }
        "#;
        
        fs::write(&script_path, updated_script).expect("Failed to write updated script");
        let reload_result = runtime.hot_reload_script(1, script_path.as_path());
        
        // Assert - Engine APIs should still be available after hot reload
        assert!(reload_result.is_ok(), "Hot reload should maintain API injection");
        
        let init_result = runtime.call_init(1);
        assert!(init_result.is_ok(), "Engine APIs should work after hot reload");
        
        let update_result = runtime.call_update(1, 0.016);
        assert!(update_result.is_ok(), "Updated API usage should work");
    }

    #[test]
    fn test_hot_reload_handles_file_system_changes() {
        // Arrange - Set up file watcher scenario
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("watched_script.ts");
        
        let initial_script = r#"
            export class WatchedScript {
                private lastModified: number = Date.now();
                
                init(): void {
                    console.log("Script loaded at:", this.lastModified);
                }
                
                getLastModified(): number {
                    return this.lastModified;
                }
            }
        "#;
        
        fs::write(&script_path, initial_script).expect("Failed to write script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        
        // Act - Simulate file system change detection
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure timestamp difference
        
        let updated_script = r#"
            export class WatchedScript {
                private lastModified: number = Date.now();
                
                init(): void {
                    console.log("Script reloaded at:", this.lastModified);
                }
                
                getLastModified(): number {
                    return this.lastModified;
                }
                
                // New method to verify reload
                isReloaded(): boolean {
                    return true;
                }
            }
        "#;
        
        fs::write(&script_path, updated_script).expect("Failed to write updated script");
        
        // Simulate file watcher detecting change
        let reload_result = runtime.hot_reload_script_if_changed(1, script_path.as_path());
        
        // Assert - Should detect change and reload
        assert!(reload_result.is_ok(), "Should detect file change and reload");
        
        let new_method_result = runtime.call_script_method(1, "isReloaded", &[]);
        assert!(new_method_result.is_ok(), "New method should be available after reload");
    }

    #[test]
    fn test_hot_reload_performance_with_large_scripts() {
        // Arrange - Create a large script to test performance
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("large_script.ts");
        
        // Generate a large script with many methods
        let mut large_script = String::from(r#"
            export class LargeScript {
                private data: number[] = [];
                
                init(): void {
                    for (let i = 0; i < 1000; i++) {
                        this.data.push(i);
                    }
                    console.log("Large script initialized with", this.data.length, "items");
                }
        "#);
        
        // Add many methods to make it large
        for i in 0..100 {
            large_script.push_str(&format!(r#"
                method{}(): number {{
                    return this.data.reduce((sum, val) => sum + val, 0) + {};
                }}
            "#, i, i));
        }
        
        large_script.push_str("\n            }");
        
        fs::write(&script_path, &large_script).expect("Failed to write large script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // Act - Measure hot reload performance
        let start_time = std::time::Instant::now();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        let initial_compile_time = start_time.elapsed();
        
        // Modify and hot reload
        let modified_large_script = large_script.replace("Large script initialized", "Large script reloaded");
        fs::write(&script_path, &modified_large_script).expect("Failed to write modified script");
        
        let reload_start = std::time::Instant::now();
        let reload_result = runtime.hot_reload_script(1, script_path.as_path());
        let reload_time = reload_start.elapsed();
        
        // Assert - Hot reload should complete within reasonable time
        assert!(reload_result.is_ok(), "Large script hot reload should succeed");
        assert!(reload_time.as_millis() < 5000, "Hot reload should complete within 5 seconds");
        
        // Performance should be reasonable compared to initial compilation
        let performance_ratio = reload_time.as_millis() as f64 / initial_compile_time.as_millis() as f64;
        assert!(performance_ratio < 3.0, "Hot reload shouldn't be more than 3x slower than initial compilation");
    }

    #[test]
    fn test_hot_reload_memory_management() {
        // Arrange - Test memory cleanup during hot reload
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let script_path = temp_dir.path().join("memory_test_script.ts");
        
        let script_template = |version: u32| -> String {
            format!(r#"
                export class MemoryTestScript {{
                    private version: number = {};
                    private largeArray: number[] = new Array(10000).fill({});
                    
                    init(): void {{
                        console.log("Version {} loaded");
                    }}
                    
                    getVersion(): number {{
                        return this.version;
                    }}
                    
                    getArraySize(): number {{
                        return this.largeArray.length;
                    }}
                }}
            "#, version, version, version)
        };
        
        fs::write(&script_path, script_template(1)).expect("Failed to write script");
        
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        runtime.load_and_compile_script_from_file(1, script_path.as_path()).unwrap();
        runtime.call_init(1).unwrap();
        
        // Act - Perform multiple hot reloads to test memory management
        for version in 2..=5 {
            fs::write(&script_path, script_template(version)).expect("Failed to write script version");
            let reload_result = runtime.hot_reload_script(1, script_path.as_path());
            assert!(reload_result.is_ok(), "Hot reload version {} should succeed", version);
            
            // Verify new version is active
            runtime.call_init(1).unwrap();
        }
        
        // Force garbage collection if available
        runtime.force_garbage_collection();
        
        // Assert - Memory should be managed properly (no excessive growth)
        let memory_stats = runtime.get_memory_stats();
        assert!(memory_stats.is_ok(), "Should be able to get memory statistics");
        
        // Verify the latest version is still functional
        let final_init = runtime.call_init(1);
        assert!(final_init.is_ok(), "Final script version should work after multiple reloads");
    }
}