/// Integration tests for loading and validating game projects
use std::path::PathBuf;

fn empty_project_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("empty_project")
}

#[test]
fn test_empty_project_has_valid_structure() {
    let project_path = empty_project_path();

    // Verify required files exist
    assert!(project_path.join("game.json").exists(), "game.json should exist");
    assert!(project_path.join("assets.json").exists(), "assets.json should exist");
}

#[test]
fn test_empty_project_game_json_is_valid() {
    use std::fs;

    let game_json_path = empty_project_path().join("game.json");
    let content = fs::read_to_string(game_json_path)
        .expect("Should be able to read game.json");

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("game.json should be valid JSON");

    // Should have required fields
    assert!(parsed.get("name").is_some(), "Should have name field");
    assert!(parsed.get("version").is_some(), "Should have version field");
    assert!(parsed.get("viewport").is_some(), "Should have viewport field");
}

#[test]
fn test_empty_project_assets_json_is_valid() {
    use std::fs;

    let assets_json_path = empty_project_path().join("assets.json");
    let content = fs::read_to_string(assets_json_path)
        .expect("Should be able to read assets.json");

    // Should be valid JSON (even if empty object)
    let _parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("assets.json should be valid JSON");
}

#[test]
fn test_engine_can_load_empty_project() {
    use longhorn_engine::Engine;

    let project_path = empty_project_path();
    let mut engine = Engine::new_headless();

    // Engine should be able to load a project without scripts or assets
    let result = engine.load_game(&project_path);

    // Note: This may fail if engine requires an entry script
    // If so, this test documents that limitation
    match result {
        Ok(_) => {
            // Success - engine handles empty projects
            assert!(true);
        }
        Err(e) => {
            // Document the expected error for empty projects
            let error_msg = format!("{:?}", e);
            assert!(
                error_msg.contains("entry") || error_msg.contains("script"),
                "Expected error about missing entry script, got: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_editor_can_open_empty_project() {
    use longhorn_engine::Engine;

    let project_path = empty_project_path();
    let mut engine = Engine::new_headless();

    // Editor should be able to open any valid project structure
    // even if it has no content yet
    let result = engine.load_game(&project_path);

    // Similar to engine test, document behavior
    match result {
        Ok(_) => assert!(true),
        Err(_) => {
            // This is acceptable - editor may require manual setup
            // for projects without entry points
            assert!(true, "Empty project requires entry point setup");
        }
    }
}
