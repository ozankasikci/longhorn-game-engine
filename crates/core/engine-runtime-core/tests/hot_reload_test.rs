//! Tests for hot reload functionality

use engine_runtime_core::{HotReloadManager, HotReloadEvent, HotReloadError, AssetType};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct TestReloadHandler {
    reloaded_assets: Arc<Mutex<Vec<(PathBuf, AssetType)>>>,
    reload_count: Arc<Mutex<u32>>,
}

impl TestReloadHandler {
    fn new() -> Self {
        Self::default()
    }
    
    fn get_reload_count(&self) -> u32 {
        *self.reload_count.lock().unwrap()
    }
    
    fn get_reloaded_assets(&self) -> Vec<(PathBuf, AssetType)> {
        self.reloaded_assets.lock().unwrap().clone()
    }
}

#[test]
fn test_hot_reload_manager_creation() {
    let manager = HotReloadManager::new();
    assert!(manager.is_enabled());
    assert_eq!(manager.get_watched_paths().len(), 0);
}

#[test]
fn test_hot_reload_watch_path() {
    let mut manager = HotReloadManager::new();
    let path = PathBuf::from("test_hot_reload_watch");
    
    // Create test directory
    std::fs::create_dir_all(&path).unwrap();
    
    let result = manager.watch_path(&path, AssetType::Texture);
    assert!(result.is_ok());
    assert_eq!(manager.get_watched_paths().len(), 1);
    assert!(manager.is_watching(&path));
    
    // Cleanup
    std::fs::remove_dir_all(&path).ok();
}

#[test]
fn test_hot_reload_unwatch_path() {
    let mut manager = HotReloadManager::new();
    let path = PathBuf::from("test_hot_reload_unwatch");
    
    // Create test directory
    std::fs::create_dir_all(&path).unwrap();
    
    manager.watch_path(&path, AssetType::Model).unwrap();
    assert!(manager.is_watching(&path));
    
    manager.unwatch_path(&path);
    assert!(!manager.is_watching(&path));
    assert_eq!(manager.get_watched_paths().len(), 0);
    
    // Cleanup
    std::fs::remove_dir_all(&path).ok();
}

#[test]
fn test_hot_reload_event_detection() {
    let mut manager = HotReloadManager::new();
    let test_file = PathBuf::from("test_assets/test.txt");
    
    // Create test directory and file
    std::fs::create_dir_all("test_assets").unwrap();
    std::fs::write(&test_file, "initial content").unwrap();
    
    // Watch the directory
    manager.watch_path(&PathBuf::from("test_assets"), AssetType::Script).unwrap();
    
    // Modify the file
    std::thread::sleep(Duration::from_millis(200));
    std::fs::write(&test_file, "modified content").unwrap();
    
    // Check for events (give watcher more time to detect changes)
    std::thread::sleep(Duration::from_millis(500));
    let events = manager.poll_events();
    
    assert!(!events.is_empty(), "No events detected");
    
    // Debug print events
    for event in &events {
        println!("Event: {:?}", event);
    }
    
    assert!(events.iter().any(|e| match e {
        HotReloadEvent::FileModified(p, _) => p.ends_with("test.txt"),
        _ => false
    }), "No FileModified event for test.txt found");
    
    // Cleanup
    std::fs::remove_dir_all("test_assets").ok();
}

#[test]
fn test_hot_reload_handler_registration() {
    let mut manager = HotReloadManager::new();
    let handler = TestReloadHandler::new();
    let reload_count = Arc::clone(&handler.reload_count);
    
    // Register handler
    manager.register_handler(AssetType::Script, Box::new(move |path, asset_type| {
        let mut count = reload_count.lock().unwrap();
        *count += 1;
        log::info!("Reloading {:?} asset: {}", asset_type, path.display());
        Ok(())
    }));
    
    // Trigger reload
    let test_path = PathBuf::from("scripts/test.lua");
    manager.trigger_reload(&test_path, AssetType::Script).unwrap();
    
    assert_eq!(handler.get_reload_count(), 1);
}

#[test]
fn test_hot_reload_disable_enable() {
    let mut manager = HotReloadManager::new();
    assert!(manager.is_enabled());
    
    manager.set_enabled(false);
    assert!(!manager.is_enabled());
    
    // Should not process events when disabled
    let path = PathBuf::from("assets/disabled");
    let result = manager.watch_path(&path, AssetType::Texture);
    assert!(matches!(result, Err(HotReloadError::Disabled)));
    
    manager.set_enabled(true);
    assert!(manager.is_enabled());
}

#[test]
fn test_hot_reload_batch_events() {
    let mut manager = HotReloadManager::new();
    
    // Enable batching with 100ms window
    manager.set_batch_window(Duration::from_millis(100));
    
    // Simulate multiple rapid file changes
    let files = vec![
        (PathBuf::from("texture1.png"), AssetType::Texture),
        (PathBuf::from("texture2.png"), AssetType::Texture),
        (PathBuf::from("model.obj"), AssetType::Model),
    ];
    
    for (path, asset_type) in &files {
        manager.queue_reload_event(HotReloadEvent::FileModified(path.clone(), *asset_type));
    }
    
    // First call with batch window returns empty (waiting for window)
    let batched = manager.get_batched_events();
    if manager.batch_window() > Duration::ZERO {
        assert_eq!(batched.len(), 0);
        
        // Wait for batch window to expire
        std::thread::sleep(Duration::from_millis(110));
        
        // Now should get all events
        let batched = manager.get_batched_events();
        assert_eq!(batched.len(), files.len());
    } else {
        assert_eq!(batched.len(), files.len());
    }
}

#[test]
fn test_hot_reload_error_handling() {
    let mut manager = HotReloadManager::new();
    
    // Register handler that fails
    manager.register_handler(AssetType::Shader, Box::new(|_path, _| {
        Err(HotReloadError::ReloadFailed("Shader compilation failed".to_string()))
    }));
    
    let result = manager.trigger_reload(&PathBuf::from("shader.wgsl"), AssetType::Shader);
    assert!(matches!(result, Err(HotReloadError::ReloadFailed(_))));
}

#[test]
fn test_hot_reload_recursive_watch() {
    let mut manager = HotReloadManager::new();
    
    // Create nested directory structure
    std::fs::create_dir_all("test_recursive/sub1/sub2").unwrap();
    std::fs::write("test_recursive/file1.txt", "content").unwrap();
    std::fs::write("test_recursive/sub1/file2.txt", "content").unwrap();
    std::fs::write("test_recursive/sub1/sub2/file3.txt", "content").unwrap();
    
    // Watch recursively
    manager.watch_recursive(&PathBuf::from("test_recursive"), AssetType::Script).unwrap();
    
    // Should watch all subdirectories
    assert!(manager.is_watching(&PathBuf::from("test_recursive")));
    assert!(manager.is_watching(&PathBuf::from("test_recursive/sub1")));
    assert!(manager.is_watching(&PathBuf::from("test_recursive/sub1/sub2")));
    
    // Cleanup
    std::fs::remove_dir_all("test_recursive").ok();
}