//! Asset-related integration tests for the Longhorn Editor.
//!
//! Tests for asset browser operations, texture loading, and asset management.
//!
//! These tests require a running editor instance.
//! Run with: cargo test --test editor_assets -- --ignored

use longhorn_test_client::EditorClient;

/// Test getting the asset browser state.
#[test]
#[ignore]
fn test_get_asset_browser_state() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let state = client.get_asset_browser_state().expect("Failed to get asset browser state");

    println!("Asset Browser State:");
    println!("  Selected folder: {}", state.selected_folder);
    println!("  Selected file: {:?}", state.selected_file);
    println!("  Files: {}", state.files.len());
    for file in &state.files {
        println!("    - {} ({}) @ {}", file.name, file.file_type, file.path);
    }
}

/// Test getting the list of registered assets.
#[test]
#[ignore]
fn test_get_assets() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let assets = client.get_assets().expect("Failed to get assets");

    println!("Found {} registered assets:", assets.len());
    for asset in &assets {
        println!("  - id: {}, path: {}, loaded: {}", asset.id, asset.path, asset.loaded);
    }
}

/// Test getting the render state (texture info).
#[test]
#[ignore]
fn test_get_render_state() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let state = client.get_render_state().expect("Failed to get render state");

    println!("Render State:");
    println!("  Loaded texture count: {}", state.loaded_texture_count);
    println!("  Texture IDs: {:?}", state.texture_ids);
    println!("  Sprite count: {}", state.sprite_count);
}

/// Test selecting a file in the asset browser.
#[test]
#[ignore]
fn test_select_asset_file() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get the initial state
    let initial_state = client.get_asset_browser_state().expect("Failed to get asset browser state");

    // If there are files, try to select one
    if !initial_state.files.is_empty() {
        let file_path = &initial_state.files[0].path;
        println!("Selecting file: {}", file_path);

        client.select_asset_file(file_path).expect("Failed to select asset file");

        // Verify the file is now selected
        let new_state = client.get_asset_browser_state().expect("Failed to get asset browser state");
        assert_eq!(
            new_state.selected_file,
            Some(file_path.clone()),
            "Selected file should match"
        );
    } else {
        println!("No files in asset browser to select");
    }
}

/// Test loading all textures.
#[test]
#[ignore]
fn test_load_all_textures() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let results = client.load_all_textures().expect("Failed to load all textures");

    println!("Loaded {} textures:", results.len());
    for result in &results {
        if result.success {
            println!("  ✓ id: {}, path: {}", result.id, result.path);
        } else {
            println!(
                "  ✗ id: {}, path: {}, error: {:?}",
                result.id, result.path, result.error
            );
        }
    }
}

/// Test loading a specific texture by asset ID.
#[test]
#[ignore]
fn test_load_texture_by_id() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // First get the registered assets
    let assets = client.get_assets().expect("Failed to get assets");

    // Find a texture asset (look for common image extensions)
    let texture_asset = assets.iter().find(|a| {
        let path = a.path.to_lowercase();
        path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
    });

    if let Some(asset) = texture_asset {
        println!("Loading texture: {} (id: {})", asset.path, asset.id);

        let result = client.load_texture(asset.id).expect("Failed to load texture");

        println!("Load result:");
        println!("  Success: {}", result.success);
        println!("  Path: {}", result.path);
        if let Some(error) = &result.error {
            println!("  Error: {}", error);
        }

        assert!(result.success, "Texture should load successfully");
    } else {
        println!("No texture assets found to test loading");
    }
}

/// Test that render state updates after loading textures.
#[test]
#[ignore]
fn test_render_state_updates_after_texture_load() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get initial render state
    let initial_state = client.get_render_state().expect("Failed to get render state");
    println!(
        "Initial loaded texture count: {}",
        initial_state.loaded_texture_count
    );

    // Load all textures
    let results = client.load_all_textures().expect("Failed to load all textures");
    let successful_loads = results.iter().filter(|r| r.success).count();
    println!("Successfully loaded {} textures", successful_loads);

    // Get updated render state
    let updated_state = client.get_render_state().expect("Failed to get render state");
    println!(
        "Updated loaded texture count: {}",
        updated_state.loaded_texture_count
    );

    // Verify texture count increased (or stayed same if already loaded)
    assert!(
        updated_state.loaded_texture_count >= initial_state.loaded_texture_count,
        "Loaded texture count should not decrease"
    );
}

/// Test opening an asset file.
#[test]
#[ignore]
fn test_open_asset_file() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get the asset browser state
    let state = client.get_asset_browser_state().expect("Failed to get asset browser state");

    // Find a text-editable file
    let editable_file = state.files.iter().find(|f| f.is_text_editable);

    if let Some(file) = editable_file {
        println!("Opening file: {}", file.path);
        client
            .open_asset_file(&file.path)
            .expect("Failed to open asset file");
        println!("File opened successfully");
    } else {
        println!("No text-editable files found to test opening");
    }
}

/// Test double-clicking an asset file (should open it).
#[test]
#[ignore]
fn test_double_click_asset_file() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get the asset browser state
    let state = client.get_asset_browser_state().expect("Failed to get asset browser state");

    if !state.files.is_empty() {
        let file_path = &state.files[0].path;
        println!("Double-clicking file: {}", file_path);

        client
            .double_click_asset_file(file_path)
            .expect("Failed to double-click asset file");

        println!("Double-click action completed");
    } else {
        println!("No files in asset browser to double-click");
    }
}

/// Test asset context menu - open in editor.
#[test]
#[ignore]
fn test_asset_context_open_in_editor() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Get the asset browser state
    let state = client.get_asset_browser_state().expect("Failed to get asset browser state");

    // Find a text-editable file (like a script)
    let editable_file = state.files.iter().find(|f| f.is_text_editable);

    if let Some(file) = editable_file {
        println!("Opening in editor via context menu: {}", file.path);

        client
            .asset_context_open_in_editor(&file.path)
            .expect("Failed to open in editor");

        println!("Context menu action completed");
    } else {
        println!("No text-editable files found for context menu test");
    }
}
