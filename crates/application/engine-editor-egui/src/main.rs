// Longhorn Game Engine editor built with EGUI and dockable panels
// Provides a modern, responsive interface with drag-and-drop docking

mod editor_state;
mod types;
mod panels;
mod ui;
mod styling;
mod bridge;
mod utils;
mod play_state;
mod world_setup;
mod assets;
mod editor_coordinator;
mod settings;

use std::sync::Arc;
use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex};
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use engine_components_ui::Name;
use editor_state::ConsoleMessage;
use types::{PlayState, SceneNavigation, GizmoSystem, TextureAsset, ProjectAsset, PanelType, HierarchyObject};
use styling::{setup_custom_fonts, setup_custom_style};
use editor_coordinator::EditorCoordinator;
use settings::EditorSettings;
use ui::SettingsDialog;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1000.0])
            .with_title("Longhorn Game Engine Editor (EGUI + Docking)"),
        ..Default::default()
    };

    eframe::run_native(
        "Longhorn Game Engine Editor",
        options,
        Box::new(|cc| {
            // Load custom design constraints if available
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);
            
            Ok(Box::new(LonghornEditor::new(cc)))
        }),
    )
}

/// Longhorn Game Engine editor application with dockable panels
pub struct LonghornEditor {
    // Docking system
    dock_state: DockState<PanelType>,
    
    // ECS v2 Integration
    world: World,
    selected_entity: Option<Entity>,
    
    // Editor state
    selected_object: Option<String>,
    
    // Panel data
    hierarchy_objects: Vec<HierarchyObject>,
    project_assets: Vec<ProjectAsset>,
    
    // Texture asset system
    texture_assets: std::collections::HashMap<u64, TextureAsset>,
    next_texture_handle: u64,
    
    // Editor coordination
    coordinator: EditorCoordinator,
    
    // UI state
    scene_view_active: bool,
    show_add_component_dialog: bool,
    inspector_panel: panels::inspector::InspectorPanel,
    hierarchy_panel: panels::hierarchy::HierarchyPanel,
    console_panel: panels::console::ConsolePanel,
    project_panel: panels::project::ProjectPanel,
    toolbar: ui::toolbar::Toolbar,
    menu_bar: ui::menu_bar::MenuBar,
    game_view_panel: panels::game_view::GameViewPanel,
    scene_view_panel: panels::scene_view::SceneViewPanel,
    
    // Gizmo system
    gizmo_system: GizmoSystem,
    
    // Scene navigation system
    scene_navigation: SceneNavigation,
    
    // 3D scene renderer (using engine-renderer-3d)
    scene_view_renderer: panels::scene_view::scene_view_impl::SceneViewRenderer,
    
    // Phase 10.2: Track entity counts for change detection
    last_rendered_entity_count: usize,
    
    // Editor settings
    settings: EditorSettings,
    settings_dialog: SettingsDialog,
}

impl LonghornEditor {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Create Longhorn-style dock layout with Scene and Game views
        let mut dock_state = DockState::new(vec![PanelType::SceneView, PanelType::GameView]);
        
        // Add Hierarchy to the left
        let [_main, _left] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2,
            vec![PanelType::Hierarchy]
        );
        
        // Add Inspector to the right
        let [_main, _right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.8,
            vec![PanelType::Inspector]
        );
        
        // Add Console to the bottom
        let [_main, _bottom] = dock_state.main_surface_mut().split_below(
            NodeIndex::root(),
            0.7,
            vec![PanelType::Console]
        );
        
        // Initialize world with default entities
        let (world, camera_entity) = world_setup::create_default_world();
        
        // Verify world has entities immediately after creation
        
        // Load editor settings
        let settings = EditorSettings::load();
        let settings_dialog = SettingsDialog::new(settings.clone());
        
        // Create scene navigation with settings applied
        let mut scene_navigation = SceneNavigation::default();
        scene_navigation.movement_speed = settings.camera.movement_speed;
        scene_navigation.fast_movement_multiplier = settings.camera.fast_multiplier;
        scene_navigation.rotation_sensitivity = settings.camera.rotation_sensitivity;
        
        // Initialize scene view renderer
        let mut scene_view_renderer = panels::scene_view::scene_view_impl::SceneViewRenderer::new();
        
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
        
        Self {
            dock_state,
            world,
            selected_entity: Some(camera_entity),
            selected_object: None,
            hierarchy_objects: world_setup::create_default_hierarchy(),
            project_assets: world_setup::create_default_project_assets(),
            texture_assets: assets::create_default_textures(),
            next_texture_handle: 1000, // Start texture handles at 1000
            coordinator: EditorCoordinator::new(),
            scene_view_active: true,
            show_add_component_dialog: false,
            inspector_panel: panels::inspector::InspectorPanel::new(),
            hierarchy_panel: panels::hierarchy::HierarchyPanel::new(),
            console_panel: panels::console::ConsolePanel::new(),
            project_panel: panels::project::ProjectPanel::new(),
            toolbar: ui::toolbar::Toolbar::new(),
            menu_bar: ui::menu_bar::MenuBar::new(),
            game_view_panel: panels::game_view::GameViewPanel::new(),
            scene_view_panel: panels::scene_view::SceneViewPanel::new(),
            gizmo_system: GizmoSystem::new(),
            scene_navigation,
            scene_view_renderer,
            last_rendered_entity_count: 0,
            settings,
            settings_dialog,
        }
    }
}

impl eframe::App for LonghornEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update play state timing
        self.coordinator.update_delta_time();
        
        // Apply custom styling based on play state
        if self.coordinator.get_play_state() != PlayState::Editing {
            // Apply Longhorn-style play mode tint (subtle blue)
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(45, 45, 55, 240);
            ctx.set_style(style);
        } else {
            styling::apply_longhorn_style(ctx);
        }
        
        // Show settings dialog if open
        self.settings_dialog.show(ctx);
        
        // Apply settings to scene navigation if changed
        if self.settings_dialog.settings.camera.movement_speed != self.settings.camera.movement_speed ||
           self.settings_dialog.settings.camera.fast_multiplier != self.settings.camera.fast_multiplier ||
           self.settings_dialog.settings.camera.rotation_sensitivity != self.settings.camera.rotation_sensitivity {
            self.settings = self.settings_dialog.settings.clone();
            self.scene_navigation.movement_speed = self.settings.camera.movement_speed;
            self.scene_navigation.fast_movement_multiplier = self.settings.camera.fast_multiplier;
            self.scene_navigation.rotation_sensitivity = self.settings.camera.rotation_sensitivity;
        }
        
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
                .show_inside(ui, &mut ui::tab_viewer::EditorTabViewer { editor: self });
            
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
            if let ConsoleMessage::UserAction(action) = msg {
                if action == "open_settings" {
                    self.settings_dialog.open = true;
                }
            }
        }
    }
    
    pub fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the toolbar module
        let actions = self.toolbar.show(
            ui,
            self.coordinator.get_play_state_mut(),
            &mut self.gizmo_system,
            &self.world,
            self.selected_entity,
            &self.selected_object,
        );
        
        // Handle toolbar actions
        if actions.start_play {
            self.coordinator.start_play();
        }
        if actions.pause_play {
            self.coordinator.pause_play();
        }
        if actions.resume_play {
            self.coordinator.resume_play();
        }
        if actions.stop_play {
            self.coordinator.stop_play();
        }
        
        // Handle test move action
        if actions.test_move {
            if let Some(selected_entity) = self.selected_entity {
                if let Some(transform_mut) = self.world.get_component_mut::<Transform>(selected_entity) {
                    let old_pos = transform_mut.position;
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
        self.hierarchy_panel.show(ui, &mut self.world, &mut self.selected_entity, &mut self.gizmo_system);
    }
    
    pub fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        self.inspector_panel.show(ui, &mut self.world, self.selected_entity);
    }
    
    pub fn show_scene_view(&mut self, ui: &mut egui::Ui) {
        let console_messages = self.scene_view_panel.show(
            ui,
            &mut self.world,
            self.selected_entity,
            &mut self.scene_navigation,
            &mut self.gizmo_system,
            &mut self.scene_view_renderer,
            self.coordinator.get_play_state(),
        );
        
        // Add camera logging messages to console
        if !console_messages.is_empty() {
            self.console_panel.add_messages(console_messages);
        }
    }

    /// Render the scene from the main camera's perspective using 3D renderer
    fn render_camera_perspective(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        log::info!("render_camera_perspective called with rect: {:?}", rect);
        
        // Debug: List all cameras
        let cameras: Vec<_> = self.world.query_legacy::<engine_components_3d::Camera>()
            .map(|(e, _)| e)
            .collect();
        log::info!("Total cameras in world: {}", cameras.len());
        
        // Find the main camera entity, or use the scene editor camera
        let main_camera_entity = engine_camera_impl::find_main_camera(&self.world)
            .or_else(|| engine_camera_impl::find_active_camera(&self.world))
            .or_else(|| {
                // Fallback to editor camera if no main camera
                self.scene_view_renderer.editor_camera.camera_entity
            });
        
        if let Some(camera_entity) = main_camera_entity {
            log::info!("Found main camera entity: {:?}", camera_entity);
            
            // Get camera transform and component
            if let (Some(transform), Some(cam)) = (
                self.world.get_component::<engine_components_3d::Transform>(camera_entity),
                self.world.get_component::<engine_components_3d::Camera>(camera_entity)
            ) {
                log::info!("Camera transform: pos={:?}, rot={:?}", transform.position, transform.rotation);
                
                // Store transform values to detect changes
                static mut LAST_POS: [f32; 3] = [0.0, 0.0, 0.0];
                static mut LAST_ROT: [f32; 3] = [0.0, 0.0, 0.0];
                
                unsafe {
                    if LAST_POS != transform.position || LAST_ROT != transform.rotation {
                        log::warn!("CAMERA MOVED! Old pos={:?}, New pos={:?}", LAST_POS, transform.position);
                        log::warn!("CAMERA ROTATED! Old rot={:?}, New rot={:?}", LAST_ROT, transform.rotation);
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
                
                log::info!("Created render camera at pos={:?}, target={:?}", camera.position, camera.target);
                
                // Use the scene view renderer to render from this camera
                self.scene_view_renderer.render_game_camera_view(
                    &mut self.world,
                    ui,
                    rect,
                    camera,
                );
            } else {
                log::warn!("Camera components missing");
                // Show no camera message
                panels::scene_view::rendering::SceneRenderer::show_no_camera_message(
                    ui, rect, "Camera components missing"
                );
            }
        } else {
            log::warn!("No main camera found");
            // Show no camera message
            panels::scene_view::rendering::SceneRenderer::show_no_camera_message(
                ui, rect, "No main camera found"
            );
        }
    }

    pub fn show_game_view(&mut self, ui: &mut egui::Ui) {
        let (_, render_rect) = self.game_view_panel.show(ui, self.coordinator.get_play_state());
        
        // If we got a rect back, render the camera perspective
        if let Some(rect) = render_rect {
            self.render_camera_perspective(ui, rect);
        }
    }
    
    pub fn show_console_panel(&mut self, ui: &mut egui::Ui) {
        // Use the internal console messages
        let mut console_messages = std::mem::take(&mut self.console_panel.console_messages);
        self.console_panel.show(ui, &mut console_messages);
        self.console_panel.console_messages = console_messages;
    }
    
    pub fn show_project_panel(&mut self, ui: &mut egui::Ui) {
        self.project_panel.show(ui, &self.project_assets);
    }
}
