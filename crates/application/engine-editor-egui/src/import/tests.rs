// Test-Driven Development for Phase 20.4: Editor Integration
//
// This test defines the expected behavior for asset import integration in the editor

use engine_resource_core::ResourceId;
use std::path::PathBuf;

use crate::import::*;
use crate::panels::asset_browser::{AssetBrowser, AssetBrowserState, AssetInfo, AssetType};

#[test]
fn test_import_dialog_creation() {
    // Test 1: Verify import dialog can be created and shown
    let mut import_dialog = ImportDialog::new();

    // Check dialog initial state
    assert!(!import_dialog.is_visible());
    assert!(import_dialog.selected_files().is_empty());
    assert!(import_dialog.import_settings().is_default());

    // Show dialog with files
    let files = vec![
        PathBuf::from("assets/model.obj"),
        PathBuf::from("assets/texture.png"),
    ];
    import_dialog.show_with_files(files.clone());

    assert!(import_dialog.is_visible());
    assert_eq!(import_dialog.selected_files(), &files);
}

#[test]
fn test_import_settings_ui() {
    // Test 2: Verify import settings can be modified through UI
    let mut settings = ImportSettings::default();

    // Check default settings
    assert_eq!(settings.scale, 1.0);
    assert!(!settings.generate_lods);
    assert!(settings.optimize_meshes);
    assert!(!settings.auto_generate_collision);

    // Modify settings
    settings.scale = 2.0;
    settings.generate_lods = true;
    settings.lod_levels = vec![10.0, 50.0, 100.0];
    settings.auto_generate_collision = true;
    settings.collision_type = CollisionType::ConvexHull;

    // Verify modifications
    assert_eq!(settings.scale, 2.0);
    assert!(settings.generate_lods);
    assert_eq!(settings.lod_levels.len(), 3);
    assert!(settings.auto_generate_collision);
}

#[test]
fn test_import_preview() {
    // Test 3: Verify import preview functionality
    let preview = ImportPreview::new();

    // Load a preview for a mesh file
    let mesh_path = PathBuf::from("test_assets/cube.obj");
    let preview_result = preview.generate_preview(&mesh_path);

    assert!(preview_result.is_ok());
    let preview_data = preview_result.unwrap();

    // Check preview data
    assert!(preview_data.has_mesh);
    assert_eq!(preview_data.vertex_count, 8);
    assert_eq!(preview_data.triangle_count, 12);
    assert!(preview_data.bounds.is_some());
    assert!(preview_data.thumbnail.is_some());
}

#[test]
fn test_asset_browser_integration() {
    // Test 4: Verify asset browser shows imported assets
    let _asset_browser = AssetBrowser::new();
    let mut browser_state = AssetBrowserState::default();

    // Import an asset
    let imported_asset_id = ResourceId::generate();
    let asset_info = AssetInfo {
        id: imported_asset_id,
        name: "ImportedCube".to_string(),
        path: PathBuf::from("assets/imported/cube.mesh"),
        asset_type: AssetType::Mesh,
        size_bytes: 1024,
        import_time: std::time::SystemTime::now(),
    };

    // Add to browser
    browser_state.add_asset(asset_info.clone());

    // Verify asset appears
    assert!(browser_state.has_asset(&imported_asset_id));
    let found_asset = browser_state.get_asset(&imported_asset_id).unwrap();
    assert_eq!(found_asset.name, "ImportedCube");
    assert_eq!(found_asset.asset_type, AssetType::Mesh);
}

#[test]
fn test_drag_drop_import() {
    // Test 5: Verify drag and drop import functionality
    use DragDropHandler;

    let mut drag_drop = DragDropHandler::new();

    // Simulate dragging files
    let dropped_files = vec![
        PathBuf::from("/Users/test/model.fbx"),
        PathBuf::from("/Users/test/texture.jpg"),
        PathBuf::from("/Users/test/sound.wav"),
    ];

    drag_drop.handle_drop(dropped_files.clone());

    // Check that files are queued for import
    assert!(drag_drop.has_pending_imports());
    let pending = drag_drop.get_pending_imports();
    assert_eq!(pending.len(), 3);

    // Check file type detection
    assert_eq!(
        drag_drop.detect_file_type(&dropped_files[0]),
        Some(FileType::Mesh)
    );
    assert_eq!(
        drag_drop.detect_file_type(&dropped_files[1]),
        Some(FileType::Texture)
    );
    assert_eq!(
        drag_drop.detect_file_type(&dropped_files[2]),
        Some(FileType::Audio)
    );
}

#[test]
fn test_import_progress_tracking() {
    // Test 6: Verify import progress UI
    use crate::import::progress::{ImportProgress, ImportStatus, ImportTask};

    let mut progress = ImportProgress::new();

    // Start import tasks
    let task1 = ImportTask {
        id: 1,
        file_path: PathBuf::from("model1.obj"),
        total_bytes: 1000,
        processed_bytes: 0,
        status: ImportStatus::Pending,
    };

    let task2 = ImportTask {
        id: 2,
        file_path: PathBuf::from("model2.fbx"),
        total_bytes: 2000,
        processed_bytes: 0,
        status: ImportStatus::Pending,
    };

    progress.add_task(task1);
    progress.add_task(task2);

    assert_eq!(progress.active_tasks(), 2);
    assert_eq!(progress.overall_progress(), 0.0);

    // Update progress
    progress.update_task(1, 500, ImportStatus::Processing);
    assert_eq!(progress.get_task(1).unwrap().processed_bytes, 500);
    assert!((progress.overall_progress() - 0.167).abs() < 0.01); // 500/3000

    // Complete a task
    progress.update_task(1, 1000, ImportStatus::Completed);
    assert_eq!(progress.active_tasks(), 1);
}

#[test]
fn test_import_error_handling() {
    // Test 7: Verify import error handling and display
    use crate::import::error::{ImportError, ImportErrorDialog, ImportErrorType};

    let mut error_dialog = ImportErrorDialog::new();

    // Add import errors
    let error1 = ImportError {
        file_path: PathBuf::from("broken.obj"),
        error_type: ImportErrorType::InvalidFormat,
        message: "Invalid OBJ format: missing vertices".to_string(),
        recoverable: false,
    };

    let error2 = ImportError {
        file_path: PathBuf::from("huge.fbx"),
        error_type: ImportErrorType::FileTooLarge,
        message: "File exceeds maximum size of 100MB".to_string(),
        recoverable: true,
    };

    error_dialog.add_error(error1);
    error_dialog.add_error(error2);

    assert!(error_dialog.has_errors());
    assert_eq!(error_dialog.error_count(), 2);
    assert_eq!(error_dialog.recoverable_errors().len(), 1);
}

#[test]
fn test_import_history() {
    // Test 8: Verify import history tracking
    use {ImportHistory, ImportRecord};

    let mut history = ImportHistory::new();

    // Add import records
    let record1 = ImportRecord {
        timestamp: std::time::SystemTime::now(),
        source_path: PathBuf::from("original/model.obj"),
        imported_path: PathBuf::from("assets/imported/model.mesh"),
        resource_id: ResourceId::generate(),
        import_settings: ImportSettings::default(),
        success: true,
    };

    history.add_record(record1.clone());

    // Check history
    assert_eq!(history.total_imports(), 1);
    assert_eq!(history.successful_imports(), 1);
    assert_eq!(history.failed_imports(), 0);

    // Find by source path
    let found = history.find_by_source(&PathBuf::from("original/model.obj"));
    assert!(found.is_some());
    assert_eq!(found.unwrap().resource_id, record1.resource_id);

    // Check recent imports
    let recent = history.get_recent(10);
    assert_eq!(recent.len(), 1);
}

#[test]
fn test_batch_import() {
    // Test 9: Verify batch import functionality
    use {BatchImportOptions, BatchImporter};

    let mut batch_importer = BatchImporter::new();

    let files = vec![
        PathBuf::from("models/character.fbx"),
        PathBuf::from("models/weapon.obj"),
        PathBuf::from("models/environment.gltf"),
    ];

    let options = BatchImportOptions {
        use_same_settings: true,
        base_settings: ImportSettings {
            scale: 0.01, // Convert from cm to m
            optimize_meshes: true,
            generate_lods: true,
            ..Default::default()
        },
        output_directory: PathBuf::from("assets/imported/batch"),
    };

    // Start batch import
    let batch_id = batch_importer.start_batch(files, options);

    assert!(batch_importer.is_batch_active(batch_id));
    assert_eq!(batch_importer.batch_total_files(batch_id), 3);
    assert_eq!(batch_importer.batch_completed_files(batch_id), 0);
}

#[test]
fn test_import_hot_reload() {
    // Test 10: Verify hot reload of imported assets
    use {HotReloadEvent, HotReloadWatcher};

    let mut watcher = HotReloadWatcher::new();

    // Watch an imported asset
    let asset_path = PathBuf::from("assets/imported/model.mesh");
    let source_path = PathBuf::from("original/model.obj");
    let resource_id = ResourceId::generate();

    watcher.watch_asset(resource_id, asset_path.clone(), source_path.clone());

    assert!(watcher.is_watching(&resource_id));

    // Simulate file change
    let event = HotReloadEvent::SourceModified {
        resource_id,
        source_path: source_path.clone(),
    };

    // Check that reimport is triggered
    let actions = watcher.handle_event(event);
    assert!(!actions.is_empty());
    assert!(matches!(actions[0], HotReloadAction::Reimport { .. }));
}
