use longhorn_core::{Script, World};
use longhorn_scripting::ScriptRuntime;
use std::path::PathBuf;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test_game")
}

#[test]
fn test_load_game_with_scripts() {
    let mut runtime = ScriptRuntime::new();
    let result = runtime.load_game(fixtures_path());

    assert!(result.is_ok());
    assert!(runtime.is_loaded());

    let scripts = runtime.available_scripts();
    assert!(!scripts.is_empty());
    assert!(scripts.contains(&"TestScript.ts"));
}

#[test]
fn test_script_properties_parsed() {
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(fixtures_path()).unwrap();

    let props = runtime.script_properties("TestScript.ts");
    assert!(props.is_some());

    let props = props.unwrap();
    assert!(props.contains_key("speed"));
    assert!(props.contains_key("name"));
}

#[test]
fn test_script_lifecycle() {
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(fixtures_path()).unwrap();

    let mut world = World::new();

    // Create an entity with a script
    world.spawn()
        .with(Script::new("TestScript.ts"))
        .build();

    // Initialize should succeed
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok());
    assert!(runtime.is_initialized());

    // Update should succeed
    let result = runtime.update(&mut world, 0.016);
    assert!(result.is_ok());
}
