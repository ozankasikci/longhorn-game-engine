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
            
            // Start in play mode if requested
            if args.play {
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
        let script_entities: Vec<_> = self.world.query_legacy::<engine_scripting::components::LuaScript>()
            .map(|(entity, script)| (entity, script.clone()))
            .collect();
            
        println!("[LonghornEditor] Syncing {} entities with LuaScript components to coordinator", script_entities.len());
        
        for (entity, script) in script_entities {
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
            
            println!("  Copied entity {:?} -> {:?} with script: {}", entity, new_entity, 
                coordinator_world.get_component::<engine_scripting::components::LuaScript>(new_entity)
                    .map(|s| s.script_path.as_str()).unwrap_or("Unknown"));
        }
        
        drop(coordinator_world);
        println!("[LonghornEditor] World sync complete!");
    }
    
    /// Poll for Lua console messages and add them to the console panel
    fn poll_lua_console_messages(&mut self) {
        let lua_messages = engine_scripting::get_and_clear_console_messages();
        if !lua_messages.is_empty() {
            let panel_messages: Vec<engine_editor_panels::types::ConsoleMessage> = lua_messages
                .into_iter()
                .map(|msg| engine_editor_panels::types::ConsoleMessage::info(&msg.message))
                .collect();
            self.console_panel.add_messages(panel_messages);
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
        }
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
        
        // Poll for Lua console messages and add them to the console panel
        self.poll_lua_console_messages();
        
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
