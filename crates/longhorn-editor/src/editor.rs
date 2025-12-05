use egui::Context;
use longhorn_engine::Engine;
use crate::{EditorState, EditorMode, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, SceneSnapshot};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    scene_snapshot: Option<SceneSnapshot>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            scene_snapshot: None,
        }
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    pub fn viewport_mut(&mut self) -> &mut ViewportPanel {
        &mut self.viewport
    }

    /// Handle toolbar action and update state
    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, engine: &mut Engine) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::Play => {
                // Capture scene state before playing
                self.scene_snapshot = Some(SceneSnapshot::capture(engine.world()));
                self.state.mode = EditorMode::Play;
                self.state.paused = false;
                log::info!("Entering Play mode");
            }
            ToolbarAction::Pause => {
                self.state.paused = true;
                log::info!("Game paused");
            }
            ToolbarAction::Resume => {
                self.state.paused = false;
                log::info!("Game resumed");
            }
            ToolbarAction::Stop => {
                // Restore scene state
                if let Some(snapshot) = self.scene_snapshot.take() {
                    snapshot.restore(engine.world_mut());
                    log::info!("Scene restored");
                }
                self.state.mode = EditorMode::Scene;
                self.state.paused = false;
                log::info!("Entering Scene mode");
            }
        }
    }

    pub fn show(&mut self, ctx: &Context, engine: &mut Engine, viewport_texture: Option<egui::TextureId>) -> bool {
        let mut should_exit = false;
        let mut toolbar_action = ToolbarAction::None;

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

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar_action = self.toolbar.show(ui, &self.state);
        });

        // Handle toolbar action
        self.handle_toolbar_action(toolbar_action, engine);

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
                // In play mode, show read-only indicator
                if self.state.is_playing() {
                    ui.label("(Read-only during play)");
                    ui.separator();
                }
                self.inspector.show(ui, engine.world_mut(), &self.state);
            });

        // Center panel - Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewport.show(ui, viewport_texture);
        });

        should_exit
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
