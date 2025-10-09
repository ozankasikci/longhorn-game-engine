//! Test client for the editor control system

use engine_editor_control::{EditorControlClient, EditorCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Editor Control System");
    
    let mut client = EditorControlClient::new(9999);
    
    // Connect to the editor
    println!("🔌 Connecting to editor on port 9999...");
    match client.connect().await {
        Ok(()) => println!("✅ Connected successfully!"),
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            println!("Make sure the editor is running with control server enabled.");
            return Ok(());
        }
    }
    
    // Test ping
    println!("\n📡 Testing ping...");
    match client.ping().await {
        Ok(true) => println!("✅ Ping successful!"),
        Ok(false) => println!("❌ Ping failed"),
        Err(e) => println!("❌ Ping error: {}", e),
    }
    
    // Get scene objects
    println!("\n🏗️  Getting scene objects...");
    match client.get_scene_objects().await {
        Ok(objects) => {
            println!("✅ Found {} scene objects:", objects.len());
            for obj in &objects {
                println!("  - Entity {}: {} (scripts: {:?})", obj.entity_id, obj.name, obj.scripts);
            }
            
            // Test script operations - use existing entity or try entity 1
            let test_entity_id = if let Some(first_obj) = objects.first() {
                first_obj.entity_id
            } else {
                println!("⚠️  No scene objects found, trying to add script to entity 1...");
                1
            };
            
            {
                let entity_id = test_entity_id;
                
                println!("\n🔄 Testing script operations on entity {}...", entity_id);
                
                // Get current scripts
                match client.get_entity_scripts(entity_id).await {
                    Ok(scripts) => {
                        println!("📋 Current scripts: {:?}", scripts);
                        
                        // Test adding a script
                        println!("➕ Adding test script...");
                        match client.add_script(entity_id, "assets/scripts/typescript_hello_world.ts".to_string()).await {
                            Ok(true) => {
                                println!("✅ Script added successfully!");
                                
                                // Verify scripts
                                match client.get_entity_scripts(entity_id).await {
                                    Ok(new_scripts) => println!("📋 Scripts after add: {:?}", new_scripts),
                                    Err(e) => println!("❌ Error getting scripts: {}", e),
                                }
                                
                                // Test removing the script
                                println!("➖ Removing test script...");
                                match client.remove_script(entity_id, "assets/scripts/typescript_hello_world.ts".to_string()).await {
                                    Ok(true) => {
                                        println!("✅ Script removed successfully!");
                                        
                                        // Verify scripts
                                        match client.get_entity_scripts(entity_id).await {
                                            Ok(final_scripts) => println!("📋 Scripts after remove: {:?}", final_scripts),
                                            Err(e) => println!("❌ Error getting scripts: {}", e),
                                        }
                                    },
                                    Ok(false) => println!("❌ Failed to remove script"),
                                    Err(e) => println!("❌ Error removing script: {}", e),
                                }
                            },
                            Ok(false) => println!("❌ Failed to add script"),
                            Err(e) => println!("❌ Error adding script: {}", e),
                        }
                        
                        // Test script replacement
                        if !scripts.is_empty() {
                            let old_script = &scripts[0];
                            println!("🔄 Testing script replacement...");
                            match client.replace_script(entity_id, old_script.clone(), "assets/scripts/typescript_hello_world.ts".to_string()).await {
                                Ok(true) => {
                                    println!("✅ Script replaced successfully!");
                                    
                                    // Replace it back
                                    match client.replace_script(entity_id, "assets/scripts/typescript_hello_world.ts".to_string(), old_script.clone()).await {
                                        Ok(true) => println!("✅ Script replaced back successfully!"),
                                        Ok(false) => println!("❌ Failed to replace script back"),
                                        Err(e) => println!("❌ Error replacing script back: {}", e),
                                    }
                                },
                                Ok(false) => println!("❌ Failed to replace script"),
                                Err(e) => println!("❌ Error replacing script: {}", e),
                            }
                        }
                    },
                    Err(e) => println!("❌ Error getting entity scripts: {}", e),
                }
            }
        },
        Err(e) => println!("❌ Error getting scene objects: {}", e),
    }
    
    // Re-add script for execution test
    println!("\n🔄 Re-adding script for execution test...");
    match client.add_script(1, "assets/scripts/typescript_hello_world.ts".to_string()).await {
        Ok(true) => {
            println!("✅ Script re-added successfully!");
            
            // Wait for script execution
            println!("⏳ Waiting 3 seconds for script execution...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        },
        Ok(false) => println!("❌ Failed to re-add script"),
        Err(e) => println!("❌ Error re-adding script: {}", e),
    }

    // Get logs
    println!("\n📝 Getting recent logs...");
    match client.get_logs(Some(20)).await {
        Ok(logs) => {
            println!("✅ Recent logs ({} entries):", logs.len());
            let mut script_logs_found = false;
            for log in &logs {
                println!("  {}", log);
                if log.contains("🔥 SCRIPT") || log.contains("⚡ SCRIPT") || log.contains("🚀 SCRIPT") || log.contains("Init called") || log.contains("Update called") || log.contains("running!") {
                    script_logs_found = true;
                }
            }
            
            if script_logs_found {
                println!("\n🎉 SUCCESS: Found script execution logs!");
            } else {
                println!("\n❌ FAILURE: No script execution logs found");
                println!("🔍 Scripts may not be executing in the editor");
            }
        },
        Err(e) => println!("❌ Error getting logs: {}", e),
    }
    
    // Disconnect
    client.disconnect().await;
    println!("\n🏁 Test complete!");
    
    Ok(())
}