//! Tests for TypeScript Hot Reload functionality
//! 
//! These tests define the expected behavior of the hot reload system for TypeScript scripts.
//! Following TDD principles, these tests are written before implementation.

use crate::initialize_v8_platform;
use crate::runtime::TypeScriptRuntime;
use engine_scripting::{
    runtime::ScriptRuntime,
    types::{ScriptId, ScriptMetadata, ScriptType},
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_modification_detection() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create temporary directory for test scripts
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_script.ts");
        
        // Write initial script
        let initial_script = r#"
            let counter = 0;
            function getCounter(): number {
                return counter;
            }
            function increment(): void {
                counter++;
            }
        "#;
        fs::write(&script_path, initial_script).unwrap();
        
        let script_id = ScriptId(1);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: script_path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        // Load and execute initial script
        runtime.load_script(metadata.clone(), initial_script).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test initial state
        let result = runtime.execute_function("getCounter", vec![]).unwrap();
        assert_eq!(result, "0");
        
        // Increment counter
        runtime.execute_function("increment", vec![]).unwrap();
        let result = runtime.execute_function("getCounter", vec![]).unwrap();
        assert_eq!(result, "1");
        
        // Check if hot reload can detect file modification
        let modified = runtime.has_script_changed(script_id).unwrap();
        assert_eq!(modified, false, "Script should not be marked as modified initially");
        
        // Modify the script file
        let modified_script = r#"
            let counter = 0;
            function getCounter(): number {
                return counter;
            }
            function increment(): void {
                counter += 2; // Changed increment behavior
            }
        "#;
        fs::write(&script_path, modified_script).unwrap();
        
        // Check if modification is detected
        let modified = runtime.has_script_changed(script_id).unwrap();
        assert_eq!(modified, true, "Script modification should be detected");
    }

    #[test]
    fn test_hot_reload_with_state_preservation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create temporary directory for test scripts
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("stateful_script.ts");
        
        // Write initial script with state
        let initial_script = r#"
            let gameScore = 100;
            let playerName = "Alice";
            
            function getScore(): number {
                return gameScore;
            }
            
            function addScore(points: number): void {
                gameScore += points;
            }
            
            function getPlayerName(): string {
                return playerName;
            }
        "#;
        fs::write(&script_path, initial_script).unwrap();
        
        let script_id = ScriptId(2);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: script_path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        // Load and execute initial script
        runtime.load_script(metadata.clone(), initial_script).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Modify game state
        runtime.execute_function("addScore", vec!["50".to_string()]).unwrap();
        let score_before = runtime.execute_function("getScore", vec![]).unwrap();
        assert_eq!(score_before, "150");
        
        // Modify the script (add new function, keep existing ones)
        let modified_script = r#"
            let gameScore = 100;
            let playerName = "Alice";
            
            function getScore(): number {
                return gameScore;
            }
            
            function addScore(points: number): void {
                gameScore += points * 2; // Changed multiplier
            }
            
            function getPlayerName(): string {
                return playerName;
            }
            
            function resetScore(): void {
                gameScore = 0;
            }
        "#;
        fs::write(&script_path, modified_script).unwrap();
        
        // Perform hot reload
        runtime.hot_reload_script(script_id).unwrap();
        
        // Test that state was preserved
        let score_after = runtime.execute_function("getScore", vec![]).unwrap();
        assert_eq!(score_after, "150", "State should be preserved during hot reload");
        
        // Test new function works
        runtime.execute_function("resetScore", vec![]).unwrap();
        let score_reset = runtime.execute_function("getScore", vec![]).unwrap();
        assert_eq!(score_reset, "0");
        
        // Test modified behavior
        runtime.execute_function("addScore", vec!["10".to_string()]).unwrap();
        let score_with_multiplier = runtime.execute_function("getScore", vec![]).unwrap();
        assert_eq!(score_with_multiplier, "20", "Modified function should use new logic");
    }

    #[test]
    fn test_dependency_tracking() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create temporary directory for test scripts
        let temp_dir = TempDir::new().unwrap();
        let main_script_path = temp_dir.path().join("main.ts");
        let helper_script_path = temp_dir.path().join("helper.ts");
        
        // Write helper script
        let helper_script = r#"
            function calculateBonus(base: number): number {
                return base * 0.1;
            }
        "#;
        fs::write(&helper_script_path, helper_script).unwrap();
        
        // Write main script that depends on helper
        let main_script = r#"
            // This script depends on helper.ts
            function getTotalScore(baseScore: number): number {
                const bonus = calculateBonus(baseScore);
                return baseScore + bonus;
            }
        "#;
        fs::write(&main_script_path, main_script).unwrap();
        
        let main_script_id = ScriptId(3);
        let helper_script_id = ScriptId(4);
        
        let main_metadata = ScriptMetadata {
            id: main_script_id,
            script_type: ScriptType::TypeScript,
            path: main_script_path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        let helper_metadata = ScriptMetadata {
            id: helper_script_id,
            script_type: ScriptType::TypeScript,
            path: helper_script_path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        // Load helper script first
        runtime.load_script(helper_metadata, helper_script).unwrap();
        runtime.execute_script(helper_script_id).unwrap();
        
        // Load main script and register dependency
        runtime.load_script(main_metadata, main_script).unwrap();
        runtime.add_script_dependency(main_script_id, helper_script_id).unwrap();
        runtime.execute_script(main_script_id).unwrap();
        
        // Test initial functionality
        let result = runtime.execute_function("getTotalScore", vec!["100".to_string()]).unwrap();
        assert_eq!(result, "110"); // 100 + (100 * 0.1)
        
        // Modify helper script
        let modified_helper = r#"
            function calculateBonus(base: number): number {
                return base * 0.2; // Changed bonus multiplier
            }
        "#;
        fs::write(&helper_script_path, modified_helper).unwrap();
        
        // Check if dependency system detects the change
        let main_needs_reload = runtime.script_dependencies_changed(main_script_id).unwrap();
        assert_eq!(main_needs_reload, true, "Main script should need reload when dependency changes");
        
        // Perform hot reload on helper script
        runtime.hot_reload_script(helper_script_id).unwrap();
        
        // Test that main script now uses updated helper logic
        let updated_result = runtime.execute_function("getTotalScore", vec!["100".to_string()]).unwrap();
        assert_eq!(updated_result, "120"); // 100 + (100 * 0.2)
    }

    #[test]
    fn test_incremental_compilation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create a script with multiple functions
        let typescript_code = r#"
            let moduleLevel = "initial";
            
            function function1(): string {
                return "function1_initial";
            }
            
            function function2(): string {
                return "function2_initial";
            }
            
            function getModuleLevel(): string {
                return moduleLevel;
            }
        "#;
        
        let script_id = ScriptId(5);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "incremental_test.ts".to_string(),
            entry_point: None,
        };
        
        // Load and execute initial script
        runtime.load_script(metadata.clone(), typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test initial state
        let result1 = runtime.execute_function("function1", vec![]).unwrap();
        let result2 = runtime.execute_function("function2", vec![]).unwrap();
        let module_result = runtime.execute_function("getModuleLevel", vec![]).unwrap();
        
        assert_eq!(result1, "function1_initial");
        assert_eq!(result2, "function2_initial");
        assert_eq!(module_result, "initial");
        
        // Perform incremental update (only modify function1)
        let updated_code = r#"
            let moduleLevel = "initial";
            
            function function1(): string {
                return "function1_updated";
            }
            
            function function2(): string {
                return "function2_initial";
            }
            
            function getModuleLevel(): string {
                return moduleLevel;
            }
        "#;
        
        // Test incremental reload
        runtime.incremental_reload_script(script_id, updated_code).unwrap();
        
        // Test that only modified function changed
        let updated_result1 = runtime.execute_function("function1", vec![]).unwrap();
        let unchanged_result2 = runtime.execute_function("function2", vec![]).unwrap();
        let unchanged_module = runtime.execute_function("getModuleLevel", vec![]).unwrap();
        
        assert_eq!(updated_result1, "function1_updated");
        assert_eq!(unchanged_result2, "function2_initial"); // Should remain unchanged
        assert_eq!(unchanged_module, "initial"); // Should remain unchanged
    }

    #[test]
    fn test_hot_reload_error_handling() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create temporary directory for test scripts
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("error_test.ts");
        
        // Write valid initial script
        let valid_script = r#"
            function validFunction(): string {
                return "valid";
            }
        "#;
        fs::write(&script_path, valid_script).unwrap();
        
        let script_id = ScriptId(6);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: script_path.to_string_lossy().to_string(),
            entry_point: None,
        };
        
        // Load and execute valid script
        runtime.load_script(metadata.clone(), valid_script).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test initial functionality
        let result = runtime.execute_function("validFunction", vec![]).unwrap();
        assert_eq!(result, "valid");
        
        // Write invalid script (syntax error)
        let invalid_script = r#"
            function invalidFunction(): string {
                return "invalid" // Missing semicolon and other errors
                invalid syntax here
            }
        "#;
        fs::write(&script_path, invalid_script).unwrap();
        
        // Attempt hot reload with invalid script
        let reload_result = runtime.hot_reload_script(script_id);
        assert!(reload_result.is_err(), "Hot reload should fail with invalid script");
        
        // Test that original script still works (rollback)
        let still_valid = runtime.execute_function("validFunction", vec![]).unwrap();
        assert_eq!(still_valid, "valid", "Original script should still work after failed reload");
        
        // Write corrected script
        let corrected_script = r#"
            function validFunction(): string {
                return "updated_and_valid";
            }
        "#;
        fs::write(&script_path, corrected_script).unwrap();
        
        // Test successful reload after error
        runtime.hot_reload_script(script_id).unwrap();
        let corrected_result = runtime.execute_function("validFunction", vec![]).unwrap();
        assert_eq!(corrected_result, "updated_and_valid");
    }

    #[test]
    fn test_multiple_script_hot_reload() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        // Create multiple scripts
        let script1_code = r#"
            function script1Function(): string {
                return "script1_initial";
            }
        "#;
        
        let script2_code = r#"
            function script2Function(): string {
                return "script2_initial";
            }
        "#;
        
        let script1_id = ScriptId(7);
        let script2_id = ScriptId(8);
        
        let metadata1 = ScriptMetadata {
            id: script1_id,
            script_type: ScriptType::TypeScript,
            path: "script1.ts".to_string(),
            entry_point: None,
        };
        
        let metadata2 = ScriptMetadata {
            id: script2_id,
            script_type: ScriptType::TypeScript,
            path: "script2.ts".to_string(),
            entry_point: None,
        };
        
        // Load both scripts
        runtime.load_script(metadata1, script1_code).unwrap();
        runtime.load_script(metadata2, script2_code).unwrap();
        runtime.execute_script(script1_id).unwrap();
        runtime.execute_script(script2_id).unwrap();
        
        // Test initial state
        let result1 = runtime.execute_function("script1Function", vec![]).unwrap();
        let result2 = runtime.execute_function("script2Function", vec![]).unwrap();
        assert_eq!(result1, "script1_initial");
        assert_eq!(result2, "script2_initial");
        
        // Update only script1
        let updated_script1 = r#"
            function script1Function(): string {
                return "script1_updated";
            }
        "#;
        
        runtime.incremental_reload_script(script1_id, updated_script1).unwrap();
        
        // Test that only script1 changed
        let updated_result1 = runtime.execute_function("script1Function", vec![]).unwrap();
        let unchanged_result2 = runtime.execute_function("script2Function", vec![]).unwrap();
        assert_eq!(updated_result1, "script1_updated");
        assert_eq!(unchanged_result2, "script2_initial");
        
        // Update script2
        let updated_script2 = r#"
            function script2Function(): string {
                return "script2_updated";
            }
        "#;
        
        runtime.incremental_reload_script(script2_id, updated_script2).unwrap();
        
        // Test both scripts are updated
        let final_result1 = runtime.execute_function("script1Function", vec![]).unwrap();
        let final_result2 = runtime.execute_function("script2Function", vec![]).unwrap();
        assert_eq!(final_result1, "script1_updated");
        assert_eq!(final_result2, "script2_updated");
    }
}