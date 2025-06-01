mod settings_app;

use settings_app::run_settings_app;

fn main() -> glib::ExitCode {
    println!("🎨 Mobile Game Engine - Settings Application");
    println!("🔧 Initializing GTK4 application...");

    println!("✅ Settings application initialized successfully");
    println!("🖥️  Starting GTK application...");

    run_settings_app()
}