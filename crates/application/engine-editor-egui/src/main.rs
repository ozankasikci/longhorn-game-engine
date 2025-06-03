// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

mod editor_state;
mod scene_renderer;
mod types;
mod panels;
mod ui;
mod bridge;
mod utils;
mod play_state;
mod world_setup;
mod assets;
mod editor_coordinator;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex};
use engine_ecs_core::{World, Entity};
use engine_components_3d::Transform;
use engine_components_ui::Name;
use editor_state::ConsoleMessage;
use scene_renderer::SceneRenderer;
use types::{PlayState, SceneNavigation, GizmoSystem, TextureAsset, ProjectAsset, PanelType, HierarchyObject};
use ui::style::{setup_custom_fonts, setup_custom_style};
use editor_coordinator::EditorCoordinator;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1000.0])
            .with_title("Unity Editor - Mobile Game Engine (EGUI + Docking)"),
        ..Default::default()
    };

    eframe::run_native(
        "Unity Editor",
        options,
        Box::new(|cc| {
            // Load custom design constraints if available
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);
            
            Ok(Box::new(UnityEditor::new(cc)))
        }),
    )
}

/// Unity-style editor application with dockable panels
pub struct UnityEditor {
    // Docking system
    dock_state: DockState<PanelType>,
    
    // ECS v2 Integration
    world: World,
    selected_entity: Option<Entity>,
    
    // Editor state
    selected_object: Option<String>,
    pub console_messages: Vec<ConsoleMessage>,
    
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
    
    // 3D scene renderer
    scene_renderer: SceneRenderer,
    scene_view_renderer: panels::scene_view::scene_view_impl::SceneViewRenderer,
    
    // Phase 10.2: Track entity counts for change detection
    last_rendered_entity_count: usize,
}

impl UnityEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create Unity-style dock layout with Scene and Game views
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
        let (world, camera_entity, init_messages) = world_setup::create_default_world();
        
        Self {
            dock_state,
            world,
            selected_entity: Some(camera_entity),
            selected_object: None,
            console_messages: {
                let mut messages = vec![
                    ConsoleMessage::info("🎮 Unity Editor initialized with dockable panels"),
                    ConsoleMessage::info("✅ EGUI docking system active"),
                ];
                messages.extend(init_messages);
                messages.extend(vec![
                    ConsoleMessage::info("📝 Debug logs are being written to debug_console.log"),
                    ConsoleMessage::info("💡 Use 📋 Copy All or 💾 Export buttons to get logs"),
                    ConsoleMessage::info("🔧 Select an entity and use the move tool to test gizmos"),
                ]);
                messages
            },
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
            scene_navigation: SceneNavigation::default(),
            scene_renderer: SceneRenderer::new(),
            scene_view_renderer: panels::scene_view::scene_view_impl::SceneViewRenderer::new(),
            last_rendered_entity_count: 0,
        }
    }
}

impl eframe::App for UnityEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update play state timing
        self.coordinator.update_delta_time();
        
        // Apply custom styling based on play state
        if self.coordinator.get_play_state() != PlayState::Editing {
            // Apply Unity-style play mode tint (subtle blue)
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(45, 45, 55, 240);
            ctx.set_style(style);
        } else {
            ui::style::apply_unity_style(ctx);
        }
        
        // Top menu bar (macOS style)
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });
        
        // Unity toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.show_toolbar(ui);
        });
        
        // Main docking area - this is where the magic happens!
        egui::CentralPanel::default().show(ctx, |ui| {
            let style = {
                let mut style = egui_dock::Style::from_egui(ui.style());
                // Customize docking appearance to match Unity
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
                .show_inside(ui, &mut ui::EditorTabViewer { editor: self });
            
            // Put dock_state back
            self.dock_state = dock_state;
        });
    }
}

impl UnityEditor {
    pub fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the menu bar module
        let messages = self.menu_bar.show(ui, &mut self.dock_state);
        self.console_messages.extend(messages);
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
            let messages = self.coordinator.start_play();
            self.console_messages.extend(messages);
        }
        if actions.pause_play {
            let messages = self.coordinator.pause_play();
            self.console_messages.extend(messages);
        }
        if actions.resume_play {
            let messages = self.coordinator.resume_play();
            self.console_messages.extend(messages);
        }
        if actions.stop_play {
            let messages = self.coordinator.stop_play();
            self.console_messages.extend(messages);
        }
        
        // Handle test move action
        if actions.test_move {
            if let Some(selected_entity) = self.selected_entity {
                if let Some(transform_mut) = self.world.get_component_mut::<Transform>(selected_entity) {
                    let old_pos = transform_mut.position;
                    transform_mut.position[0] += 1.0; // Move 1 unit in X
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🔧 TEST: Moved object from [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
                        old_pos[0], old_pos[1], old_pos[2],
                        transform_mut.position[0], transform_mut.position[1], transform_mut.position[2]
                    )));
                } else {
                    self.console_messages.push(ConsoleMessage::info("🔧 TEST: Failed to get mutable transform"));
                }
            } else {
                self.console_messages.push(ConsoleMessage::info("🔧 TEST: No object selected"));
            }
        }
        
        // Add any messages from toolbar
        self.console_messages.extend(actions.messages);
    }
    
    pub fn show_hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        let messages = self.hierarchy_panel.show(ui, &mut self.world, &mut self.selected_entity, &mut self.gizmo_system);
        self.console_messages.extend(messages);
    }
    
    pub fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        let messages = self.inspector_panel.show(ui, &mut self.world, self.selected_entity);
        self.console_messages.extend(messages);
    }
    
    pub fn show_scene_view(&mut self, ui: &mut egui::Ui) {
        let messages = self.scene_view_panel.show(
            ui,
            &mut self.world,
            self.selected_entity,
            &mut self.scene_navigation,
            &mut self.gizmo_system,
            &mut self.scene_view_renderer,
            self.coordinator.get_play_state(),
        );
        self.console_messages.extend(messages);
    }

    /// Render the scene from the main camera's perspective
    fn render_camera_perspective(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let messages = panels::scene_view::rendering::SceneRenderer::render_camera_perspective(
            &self.world,
            ui,
            rect
        );
        self.console_messages.extend(messages);
    }

    pub fn show_game_view(&mut self, ui: &mut egui::Ui) {
        let (messages, render_rect) = self.game_view_panel.show(ui, self.coordinator.get_play_state());
        self.console_messages.extend(messages);
        
        // If we got a rect back, render the camera perspective
        if let Some(rect) = render_rect {
            self.render_camera_perspective(ui, rect);
        }
    }
    
    pub fn show_console_panel(&mut self, ui: &mut egui::Ui) {
        self.console_panel.show(ui, &mut self.console_messages);
    }
    
    pub fn show_project_panel(&mut self, ui: &mut egui::Ui) {
        self.project_panel.show(ui, &self.project_assets, &mut self.console_messages);
    }
}
