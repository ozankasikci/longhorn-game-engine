use egui::{Context, Ui};
use egui_dock::DockState;
use longhorn_engine::Engine;
use longhorn_scripting::set_console_callback;
use std::sync::Arc;
use crate::{EditorState, EditorMode, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, SceneSnapshot, ConsolePanel, ScriptConsole, EditorAction, ScriptEditorState, ScriptEditorPanel, ScriptError};
use crate::docking::{PanelType, PanelRenderer, create_default_dock_state, show_dock_area};
use crate::remote::{RemoteCommand, RemoteResponse, ResponseData, EntityInfo, EntityDetails, TransformData, UiStateData, PanelInfo, ClickableInfo};
use crate::ui_state::UiStateTracker;
use longhorn_core::{Name, Transform, World, EntityHandle};
use crate::{AssetBrowserState, AssetBrowserPanel, AssetBrowserAction, DirectoryNode, ContextAction};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    scene_snapshot: Option<SceneSnapshot>,
    console_panel: ConsolePanel,
    console: ScriptConsole,
    dock_state: DockState<PanelType>,
    pending_action: EditorAction,
    script_editor_state: ScriptEditorState,
    script_editor_panel: ScriptEditorPanel,
    ui_state: UiStateTracker,
    asset_browser_state: AssetBrowserState,
    asset_browser_panel: AssetBrowserPanel,
    asset_tree: Option<DirectoryNode>,
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
            dock_state: create_default_dock_state(),
            pending_action: EditorAction::None,
            script_editor_state: ScriptEditorState::new(),
            script_editor_panel: ScriptEditorPanel::new(),
            ui_state: UiStateTracker::new(),
            asset_browser_state: AssetBrowserState::new(),
            asset_browser_panel: AssetBrowserPanel::new(),
            asset_tree: None,
        }
    }

    /// Get a reference to the UI state tracker
    pub fn ui_state(&self) -> &UiStateTracker {
        &self.ui_state
    }

    /// Get a mutable reference to the UI state tracker
    pub fn ui_state_mut(&mut self) -> &mut UiStateTracker {
        &mut self.ui_state
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

    /// Get and clear any pending editor action
    pub fn take_pending_action(&mut self) -> EditorAction {
        let action = self.pending_action.clone();
        self.pending_action = EditorAction::None;
        match &action {
            EditorAction::None => {},
            a => log::info!("take_pending_action returning: {:?}", a),
        }
        action
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
                // Console is now always visible in dock, this is a no-op
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

            RemoteCommand::GetEntity { id } => {
                // Find entity by raw ID (matching get_entities format)
                let found = engine.world().inner().iter()
                    .find(|e| e.entity().id() as u64 == id);

                match found {
                    Some(entity_ref) => {
                        let entity = entity_ref.entity();
                        let handle = EntityHandle::new(entity);

                        // Get name
                        let name = engine.world().get::<Name>(handle)
                            .ok()
                            .map(|n| n.0.clone())
                            .unwrap_or_else(|| format!("Entity {}", id));

                        // Get transform
                        let transform = engine.world().get::<Transform>(handle)
                            .ok()
                            .map(|t| TransformData {
                                position_x: t.position.x,
                                position_y: t.position.y,
                                rotation: t.rotation,
                                scale_x: t.scale.x,
                                scale_y: t.scale.y,
                            });

                        RemoteResponse::with_data(ResponseData::Entity(EntityDetails {
                            id,
                            name,
                            transform,
                        }))
                    }
                    None => RemoteResponse::error(format!("Entity not found: {}", id)),
                }
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
                        self.refresh_asset_tree();
                        RemoteResponse::ok()
                    }
                    Err(e) => RemoteResponse::error(format!("Failed to load project: {}", e)),
                }
            }

            RemoteCommand::OpenScript { path } => {
                log::info!("Remote: Opening script '{}'", path);
                if let Some(project_path) = engine.game_path() {
                    let script_path = std::path::PathBuf::from(&path);
                    match self.script_editor_state.open(script_path, project_path) {
                        Ok(()) => {
                            log::info!("Script opened successfully: {}", path);
                            self.recheck_script_errors();
                            self.ensure_script_editor_visible();
                            RemoteResponse::ok()
                        }
                        Err(e) => {
                            log::error!("Failed to open script '{}': {}", path, e);
                            RemoteResponse::error(format!("Failed to open script: {}", e))
                        }
                    }
                } else {
                    log::error!("Cannot open script: No project loaded");
                    RemoteResponse::error("No project loaded")
                }
            }

            RemoteCommand::SaveScript => {
                log::info!("Remote: Saving script");
                if self.script_editor_state.is_open() {
                    match self.script_editor_state.save() {
                        Ok(()) => {
                            log::info!("Script saved successfully");
                            self.recheck_script_errors();
                            RemoteResponse::ok()
                        }
                        Err(e) => {
                            log::error!("Failed to save script: {}", e);
                            RemoteResponse::error(format!("Failed to save script: {}", e))
                        }
                    }
                } else {
                    RemoteResponse::error("No script is open")
                }
            }

            RemoteCommand::GetScriptEditorState => {
                use crate::remote::{ScriptEditorData, ScriptErrorData};
                let data = ScriptEditorData {
                    is_open: self.script_editor_state.is_open(),
                    file_path: self.script_editor_state.open_file.as_ref()
                        .map(|p| p.display().to_string()),
                    is_dirty: self.script_editor_state.is_dirty(),
                    error_count: self.script_editor_state.errors.len(),
                    errors: self.script_editor_state.errors.iter()
                        .map(|e| ScriptErrorData {
                            line: e.line,
                            message: e.message.clone(),
                        })
                        .collect(),
                };
                RemoteResponse::with_data(ResponseData::ScriptEditor(data))
            }

            // UI State Commands
            RemoteCommand::GetUiState => {
                let snapshot = self.ui_state.snapshot();
                let data = UiStateData {
                    focused_panel: snapshot.focused_panel,
                    panels: snapshot.panels.into_iter().map(|p| PanelInfo {
                        id: p.id,
                        title: p.title,
                        is_focused: p.is_focused,
                    }).collect(),
                    clickable_elements: snapshot.clickable_elements.into_iter().map(|c| ClickableInfo {
                        id: c.id,
                        label: c.label,
                        element_type: c.element_type,
                    }).collect(),
                };
                RemoteResponse::with_data(ResponseData::UiState(data))
            }

            RemoteCommand::ListPanels => {
                let panels: Vec<PanelInfo> = self.ui_state.panels().iter().map(|p| PanelInfo {
                    id: p.id.clone(),
                    title: p.title.clone(),
                    is_focused: p.is_focused,
                }).collect();
                RemoteResponse::with_data(ResponseData::Panels(panels))
            }

            RemoteCommand::GetClickableElements => {
                let elements: Vec<ClickableInfo> = self.ui_state.clickable_elements().iter().map(|c| ClickableInfo {
                    id: c.id.clone(),
                    label: c.label.clone(),
                    element_type: c.element_type.clone(),
                }).collect();
                RemoteResponse::with_data(ResponseData::Clickables(elements))
            }

            RemoteCommand::FocusPanel { panel } => {
                self.ui_state.request_focus(panel);
                RemoteResponse::ok()
            }

            RemoteCommand::TriggerElement { id } => {
                self.ui_state.request_trigger(id);
                RemoteResponse::ok()
            }

            // Scene Tree Commands
            RemoteCommand::ExpandTreeNode { path } => {
                self.ui_state.request_tree_expand(path);
                RemoteResponse::ok()
            }

            RemoteCommand::CollapseTreeNode { path } => {
                self.ui_state.request_tree_collapse(path);
                RemoteResponse::ok()
            }

            RemoteCommand::SelectByPath { path } => {
                // Find entity by name path (for now, simple name match)
                // Path format: "EntityName" or "Parent/Child/Entity"
                let entity_name = path.split('/').last().unwrap_or(&path);
                let found = engine.world().inner().iter()
                    .find(|entity_ref| {
                        let handle = EntityHandle::new(entity_ref.entity());
                        engine.world().get::<Name>(handle)
                            .ok()
                            .map(|n| n.0 == entity_name)
                            .unwrap_or(false)
                    })
                    .map(|e| e.entity());

                match found {
                    Some(entity) => {
                        self.state.select(Some(entity));
                        RemoteResponse::ok()
                    }
                    None => RemoteResponse::error(format!("Entity not found by path: {}", path)),
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

    /// Re-check for TypeScript errors in the current script
    fn recheck_script_errors(&mut self) {
        // Use TypeScriptCompiler to check for errors
        use longhorn_scripting::TypeScriptCompiler;
        let mut compiler = TypeScriptCompiler::new();
        let (_, diagnostics) = compiler.compile_with_diagnostics(&self.script_editor_state.content, "script.ts");
        let errors: Vec<ScriptError> = diagnostics.into_iter()
            .map(|d| ScriptError { line: d.line, message: d.message })
            .collect();
        self.script_editor_state.set_errors(errors);
    }

    /// Refresh the asset tree from disk
    pub fn refresh_asset_tree(&mut self) {
        if let Some(game_path) = &self.state.game_path {
            let assets_path = std::path::Path::new(game_path).join("assets");
            if assets_path.exists() {
                match DirectoryNode::scan(&assets_path) {
                    Ok(tree) => {
                        // Set selected folder to root if not set
                        if self.asset_browser_state.selected_folder.as_os_str().is_empty() {
                            self.asset_browser_state.selected_folder = tree.path.clone();
                        }
                        self.asset_tree = Some(tree);
                    }
                    Err(e) => {
                        log::error!("Failed to scan assets directory: {}", e);
                    }
                }
            }
        }
    }

    /// Ensure the ScriptEditor panel is visible in the dock
    fn ensure_script_editor_visible(&mut self) {
        // Check if ScriptEditor is already in dock, if not add it
        // For simplicity, we can add it as a new tab in the main area
        // Add to root node as a new tab
        self.dock_state.main_surface_mut().push_to_focused_leaf(PanelType::ScriptEditor);
    }

    /// Handle toolbar action and update state
    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, engine: &mut Engine) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::ToggleConsole => {
                // Console is always visible in dock now
            }
            ToolbarAction::Play => {
                // Capture scene state before playing
                log::debug!("Capturing scene snapshot ({} entities)", engine.world().len());
                self.scene_snapshot = Some(SceneSnapshot::capture(engine.world()));
                self.state.mode = EditorMode::Play;
                self.state.paused = false;
                // Reload scripts from disk before starting to pick up any edits
                engine.reset_scripting();
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
        // Reset UI state tracking for this frame
        self.ui_state.begin_frame();

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
                            self.refresh_asset_tree();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        should_exit = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.dock_state = create_default_dock_state();
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
        if toolbar_action != ToolbarAction::None && toolbar_action != ToolbarAction::ToggleConsole {
            self.handle_toolbar_action(toolbar_action, engine);
        }

        // Handle inspector actions
        let action = self.take_pending_action();
        match action {
            EditorAction::OpenScriptEditor { path } => {
                log::info!("EditorAction::OpenScriptEditor received for path: {}", path);
                // Get project path from engine
                if let Some(project_path) = engine.game_path() {
                    log::info!("Project path: {:?}", project_path);
                    // Scripts are stored in the scripts/ subdirectory
                    let script_path = std::path::PathBuf::from("scripts").join(&path);
                    log::info!("Full script path: {:?}", project_path.join(&script_path));
                    if let Err(e) = self.script_editor_state.open(script_path, project_path) {
                        log::error!("Failed to open script: {}", e);
                    } else {
                        log::info!("Opened script: {}", path);
                        self.recheck_script_errors();
                        // Add ScriptEditor panel to dock if not already there
                        self.ensure_script_editor_visible();
                    }
                } else {
                    log::error!("Cannot open script: No project loaded (game_path() returned None)");
                }
            }
            EditorAction::None => {}
        }

        // Main dock area
        egui::CentralPanel::default().show(ctx, |ui| {
            // We need to render panels which require access to engine/viewport_texture
            // Use a wrapper struct that implements PanelRenderer
            let mut wrapper = EditorPanelWrapper {
                editor: self,
                engine,
                viewport_texture,
            };

            // Take dock_state temporarily to avoid borrow issues
            let mut dock_state = std::mem::replace(&mut wrapper.editor.dock_state, create_default_dock_state());
            show_dock_area(ui, &mut dock_state, &mut wrapper);
            wrapper.editor.dock_state = dock_state;
        });

        should_exit
    }
}

/// Wrapper struct that provides PanelRenderer implementation with access to Engine
struct EditorPanelWrapper<'a> {
    editor: &'a mut Editor,
    engine: &'a mut Engine,
    viewport_texture: Option<egui::TextureId>,
}

impl<'a> PanelRenderer for EditorPanelWrapper<'a> {
    fn show_panel(&mut self, ui: &mut Ui, panel_type: PanelType) {
        // Get panel ID and title for UI state tracking
        let (panel_id, panel_title) = match panel_type {
            PanelType::Hierarchy => ("hierarchy", "Hierarchy"),
            PanelType::Inspector => ("inspector", "Inspector"),
            PanelType::SceneView => ("scene_view", "Scene"),
            PanelType::GameView => ("game_view", "Game"),
            PanelType::Console => ("console", "Console"),
            PanelType::Project => ("project", "Project"),
            PanelType::ScriptEditor => ("script_editor", "Script Editor"),
            PanelType::AssetBrowser => ("asset_browser", "Assets"),
        };

        // Register panel with UI state tracker
        // Note: We set is_focused to false since egui doesn't expose focus tracking publicly.
        // For more accurate focus tracking, we'd need to check if any child widget has focus.
        self.editor.ui_state.register_panel(panel_id, panel_title, false);

        match panel_type {
            PanelType::Hierarchy => {
                self.editor.scene_tree.show(
                    ui,
                    self.engine.world(),
                    &mut self.editor.state,
                    &mut self.editor.ui_state,
                );
            }
            PanelType::Inspector => {
                // In play mode, show read-only indicator
                if self.editor.state.is_playing() {
                    ui.label("(Read-only during play)");
                    ui.separator();
                }
                let action = self.editor.inspector.show(
                    ui,
                    self.engine.world_mut(),
                    &self.editor.state,
                    &mut self.editor.ui_state,
                );

                // Store the action for processing later
                match &action {
                    EditorAction::None => {},
                    a => log::info!("Storing action in editor: {:?}", a),
                }
                self.editor.pending_action = action;
            }
            PanelType::SceneView | PanelType::GameView => {
                // Both Scene and Game view show the viewport for now
                self.editor.viewport.show(ui, self.viewport_texture);
            }
            PanelType::Console => {
                self.editor.console_panel.show(ui, &self.editor.console);
            }
            PanelType::Project => {
                // Project browser - placeholder for now
                ui.label("Project browser coming soon...");
            }
            PanelType::ScriptEditor => {
                let save_triggered = self.editor.script_editor_panel.show(ui, &mut self.editor.script_editor_state);
                if save_triggered {
                    if let Err(e) = self.editor.script_editor_state.save() {
                        log::error!("Failed to save script: {}", e);
                    } else {
                        log::info!("Script saved");
                        // Re-check for errors after save
                        self.editor.recheck_script_errors();
                    }
                }
            }
            PanelType::AssetBrowser => {
                if let Some(action) = self.editor.asset_browser_panel.show(
                    ui,
                    &mut self.editor.asset_browser_state,
                    self.editor.asset_tree.as_ref(),
                ) {
                    match action {
                        AssetBrowserAction::OpenScript(path) => {
                            if let Some(game_path) = &self.editor.state.game_path {
                                let project_path = std::path::Path::new(game_path);
                                if let Ok(relative) = path.strip_prefix(project_path.join("assets")) {
                                    let script_path = std::path::PathBuf::from("assets").join(relative);
                                    if let Err(e) = self.editor.script_editor_state.open(script_path, project_path) {
                                        log::error!("Failed to open script: {}", e);
                                    } else {
                                        self.editor.recheck_script_errors();
                                        self.editor.ensure_script_editor_visible();
                                    }
                                }
                            }
                        }
                        AssetBrowserAction::OpenImage(path) => {
                            log::info!("TODO: Open image preview: {:?}", path);
                        }
                        AssetBrowserAction::OpenExternal(path) => {
                            if let Err(e) = open::that(&path) {
                                log::error!("Failed to open external: {}", e);
                            }
                        }
                        AssetBrowserAction::Context(context_action) => {
                            match context_action {
                                ContextAction::CreateFolder => {
                                    // TODO: Show dialog for folder name
                                    log::info!("TODO: Create folder dialog");
                                }
                                ContextAction::Rename(path) => {
                                    self.editor.asset_browser_state.renaming = Some(path);
                                }
                                ContextAction::Delete(path) => {
                                    if let Err(e) = crate::delete(&path) {
                                        log::error!("Failed to delete: {}", e);
                                    } else {
                                        log::info!("Deleted: {:?}", path);
                                    }
                                    self.editor.refresh_asset_tree();
                                }
                                ContextAction::Refresh => {
                                    self.editor.refresh_asset_tree();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
