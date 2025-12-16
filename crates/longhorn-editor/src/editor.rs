use egui::{Context, Ui};
use egui_dock::DockState;
use longhorn_engine::Engine;
use longhorn_scripting::set_console_callback;
use std::sync::Arc;
use crate::{EditorState, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, ConsolePanel, ScriptConsole, EditorAction, ScriptEditorState, ScriptEditorPanel, ScriptError};
use crate::docking::{PanelType, PanelRenderer, create_default_dock_state, show_dock_area};
use longhorn_remote::{RemoteCommand, RemoteResponse};
use crate::ui_state::UiStateTracker;
use crate::{ProjectPanelState, ProjectPanel, ProjectPanelAction, DirectoryNode, ContextAction};
use crate::texture_picker::{TexturePickerState, TexturePickerAction};
use crate::EditorCamera;
use crate::{GizmoState, GizmoConfig, GizmoMode};
use crate::{Project, DirtyState, StartupPanel, StartupAction, NewProjectDialog, NewProjectResult, UnsavedChangesDialog, UnsavedChangesResult};

pub struct Editor {
    editor_camera: EditorCamera,
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    console_panel: ConsolePanel,
    console: ScriptConsole,
    dock_state: DockState<PanelType>,
    pending_action: EditorAction,
    script_editor_state: ScriptEditorState,
    script_editor_panel: ScriptEditorPanel,
    ui_state: UiStateTracker,
    project_panel_state: ProjectPanelState,
    project_panel: ProjectPanel,
    project_tree: Option<DirectoryNode>,
    /// Flag to show script editor on next frame (deferred to avoid dock state borrow issues)
    pending_show_script_editor: bool,
    /// Texture picker state
    texture_picker_state: TexturePickerState,
    /// Pending screenshot request (path to save)
    pending_screenshot: Option<String>,
    /// Gizmo state for transform manipulation
    gizmo_state: GizmoState,
    /// Gizmo visual configuration
    gizmo_config: GizmoConfig,
    /// Currently loaded project (None = startup screen)
    project: Option<Project>,
    /// Dirty state tracking
    dirty_state: DirtyState,
    /// Startup screen panel
    startup_panel: StartupPanel,
    /// New project dialog
    new_project_dialog: NewProjectDialog,
    /// Unsaved changes dialog
    unsaved_changes_dialog: UnsavedChangesDialog,
    /// Pending action after unsaved changes dialog
    pending_close_action: Option<CloseAction>,
}

/// Actions that can be pending after unsaved changes dialog
#[derive(Debug, Clone)]
enum CloseAction {
    CloseProject,
    OpenProject(std::path::PathBuf),
    Quit,
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
            editor_camera: EditorCamera::new(),
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            console_panel: ConsolePanel::new(),
            console,
            dock_state: create_default_dock_state(),
            pending_action: EditorAction::None,
            script_editor_state: ScriptEditorState::new(),
            script_editor_panel: ScriptEditorPanel::new(),
            ui_state: UiStateTracker::new(),
            project_panel_state: ProjectPanelState::new(),
            project_panel: ProjectPanel::new(),
            project_tree: None,
            pending_show_script_editor: false,
            texture_picker_state: TexturePickerState::new(),
            pending_screenshot: None,
            gizmo_state: GizmoState::new(GizmoMode::Move),
            gizmo_config: GizmoConfig::default(),
            project: None,
            dirty_state: DirtyState::new(),
            startup_panel: StartupPanel::new(),
            new_project_dialog: NewProjectDialog::new(),
            unsaved_changes_dialog: UnsavedChangesDialog::new(),
            pending_close_action: None,
        }
    }

    /// Set up event subscriptions for debugging world events
    pub fn setup_event_subscriptions(&mut self, engine: &mut Engine) {
        use longhorn_events::EventType;

        engine.event_bus_mut().subscribe(
            EventType::EntitySpawned,
            |event| {
                log::debug!("Entity spawned: {:?}", event.data);
            },
        );

        engine.event_bus_mut().subscribe(
            EventType::EntityDespawned,
            |event| {
                log::debug!("Entity despawned: {:?}", event.data);
            },
        );
    }

    /// Get a reference to the UI state tracker
    pub fn ui_state(&self) -> &UiStateTracker {
        &self.ui_state
    }

    /// Get a mutable reference to the UI state tracker
    pub fn ui_state_mut(&mut self) -> &mut UiStateTracker {
        &mut self.ui_state
    }

    /// Request a screenshot to be taken on the next frame
    pub fn request_screenshot(&mut self, path: String) {
        log::info!("Screenshot requested: {}", path);
        self.pending_screenshot = Some(path);
    }

    /// Take and consume a pending screenshot request
    pub fn take_pending_screenshot(&mut self) -> Option<String> {
        self.pending_screenshot.take()
    }

    /// Request waiting for N frames (placeholder for future implementation)
    pub fn request_wait_frames(&self, _count: u32) {
        // TODO: Implement frame waiting
        // This would need to be handled in the main render loop
        log::info!("Wait frames requested (not yet implemented)");
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    pub fn editor_camera(&self) -> &EditorCamera {
        &self.editor_camera
    }

    pub fn viewport_mut(&mut self) -> &mut ViewportPanel {
        &mut self.viewport
    }

    pub fn console(&self) -> &ScriptConsole {
        &self.console
    }

    /// Get a reference to the script editor state
    pub fn script_editor_state(&self) -> &ScriptEditorState {
        &self.script_editor_state
    }

    /// Get a mutable reference to the script editor state
    pub fn script_editor_state_mut(&mut self) -> &mut ScriptEditorState {
        &mut self.script_editor_state
    }

    /// Get a reference to the project panel state
    pub fn project_panel_state(&self) -> &ProjectPanelState {
        &self.project_panel_state
    }

    /// Get a mutable reference to the project panel state
    pub fn project_panel_state_mut(&mut self) -> &mut ProjectPanelState {
        &mut self.project_panel_state
    }

    /// Get a reference to the project tree
    pub fn project_tree(&self) -> Option<&DirectoryNode> {
        self.project_tree.as_ref()
    }

    /// Get a mutable reference to the texture picker state
    pub fn texture_picker_state_mut(&mut self) -> &mut TexturePickerState {
        &mut self.texture_picker_state
    }

    /// Get a reference to the gizmo state
    pub fn gizmo_state(&self) -> &GizmoState {
        &self.gizmo_state
    }

    /// Check if a project is loaded
    pub fn has_project(&self) -> bool {
        self.project.is_some()
    }

    /// Get the current project
    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    /// Get the dirty state
    pub fn dirty_state(&self) -> &DirtyState {
        &self.dirty_state
    }

    /// Get mutable dirty state
    pub fn dirty_state_mut(&mut self) -> &mut DirtyState {
        &mut self.dirty_state
    }

    /// Get the window title based on project state
    pub fn window_title(&self) -> String {
        match &self.project {
            Some(project) => {
                let dirty = if self.dirty_state.any_dirty() { " *" } else { "" };
                format!("{}{} - Longhorn Editor", project.manifest.name, dirty)
            }
            None => "Longhorn Editor".to_string(),
        }
    }

    /// Save the currently focused file
    fn save_current(&mut self, engine: &Engine) {
        // Save scene if dirty
        if self.dirty_state.scene {
            // TODO: Implement scene save
            log::info!("TODO: Save scene");
            self.dirty_state.scene = false;
        }

        // Save current script if dirty
        if self.script_editor_state.is_dirty() {
            if let Err(e) = self.script_editor_state.save() {
                log::error!("Failed to save script: {}", e);
            } else {
                if let Some(path) = self.script_editor_state.open_file.clone() {
                    self.dirty_state.scripts.remove(&path);
                }
            }
        }

        // Save project settings if dirty
        if self.dirty_state.project_settings {
            if let Some(project) = &self.project {
                if let Err(e) = project.save_manifest() {
                    log::error!("Failed to save project settings: {}", e);
                } else {
                    self.dirty_state.project_settings = false;
                }
            }
        }
    }

    /// Save all dirty files
    fn save_all(&mut self, engine: &Engine) {
        self.save_current(engine);
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
        crate::commands::process_remote_command(self, command, engine)
    }

    /// Re-check for TypeScript errors in the current script
    pub fn recheck_script_errors(&mut self) {
        // Use TypeScriptCompiler to check for errors
        use longhorn_scripting::TypeScriptCompiler;
        let mut compiler = TypeScriptCompiler::new();
        let (_, diagnostics) = compiler.compile_with_diagnostics(&self.script_editor_state.content, "script.ts");
        let errors: Vec<ScriptError> = diagnostics.into_iter()
            .map(|d| ScriptError { line: d.line, message: d.message })
            .collect();
        self.script_editor_state.set_errors(errors);
    }

    /// Save the project panel state to disk
    pub fn save_panel_state(&self, engine: &Engine) {
        if let Some(game_path) = engine.game_path() {
            if let Err(e) = self.project_panel_state.save_to_file(game_path) {
                log::warn!("Failed to save project panel state: {}", e);
            }
        }
    }

    /// Refresh the project tree from disk
    pub fn refresh_project_tree(&mut self, engine: &Engine) {
        let game_path = engine.game_path();
        log::info!("refresh_project_tree called, game_path = {:?}", game_path);
        if let Some(game_path) = game_path {
            // Also sync to editor state for other uses
            self.state.game_path = Some(game_path.to_string_lossy().to_string());

            // Scan the project root directory (not just assets/)
            log::info!("Scanning project at: {:?}", game_path);
            match DirectoryNode::scan(game_path) {
                Ok(tree) => {
                    log::info!("Scanned project tree: {} files, {} folders", tree.files.len(), tree.children.len());

                    // Load saved panel state
                    if let Err(e) = self.project_panel_state.load_from_file(game_path) {
                        log::warn!("Failed to load project panel state: {}", e);
                    }

                    // Set selected folder to root if not set
                    if self.project_panel_state.selected_folder.as_os_str().is_empty() {
                        self.project_panel_state.selected_folder = tree.path.clone();
                    }
                    self.project_tree = Some(tree);
                }
                Err(e) => {
                    log::error!("Failed to scan project directory: {}", e);
                }
            }
        } else {
            log::warn!("refresh_project_tree: No game_path set");
        }
    }

    /// Request the ScriptEditor panel to be shown (deferred to avoid dock state borrow issues)
    pub fn ensure_script_editor_visible(&mut self) {
        log::info!("=== ensure_script_editor_visible called - setting pending flag ===");
        self.pending_show_script_editor = true;
    }

    /// Actually show the script editor (call after dock rendering)
    fn apply_pending_show_script_editor(&mut self) {
        if self.pending_show_script_editor {
            log::info!("=== apply_pending_show_script_editor: Adding ScriptEditor to dock ===");
            self.dock_state.main_surface_mut().push_to_focused_leaf(PanelType::ScriptEditor);
            self.pending_show_script_editor = false;
            log::info!("  ScriptEditor added to dock");
        }
    }

    /// Handle toolbar action and update state
    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, engine: &mut Engine) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::ToggleConsole => {
                // Console is always visible in dock now
            }
            ToolbarAction::Play => {
                // Clear selection because entity IDs will change during Play mode
                self.state.selected_entity = None;

                // Enter play mode using the new snapshot system
                if let Err(e) = self.state.enter_play_mode(engine.world(), engine.assets()) {
                    eprintln!("Failed to enter play mode: {}", e);
                    return;
                }

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
                // Exit play mode using the new snapshot system
                let (world, assets) = engine.world_and_assets_mut();
                if let Err(e) = self.state.exit_play_mode(world, assets) {
                    eprintln!("Failed to exit play mode: {}", e);
                    return;
                }

                // Clear selection because entity IDs have changed after restoring snapshot
                self.state.selected_entity = None;

                // Reset script runtime so it re-initializes on next Play
                engine.reset_scripting();
                self.state.paused = false;
                log::info!("Entering Scene mode");
            }
        }
    }

    pub fn show(&mut self, ctx: &Context, engine: &mut Engine, viewport_texture: Option<egui::TextureId>, viewport_texture_size: glam::Vec2, game_texture: Option<egui::TextureId>) -> bool {
        // Reset UI state tracking for this frame
        self.ui_state.begin_frame();

        // If no project is loaded, show startup screen
        if self.project.is_none() {
            // Show any open dialogs first
            match self.new_project_dialog.show(ctx) {
                NewProjectResult::Create { path, name } => {
                    match Project::create(&path, &name) {
                        Ok(project) => {
                            self.project = Some(project.clone());
                            if let Err(e) = engine.load_game(&path) {
                                log::error!("Failed to load created project: {}", e);
                            } else {
                                self.refresh_project_tree(engine);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create project: {}", e);
                        }
                    }
                }
                NewProjectResult::Cancel | NewProjectResult::None => {}
            }

            let action = self.startup_panel.show(ctx);
            match action {
                StartupAction::NewProject => {
                    self.new_project_dialog.open();
                }
                StartupAction::OpenProject => {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        match Project::load(&path) {
                            Ok(project) => {
                                self.project = Some(project);
                                if let Err(e) = engine.load_game(&path) {
                                    log::error!("Failed to load project: {}", e);
                                } else {
                                    self.refresh_project_tree(engine);
                                }
                            }
                            Err(e) => {
                                log::error!("Not a valid Longhorn project: {}", e);
                            }
                        }
                    }
                }
                StartupAction::None => {}
            }

            return false;
        }

        // Handle unsaved changes dialog
        match self.unsaved_changes_dialog.show(ctx) {
            UnsavedChangesResult::Save => {
                self.save_all(engine);
                if let Some(action) = self.pending_close_action.take() {
                    match action {
                        CloseAction::CloseProject => {
                            self.project = None;
                            self.dirty_state.clear();
                        }
                        CloseAction::OpenProject(path) => {
                            match Project::load(&path) {
                                Ok(project) => {
                                    self.project = Some(project);
                                    self.dirty_state.clear();
                                    if let Err(e) = engine.load_game(&path) {
                                        log::error!("Failed to load project: {}", e);
                                    } else {
                                        self.refresh_project_tree(engine);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Not a valid Longhorn project: {}", e);
                                }
                            }
                        }
                        CloseAction::Quit => {
                            return true; // Signal exit
                        }
                    }
                }
            }
            UnsavedChangesResult::DontSave => {
                if let Some(action) = self.pending_close_action.take() {
                    match action {
                        CloseAction::CloseProject => {
                            self.project = None;
                            self.dirty_state.clear();
                        }
                        CloseAction::OpenProject(path) => {
                            match Project::load(&path) {
                                Ok(project) => {
                                    self.project = Some(project);
                                    self.dirty_state.clear();
                                    if let Err(e) = engine.load_game(&path) {
                                        log::error!("Failed to load project: {}", e);
                                    } else {
                                        self.refresh_project_tree(engine);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Not a valid Longhorn project: {}", e);
                                }
                            }
                        }
                        CloseAction::Quit => {
                            return true; // Signal exit
                        }
                    }
                }
            }
            UnsavedChangesResult::Cancel => {
                self.pending_close_action = None;
            }
            UnsavedChangesResult::None => {}
        }

        // Check for pending import from file picker
        let pending_import: Option<(std::path::PathBuf, std::path::PathBuf, String)> = ctx.data_mut(|d| {
            d.remove_temp(egui::Id::new("pending_import"))
        });

        if let Some((source_path, target_folder, file_name)) = pending_import {
            log::info!("Processing pending import: {:?} -> {:?}/{}", source_path, target_folder, file_name);

            // Get project root to build relative path
            if let Some(project_root) = engine.game_path() {
                // Build destination path relative to project root
                let relative_target = if let Ok(rel) = target_folder.strip_prefix(project_root) {
                    rel.join(&file_name)
                } else {
                    // If target_folder is not under project_root, just use file_name
                    std::path::PathBuf::from(&file_name)
                };

                let dest_relative_str = relative_target.to_str().unwrap_or(&file_name);

                log::info!("Importing asset: {} -> {}", source_path.display(), dest_relative_str);

                // Import the asset
                match engine.assets_mut().import_asset(&source_path, dest_relative_str) {
                    Ok(asset_id) => {
                        log::info!("Successfully imported asset with ID: {}", asset_id.0);
                        // Refresh the project tree to show the new file
                        self.refresh_project_tree(engine);
                    }
                    Err(e) => {
                        log::error!("Failed to import asset: {}", e);
                    }
                }
            } else {
                log::error!("Cannot import asset: No project loaded");
            }
        }

        let mut should_exit = false;
        let mut toolbar_action = ToolbarAction::None;

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project...").clicked() {
                        if self.dirty_state.any_dirty() {
                            self.unsaved_changes_dialog.open(self.dirty_state.dirty_files());
                            self.pending_close_action = Some(CloseAction::CloseProject);
                        } else {
                            self.project = None;
                            self.dirty_state.clear();
                        }
                        self.new_project_dialog.open();
                        ui.close_menu();
                    }

                    if ui.button("Open Project...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            if self.dirty_state.any_dirty() {
                                self.unsaved_changes_dialog.open(self.dirty_state.dirty_files());
                                self.pending_close_action = Some(CloseAction::OpenProject(path));
                            } else {
                                match Project::load(&path) {
                                    Ok(project) => {
                                        self.project = Some(project);
                                        self.dirty_state.clear();
                                        if let Err(e) = engine.load_game(&path) {
                                            log::error!("Failed to load project: {}", e);
                                        } else {
                                            self.refresh_project_tree(engine);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Not a valid Longhorn project: {}", e);
                                    }
                                }
                            }
                        }
                        ui.close_menu();
                    }

                    if ui.add_enabled(self.project.is_some(), egui::Button::new("Close Project")).clicked() {
                        if self.dirty_state.any_dirty() {
                            self.unsaved_changes_dialog.open(self.dirty_state.dirty_files());
                            self.pending_close_action = Some(CloseAction::CloseProject);
                        } else {
                            self.project = None;
                            self.dirty_state.clear();
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.add_enabled(self.dirty_state.any_dirty(), egui::Button::new("Save")).clicked() {
                        self.save_current(engine);
                        ui.close_menu();
                    }

                    if ui.add_enabled(self.dirty_state.any_dirty(), egui::Button::new("Save All")).clicked() {
                        self.save_all(engine);
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        if self.dirty_state.any_dirty() {
                            self.unsaved_changes_dialog.open(self.dirty_state.dirty_files());
                            self.pending_close_action = Some(CloseAction::Quit);
                        } else {
                            should_exit = true;
                        }
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
            EditorAction::OpenTexturePicker { entity } => {
                log::info!("EditorAction::OpenTexturePicker received for entity ID: {} (raw: {:?})", entity.id(), entity);
                self.texture_picker_state.open_for_entity(entity);
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
                viewport_texture_size,
                game_texture,
            };

            // Take dock_state temporarily to avoid borrow issues
            let mut dock_state = std::mem::replace(&mut wrapper.editor.dock_state, create_default_dock_state());
            show_dock_area(ui, &mut dock_state, &mut wrapper);
            wrapper.editor.dock_state = dock_state;

            // Apply any pending dock state changes AFTER rendering completes
            wrapper.editor.apply_pending_show_script_editor();
        });

        // Show texture picker popup (if open)
        if self.texture_picker_state.is_open {
            // Collect all image files from the project tree
            let image_files = if let Some(ref tree) = self.project_tree {
                crate::texture_picker::collect_image_files(tree)
            } else {
                Vec::new()
            };

            // Get project root before mutable borrow (clone to avoid lifetime issues)
            let project_root = engine.game_path().map(|p| p.to_path_buf());

            // Show the texture picker
            let picker_action = crate::texture_picker::show_texture_picker(
                ctx,
                &mut self.texture_picker_state,
                &image_files,
                engine.assets_mut(),
                project_root.as_deref(),
            );

            // Handle the action
            match picker_action {
                TexturePickerAction::SelectTexture { entity, asset_id, path } => {
                    log::info!("Texture selected: {} (ID: {}) for entity ID: {} (raw: {:?})", path, asset_id.0, entity.id(), entity);

                    // Load the texture into the AssetManager cache so it's available for rendering
                    match engine.assets_mut().load_texture_by_id(asset_id) {
                        Ok(_) => {
                            log::info!("Loaded texture {} into cache", asset_id.0);
                        }
                        Err(e) => {
                            log::error!("Failed to load texture {}: {}", asset_id.0, e);
                        }
                    }

                    // Update the Sprite component with the new texture
                    let handle = longhorn_core::EntityHandle::new(entity);
                    match engine.world_mut().get_mut::<longhorn_core::Sprite>(handle) {
                        Ok(mut sprite) => {
                            let old_texture = sprite.texture;
                            sprite.texture = asset_id;
                            log::info!("[TEXTURE_CHANGE] Entity {} texture: {} -> {}", entity.id(), old_texture.0, asset_id.0);
                        }
                        Err(e) => {
                            log::error!("Failed to get Sprite component for entity {:?}: {}", entity, e);
                        }
                    }

                    // DIAGNOSTIC: Dump all entity textures after the change to verify isolation
                    log::info!("[TEXTURE_VERIFY] All entity textures after change:");
                    for (eid, sprite) in engine.world().query::<&longhorn_core::Sprite>().iter() {
                        log::info!("[TEXTURE_VERIFY]   Entity {}: texture ID = {}", eid.id(), sprite.texture.0);
                    }
                }
                TexturePickerAction::None => {}
            }
        }

        should_exit
    }
}

/// Wrapper struct that provides PanelRenderer implementation with access to Engine
struct EditorPanelWrapper<'a> {
    editor: &'a mut Editor,
    engine: &'a mut Engine,
    viewport_texture: Option<egui::TextureId>,
    viewport_texture_size: glam::Vec2,
    game_texture: Option<egui::TextureId>,
}

impl<'a> EditorPanelWrapper<'a> {
    fn show_game_placeholder(&self, ui: &mut Ui, message: &str) {
        let available = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());

        // Draw dark background
        ui.painter().rect_filled(
            rect,
            0.0,
            crate::styling::Colors::BG_VIEWPORT,
        );

        // Draw centered message
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            message,
            egui::FontId::proportional(20.0),
            crate::styling::Colors::TEXT_SECONDARY,
        );
    }
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
        };

        // Register panel with UI state tracker
        // Note: We set is_focused to false since egui doesn't expose focus tracking publicly.
        // For more accurate focus tracking, we'd need to check if any child widget has focus.
        self.editor.ui_state.register_panel(panel_id, panel_title, false);

        match panel_type {
            PanelType::Hierarchy => {
                // Get game_path before the mutable borrow (convert to owned PathBuf)
                let game_path = self.engine.game_path().map(|p| p.to_path_buf());
                // Split borrows: we need both world and assets mutably
                let (world, assets) = self.engine.world_and_assets_mut();
                self.editor.scene_tree.show(
                    ui,
                    world,
                    &mut self.editor.state,
                    &mut self.editor.ui_state,
                    game_path.as_deref(),
                    assets,
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
                );

                // Store the action for processing later
                match &action {
                    EditorAction::None => {},
                    a => log::info!("Storing action in editor: {:?}", a),
                }
                self.editor.pending_action = action;
            }
            PanelType::SceneView => {
                // Get selected entity transforms (both local and global)
                let (selected_transform, selected_global_transform) = self.editor.state.selected_entity
                    .map(|entity| {
                        let handle = longhorn_core::EntityHandle::new(entity);
                        let transform = self.engine.world().get::<longhorn_core::Transform>(handle).ok().map(|t| *t);
                        let global_transform = self.engine.world().get::<longhorn_core::GlobalTransform>(handle).ok().map(|t| *t);
                        (transform, global_transform)
                    })
                    .unwrap_or((None, None));

                // Scene view - capture camera input and apply to editor camera
                let (camera_input, action) = self.editor.viewport.show(
                    ui,
                    self.viewport_texture,
                    self.viewport_texture_size,
                    &mut self.editor.gizmo_state,
                    &self.editor.gizmo_config,
                    selected_transform,
                    selected_global_transform,
                    self.editor.editor_camera.transform.position,
                    self.editor.editor_camera.zoom,
                    self.engine.world(),
                );
                self.editor.editor_camera.handle_input(&camera_input);

                // Handle transform updates from gizmo
                if let Some(new_transform) = action.transform_update {
                    if let Some(selected) = self.editor.state.selected_entity {
                        let handle = longhorn_core::EntityHandle::new(selected);
                        if let Ok(mut transform) = self.engine.world_mut().get_mut::<longhorn_core::Transform>(handle) {
                            *transform = new_transform;
                        }
                    }
                }

                // Handle entity selection by clicking
                if let Some(clicked_entity) = action.entity_clicked {
                    self.editor.state.selected_entity = Some(clicked_entity);
                    log::info!("Entity selected by clicking: {}", clicked_entity.id());
                }

                // Handle F key to frame selected entity
                if action.frame_selected {
                    if let Some(selected) = self.editor.state.selected_entity {
                        let handle = longhorn_core::EntityHandle::new(selected);
                        if let Ok(transform) = self.engine.world().get::<longhorn_core::Transform>(handle) {
                            // Get sprite size if available, otherwise use default
                            let entity_size = if let Ok(sprite) = self.engine.world().get::<longhorn_core::Sprite>(handle) {
                                sprite.size
                            } else {
                                glam::Vec2::new(64.0, 64.0) // Default size
                            };
                            self.editor.editor_camera.frame_entity(transform.position, entity_size);
                        }
                    }
                }
            }
            PanelType::GameView => {
                // Game view - shows game camera perspective when playing
                ui.heading("Game View (Main Camera - Play Mode Only)");
                ui.separator();

                if self.editor.state.is_playing() {
                    if let Some(game_tex) = self.game_texture {
                        // Show game texture when in Play mode - simple display without gizmos
                        let available = ui.available_size();
                        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());

                        ui.painter().image(
                            game_tex,
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            crate::styling::Colors::TEXT_ON_ACCENT,
                        );
                    } else {
                        // In Play mode but no MainCamera found
                        self.show_game_placeholder(ui, "⚠ No MainCamera in scene");
                    }
                } else {
                    // Not in Play mode
                    self.show_game_placeholder(ui, "Press ▶ Play to start");
                }
            }
            PanelType::Console => {
                self.editor.console_panel.show(ui, &self.editor.console);
            }
            PanelType::Project => {
                if let Some(action) = self.editor.project_panel.show(
                    ui,
                    &mut self.editor.project_panel_state,
                    self.editor.project_tree.as_ref(),
                    &mut self.editor.ui_state,
                ) {
                    log::info!("=== EDITOR received ProjectPanelAction: {:?} ===", action);
                    match action {
                        ProjectPanelAction::OpenScript(path) => {
                            log::info!("  Processing OpenScript for path: {:?}", path);
                            log::info!("  game_path: {:?}", self.editor.state.game_path);
                            if let Some(game_path) = &self.editor.state.game_path {
                                let project_path = std::path::Path::new(game_path);
                                log::info!("  project_path: {:?}", project_path);
                                // Get relative path from project root
                                if let Ok(relative) = path.strip_prefix(project_path) {
                                    let script_path = relative.to_path_buf();
                                    log::info!("  relative script_path: {:?}", script_path);
                                    log::info!("  Calling script_editor_state.open()...");
                                    if let Err(e) = self.editor.script_editor_state.open(script_path, project_path) {
                                        log::error!("  FAILED to open script: {}", e);
                                    } else {
                                        log::info!("  Script opened successfully!");
                                        self.editor.recheck_script_errors();
                                        log::info!("  Calling ensure_script_editor_visible()...");
                                        self.editor.ensure_script_editor_visible();
                                        log::info!("  Script editor should now be visible");
                                    }
                                } else {
                                    log::error!("  Script path {:?} is not under project {:?}", path, project_path);
                                }
                            } else {
                                log::error!("  No game_path set - cannot open script!");
                            }
                        }
                        ProjectPanelAction::OpenImage(path) => {
                            log::info!("TODO: Open image preview: {:?}", path);
                        }
                        ProjectPanelAction::OpenExternal(path) => {
                            if let Err(e) = open::that(&path) {
                                log::error!("Failed to open external: {}", e);
                            }
                        }
                        ProjectPanelAction::Context(context_action) => {
                            match context_action {
                                ContextAction::CreateFolder => {
                                    // TODO: Show dialog for folder name
                                    log::info!("TODO: Create folder dialog");
                                }
                                ContextAction::Rename(path) => {
                                    self.editor.project_panel_state.renaming = Some(path);
                                }
                                ContextAction::Delete(path) => {
                                    if let Err(e) = crate::delete(&path) {
                                        log::error!("Failed to delete: {}", e);
                                    } else {
                                        log::info!("Deleted: {:?}", path);
                                    }
                                    self.editor.refresh_project_tree(self.engine);
                                }
                                ContextAction::Refresh => {
                                    self.editor.refresh_project_tree(self.engine);
                                }
                                ContextAction::ImportAsset(target_folder) => {
                                    log::info!("ImportAsset context action triggered for folder: {:?}", target_folder);
                                    // Open file picker for image files
                                    let task = rfd::AsyncFileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                                        .set_title("Import Asset")
                                        .pick_file();

                                    // Spawn async task to handle file selection
                                    let target_folder = target_folder.clone();
                                    let ctx = ui.ctx().clone();
                                    std::thread::spawn(move || {
                                        if let Some(file_handle) = pollster::block_on(task) {
                                            let source_path = file_handle.path().to_path_buf();
                                            let file_name = source_path
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("unknown")
                                                .to_string();

                                            log::info!("Import asset: Selected file {:?}, target folder {:?}", source_path, target_folder);
                                            // Store the import request for processing on next frame
                                            ctx.data_mut(|d| {
                                                d.insert_temp(egui::Id::new("pending_import"), (source_path, target_folder, file_name));
                                            });
                                            ctx.request_repaint();
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
            PanelType::ScriptEditor => {
                let save_triggered = self.editor.script_editor_panel.show(ui, &mut self.editor.script_editor_state);
                if save_triggered {
                    if let Err(e) = self.editor.script_editor_state.save() {
                        log::error!("Failed to save script: {}", e);
                    } else {
                        log::info!("Script saved");
                        // Clear dirty flag for this script
                        if let Some(path) = self.editor.script_editor_state.open_file.clone() {
                            self.editor.dirty_state.scripts.remove(&path);
                        }
                        self.editor.recheck_script_errors();
                    }
                }

                // Sync dirty state from script editor
                if let Some(path) = self.editor.script_editor_state.open_file.clone() {
                    if self.editor.script_editor_state.is_dirty() {
                        self.editor.dirty_state.scripts.insert(path, true);
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
