// Test-Driven Development for Phase 19.5: Extract Asset Management
//
// This test verifies that asset management functionality is properly extracted
// from engine-editor-egui into a dedicated engine-editor-assets crate.

use std::path::Path;

#[test]
fn test_asset_crate_exists() {
    // Test 1: Verify the new asset management crate exists
    let asset_crate_path = Path::new("../engine-editor-assets/Cargo.toml");
    assert!(
        asset_crate_path.exists(),
        "engine-editor-assets crate should exist"
    );
}

#[test]
fn test_asset_crate_structure() {
    // Test 2: Verify the crate has the expected structure
    let expected_files = vec![
        "../engine-editor-assets/src/lib.rs",
        "../engine-editor-assets/src/texture_manager.rs",
        "../engine-editor-assets/src/asset_loader.rs",
        "../engine-editor-assets/src/asset_cache.rs",
        "../engine-editor-assets/src/types.rs",
    ];

    for file in expected_files {
        let path = Path::new(file);
        assert!(path.exists(), "File {} should exist", file);
    }
}

#[test]
fn test_asset_files_removed_from_original() {
    // Test 3: Verify asset-related files have been removed from the original location
    let removed_files = vec![
        "src/assets.rs", // This file should be removed or significantly reduced
    ];

    for file in removed_files {
        let path = Path::new(file);
        if path.exists() {
            // If the file still exists, it should be minimal (just re-exports or minimal code)
            let contents = std::fs::read_to_string(path).unwrap();
            assert!(
                contents.lines().count() < 20,
                "File {} should be removed or minimal (less than 20 lines)",
                file
            );
        }
    }
}

#[test]
fn test_asset_crate_dependencies() {
    // Test 4: Verify the asset crate has the correct dependencies
    let cargo_path = Path::new("../engine-editor-assets/Cargo.toml");
    if cargo_path.exists() {
        let contents = std::fs::read_to_string(cargo_path).unwrap();

        // Check for essential dependencies
        assert!(
            contents.contains("egui"),
            "Should depend on egui for TextureId"
        );
        assert!(
            contents.contains("engine-resource-core"),
            "Should depend on resource core"
        );
        assert!(
            contents.contains("serde"),
            "Should depend on serde for serialization"
        );
    }
}

#[test]
fn test_asset_types_exported() {
    // Test 5: Verify that essential asset types are properly exported
    // This test will compile only if the types are properly exported

    // Note: This is a compile-time test. If it compiles, the test passes.
    // We're checking that these types exist and are accessible
    use engine_editor_assets::{AssetHandle, AssetLoadError, TextureAsset};

    // Verify we can create instances
    let _ = AssetHandle::new(1);
    let _ = AssetHandle::invalid();

    // Verify error type can be created
    let _ = AssetLoadError::NotFound("test".to_string());
}

#[test]
fn test_texture_manager_functionality() {
    // Test 6: Verify TextureManager can be created and used
    use engine_editor_assets::TextureManager;

    let mut texture_manager = TextureManager::new();

    // Test that we can create a default texture
    let default_texture = texture_manager.get_default_texture();
    assert_eq!(default_texture.name, "default");

    // Test that we can register a texture
    let texture_id = egui::TextureId::default();
    let handle = texture_manager.register_texture(
        "test_texture".to_string(),
        texture_id,
        egui::Vec2::new(256.0, 256.0),
        "test.png".to_string(),
    );

    assert!(handle.is_valid());

    // Test that we can retrieve the texture
    let retrieved = texture_manager.get_texture(handle);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_texture");
}

#[test]
fn test_asset_loader_interface() {
    // Test 7: Verify AssetLoader trait and implementations exist
    use engine_editor_assets::{AssetLoader, TextureLoader};

    // Create a texture loader
    let loader = TextureLoader::new();

    // Test that the loader implements the AssetLoader trait
    fn assert_loader<T: AssetLoader>(_loader: &T) {}
    assert_loader(&loader);
}

#[test]
fn test_asset_cache_functionality() {
    // Test 8: Verify AssetCache can store and retrieve assets
    use engine_editor_assets::{AssetCache, TextureAsset};

    let mut cache: AssetCache<TextureAsset> = AssetCache::new();

    // Create a test texture asset
    let texture = TextureAsset {
        id: egui::TextureId::default(),
        name: "cached_texture".to_string(),
        size: egui::Vec2::new(512.0, 512.0),
        path: "cache_test.png".to_string(),
    };

    // Cache the asset
    let handle = cache.insert("test_key".to_string(), texture.clone());

    // Retrieve the asset
    let retrieved = cache.get(&"test_key".to_string());
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "cached_texture");

    // Test cache eviction
    assert!(cache.contains_key(&"test_key".to_string()));
    cache.clear();
    assert!(!cache.contains_key(&"test_key".to_string()));
}

#[test]
fn test_project_asset_functionality() {
    // Test 9: Verify ProjectAsset type and functionality
    use engine_editor_assets::ProjectAsset;

    // Create a file asset
    let file_asset = ProjectAsset::file("test.png");
    assert_eq!(file_asset.name, "test.png");
    assert!(file_asset.children.is_none());

    // Create a folder asset with children
    let child1 = ProjectAsset::file("child1.png");
    let child2 = ProjectAsset::file("child2.png");
    let folder_asset = ProjectAsset::folder("textures", vec![child1, child2]);

    assert_eq!(folder_asset.name, "textures");
    assert!(folder_asset.children.is_some());
    assert_eq!(folder_asset.children.unwrap().len(), 2);
}

#[test]
fn test_asset_handle_system() {
    // Test 10: Verify AssetHandle system works correctly
    use engine_editor_assets::{AssetHandle, AssetHandleGenerator};

    let mut generator = AssetHandleGenerator::new();

    // Generate handles
    let handle1 = generator.generate();
    let handle2 = generator.generate();

    // Handles should be unique
    assert_ne!(handle1, handle2);

    // Handles should be valid
    assert!(handle1.is_valid());
    assert!(handle2.is_valid());

    // Invalid handle should not be valid
    let invalid = AssetHandle::invalid();
    assert!(!invalid.is_valid());
}

#[test]
fn test_editor_integration() {
    // Test 11: Verify that engine-editor-egui properly depends on engine-editor-assets
    let cargo_path = Path::new("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_path).unwrap();

    assert!(
        contents.contains("engine-editor-assets"),
        "engine-editor-egui should depend on engine-editor-assets"
    );
}

#[test]
fn test_asset_default_creation() {
    // Test 12: Verify default asset creation functions work
    use engine_editor_assets::{create_default_project_assets, create_default_textures};

    // Test default textures
    let textures = create_default_textures();
    assert!(!textures.is_empty(), "Should create default textures");

    // Test default project assets
    let project_assets = create_default_project_assets();
    assert!(
        !project_assets.is_empty(),
        "Should create default project assets"
    );

    // Verify structure of default project assets
    let has_assets_folder = project_assets.iter().any(|asset| asset.name == "Assets");
    assert!(has_assets_folder, "Should have an Assets folder");
}
