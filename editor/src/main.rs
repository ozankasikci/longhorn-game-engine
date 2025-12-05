use eframe::egui;
use longhorn_editor::Editor;
use longhorn_engine::Engine;
use longhorn_core::{Name, Transform, Sprite, Enabled};
use glam::Vec2;

struct EditorApp {
    engine: Engine,
    editor: Editor,
}

impl EditorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut engine = Engine::new_headless();

        // Spawn test entities
        engine.world_mut()
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Sprite::new(
                longhorn_core::AssetId::new(1),
                Vec2::new(32.0, 32.0),
            ))
            .with(Enabled::default())
            .build();

        engine.world_mut()
            .spawn()
            .with(Name::new("Enemy"))
            .with(Transform::from_position(Vec2::new(300.0, 150.0)))
            .with(Sprite::new(
                longhorn_core::AssetId::new(2),
                Vec2::new(64.0, 64.0),
            ))
            .with(Enabled::default())
            .build();

        Self {
            engine,
            editor: Editor::new(),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let should_exit = self.editor.show(ctx, &mut self.engine);

        if should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("Longhorn Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Longhorn Editor",
        options,
        Box::new(|cc| Box::new(EditorApp::new(cc))),
    )
}
