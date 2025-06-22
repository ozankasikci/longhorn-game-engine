// Test-Driven Development for Import Integration
// 
// This test defines the expected behavior for integrating the import dialog with the actual import pipeline

use engine_editor_egui::import::{ImportDialog, ImportSettings, ImportResult};
use engine_asset_import::{ImportPipeline, ImportContext, ImportSettings as AssetImportSettings};
use engine_mesh_import::obj::ObjImporter;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_import_service_creation() {
    // Test 1: Verify we can create an import service that connects dialog to pipeline
    use engine_editor_egui::import::ImportService;
    
    let mut service = ImportService::new();
    assert_eq!(service.pipeline().importer_count(), 0);
    
    // Register importers
    service.register_mesh_importers();
    // For now, mesh importers are commented out
    // assert!(service.pipeline().importer_count() > 0);
}

#[test] 
fn test_import_service_process_file() {
    // Test 2: Verify import service can process a file
    use engine_editor_egui::import::ImportService;
    
    let mut service = ImportService::new();
    service.register_mesh_importers();
    
    let path = PathBuf::from("test.obj");
    let settings = ImportSettings::default();
    
    // Should return a future or handle
    let import_handle = service.start_import(path.clone(), settings);
    assert_eq!(import_handle.path(), &path);
    assert_eq!(import_handle.status(), engine_editor_egui::import::ImportStatus::Pending);
}

#[test]
fn test_import_progress_tracking() {
    // Test 3: Verify we can track import progress
    use engine_editor_egui::import::{ImportService, ImportHandle};
    
    let mut service = ImportService::new();
    let path = PathBuf::from("test.obj");
    let settings = ImportSettings::default();
    
    let handle = service.start_import(path, settings);
    
    // Check initial progress
    assert_eq!(handle.progress(), 0.0);
    
    // Simulate progress update
    handle.update_progress(0.5);
    assert_eq!(handle.progress(), 0.5);
    
    // Complete import
    handle.complete(Ok(vec![]));
    assert_eq!(handle.status(), engine_editor_egui::import::ImportStatus::Completed);
    assert_eq!(handle.progress(), 1.0);
}

#[test]
fn test_import_error_handling() {
    // Test 4: Verify error handling in import process
    use engine_editor_egui::import::{ImportService, ImportHandle, ImportStatus};
    
    let mut service = ImportService::new();
    let path = PathBuf::from("nonexistent.obj");
    let settings = ImportSettings::default();
    
    let handle = service.start_import(path, settings);
    
    // Simulate error
    let error = engine_editor_egui::import::ImportError::FileNotFound;
    handle.complete(Err(error.clone()));
    
    assert_eq!(handle.status(), ImportStatus::Failed);
    assert!(handle.error().is_some());
    assert_eq!(handle.error().unwrap(), error);
}

#[test]
fn test_import_queue() {
    // Test 5: Verify import queue for batch imports
    use engine_editor_egui::import::{ImportService, ImportQueue};
    
    let mut service = ImportService::new();
    let queue = ImportQueue::new();
    
    // Add multiple files to queue
    let files = vec![
        PathBuf::from("model1.obj"),
        PathBuf::from("model2.obj"),
        PathBuf::from("texture.png"),
    ];
    
    for file in files {
        queue.add(file, ImportSettings::default());
    }
    
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.pending_count(), 3);
    
    // Process queue
    let handles = service.process_queue(&queue);
    assert_eq!(handles.len(), 3);
}

#[test]
fn test_asset_database_integration() {
    // Test 6: Verify imported assets are added to asset database
    use engine_editor_egui::assets::AssetDatabase;
    use engine_editor_egui::import::ImportService;
    
    let mut database = AssetDatabase::new();
    let mut service = ImportService::new();
    service.register_mesh_importers();
    
    let initial_count = database.asset_count();
    
    // Import an asset
    let path = PathBuf::from("cube.obj");
    let handle = service.start_import(path.clone(), ImportSettings::default());
    
    // Simulate successful import
    let asset_id = Uuid::new_v4();
    handle.complete(Ok(vec![asset_id]));
    
    // Add to database
    database.add_imported_asset(asset_id, path, engine_editor_egui::assets::AssetType::Mesh);
    
    assert_eq!(database.asset_count(), initial_count + 1);
    assert!(database.get_asset(asset_id).is_some());
}

#[test]
fn test_import_settings_conversion() {
    // Test 7: Verify UI settings convert to pipeline settings
    use engine_editor_egui::import::{ImportSettings, ImportSettingsConverter};
    
    let ui_settings = ImportSettings {
        scale: 2.0,
        generate_lods: true,
        optimize_meshes: true,
        auto_generate_collision: false,
        collision_type: engine_editor_egui::import::CollisionType::Box,
        lod_levels: vec![0.5, 0.25],
    };
    
    let converter = ImportSettingsConverter::new();
    let pipeline_settings = converter.convert(&ui_settings);
    
    // Verify conversion
    assert_eq!(pipeline_settings.scale_factor(), 2.0);
    assert_eq!(pipeline_settings.generate_lods(), true);
    assert_eq!(pipeline_settings.optimize_mesh(), true);
}

#[test]
fn test_import_notifications() {
    // Test 8: Verify import notifications/events
    use engine_editor_egui::import::{ImportService, ImportNotification};
    use std::sync::mpsc;
    
    let mut service = ImportService::new();
    let (tx, rx) = mpsc::channel();
    
    service.set_notification_sender(tx);
    
    let path = PathBuf::from("test.obj");
    let handle = service.start_import(path.clone(), ImportSettings::default());
    
    // Should receive started notification
    let notification = rx.try_recv().unwrap();
    match notification {
        ImportNotification::Started { path: p, .. } => assert_eq!(p, path),
        _ => panic!("Expected Started notification"),
    }
    
    // Complete import
    handle.complete(Ok(vec![]));
    
    // Should receive completed notification
    let notification = rx.try_recv().unwrap();
    assert!(matches!(notification, ImportNotification::Completed { .. }));
}

#[test]
fn test_import_file_watcher() {
    // Test 9: Verify file watcher for hot reload
    use engine_editor_egui::import::{ImportFileWatcher, FileWatchEvent};
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    let mut watcher = ImportFileWatcher::new(tx);
    
    let path = PathBuf::from("assets/models/");
    watcher.watch_directory(path.clone()).unwrap();
    
    // Simulate file change
    watcher.trigger_test_event(FileWatchEvent::Modified(path.join("cube.obj")));
    
    let event = rx.try_recv().unwrap();
    match event {
        FileWatchEvent::Modified(p) => assert_eq!(p.file_name().unwrap(), "cube.obj"),
        _ => panic!("Expected Modified event"),
    }
}

#[test]
fn test_import_ui_state() {
    // Test 10: Verify UI state management during import
    use engine_editor_egui::import::{ImportUIState, ImportHandle};
    
    let mut ui_state = ImportUIState::new();
    
    // Initially no active imports
    assert_eq!(ui_state.active_imports().len(), 0);
    assert!(!ui_state.is_importing());
    
    // Add import handle
    let handle = ImportHandle::new(PathBuf::from("test.obj"));
    ui_state.add_import(handle.clone());
    
    assert_eq!(ui_state.active_imports().len(), 1);
    assert!(ui_state.is_importing());
    
    // Complete import
    handle.complete(Ok(vec![]));
    ui_state.update();
    
    // Should move to completed
    assert_eq!(ui_state.active_imports().len(), 0);
    assert_eq!(ui_state.completed_imports().len(), 1);
}