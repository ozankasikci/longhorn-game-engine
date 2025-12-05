use egui::Context;
use longhorn_engine::Engine;
use crate::{EditorState, SceneTreePanel, InspectorPanel, ViewportPanel};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, engine: &mut Engine) -> bool {
        let mut should_exit = false;

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Game").clicked() {
                        log::info!("Open Game clicked (not implemented)");
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        should_exit = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Left panel - Scene Tree
        egui::SidePanel::left("scene_tree")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.scene_tree.show(ui, engine.world(), &mut self.state);
            });

        // Right panel - Inspector
        egui::SidePanel::right("inspector")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.inspector.show(ui, engine.world_mut(), &self.state);
            });

        // Center panel - Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewport.show(ui);
        });

        should_exit
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
