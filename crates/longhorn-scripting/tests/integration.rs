use longhorn_core::{Script, Transform, Vec2, World};
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

#[test]
fn test_script_execution_logs() {
    // Create a temporary test game directory
    let test_dir = std::env::temp_dir().join("test_game_execution");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Create a test script that logs
    let script_content = r#"
export default class LoggingScript {
    speed = 42.0;
    static executionOrder = 0;

    onStart(self) {
        console.log("onStart called with speed:", this.speed);
    }

    onUpdate(self, dt) {
        console.log("onUpdate dt:", dt);
    }
}
"#;
    std::fs::write(scripts_dir.join("LoggingScript.ts"), script_content).unwrap();

    // Load and initialize
    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    world.spawn()
        .with(Script::new("LoggingScript.ts"))
        .build();

    // Initialize (calls onStart)
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok(), "Initialize failed: {:?}", result);

    // Update (calls onUpdate)
    let result = runtime.update(&mut world, 0.016);
    assert!(result.is_ok(), "Update failed: {:?}", result);

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_script_reads_transform() {
    let test_dir = std::env::temp_dir().join("test_script_reads_transform");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that logs transform position
    let script = r#"
export default class ReadTransform {
    onStart(self) {
        console.log("pos:", self.transform.position.x, self.transform.position.y);
    }
}
"#;
    std::fs::write(scripts_dir.join("ReadTransform.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    let entity = world
        .spawn()
        .with(Script::new("ReadTransform.ts"))
        .with(Transform::from_position(Vec2::new(100.0, 200.0)))
        .build();

    runtime.initialize(&mut world).unwrap();

    // Verify transform is still there (wasn't corrupted)
    let t = world.get::<Transform>(entity).unwrap();
    assert_eq!(t.position.x, 100.0);
    assert_eq!(t.position.y, 200.0);

    std::fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_script_modifies_transform() {
    let test_dir = std::env::temp_dir().join("test_script_modifies_transform");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that modifies position
    let script = r#"
export default class MoveRight {
    onUpdate(self, dt) {
        self.transform.position.x += 10;
    }
}
"#;
    std::fs::write(scripts_dir.join("MoveRight.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    let entity = world
        .spawn()
        .with(Script::new("MoveRight.ts"))
        .with(Transform::from_position(Vec2::new(0.0, 0.0)))
        .build();

    runtime.initialize(&mut world).unwrap();

    // Run update
    runtime.update(&mut world, 0.016).unwrap();

    // Verify position changed
    {
        let t = world.get::<Transform>(entity).unwrap();
        assert_eq!(t.position.x, 10.0);
    }

    // Run another update
    runtime.update(&mut world, 0.016).unwrap();

    {
        let t = world.get::<Transform>(entity).unwrap();
        assert_eq!(t.position.x, 20.0);
    }

    std::fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_script_handles_null_sprite() {
    let test_dir = std::env::temp_dir().join("test_script_null_sprite");
    let scripts_dir = test_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();

    // Script that checks for null sprite
    let script = r#"
export default class CheckSprite {
    onStart(self) {
        if (self.sprite === null) {
            console.log("no sprite");
        } else {
            console.log("has sprite");
        }
    }
}
"#;
    std::fs::write(scripts_dir.join("CheckSprite.ts"), script).unwrap();

    let mut runtime = ScriptRuntime::new();
    runtime.load_game(&test_dir).unwrap();

    let mut world = World::new();
    // Entity with transform but NO sprite
    world
        .spawn()
        .with(Script::new("CheckSprite.ts"))
        .with(Transform::new())
        .build();

    // Should not crash
    let result = runtime.initialize(&mut world);
    assert!(result.is_ok());

    std::fs::remove_dir_all(&test_dir).ok();
}
