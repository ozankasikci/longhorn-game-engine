// Example: Load and run a TypeScript script
//
// Run with: cargo run -p longhorn-scripting --example run_script

use longhorn_core::{Script, Transform, World};
use longhorn_scripting::ScriptRuntime;
use std::path::PathBuf;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_project");

    println!("Loading scripts from: {:?}", project_path);

    // Create script runtime
    let mut runtime = ScriptRuntime::new();

    // Load the game project (compiles all TypeScript files)
    match runtime.load_game(&project_path) {
        Ok(()) => {
            println!("Scripts loaded successfully!");
            println!("Available scripts: {:?}", runtime.available_scripts());

            // Show parsed properties
            if let Some(props) = runtime.script_properties("PlayerController.ts") {
                println!("\nPlayerController.ts properties:");
                for (name, type_name) in props {
                    println!("  {} : {}", name, type_name);
                }
            }
        }
        Err(e) => {
            println!("Failed to load scripts: {}", e);
            return;
        }
    }

    // Create a world with an entity that has the script and a transform
    let mut world = World::new();
    let entity = world.spawn()
        .with(Script::new("PlayerController.ts"))
        .with(Transform::new())
        .build();

    println!("\nCreated entity {:?} with PlayerController script", entity);

    // Initialize the runtime (calls onStart)
    println!("\n--- Initializing (calling onStart) ---");
    if let Err(e) = runtime.initialize(&mut world) {
        println!("Initialize error: {}", e);
    }

    // Run a few update frames
    println!("\n--- Running 3 update frames ---");
    for frame in 0..3 {
        let dt = 0.016; // ~60fps
        println!("\nFrame {}: dt={:.3}s", frame + 1, dt);
        if let Err(e) = runtime.update(&mut world, dt) {
            println!("Update error: {}", e);
            break;
        }

        // Print current position
        if let Ok(transform) = world.get::<Transform>(entity) {
            println!("  Position: ({:.2}, {:.2})", transform.position.x, transform.position.y);
        }
    }

    println!("\nDone!");
}
