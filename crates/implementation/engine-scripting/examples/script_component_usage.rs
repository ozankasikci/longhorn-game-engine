//! Example showing how to use the LuaScript component system
//! This demonstrates how to solve the "Script file not found" issue from the Entity Inspector

use engine_scripting::{
    LuaScriptComponentManager, ScriptComponentStatus, ScriptValidation,
    components::LuaScript
};
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LuaScript Component Management Example ===\n");
    
    // Simulate the scenario from the Entity Inspector UI
    let temp_dir = TempDir::new()?;
    let mut manager = LuaScriptComponentManager::new(temp_dir.path().to_path_buf())?;
    
    println!("1. Checking status of missing script 'new_script.lua':");
    let status = manager.get_script_status(2, "new_script.lua"); // Entity ID 2 from screenshot
    println!("   Status: {:?}", status);
    assert_eq!(status, ScriptComponentStatus::FileNotFound);
    
    println!("\n2. Creating the missing script file:");
    let created_path = manager.create_script_file("new_script.lua", Some("entity"))?;
    println!("   Created script at: {:?}", created_path);
    
    println!("\n3. Verifying the script is now valid:");
    let validation = manager.validate_script_path("new_script.lua");
    println!("   Validation result: {:?}", validation);
    assert_eq!(validation, ScriptValidation::Valid);
    
    println!("\n4. Registering LuaScript component with entity (like in the inspector):");
    let lua_script = LuaScript {
        script_path: "new_script.lua".to_string(),
        enabled: true,
        instance_id: None,
        execution_order: 0,
    };
    
    manager.register_script_component(2, &lua_script)?;
    println!("   Component registered successfully");
    
    println!("\n5. Checking final status:");
    let final_status = manager.get_script_status(2, "new_script.lua");
    println!("   Final status: {:?}", final_status);
    assert_eq!(final_status, ScriptComponentStatus::Ready);
    
    println!("\n6. Demonstrating error handling for invalid script:");
    // Create a script with syntax errors
    std::fs::write(temp_dir.path().join("broken_script.lua"), "function broken_syntax(")?;
    
    let broken_status = manager.get_script_status(3, "broken_script.lua");
    println!("   Broken script status: {:?}", broken_status);
    match broken_status {
        ScriptComponentStatus::SyntaxError(_) => println!("   ✓ Syntax error correctly detected"),
        _ => panic!("Expected syntax error"),
    }
    
    println!("\n7. Listing all available scripts:");
    let available_scripts = manager.list_available_scripts()?;
    println!("   Available scripts: {:?}", available_scripts);
    
    println!("\n8. Testing component state changes:");
    // Disable the script
    let disabled_script = LuaScript {
        script_path: "new_script.lua".to_string(),
        enabled: false,
        instance_id: None,
        execution_order: 0,
    };
    
    manager.update_script_component(2, &disabled_script)?;
    let disabled_status = manager.get_script_status(2, "new_script.lua");
    println!("   Disabled script status: {:?}", disabled_status);
    assert_eq!(disabled_status, ScriptComponentStatus::Disabled);
    
    println!("\n=== Summary ===");
    println!("This example demonstrates how to:");
    println!("- Detect missing script files (like in the Entity Inspector)");
    println!("- Create new script files from templates");
    println!("- Validate script syntax");
    println!("- Register and manage LuaScript components");
    println!("- Handle script enable/disable state");
    println!("- Detect and report syntax errors");
    
    println!("\n✅ All operations completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_component_workflow() {
        // This test demonstrates the complete workflow
        let temp_dir = TempDir::new().unwrap();
        let mut manager = LuaScriptComponentManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        // 1. Missing script detection
        assert_eq!(manager.get_script_status(1, "missing.lua"), ScriptComponentStatus::FileNotFound);
        
        // 2. Script creation
        let path = manager.create_script_file("test_script.lua", Some("basic")).unwrap();
        assert!(path.exists());
        
        // 3. Script validation
        assert_eq!(manager.validate_script_path("test_script.lua"), ScriptValidation::Valid);
        
        // 4. Component registration
        let component = LuaScript {
            script_path: "test_script.lua".to_string(),
            enabled: true,
            instance_id: None,
            execution_order: 0,
        };
        
        manager.register_script_component(1, &component).unwrap();
        assert_eq!(manager.get_script_status(1, "test_script.lua"), ScriptComponentStatus::Ready);
        
        // 5. Component state management
        let disabled_component = LuaScript {
            script_path: "test_script.lua".to_string(),
            enabled: false,
            instance_id: None,
            execution_order: 0,
        };
        
        manager.update_script_component(1, &disabled_component).unwrap();
        assert_eq!(manager.get_script_status(1, "test_script.lua"), ScriptComponentStatus::Disabled);
    }
}