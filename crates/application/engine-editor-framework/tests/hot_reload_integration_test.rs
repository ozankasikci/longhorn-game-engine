//! Tests for hot reload integration with the unified editor

use engine_editor_framework::UnifiedEditorCoordinator;
use engine_runtime_core::{HotReloadManager, HotReloadEvent, AssetType};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_unified_coordinator_with_hot_reload() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    
    // Should have hot reload manager
    assert!(coordinator.hot_reload_manager().is_enabled());
}

#[test]
fn test_hot_reload_asset_reloading() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    let reloaded_assets = Arc::new(Mutex::new(Vec::new()));
    let assets_clone = Arc::clone(&reloaded_assets);
    
    // Register a test handler for script reloading
    coordinator.hot_reload_manager_mut().register_handler(
        AssetType::Script,
        Box::new(move |path, asset_type| {
            assets_clone.lock().unwrap().push((path.to_path_buf(), asset_type));
            log::info!("Reloaded script: {}", path.display());
            Ok(())
        })
    );
    
    // Trigger a reload event
    let script_path = PathBuf::from("scripts/test.lua");
    coordinator.hot_reload_manager_mut()
        .trigger_reload(&script_path, AssetType::Script)
        .unwrap();
    
    // Check that the handler was called
    let assets = reloaded_assets.lock().unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].0, script_path);
    assert_eq!(assets[0].1, AssetType::Script);
}

#[test]
fn test_hot_reload_during_play_mode() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    
    // Start play mode
    coordinator.play_state_manager_mut().start();
    
    // Hot reload should still work during play
    assert!(coordinator.hot_reload_manager().is_enabled());
    
    // Create test directory and watch it
    std::fs::create_dir_all("test_play_reload").unwrap();
    let result = coordinator.hot_reload_manager_mut()
        .watch_path(&PathBuf::from("test_play_reload"), AssetType::Texture);
    assert!(result.is_ok());
    
    // Cleanup
    std::fs::remove_dir_all("test_play_reload").ok();
}

#[test]
fn test_hot_reload_texture_handling() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    let reloaded_textures = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&reloaded_textures);
    
    // Register texture reload handler
    coordinator.hot_reload_manager_mut().register_handler(
        AssetType::Texture,
        Box::new(move |_path, _| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        })
    );
    
    // Trigger multiple texture reloads
    let textures = vec![
        "diffuse.png",
        "normal.jpg", 
        "roughness.jpeg"
    ];
    
    for texture in &textures {
        coordinator.hot_reload_manager_mut()
            .trigger_reload(&PathBuf::from(texture), AssetType::Texture)
            .unwrap();
    }
    
    assert_eq!(*reloaded_textures.lock().unwrap(), textures.len());
}

#[test]
fn test_hot_reload_shader_compilation() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    let shader_compiles = Arc::new(Mutex::new(Vec::new()));
    let compiles_clone = Arc::clone(&shader_compiles);
    
    // Register shader reload handler
    coordinator.hot_reload_manager_mut().register_handler(
        AssetType::Shader,
        Box::new(move |path, _| {
            // Simulate shader compilation
            if path.to_string_lossy().contains("invalid") {
                Err(engine_runtime_core::HotReloadError::ReloadFailed(
                    "Shader compilation failed".to_string()
                ))
            } else {
                compiles_clone.lock().unwrap().push(path.to_path_buf());
                Ok(())
            }
        })
    );
    
    // Test successful shader reload
    let valid_shader = PathBuf::from("shaders/basic.wgsl");
    let result = coordinator.hot_reload_manager_mut()
        .trigger_reload(&valid_shader, AssetType::Shader);
    assert!(result.is_ok());
    
    // Test failed shader reload
    let invalid_shader = PathBuf::from("shaders/invalid.wgsl");
    let result = coordinator.hot_reload_manager_mut()
        .trigger_reload(&invalid_shader, AssetType::Shader);
    assert!(result.is_err());
    
    let compiles = shader_compiles.lock().unwrap();
    assert_eq!(compiles.len(), 1);
    assert_eq!(compiles[0], valid_shader);
}

#[test]
fn test_hot_reload_disable_in_release() {
    let mut coordinator = UnifiedEditorCoordinator::new();
    
    // Hot reload should be enabled in debug mode
    assert!(coordinator.hot_reload_manager().is_enabled());
    
    // Can be disabled manually
    coordinator.hot_reload_manager_mut().set_enabled(false);
    assert!(!coordinator.hot_reload_manager().is_enabled());
    
    // Re-enable
    coordinator.hot_reload_manager_mut().set_enabled(true);
    assert!(coordinator.hot_reload_manager().is_enabled());
}