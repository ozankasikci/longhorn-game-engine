use egui::Context;
use longhorn_engine::Engine;
use longhorn_scripting::set_console_callback;
use std::sync::Arc;
use crate::{EditorState, EditorMode, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, SceneSnapshot, ConsolePanel, ScriptConsole};
use crate::remote::{RemoteCommand, RemoteResponse, ResponseData, EntityInfo};
use longhorn_core::{Name, Transform, World, EntityHandle};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    scene_snapshot: Option<SceneSnapshot>,
    console_panel: ConsolePanel,
    console: ScriptConsole,
    console_open: bool,
}

impl Editor {
    pub fn new() -> Self {
        let console = ScriptConsole::new();

        // Set up console callback for script runtime
        let console_clone = console.clone();
        set_console_callback(Some(Arc::new(move |level: &str, message: &str| {
            match level {
                "error" => console_clone.error(message.to_string()),
                "warn" => console_clone.warn(message.to_string()),
                _ => console_clone.log(message.to_string()),
            }
        })));

        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            scene_snapshot: None,
            console_panel: ConsolePanel::new(),
            console,
            console_open: false,
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

    pub fn console(&self) -> &ScriptConsole {
        &self.console
    }

    /// Process a remote command and return a response
    pub fn process_remote_command(
        &mut self,
        command: RemoteCommand,
        engine: &mut Engine,
    ) -> RemoteResponse {
        match command {
            RemoteCommand::Ping => {
                RemoteResponse::ok()
            }

            RemoteCommand::Play => {
                self.handle_toolbar_action(crate::ToolbarAction::Play, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Pause => {
                self.handle_toolbar_action(crate::ToolbarAction::Pause, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Resume => {
                self.handle_toolbar_action(crate::ToolbarAction::Resume, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Stop => {
                self.handle_toolbar_action(crate::ToolbarAction::Stop, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::ToggleConsole => {
                self.console_open = !self.console_open;
                RemoteResponse::ok()
            }

            RemoteCommand::GetState => {
                let selected = self.state.selected_entity
                    .map(|e| e.id() as u64);
                RemoteResponse::with_data(ResponseData::State {
                    mode: format!("{:?}", self.state.mode),
                    paused: self.state.paused,
                    entity_count: engine.world().len(),
                    selected_entity: selected,
                })
            }

            RemoteCommand::GetEntities => {
                let entities: Vec<EntityInfo> = engine.world().inner().iter()
                    .map(|entity_ref| {
                        let entity = entity_ref.entity();
                        let handle = EntityHandle::new(entity);
                        let name = engine.world().get::<Name>(handle)
                            .ok()
                            .map(|n| n.0.clone())
                            .unwrap_or_else(|| format!("Entity {}", entity.id()));
                        EntityInfo {
                            id: entity.id() as u64,
                            name,
                        }
                    })
                    .collect();
                RemoteResponse::with_data(ResponseData::Entities(entities))
            }

            RemoteCommand::SelectEntity { id } => {
                // Find entity by ID
                let found = engine.world().inner().iter()
                    .find(|e| e.entity().id() as u64 == id)
                    .map(|e| e.entity());

                match found {
                    Some(entity) => {
                        self.state.select(Some(entity));
                        RemoteResponse::ok()
                    }
                    None => RemoteResponse::error(format!("Entity not found: {}", id)),
                }
            }

            RemoteCommand::CreateEntity { name } => {
                let entity = engine.world_mut()
                    .spawn()
                    .with(Name::new(&name))
                    .with(Transform::default())
                    .build();
                let id = entity.id().to_bits().get();
                log::info!("Created entity '{}' with id {}", name, id);
                RemoteResponse::with_data(ResponseData::Created { id })
            }

            RemoteCommand::DeleteEntity { id } => {
                use longhorn_core::EntityId;
                match EntityId::from_bits(id) {
                    Some(entity_id) => {
                        let handle = EntityHandle::new(entity_id);
                        if engine.world_mut().despawn(handle).is_ok() {
                            // Deselect if this was selected
                            if self.state.selected_entity.map(|e| e.id() as u64) == Some(id) {
                                self.state.select(None);
                            }
                            log::info!("Deleted entity {}", id);
                            RemoteResponse::ok()
                        } else {
                            RemoteResponse::error(format!("Entity not found: {}", id))
                        }
                    }
                    None => RemoteResponse::error(format!("Invalid entity id: {}", id)),
                }
            }

            RemoteCommand::SetProperty { entity, component, field, value } => {
                Self::set_entity_property(engine.world_mut(), entity, &component, &field, value)
            }

            RemoteCommand::LoadProject { path } => {
                match engine.load_game(&path) {
                    Ok(()) => {
                        log::info!("Loaded project: {}", path);
                        RemoteResponse::ok()
                    }
                    Err(e) => RemoteResponse::error(format!("Failed to load project: {}", e)),
                }
            }
        }
    }

    fn set_entity_property(
        world: &mut World,
        entity_id: u64,
        component: &str,
        field: &str,
        value: serde_json::Value,
    ) -> RemoteResponse {
        use longhorn_core::EntityId;

        let entity_id = match EntityId::from_bits(entity_id) {
            Some(id) => id,
            None => return RemoteResponse::error(format!("Invalid entity id: {}", entity_id)),
        };
        let handle = EntityHandle::new(entity_id);

        match component {
            "Transform" => {
                let mut transform = match world.get::<Transform>(handle) {
                    Ok(t) => (*t).clone(),
                    Err(_) => return RemoteResponse::error("Entity has no Transform"),
                };

                match field {
                    "position.x" => {
                        if let Some(v) = value.as_f64() {
                            transform.position.x = v as f32;
                        }
                    }
                    "position.y" => {
                        if let Some(v) = value.as_f64() {
                            transform.position.y = v as f32;
                        }
                    }
                    "rotation" => {
                        if let Some(v) = value.as_f64() {
                            transform.rotation = v as f32;
                        }
                    }
                    "scale.x" => {
                        if let Some(v) = value.as_f64() {
                            transform.scale.x = v as f32;
                        }
                    }
                    "scale.y" => {
                        if let Some(v) = value.as_f64() {
                            transform.scale.y = v as f32;
                        }
                    }
                    _ => return RemoteResponse::error(format!("Unknown field: {}", field)),
                }

                if world.set(handle, transform).is_err() {
                    return RemoteResponse::error("Failed to set Transform");
                }
                RemoteResponse::ok()
            }
            "Name" => {
                if field == "name" || field == "0" {
                    if let Some(s) = value.as_str() {
                        if world.set(handle, Name::new(s)).is_err() {
                            return RemoteResponse::error("Failed to set Name");
                        }
                        return RemoteResponse::ok();
                    }
                }
                RemoteResponse::error(format!("Invalid Name field: {}", field))
            }
            _ => RemoteResponse::error(format!("Unknown component: {}", component)),
        }
    }

    /// Handle toolbar action and update state
    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, engine: &mut Engine) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::ToggleConsole => {
                // Handled in show() method
            }
            ToolbarAction::Play => {
                // Capture scene state before playing
                log::debug!("Capturing scene snapshot ({} entities)", engine.world().len());
                self.scene_snapshot = Some(SceneSnapshot::capture(engine.world()));
                self.state.mode = EditorMode::Play;
                self.state.paused = false;
                log::debug!("Calling engine.start()");
                if let Err(e) = engine.start() {
                    log::error!("Failed to start engine: {}", e);
                }
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
                    log::debug!("Restoring scene snapshot ({} entities)", snapshot.entities.len());
                    snapshot.restore(engine.world_mut());
                    log::info!("Scene restored ({} entities)", engine.world().len());
                }
                // Reset script runtime so it re-initializes on next Play
                engine.reset_scripting();
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
                        // For now, load test_project from workspace root
                        let test_project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .join("test_project");

                        if let Err(e) = engine.load_game(&test_project) {
                            log::error!("Failed to load game: {}", e);
                        } else {
                            log::info!("Loaded game from: {:?}", test_project);
                        }
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
        match toolbar_action {
            ToolbarAction::ToggleConsole => {
                self.console_open = !self.console_open;
            }
            _ => self.handle_toolbar_action(toolbar_action, engine),
        }

        // Console panel (bottom, collapsible)
        if self.console_open {
            egui::TopBottomPanel::bottom("console")
                .resizable(true)
                .default_height(150.0)
                .min_height(100.0)
                .max_height(400.0)
                .show(ctx, |ui| {
                    self.console_panel.show(ui, &self.console);
                });
        }

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
