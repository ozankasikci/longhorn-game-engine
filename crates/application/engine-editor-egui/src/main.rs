// Longhorn Game Engine editor built with EGUI and dockable panels
// Provides a modern, responsive interface with drag-and-drop docking

mod assets;
mod bridge;
// mod editor_coordinator;  // Replaced by UnifiedEditorCoordinator
mod editor_state;
mod import;
mod panels;
mod play_state;
mod settings;
mod types;
mod utils;
mod world_setup;

#[cfg(test)]
mod compilation_toast_tests;

#[cfg(test)]
mod compilation_events_integration_tests;

#[cfg(test)]
mod typescript_cache_invalidation_tests;

use engine_editor_framework::UnifiedEditorCoordinator;
use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex};
use engine_components_3d::Transform;
use engine_ecs_core::{Entity, World};
use engine_editor_panels::{
    ConsolePanel, GameViewPanel, HierarchyPanel, InspectorPanel, ProjectPanel,
};
use engine_editor_scene_view::{
    scene_view::rendering::SceneRenderer,
    scene_view::scene_view_impl::SceneViewRenderer,
    types::{PlayState, SceneNavigation},
    SceneView as SceneViewPanel,
};
use engine_editor_ui::PanelType;
use engine_editor_ui::{
    setup_custom_fonts, setup_custom_style, EditorTabViewer, MenuBar, SettingsDialog, Toolbar,
};
use import::dialog::{ImportDialog, ImportResult};
use types::{GizmoSystem, HierarchyObject, TextureAsset};
use clap::Parser;
use engine_runtime::{StandaloneRuntime, StandaloneConfig};
use engine_editor_control::{EditorControlServer, EditorCommandHandler, types::*};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Longhorn Game Engine - Unified Editor and Runtime
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in standalone mode without editor UI
    #[arg(short, long)]
    standalone: bool,
    
    /// Path to game project to load
    #[arg(short, long)]
    project: Option<std::path::PathBuf>,
    
    /// Start in play mode immediately (editor only)
    #[arg(long)]
    play: bool,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    let args = Args::parse();
    
    if args.standalone {
        // Run in standalone mode without editor UI
        log::info!("Running in standalone mode");
        return run_standalone(args.project);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1000.0])
            .with_title("Longhorn Game Engine Editor (EGUI + Docking)"),
        ..Default::default()
    };

    eframe::run_native(
        "Longhorn Game Engine Editor",
        options,
        Box::new(move |cc| {
            // Load custom design constraints if available
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);

            let mut editor = LonghornEditor::new(cc);
            
            // Start the control server for remote commands
            editor.start_control_server();
            
            // Start in play mode if requested
            if args.play {
                // Sync the editor world to coordinator before starting play mode
                editor.sync_editor_world_to_coordinator();
                editor.coordinator.play_state_manager_mut().start();
            }
            
            Ok(Box::new(editor))
        }),
    )
}

/// Run the engine in standalone mode without editor UI
fn run_standalone(project_path: Option<std::path::PathBuf>) -> Result<(), eframe::Error> {
    use engine_runtime_core::Application;
    
    // Create standalone runtime configuration
    let config = StandaloneConfig::builder()
        .title("Longhorn Game Engine")
        .resolution(1280, 720)
        .target_fps(60.0)
        .build();
    
    // Create runtime
    let mut runtime = match project_path {
        Some(path) => {
            log::info!("Loading project from: {}", path.display());
            match StandaloneRuntime::from_project(path) {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    return Ok(());
                }
            }
        }
        None => {
            log::info!("Starting standalone runtime with default configuration");
            match StandaloneRuntime::new(config) {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to create standalone runtime: {}", e);
                    return Ok(());
                }
            }
        }
    };
    
    // Create a demo application
    struct DemoApp {
        frame_count: u32,
    }
    
    impl Application for DemoApp {
        fn update(&mut self, delta_time: std::time::Duration, _input: &engine_input::InputManager) -> engine_runtime_core::Result<()> {
            self.frame_count += 1;
            if self.frame_count % 60 == 0 {
                log::info!("Frame {}: delta = {:?}", self.frame_count, delta_time);
            }
            Ok(())
        }
        
        fn render(&mut self, interpolation: f32) -> engine_runtime_core::Result<()> {
            // In a real implementation, this would render the game
            let _ = interpolation;
            Ok(())
        }
        
        fn should_exit(&self) -> bool {
            // Exit after 300 frames (5 seconds at 60 FPS) for demo
            self.frame_count >= 300
        }
    }
    
    // Set the application
    runtime.set_application(Box::new(DemoApp { frame_count: 0 }));
    
    // Add example systems
    // TODO: Add real game systems here
    
    // Run the game
    match runtime.run() {
        Ok(_) => {
            log::info!("Standalone runtime exited successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Standalone runtime error: {}", e);
            Ok(())
        }
    }
}

/// Toast notification for TypeScript compilation status
#[derive(Debug)]
pub struct CompilationToast {
    /// Whether the toast is currently visible
    visible: bool,
    /// Set of currently compiling scripts
    active_compilations: std::collections::HashSet<String>,
    /// Time when toast was first shown (for minimum display duration)
    show_time: Option<std::time::Instant>,
    /// Minimum duration to show toast (in seconds)
    min_display_duration: std::time::Duration,
}

impl Default for CompilationToast {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationToast {
    /// Create a new compilation toast
    pub fn new() -> Self {
        Self {
            visible: false,
            active_compilations: std::collections::HashSet::new(),
            show_time: None,
            min_display_duration: std::time::Duration::from_millis(1500), // Show for at least 1.5 seconds
        }
    }
    
    /// Check if the toast is currently visible
    pub fn is_visible(&self) -> bool {
        if !self.visible {
            return false;
        }
        
        // If no active compilations, check if minimum duration has passed
        if self.active_compilations.is_empty() {
            if let Some(show_time) = self.show_time {
                let elapsed = show_time.elapsed();
                return elapsed < self.min_display_duration;
            }
            // If no show_time set but no active compilations, hide
            return false;
        }
        
        // Has active compilations - show the toast
        true
    }
    
    /// Show compilation started (simple version)
    pub fn show_compilation_started(&mut self) {
        self.visible = true;
    }
    
    /// Hide compilation completed (simple version)
    pub fn hide_compilation_completed(&mut self) {
        self.visible = false;
    }
    
    /// Get the toast message content
    pub fn get_message(&self) -> &'static str {
        "🔄 Compiling TypeScript..."
    }
    
    /// Start compilation for a specific script
    pub fn start_compilation(&mut self, script_path: &str) {
        self.active_compilations.insert(script_path.to_string());
        if !self.visible {
            self.visible = true;
            self.show_time = Some(std::time::Instant::now());
        }
    }
    
    /// Complete compilation for a specific script
    pub fn complete_compilation(&mut self, script_path: &str) {
        self.active_compilations.remove(script_path);
        
        // If no more active compilations, start counting down minimum display time
        if self.active_compilations.is_empty() {
            // The is_visible() method will handle the minimum duration logic
            // We don't immediately set visible = false here
        }
    }
    
    /// Update method to handle minimum display duration
    pub fn update(&mut self) {
        if self.visible && self.active_compilations.is_empty() {
            if let Some(show_time) = self.show_time {
                let elapsed = show_time.elapsed();
                if elapsed >= self.min_display_duration {
                    self.visible = false;
                    self.show_time = None;
                }
            }
        }
    }
    
    /// Get the number of active compilations
    pub fn get_active_compilation_count(&self) -> usize {
        self.active_compilations.len()
    }
}

/// Longhorn Game Engine editor application with dockable panels
pub struct LonghornEditor {
    // Docking system
    dock_state: DockState<engine_editor_ui::PanelType>,

    // ECS v2 Integration
    world: World,
    selected_entity: Option<Entity>,

    // Editor state
    selected_object: Option<String>,

    // Panel data
    #[allow(dead_code)]
    hierarchy_objects: Vec<HierarchyObject>,
    project_assets: Vec<types::ProjectAsset>,

    // Texture asset system
    #[allow(dead_code)]
    texture_assets: std::collections::HashMap<u64, TextureAsset>,
    #[allow(dead_code)]
    next_texture_handle: u64,

    // Editor coordination
    coordinator: UnifiedEditorCoordinator,

    // UI state
    #[allow(dead_code)]
    scene_view_active: bool,
    #[allow(dead_code)]
    show_add_component_dialog: bool,
    inspector_panel: InspectorPanel,
    hierarchy_panel: HierarchyPanel,
    console_panel: ConsolePanel,
    project_panel: ProjectPanel,
    toolbar: Toolbar,
    menu_bar: MenuBar,
    game_view_panel: GameViewPanel,
    scene_view_panel: SceneViewPanel,

    // Gizmo system
    gizmo_system: GizmoSystem,

    // Scene navigation system
    scene_navigation: SceneNavigation,

    // 3D scene renderer (using engine-renderer-3d)
    scene_view_renderer: SceneViewRenderer,

    // Phase 10.2: Track entity counts for change detection
    #[allow(dead_code)]
    last_rendered_entity_count: usize,

    // Editor settings
    settings: engine_editor_ui::EditorSettings,
    settings_dialog: SettingsDialog,

    // Import dialog
    import_dialog: ImportDialog,
    show_import_dialog: bool,

    // Import service
    import_service: import::ImportService,
    asset_database: assets::AssetDatabase,
    
    // Timing
    last_update: std::time::Instant,
    
    // Compilation toast notification
    compilation_toast: CompilationToast,
    
    // Hot reload system for TypeScript file watching
    hot_reload_manager: engine_runtime_core::HotReloadManager,
    
    // Editor control system
    control_server_handle: Option<std::thread::JoinHandle<()>>,
    control_logs: Arc<Mutex<Vec<String>>>,
    control_script_errors: Arc<Mutex<Vec<ScriptError>>>,
    control_compilation_events: Arc<Mutex<Vec<CompilationEvent>>>,
    action_receiver: mpsc::Receiver<EditorAction>,
    action_sender: mpsc::Sender<EditorAction>,
}

impl LonghornEditor {
    /// Sync entities from the editor world to the coordinator's world for play mode
    fn sync_editor_world_to_coordinator(&mut self) {
        // Get the coordinator's world with safe locking
        let coordinator_world_arc = self.coordinator.world();
        let mut coordinator_world = match coordinator_world_arc.try_lock() {
            Ok(world) => world,
            Err(_) => {
                println!("[LonghornEditor] Could not acquire world lock for sync, skipping");
                return;
            }
        };
        
        // Copy all entities with LuaScript components from editor world to coordinator world
        let lua_script_entities: Vec<_> = self.world.query_legacy::<engine_scripting::components::LuaScript>()
            .map(|(entity, script)| (entity, script.clone()))
            .collect();
            
        // Copy all entities with TypeScriptScript components from editor world to coordinator world
        let typescript_script_entities: Vec<_> = self.world.query_legacy::<engine_scripting::components::TypeScriptScript>()
            .map(|(entity, script)| (entity, script.clone()))
            .collect();
            
        println!("[LonghornEditor] Syncing {} entities with LuaScript components to coordinator", lua_script_entities.len());
        println!("[LonghornEditor] Syncing {} entities with TypeScriptScript components to coordinator", typescript_script_entities.len());
        
        // Process Lua script entities
        for (entity, script) in lua_script_entities {
            // Spawn entity in coordinator world
            let new_entity = coordinator_world.spawn();
            
            // Copy LuaScript component
            if let Err(e) = coordinator_world.add_component(new_entity, script) {
                println!("Failed to add LuaScript component: {:?}", e);
                continue;
            }
            
            // Copy Transform component if it exists
            if let Some(transform) = self.world.get_component::<Transform>(entity) {
                if let Err(e) = coordinator_world.add_component(new_entity, transform.clone()) {
                    println!("Failed to add Transform component: {:?}", e);
                }
            }
            
            println!("  Copied Lua entity {:?} -> {:?} with script: {}", entity, new_entity, 
                coordinator_world.get_component::<engine_scripting::components::LuaScript>(new_entity)
                    .map(|s| s.script_path.as_str()).unwrap_or("Unknown"));
        }
        
        // Process TypeScript script entities
        for (entity, script) in typescript_script_entities {
            // Spawn entity in coordinator world
            let new_entity = coordinator_world.spawn();
            
            // Copy TypeScriptScript component
            if let Err(e) = coordinator_world.add_component(new_entity, script) {
                println!("Failed to add TypeScriptScript component: {:?}", e);
                continue;
            }
            
            // Copy Transform component if it exists
            if let Some(transform) = self.world.get_component::<Transform>(entity) {
                if let Err(e) = coordinator_world.add_component(new_entity, transform.clone()) {
                    println!("Failed to add Transform component: {:?}", e);
                }
            }
            
            println!("  Copied TypeScript entity {:?} -> {:?} with script: {}", entity, new_entity, 
                coordinator_world.get_component::<engine_scripting::components::TypeScriptScript>(new_entity)
                    .map(|s| s.script_path.as_str()).unwrap_or("Unknown"));
        }
        
        drop(coordinator_world);
        println!("[LonghornEditor] World sync complete!");
    }
    
    /// Sync script removal from editor world to coordinator world during play mode
    fn sync_script_removal_to_coordinator(&mut self, entity_id: u32, script_path: String) {
        // Get the coordinator's world with safe locking
        let coordinator_world_arc = self.coordinator.world();
        let mut coordinator_world = match coordinator_world_arc.try_lock() {
            Ok(world) => world,
            Err(_) => {
                log::warn!("🗑️ SYNC SCRIPT REMOVAL: Could not acquire coordinator world lock, skipping");
                return;
            }
        };
        
        log::info!("🗑️ SYNC SCRIPT REMOVAL: Searching for entities in coordinator world with matching script");
        
        // Find all entities in coordinator world with TypeScript components that match the script
        let entities_to_update: Vec<_> = coordinator_world
            .query_legacy::<engine_scripting::components::TypeScriptScript>()
            .filter_map(|(entity, script)| {
                let has_script = script.get_all_scripts().iter().any(|s| s.as_str() == script_path.as_str());
                if has_script {
                    log::info!("🗑️ SYNC SCRIPT REMOVAL: Found entity {:?} with matching script '{}'", entity, script_path);
                    Some((entity, script.clone()))
                } else {
                    None
                }
            })
            .collect();
        
        log::info!("🗑️ SYNC SCRIPT REMOVAL: Found {} entities to update in coordinator world", entities_to_update.len());
        
        for (entity, mut script_component) in entities_to_update {
            log::info!("🗑️ SYNC SCRIPT REMOVAL: Processing entity {:?}", entity);
            
            // Remove the script from the component
            let remove_result = script_component.remove_script(&script_path);
            log::info!("🗑️ SYNC SCRIPT REMOVAL: remove_script result: {}", remove_result);
            
            if script_component.get_all_scripts().is_empty() {
                // Remove the entire component if no scripts left
                log::info!("🗑️ SYNC SCRIPT REMOVAL: No scripts left, removing entire component from entity {:?}", entity);
                let remove_component_result = coordinator_world.remove_component::<engine_scripting::components::TypeScriptScript>(entity);
                log::info!("🗑️ SYNC SCRIPT REMOVAL: remove_component result: {:?}", remove_component_result);
            } else {
                // Update the component with the remaining scripts
                log::info!("🗑️ SYNC SCRIPT REMOVAL: Updating component with remaining scripts: {:?}", script_component.get_all_scripts());
                if let Err(e) = coordinator_world.add_component(entity, script_component) {
                    log::error!("🗑️ SYNC SCRIPT REMOVAL: Failed to update component: {:?}", e);
                }
            }
        }
        
        drop(coordinator_world);
        log::info!("🗑️ SYNC SCRIPT REMOVAL: Script removal sync complete!");
    }
    
    /// Poll for Lua console messages and add them to the console panel
    fn poll_script_console_messages(&mut self) {
        // This function collects console messages from both Lua and TypeScript scripts
        // as they both use the same unified console message system
        let script_messages = engine_scripting::get_and_clear_console_messages();
        
        if !script_messages.is_empty() {
            let panel_messages: Vec<engine_editor_panels::types::ConsoleMessage> = script_messages
                .into_iter()
                .map(|msg| engine_editor_panels::types::ConsoleMessage::info(&msg.message))
                .collect();
            self.console_panel.add_messages(panel_messages);
        }
    }
    
    /// Poll for compilation events and update the toast notification
    fn poll_compilation_events(&mut self) {
        let compilation_events = engine_scripting::get_and_clear_compilation_events();
        
        for event in compilation_events {
            match &event {
                engine_scripting::CompilationEvent::Started { script_path } => {
                    log::info!("🔥 TOAST: Compilation started for {}", script_path);
                    self.compilation_toast.start_compilation(&script_path);
                    
                    // Also add to control system's event list
                    if let Ok(mut control_events) = self.control_compilation_events.lock() {
                        control_events.push(CompilationEvent {
                            script_path: script_path.clone(),
                            event_type: "started".to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            success: None,
                        });
                    }
                }
                engine_scripting::CompilationEvent::Completed { script_path, success } => {
                    log::info!("🔥 TOAST: Compilation completed for {} (success: {})", script_path, success);
                    // Note: We don't differentiate between success/failure for the toast
                    // The toast just indicates compilation activity is done
                    self.compilation_toast.complete_compilation(&script_path);
                    
                    // Also add to control system's event list
                    if let Ok(mut control_events) = self.control_compilation_events.lock() {
                        control_events.push(CompilationEvent {
                            script_path: script_path.clone(),
                            event_type: "completed".to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            success: Some(*success),
                        });
                    }
                }
            }
        }
        
        // Debug log when toast is visible
        if self.compilation_toast.is_visible() {
            log::info!("🔥 TOAST: Toast is visible with {} active compilations", 
                      self.compilation_toast.get_active_compilation_count());
        }
    }
    
    /// Poll for file changes and trigger TypeScript compilation
    fn poll_file_changes(&mut self) {
        let file_events = self.hot_reload_manager.get_batched_events();
        
        for event in file_events {
            match event {
                engine_runtime_core::HotReloadEvent::FileModified(path, asset_type) => {
                    if asset_type == engine_runtime_core::AssetType::Script {
                        if let Some(extension) = path.extension() {
                            if extension == "ts" {
                                // Filter out temporary files (common on macOS during save)
                                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                    if filename.contains(".!") || filename.starts_with('.') {
                                        log::debug!("🔥 FILE WATCHER: Ignoring temporary file: {:?}", path);
                                    } else {
                                        log::info!("🔥 FILE WATCHER: TypeScript file modified: {:?}", path);
                                        self.trigger_typescript_compilation(&path);
                                    }
                                } else {
                                    log::info!("🔥 FILE WATCHER: TypeScript file modified: {:?}", path);
                                    self.trigger_typescript_compilation(&path);
                                }
                            }
                        }
                    }
                }
                engine_runtime_core::HotReloadEvent::FileCreated(path, asset_type) => {
                    if asset_type == engine_runtime_core::AssetType::Script {
                        if let Some(extension) = path.extension() {
                            if extension == "ts" {
                                // Filter out temporary files (common on macOS during save)
                                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                    if filename.contains(".!") || filename.starts_with('.') {
                                        log::debug!("🔥 FILE WATCHER: Ignoring temporary file: {:?}", path);
                                    } else {
                                        log::info!("🔥 FILE WATCHER: TypeScript file created: {:?}", path);
                                        self.trigger_typescript_compilation(&path);
                                    }
                                } else {
                                    log::info!("🔥 FILE WATCHER: TypeScript file created: {:?}", path);
                                    self.trigger_typescript_compilation(&path);
                                }
                            }
                        }
                    }
                }
                _ => {} // Ignore other events
            }
        }
    }
    
    /// Trigger compilation for a specific TypeScript file
    fn trigger_typescript_compilation(&mut self, script_path: &std::path::Path) {
        // Convert path to string
        if let Some(script_path_str) = script_path.to_str() {
            log::info!("🔥 FILE WATCHER: Triggering compilation for modified file: {}", script_path_str);
            
            // Trigger compilation event (this will show the toast and invalidate cache)
            engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
                script_path: script_path_str.to_string(),
            });
            
            // The actual compilation will happen when the scripts are next processed
            // The TypeScript system will see the compilation event and invalidate its cache
            // Then recompile the script when it's next needed
            
            // Simulate completion to hide the toast after a brief moment
            std::thread::spawn({
                let script_path = script_path_str.to_string();
                move || {
                    // Brief delay to show the toast
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    
                    // Report completion
                    engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Completed {
                        script_path,
                        success: true,
                    });
                }
            });
        }
    }
    
    /// Compile all TypeScript scripts in the current world
    fn compile_all_typescript_scripts(&mut self) {
        log::info!("🎮 PLAY MODE: Preparing TypeScript scripts for execution");
        
        // Get all entities with TypeScript scripts
        let typescript_script_entities: Vec<_> = self.world.query_legacy::<engine_scripting::components::TypeScriptScript>()
            .map(|(entity, script)| (entity, script.clone()))
            .collect();
        
        if typescript_script_entities.is_empty() {
            log::info!("🎮 PLAY MODE: No TypeScript scripts found");
            return;
        }
        
        log::info!("🎮 PLAY MODE: Found {} TypeScript scripts ready for execution", typescript_script_entities.len());
        
        // TypeScript scripts are compiled automatically when accessed by the script system
        // We just need to trigger a single compilation event to show user feedback
        engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
            script_path: "play_mode_scripts".to_string(),
        });
        
        // Immediate completion since scripts compile on-demand
        engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Completed {
            script_path: "play_mode_scripts".to_string(),
            success: true,
        });
    }
    
    /// Process editor actions received from remote control
    fn process_editor_actions(&mut self) {
        // Process all pending actions
        while let Ok(action) = self.action_receiver.try_recv() {
            log::info!("Processing editor action: {:?}", action);
            
            match action {
                EditorAction::StartPlay => {
                    // Compile all TypeScript scripts before starting play mode
                    self.compile_all_typescript_scripts();
                    
                    // Copy entities from editor world to coordinator world for play mode
                    self.sync_editor_world_to_coordinator();
                    self.coordinator.play_state_manager_mut().start();
                    log::info!("Started play mode via remote control");
                }
                EditorAction::StopPlay => {
                    self.coordinator.play_state_manager_mut().stop();
                    log::info!("Stopped play mode via remote control");
                }
                EditorAction::PausePlay => {
                    self.coordinator.play_state_manager_mut().pause();
                    log::info!("Paused play mode via remote control");
                }
                EditorAction::ResumePlay => {
                    self.coordinator.play_state_manager_mut().resume();
                    log::info!("Resumed play mode via remote control");
                }
                EditorAction::SyncScriptRemoval { entity_id, script_path } => {
                    log::info!("🗑️ SYNC SCRIPT REMOVAL: Syncing removal of script '{}' from entity {} to coordinator world", script_path, entity_id);
                    self.sync_script_removal_to_coordinator(entity_id, script_path);
                }
                EditorAction::ForceScriptReinitialization => {
                    log::info!("🔄 FORCE SCRIPT REINITIALIZATION: Triggering complete script reinitialization");
                    self.coordinator.force_script_reinitialization();
                }
            }
        }
    }
    
    /// Render the compilation toast notification if visible
    fn render_compilation_toast(&self, ctx: &egui::Context) {
        if self.compilation_toast.is_visible() {
            egui::Area::new("compilation_toast".into())
                .fixed_pos(egui::pos2(ctx.screen_rect().max.x - 320.0, 20.0)) // Top-right corner
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_rgb(50, 50, 50)) // Dark background
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100))) // Border
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(self.compilation_toast.get_message())
                                    .color(egui::Color32::WHITE)
                                    .size(14.0));
                            });
                        });
                });
        }
    }

    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create Longhorn-style dock layout with Scene and Game views
        let mut dock_state = DockState::new(vec![
            engine_editor_ui::PanelType::SceneView,
            engine_editor_ui::PanelType::GameView,
        ]);

        // Add Hierarchy to the left
        let [_main, _left] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2,
            vec![engine_editor_ui::PanelType::Hierarchy],
        );

        // Add Inspector to the right
        let [_main, _right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.8,
            vec![engine_editor_ui::PanelType::Inspector],
        );

        // Add Console to the bottom
        let [_main, _bottom] = dock_state.main_surface_mut().split_below(
            NodeIndex::root(),
            0.7,
            vec![engine_editor_ui::PanelType::Console],
        );

        // Initialize world with default entities
        let (world, cube_entity) = world_setup::create_default_world();

        // Load editor settings
        let settings = engine_editor_ui::EditorSettings::load();
        let settings_dialog = SettingsDialog::new(settings.clone());

        // Create scene navigation with settings applied
        let scene_navigation = SceneNavigation {
            movement_speed: settings.camera.movement_speed,
            fast_movement_multiplier: settings.camera.fast_multiplier,
            rotation_sensitivity: settings.camera.rotation_sensitivity,
            ..Default::default()
        };

        // Initialize scene view renderer
        let mut scene_view_renderer = SceneViewRenderer::new();

        // Try to get WGPU render state from eframe
        if let Some(render_state) = cc.wgpu_render_state.as_ref() {
            let device = render_state.device.clone();
            let queue = render_state.queue.clone();

            // Initialize the 3D renderer
            if let Err(e) = scene_view_renderer.initialize_renderer(device, queue) {
                log::error!("Failed to initialize 3D renderer: {}", e);
            } else {
                log::info!("3D renderer initialized successfully");
            }
        } else {
            log::warn!("No WGPU render state available - 3D rendering will be disabled");
        }

        // Create import service with registered importers
        let mut import_service = import::ImportService::new();
        import_service.register_mesh_importers();
        import_service.register_texture_importers();
        import_service.register_audio_importers();

        // Create channel for editor actions from remote control
        let (action_sender, action_receiver) = mpsc::channel();

        Self {
            dock_state,
            world,
            selected_entity: Some(cube_entity),
            selected_object: None,
            hierarchy_objects: world_setup::create_default_hierarchy(),
            project_assets: world_setup::create_default_project_assets(),
            texture_assets: assets::create_default_textures(),
            next_texture_handle: 1000, // Start texture handles at 1000
            coordinator: UnifiedEditorCoordinator::new(),
            scene_view_active: true,
            show_add_component_dialog: false,
            inspector_panel: InspectorPanel::new(),
            hierarchy_panel: HierarchyPanel::new(),
            console_panel: ConsolePanel::new(),
            project_panel: {
                let mut panel = ProjectPanel::new();
                // Set the project root to the current working directory
                panel.set_project_root(
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                );
                panel
            },
            toolbar: Toolbar::new(),
            menu_bar: MenuBar::new(),
            game_view_panel: GameViewPanel::new(),
            scene_view_panel: SceneViewPanel::new(),
            gizmo_system: GizmoSystem::new(),
            scene_navigation,
            scene_view_renderer,
            last_rendered_entity_count: 0,
            settings,
            settings_dialog,
            import_dialog: ImportDialog::new(),
            show_import_dialog: false,
            import_service,
            asset_database: assets::AssetDatabase::new(),
            last_update: std::time::Instant::now(),
            compilation_toast: CompilationToast::new(),
            hot_reload_manager: {
                let mut manager = engine_runtime_core::HotReloadManager::new();
                // Watch the assets/scripts directory for TypeScript files
                if let Err(e) = manager.watch_recursive(
                    std::path::Path::new("assets/scripts"), 
                    engine_runtime_core::AssetType::Script
                ) {
                    log::warn!("Failed to setup TypeScript file watching: {}", e);
                }
                manager
            },
            control_server_handle: None,
            control_logs: Arc::new(Mutex::new(Vec::new())),
            control_script_errors: Arc::new(Mutex::new(Vec::new())),
            control_compilation_events: Arc::new(Mutex::new(Vec::new())),
            action_receiver,
            action_sender,
        }
    }

    /// Start the editor control server for remote commands
    fn start_control_server(&mut self) {
        if self.control_server_handle.is_some() {
            return; // Already started
        }

        // Use the coordinator's world instead of creating a separate world
        // This ensures the control system works with the actual game world
        let world_arc = self.coordinator.world();
        let game_state = Arc::new(Mutex::new(GameStateInfo {
            is_playing: false,
            is_paused: false,
            frame_count: 0,
            delta_time: 0.0,
        }));

        let handler = EditorCommandHandler::new(
            world_arc,
            game_state,
            self.control_logs.clone(),
            self.control_script_errors.clone(),
            self.control_compilation_events.clone(),
            Some(self.action_sender.clone()),
        );

        let server = EditorControlServer::new(handler, 9999); // Use port 9999
        
        // Start the server in a separate thread with its own tokio runtime
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                if let Err(e) = server.start().await {
                    log::error!("Editor control server error: {}", e);
                }
            });
        });

        self.control_server_handle = Some(handle);
        log::info!("Editor control server started on port 9999");
    }
}

impl eframe::App for LonghornEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        // Update the coordinator
        self.coordinator.update(delta_time);
        
        // Process editor actions from remote control
        self.process_editor_actions();
        
        // Poll for Lua console messages and add them to the console panel
        self.poll_script_console_messages();
        
        // Poll for compilation events and update toast
        self.poll_compilation_events();
        
        // Poll for file changes and trigger TypeScript compilation
        self.poll_file_changes();
        
        // Update toast timing
        self.compilation_toast.update();
        
        
        // Request continuous repaint in play mode for script execution
        if self.coordinator.play_state_manager().get_state() == PlayState::Playing {
            ctx.request_repaint();
        }

        // Apply custom styling based on play state
        if self.coordinator.play_state_manager().get_state() != PlayState::Editing {
            // Apply Longhorn-style play mode tint (subtle blue)
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(45, 45, 55, 240);
            ctx.set_style(style);
        } else {
            engine_editor_ui::apply_longhorn_style(ctx);
        }

        // Show settings dialog if open
        self.settings_dialog.show(ctx);

        // Show import dialog if open
        if self.show_import_dialog {
            self.import_dialog.open();
            self.show_import_dialog = false; // Reset flag after opening
        }

        if let Some(result) = self.import_dialog.show(ctx) {
            match result {
                ImportResult::Import(path, settings) => {
                    self.import_single_asset(path, settings);
                }
                ImportResult::ImportBatch(paths, settings) => {
                    self.console_panel.add_messages(vec![
                        engine_editor_panels::types::ConsoleMessage::info(&format!(
                            "Importing {} assets...",
                            paths.len()
                        )),
                    ]);

                    for path in paths {
                        self.import_single_asset(path, settings.clone());
                    }
                }
                ImportResult::Cancel => {
                    // Dialog closed
                }
            }
        }

        // Apply settings to scene navigation if changed
        if self.settings_dialog.settings.camera.movement_speed
            != self.settings.camera.movement_speed
            || self.settings_dialog.settings.camera.fast_multiplier
                != self.settings.camera.fast_multiplier
            || self.settings_dialog.settings.camera.rotation_sensitivity
                != self.settings.camera.rotation_sensitivity
        {
            self.settings = self.settings_dialog.settings.clone();
            self.scene_navigation.movement_speed = self.settings.camera.movement_speed;
            self.scene_navigation.fast_movement_multiplier = self.settings.camera.fast_multiplier;
            self.scene_navigation.rotation_sensitivity = self.settings.camera.rotation_sensitivity;
        }

        // Handle global keyboard shortcuts for transform tools
        ctx.input(|i| {
            // TEST: Press T to test compilation toast
            if i.key_pressed(egui::Key::T) {
                log::info!("🔥 TOAST TEST: Manually triggering compilation events");
                engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Started {
                    script_path: "test_script.ts".to_string(),
                });
                // Add completion event after a delay (simulated by next frame)
                engine_scripting::add_compilation_event(engine_scripting::CompilationEvent::Completed {
                    script_path: "test_script.ts".to_string(),
                    success: true,
                });
            }
            
            if i.key_pressed(egui::Key::Q) {
                self.scene_navigation.current_tool = engine_editor_ui::SceneTool::Select;
                self.gizmo_system
                    .set_active_tool(engine_editor_ui::SceneTool::Select);
                self.gizmo_system.disable_move_gizmo();
            } else if i.key_pressed(egui::Key::W) {
                self.scene_navigation.current_tool = engine_editor_ui::SceneTool::Move;
                self.gizmo_system
                    .set_active_tool(engine_editor_ui::SceneTool::Move);
                if let Some(entity) = self.selected_entity {
                    if let Some(_transform) = self.world.get_component::<Transform>(entity) {
                        self.gizmo_system.enable_move_gizmo();
                    }
                }
            } else if i.key_pressed(egui::Key::E) {
                self.scene_navigation.current_tool = engine_editor_ui::SceneTool::Rotate;
                self.gizmo_system
                    .set_active_tool(engine_editor_ui::SceneTool::Rotate);
            } else if i.key_pressed(egui::Key::R) {
                self.scene_navigation.current_tool = engine_editor_ui::SceneTool::Scale;
                self.gizmo_system
                    .set_active_tool(engine_editor_ui::SceneTool::Scale);
            }
        });

        // Top menu bar (macOS style)
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        // Longhorn toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.show_toolbar(ui);
        });

        // Main docking area - this is where the magic happens!
        egui::CentralPanel::default().show(ctx, |ui| {
            let style = {
                let mut style = egui_dock::Style::from_egui(ui.style());
                // Customize docking appearance to match Longhorn
                style.tab.active.bg_fill = egui::Color32::from_rgb(0, 122, 255);
                style.tab.active.text_color = egui::Color32::WHITE;
                style.tab.inactive.text_color = egui::Color32::from_rgb(180, 180, 180);
                style.tab.focused.text_color = egui::Color32::WHITE;
                style.tab.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
                style.tab.active.outline_color = egui::Color32::from_rgb(70, 70, 70);
                style.tab.active.rounding = egui::Rounding::same(4.0);
                style.separator.width = 1.0;
                style.separator.color_idle = egui::Color32::from_rgb(70, 70, 70);
                style
            };

            // Take ownership of dock_state temporarily to avoid borrowing conflicts
            let mut dock_state = std::mem::replace(&mut self.dock_state, DockState::new(vec![]));

            DockArea::new(&mut dock_state)
                .style(style)
                .show_inside(ui, &mut EditorTabViewer { editor: self });

            // Put dock_state back
            self.dock_state = dock_state;
        });
        
        // Render compilation toast notification
        self.render_compilation_toast(ctx);
    }
}

impl LonghornEditor {
    pub fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the menu bar module
        let messages = self.menu_bar.show(ui, &mut self.dock_state);

        // Handle special actions
        for msg in &messages {
            if let engine_editor_ui::ConsoleMessage::UserAction(action) = msg {
                if action == "open_settings" {
                    self.settings_dialog.open = true;
                } else if action == "open_import_dialog" {
                    self.show_import_dialog = true;
                }
            }
        }
    }

    pub fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the toolbar module
        let actions = self.toolbar.show(
            ui,
            &mut self.coordinator.play_state_manager_mut().play_state,
            &mut self.gizmo_system,
            &mut self.scene_navigation,
            &self.world,
            self.selected_entity,
            &self.selected_object,
        );

        // Handle toolbar actions
        if actions.start_play {
            // Compile all TypeScript scripts before starting play mode
            self.compile_all_typescript_scripts();
            
            // Copy entities from editor world to coordinator world for play mode
            self.sync_editor_world_to_coordinator();
            self.coordinator.play_state_manager_mut().start();
        }
        if actions.pause_play {
            self.coordinator.play_state_manager_mut().pause();
        }
        if actions.resume_play {
            self.coordinator.play_state_manager_mut().resume();
        }
        if actions.stop_play {
            self.coordinator.play_state_manager_mut().stop();
        }

        // Handle test move action
        if actions.test_move {
            if let Some(selected_entity) = self.selected_entity {
                if let Some(transform_mut) =
                    self.world.get_component_mut::<Transform>(selected_entity)
                {
                    let _old_pos = transform_mut.position;
                    transform_mut.position[0] += 1.0; // Move 1 unit in X
                                                      // Object moved successfully
                } else {
                    // Failed to get mutable transform
                }
            } else {
                // No object selected
            }
        }
        // Toolbar actions processed
    }

    pub fn show_hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        self.hierarchy_panel.show(
            ui,
            &mut self.world,
            &mut self.selected_entity,
            &mut self.gizmo_system,
        );
    }

    pub fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        self.inspector_panel
            .show(ui, &mut self.world, self.selected_entity);
    }

    pub fn show_scene_view(&mut self, ui: &mut egui::Ui) {
        let console_messages = self.scene_view_panel.show(
            ui,
            &mut self.world,
            self.selected_entity,
            &mut self.scene_navigation,
            &mut self.gizmo_system,
            &mut self.scene_view_renderer,
            self.coordinator.play_state_manager().get_state(),
        );

        // Convert and add scene view messages to console
        if !console_messages.is_empty() {
            let editor_messages: Vec<editor_state::ConsoleMessage> = console_messages
                .into_iter()
                .map(|msg| match msg.severity {
                    engine_editor_scene_view::types::MessageSeverity::Info => {
                        editor_state::ConsoleMessage::info(&msg.message)
                    }
                    engine_editor_scene_view::types::MessageSeverity::Warning => {
                        editor_state::ConsoleMessage::warning(&msg.message)
                    }
                    engine_editor_scene_view::types::MessageSeverity::Error => {
                        editor_state::ConsoleMessage::error(&msg.message)
                    }
                })
                .collect();
            // Convert to panel ConsoleMessage type
            let panel_messages: Vec<engine_editor_panels::ConsoleMessage> = editor_messages
                .into_iter()
                .map(|msg| match msg {
                    editor_state::ConsoleMessage::Message {
                        message,
                        message_type,
                        ..
                    } => match message_type {
                        editor_state::ConsoleMessageType::Info => {
                            engine_editor_panels::ConsoleMessage::info(&message)
                        }
                        editor_state::ConsoleMessageType::Warning => {
                            engine_editor_panels::ConsoleMessage::warning(&message)
                        }
                        editor_state::ConsoleMessageType::Error => {
                            engine_editor_panels::ConsoleMessage::error(&message)
                        }
                    },
                    editor_state::ConsoleMessage::UserAction(action) => {
                        engine_editor_panels::ConsoleMessage::info(&action)
                    }
                })
                .collect();
            self.console_panel.add_messages(panel_messages);
        }
    }

    /// Render the scene from the main camera's perspective using 3D renderer
    fn render_camera_perspective(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        log::info!("render_camera_perspective called with rect: {:?}", rect);

        // Debug: List all cameras
        let cameras: Vec<_> = self
            .world
            .query_legacy::<engine_components_3d::Camera>()
            .map(|(e, _)| e)
            .collect();
        log::info!("Total cameras in world: {}", cameras.len());

        // Find the main camera entity, or use the scene editor camera
        let main_camera_entity = engine_camera_impl::find_main_camera(&self.world)
            .or_else(|| engine_camera_impl::find_active_camera(&self.world))
            .or({
                // Fallback to editor camera if no main camera
                self.scene_view_renderer.editor_camera.camera_entity
            });

        if let Some(camera_entity) = main_camera_entity {
            log::info!("Found main camera entity: {:?}", camera_entity);

            // Get camera transform and component
            if let (Some(transform), Some(_cam)) = (
                self.world
                    .get_component::<engine_components_3d::Transform>(camera_entity),
                self.world
                    .get_component::<engine_components_3d::Camera>(camera_entity),
            ) {
                log::info!(
                    "Camera transform: pos={:?}, rot={:?}",
                    transform.position,
                    transform.rotation
                );

                // Store transform values to detect changes
                static mut LAST_POS: [f32; 3] = [0.0, 0.0, 0.0];
                static mut LAST_ROT: [f32; 3] = [0.0, 0.0, 0.0];

                #[allow(static_mut_refs)]
                unsafe {
                    if LAST_POS != transform.position || LAST_ROT != transform.rotation {
                        log::warn!(
                            "CAMERA MOVED! Old pos={:?}, New pos={:?}",
                            LAST_POS,
                            transform.position
                        );
                        log::warn!(
                            "CAMERA ROTATED! Old rot={:?}, New rot={:?}",
                            LAST_ROT,
                            transform.rotation
                        );
                        LAST_POS = transform.position;
                        LAST_ROT = transform.rotation;
                    }
                }

                // Create a camera for rendering
                let camera = engine_renderer_3d::Camera::from_position_rotation(
                    transform.position,
                    transform.rotation,
                    rect.aspect_ratio(),
                );

                log::info!(
                    "Created render camera at pos={:?}, target={:?}",
                    camera.position,
                    camera.target
                );

                // Use the scene view renderer to render from this camera
                self.scene_view_renderer
                    .render_game_camera_view(&mut self.world, ui, rect, camera);
            } else {
                log::warn!("Camera components missing");
                // Show no camera message
                SceneRenderer::show_no_camera_message(ui, rect, "Camera components missing");
            }
        } else {
            log::warn!("No main camera found");
            // Show no camera message
            SceneRenderer::show_no_camera_message(ui, rect, "No main camera found");
        }
    }

    pub fn show_game_view(&mut self, ui: &mut egui::Ui) {
        let play_state = self.coordinator.play_state_manager().get_state();
        let (_, render_rect) = self.game_view_panel.show(ui, play_state);

        // Only render game camera when playing or paused
        if let Some(rect) = render_rect {
            match play_state {
                PlayState::Playing | PlayState::Paused => {
                    self.render_camera_perspective(ui, rect);
                }
                PlayState::Editing => {
                    // Show "Press Play" message
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Press Play ▶ to start game view",
                        egui::FontId::proportional(20.0),
                        egui::Color32::from_gray(150),
                    );
                }
            }
        }
    }

    pub fn show_console_panel(&mut self, ui: &mut egui::Ui) {
        // Use the internal console messages
        let mut console_messages = std::mem::take(&mut self.console_panel.console_messages);
        self.console_panel.show(ui, &mut console_messages);
        self.console_panel.console_messages = console_messages;
    }

    pub fn show_project_panel(&mut self, ui: &mut egui::Ui) {
        // Convert internal ProjectAsset to panel ProjectAsset recursively
        let panel_assets: Vec<engine_editor_panels::ProjectAsset> = self
            .project_assets
            .iter()
            .map(|asset| self.convert_project_asset(asset))
            .collect();

        self.project_panel.show(ui, &panel_assets);
    }

    #[allow(clippy::only_used_in_recursion)]
    fn convert_project_asset(
        &self,
        asset: &types::ProjectAsset,
    ) -> engine_editor_panels::ProjectAsset {
        if let Some(children) = &asset.children {
            // Recursively convert children
            let panel_children: Vec<engine_editor_panels::ProjectAsset> = children
                .iter()
                .map(|child| self.convert_project_asset(child))
                .collect();
            engine_editor_panels::ProjectAsset::folder(&asset.name, panel_children)
        } else {
            engine_editor_panels::ProjectAsset::file(&asset.name)
        }
    }

    fn import_single_asset(&mut self, path: std::path::PathBuf, settings: import::ImportSettings) {
        self.console_panel
            .add_messages(vec![engine_editor_panels::types::ConsoleMessage::info(
                &format!("Importing asset: {}", path.display()),
            )]);

        // Start the import
        let handle = self.import_service.start_import(path.clone(), settings);

        // For now, simulate immediate completion
        let asset_id = uuid::Uuid::new_v4();
        handle.complete(Ok(vec![asset_id]));

        // Determine asset type from extension
        let asset_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "obj" | "gltf" | "glb" | "fbx" => Some(assets::AssetType::Mesh),
                "png" | "jpg" | "jpeg" => Some(assets::AssetType::Texture),
                "wav" | "mp3" | "ogg" => Some(assets::AssetType::Audio),
                _ => None,
            })
            .unwrap_or(assets::AssetType::Mesh);

        // Add to database
        self.asset_database
            .add_imported_asset(asset_id, path.clone(), asset_type);

        // Add to project assets view
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Find or create the appropriate folder based on asset type
        let folder_name = match asset_type {
            assets::AssetType::Mesh => "Models",
            assets::AssetType::Texture => "Textures",
            assets::AssetType::Audio => "Audio",
            _ => "Other",
        };

        // Add the asset to the project view
        // For now, we'll add it to the root since we're using a simple Vec structure
        // In a real implementation, you'd want a proper tree structure
        let new_asset = types::ProjectAsset::file(&file_name);

        // Check if we have an Assets folder
        if let Some(assets_folder) = self.project_assets.iter_mut().find(|a| a.name == "Assets") {
            // Check if we have the appropriate subfolder
            if let Some(children) = &mut assets_folder.children {
                if let Some(target_folder) = children.iter_mut().find(|c| c.name == folder_name) {
                    // Add to existing folder
                    if let Some(folder_children) = &mut target_folder.children {
                        folder_children.push(new_asset);
                    } else {
                        target_folder.children = Some(vec![new_asset]);
                    }
                } else {
                    // Create new folder
                    children.push(types::ProjectAsset::folder(folder_name, vec![new_asset]));
                }
            }
        } else {
            // Create Assets folder structure
            self.project_assets.push(types::ProjectAsset::folder(
                "Assets",
                vec![types::ProjectAsset::folder(folder_name, vec![new_asset])],
            ));
        }

        self.console_panel
            .add_messages(vec![engine_editor_panels::types::ConsoleMessage::info(
                &format!(
                    "Asset imported successfully: {} (ID: {})",
                    file_name, asset_id
                ),
            )]);

        // Debug: Log the project assets structure
        log::info!("Project assets after import: {:?}", self.project_assets);
    }
}

impl engine_editor_ui::EditorApp for LonghornEditor {
    fn show_panel(&mut self, ui: &mut egui::Ui, panel_type: PanelType) {
        match panel_type {
            PanelType::Hierarchy => self.show_hierarchy_panel(ui),
            PanelType::Inspector => self.show_inspector_panel(ui),
            PanelType::SceneView => self.show_scene_view(ui),
            PanelType::GameView => self.show_game_view(ui),
            PanelType::Console => self.show_console_panel(ui),
            PanelType::Project => self.show_project_panel(ui),
        }
    }
}
