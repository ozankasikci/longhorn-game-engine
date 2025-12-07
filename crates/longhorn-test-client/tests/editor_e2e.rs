//! End-to-end workflow integration tests for the Longhorn Editor.
//!
//! These tests simulate complete user workflows from project load to play.
//!
//! These tests require a running editor instance.
//! Run with: cargo test --test editor_e2e -- --ignored

use longhorn_test_client::EditorClient;

/// Complete workflow: Create entity, add sprite, position it, and play.
#[test]
#[ignore]
fn test_create_entity_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Start in scene mode
    let initial_state = client.get_state().expect("Failed to get state");
    assert_eq!(initial_state.mode, "Scene", "Should start in Scene mode");
    println!("Step 1: Verified in Scene mode");

    // 2. Create a new entity
    let entity_id = client
        .create_entity("WorkflowTestEntity")
        .expect("Failed to create entity");
    println!("Step 2: Created entity with id: {}", entity_id);

    // 3. Set transform position
    client
        .set_property(entity_id, "Transform", "position.x", 100.0)
        .expect("Failed to set position.x");
    client
        .set_property(entity_id, "Transform", "position.y", 50.0)
        .expect("Failed to set position.y");
    println!("Step 3: Set transform position to (100, 50)");

    // 4. Verify the entity exists and has correct properties
    let entity = client.get_entity(entity_id).expect("Failed to get entity");
    assert_eq!(entity.name, "WorkflowTestEntity");
    if let Some(transform) = &entity.transform {
        assert_eq!(transform.position_x, 100.0);
        assert_eq!(transform.position_y, 50.0);
    } else {
        panic!("Entity should have a transform");
    }
    println!("Step 4: Verified entity properties");

    // 5. Enter play mode
    client.play().expect("Failed to enter play mode");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play", "Should be in Play mode");
    println!("Step 5: Entered play mode");

    // 6. Verify entity still exists in play mode
    let entities = client.get_entities().expect("Failed to get entities");
    let found = entities.iter().any(|e| e.id == entity_id);
    assert!(found, "Entity should exist in play mode");
    println!("Step 6: Verified entity exists in play mode");

    // 7. Stop and return to scene mode
    client.stop().expect("Failed to stop");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Scene", "Should return to Scene mode");
    println!("Step 7: Returned to Scene mode");

    // 8. Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
    println!("Step 8: Cleaned up entity");

    println!("Workflow completed successfully!");
}

/// Complete workflow: Select entity, modify, verify selection state.
#[test]
#[ignore]
fn test_selection_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Create multiple entities
    let entity1 = client
        .create_entity("SelectionEntity1")
        .expect("Failed to create entity");
    let entity2 = client
        .create_entity("SelectionEntity2")
        .expect("Failed to create entity");
    println!("Created entities: {} and {}", entity1, entity2);

    // 2. Select first entity
    client
        .select_entity(entity1)
        .expect("Failed to select entity");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(
        state.selected_entity,
        Some(entity1),
        "Entity 1 should be selected"
    );
    println!("Selected entity 1");

    // 3. Modify the selected entity's transform
    client
        .set_property(entity1, "Transform", "rotation", 45.0)
        .expect("Failed to set rotation");
    println!("Modified entity 1 rotation");

    // 4. Select second entity
    client
        .select_entity(entity2)
        .expect("Failed to select entity");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(
        state.selected_entity,
        Some(entity2),
        "Entity 2 should be selected"
    );
    println!("Selected entity 2");

    // 5. Verify first entity kept its modifications
    let entity = client.get_entity(entity1).expect("Failed to get entity");
    if let Some(transform) = &entity.transform {
        assert_eq!(transform.rotation, 45.0);
        println!("Verified entity 1 rotation is preserved");
    }

    // 6. Clean up
    client
        .delete_entity(entity1)
        .expect("Failed to delete entity");
    client
        .delete_entity(entity2)
        .expect("Failed to delete entity");
    println!("Cleaned up entities");
}

/// Complete workflow: Play/Pause/Resume cycle with state verification.
#[test]
#[ignore]
fn test_play_pause_resume_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Ensure we start in scene mode
    client.stop().ok(); // Ignore if already stopped
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Scene");
    println!("Step 1: In Scene mode");

    // 2. Create an entity to observe during play
    let entity_id = client
        .create_entity("PlayTestEntity")
        .expect("Failed to create entity");
    println!("Step 2: Created entity {}", entity_id);

    // 3. Enter play mode
    client.play().expect("Failed to play");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(!state.paused);
    println!("Step 3: Playing");

    // 4. Pause
    client.pause().expect("Failed to pause");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(state.paused);
    println!("Step 4: Paused");

    // 5. Resume
    client.resume().expect("Failed to resume");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(!state.paused);
    println!("Step 5: Resumed");

    // 6. Pause again
    client.pause().expect("Failed to pause");
    let state = client.get_state().expect("Failed to get state");
    assert!(state.paused);
    println!("Step 6: Paused again");

    // 7. Stop while paused (should still work)
    client.stop().expect("Failed to stop");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Scene");
    println!("Step 7: Stopped");

    // 8. Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
    println!("Step 8: Cleaned up");

    println!("Play/Pause/Resume workflow complete!");
}

/// Complete workflow: Create scene with multiple entities and verify hierarchy.
#[test]
#[ignore]
fn test_scene_building_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Get initial entity count
    let initial_entities = client.get_entities().expect("Failed to get entities");
    let initial_count = initial_entities.len();
    println!("Initial entity count: {}", initial_count);

    // 2. Create a scene with multiple entities
    let player = client
        .create_entity("Player")
        .expect("Failed to create Player");
    let enemy1 = client
        .create_entity("Enemy1")
        .expect("Failed to create Enemy1");
    let enemy2 = client
        .create_entity("Enemy2")
        .expect("Failed to create Enemy2");
    let background = client
        .create_entity("Background")
        .expect("Failed to create Background");
    println!(
        "Created entities: Player={}, Enemy1={}, Enemy2={}, Background={}",
        player, enemy1, enemy2, background
    );

    // 3. Position entities
    client
        .set_property(player, "Transform", "position.x", 400.0)
        .expect("Failed to set position");
    client
        .set_property(player, "Transform", "position.y", 300.0)
        .expect("Failed to set position");

    client
        .set_property(enemy1, "Transform", "position.x", 100.0)
        .expect("Failed to set position");
    client
        .set_property(enemy1, "Transform", "position.y", 100.0)
        .expect("Failed to set position");

    client
        .set_property(enemy2, "Transform", "position.x", 700.0)
        .expect("Failed to set position");
    client
        .set_property(enemy2, "Transform", "position.y", 100.0)
        .expect("Failed to set position");

    client
        .set_property(background, "Transform", "position.x", 0.0)
        .expect("Failed to set position");
    client
        .set_property(background, "Transform", "position.y", 0.0)
        .expect("Failed to set position");
    println!("Positioned all entities");

    // 4. Verify entity count increased
    let entities = client.get_entities().expect("Failed to get entities");
    assert_eq!(
        entities.len(),
        initial_count + 4,
        "Should have 4 more entities"
    );
    println!("Verified entity count: {}", entities.len());

    // 5. Verify all entities exist with correct names
    let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Player"));
    assert!(names.contains(&"Enemy1"));
    assert!(names.contains(&"Enemy2"));
    assert!(names.contains(&"Background"));
    println!("Verified all entity names");

    // 6. Enter play mode to verify scene runs
    client.play().expect("Failed to play");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    println!("Scene running in play mode");

    // 7. Stop
    client.stop().expect("Failed to stop");
    println!("Stopped");

    // 8. Clean up all created entities
    client
        .delete_entity(player)
        .expect("Failed to delete Player");
    client
        .delete_entity(enemy1)
        .expect("Failed to delete Enemy1");
    client
        .delete_entity(enemy2)
        .expect("Failed to delete Enemy2");
    client
        .delete_entity(background)
        .expect("Failed to delete Background");
    println!("Cleaned up all entities");

    // 9. Verify count is back to initial
    let final_entities = client.get_entities().expect("Failed to get entities");
    assert_eq!(
        final_entities.len(),
        initial_count,
        "Should be back to initial count"
    );
    println!("Scene building workflow complete!");
}

/// Complete workflow: Entity property modification and dump verification.
#[test]
#[ignore]
fn test_entity_dump_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Create entity
    let entity_id = client
        .create_entity("DumpTestEntity")
        .expect("Failed to create entity");
    println!("Created entity: {}", entity_id);

    // 2. Set various properties
    client
        .set_property(entity_id, "Transform", "position.x", 123.0)
        .expect("Failed to set property");
    client
        .set_property(entity_id, "Transform", "position.y", 456.0)
        .expect("Failed to set property");
    client
        .set_property(entity_id, "Transform", "rotation", 90.0)
        .expect("Failed to set property");
    client
        .set_property(entity_id, "Transform", "scale.x", 2.0)
        .expect("Failed to set property");
    client
        .set_property(entity_id, "Transform", "scale.y", 2.0)
        .expect("Failed to set property");
    println!("Set transform properties");

    // 3. Dump and verify all properties
    let dump = client.dump_entity(entity_id).expect("Failed to dump entity");

    assert_eq!(dump.id, entity_id);
    assert_eq!(dump.name, Some("DumpTestEntity".to_string()));

    if let Some(transform) = &dump.transform {
        assert_eq!(transform.position_x, 123.0);
        assert_eq!(transform.position_y, 456.0);
        assert_eq!(transform.rotation, 90.0);
        assert_eq!(transform.scale_x, 2.0);
        assert_eq!(transform.scale_y, 2.0);
        println!("Verified all transform properties");
    } else {
        panic!("Expected transform in dump");
    }

    println!("Entity dump:");
    println!("  Name: {:?}", dump.name);
    println!("  Transform: {:?}", dump.transform);
    println!("  Has script: {}", dump.has_script);
    println!("  Components: {:?}", dump.component_names);

    // 4. Get components list
    let components = client
        .get_entity_components(entity_id)
        .expect("Failed to get components");
    println!("Components ({}):", components.len());
    for comp in &components {
        println!("  - {}", comp.name);
    }

    // 5. Clean up
    client
        .delete_entity(entity_id)
        .expect("Failed to delete entity");
    println!("Entity dump workflow complete!");
}

/// Complete workflow: UI panel navigation.
#[test]
#[ignore]
fn test_panel_navigation_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Get all panels
    let panels = client.list_panels().expect("Failed to list panels");
    println!("Available panels ({}):", panels.len());
    for panel in &panels {
        println!("  - {} ({})", panel.title, panel.id);
    }

    if panels.is_empty() {
        println!("No panels available, skipping workflow");
        return;
    }

    // 2. Navigate through each panel
    for panel in &panels {
        client
            .focus_panel(&panel.id)
            .expect("Failed to focus panel");
        println!("Focused: {}", panel.title);

        // Verify it's focused
        let ui_state = client.get_ui_state().expect("Failed to get UI state");
        assert_eq!(
            ui_state.focused_panel,
            Some(panel.id.clone()),
            "Panel should be focused"
        );
    }

    println!("Panel navigation workflow complete!");
}

/// Complete workflow: Log monitoring during play.
#[test]
#[ignore]
fn test_log_monitoring_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Get initial log state
    let initial_logs = client.get_log_tail(5).expect("Failed to get log tail");
    println!("Initial log entries: {}", initial_logs.entries.len());

    // 2. Perform some actions that might generate logs
    client.play().expect("Failed to play");
    client.pause().expect("Failed to pause");
    client.resume().expect("Failed to resume");
    client.stop().expect("Failed to stop");
    println!("Performed play/pause/resume/stop cycle");

    // 3. Check logs again
    let logs = client.get_log_tail(20).expect("Failed to get log tail");
    println!("Log entries after actions: {}", logs.entries.len());
    for entry in &logs.entries {
        println!("  [{}] {}: {}", entry.timestamp, entry.level, entry.message);
    }

    println!("Log monitoring workflow complete!");
}

/// Complete workflow: Asset browser exploration.
#[test]
#[ignore]
fn test_asset_exploration_workflow() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // 1. Get asset browser state
    let browser_state = client
        .get_asset_browser_state()
        .expect("Failed to get asset browser state");
    println!("Asset browser - current folder: {}", browser_state.selected_folder);
    println!("Files in view: {}", browser_state.files.len());

    // 2. Get registered assets
    let assets = client.get_assets().expect("Failed to get assets");
    println!("Registered assets: {}", assets.len());

    // 3. Get render state
    let render_state = client.get_render_state().expect("Failed to get render state");
    println!(
        "Loaded textures: {}",
        render_state.loaded_texture_count
    );

    // 4. If there are files, select one
    if !browser_state.files.is_empty() {
        let file = &browser_state.files[0];
        println!("Selecting file: {}", file.name);
        client
            .select_asset_file(&file.path)
            .expect("Failed to select file");

        let updated_state = client
            .get_asset_browser_state()
            .expect("Failed to get state");
        assert_eq!(updated_state.selected_file, Some(file.path.clone()));
        println!("File selected successfully");
    }

    // 5. Load all textures
    let texture_results = client.load_all_textures().expect("Failed to load textures");
    let successful = texture_results.iter().filter(|r| r.success).count();
    println!(
        "Loaded {} of {} textures successfully",
        successful,
        texture_results.len()
    );

    println!("Asset exploration workflow complete!");
}
