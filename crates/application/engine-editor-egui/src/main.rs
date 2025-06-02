// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

mod editor_state;
mod scene_renderer;
mod types;
mod panels;
mod ui;
mod bridge;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Visibility, Light, Material, Mesh, MeshType};
use engine_components_2d::{Sprite, SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_camera::{CameraComponent, CameraType, Viewport, Camera, Camera2D};
use editor_state::{EditorState, GameObject, ConsoleMessage, ConsoleMessageType};
use scene_renderer::SceneRenderer;
use types::{PlayState, SceneNavigation, SceneTool, GizmoSystem, GizmoAxis, GizmoPlane, GizmoComponent, GizmoInteractionState, TextureAsset, ProjectAsset, PanelType};
use panels::scene_view::gizmos::{MoveGizmo, Ray};
use std::io::Write as _IoWrite;

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
struct UnityEditor {
    // Docking system
    dock_state: DockState<PanelType>,
    
    // ECS v2 Integration
    world: World,
    selected_entity: Option<Entity>,
    
    // Editor state
    selected_object: Option<String>,
    console_messages: Vec<ConsoleMessage>,
    
    // Panel data
    hierarchy_objects: Vec<HierarchyObject>,
    project_assets: Vec<ProjectAsset>,
    
    // Texture asset system
    texture_assets: std::collections::HashMap<u64, TextureAsset>,
    next_texture_handle: u64,
    
    // Play state management
    play_state: PlayState,
    game_start_time: Option<std::time::Instant>,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    
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
    
    // Gizmo system
    gizmo_system: GizmoSystem,
    
    // Scene navigation system
    scene_navigation: SceneNavigation,
    
    // 3D scene renderer
    scene_renderer: SceneRenderer,
    
    // Phase 10.2: Track entity counts for change detection
    last_rendered_entity_count: usize,
}

impl UnityEditor {
    /// Create default colored textures for sprites
    fn create_default_textures() -> std::collections::HashMap<u64, TextureAsset> {
        let mut textures = std::collections::HashMap::new();
        
        // Create basic colored square textures
        let textures_data = [
            (1000, "White Square", [1.0, 1.0, 1.0, 1.0]),
            (1001, "Red Square", [1.0, 0.2, 0.2, 1.0]),
            (1002, "Green Square", [0.2, 1.0, 0.2, 1.0]),
            (1003, "Blue Square", [0.2, 0.2, 1.0, 1.0]),
            (1004, "Yellow Square", [1.0, 1.0, 0.2, 1.0]),
            (1005, "Purple Square", [1.0, 0.2, 1.0, 1.0]),
            (1006, "Cyan Square", [0.2, 1.0, 1.0, 1.0]),
            (1007, "Orange Square", [1.0, 0.5, 0.2, 1.0]),
        ];
        
        for (handle, name, _color) in textures_data {
            textures.insert(handle, TextureAsset {
                id: egui::TextureId::default(), // Placeholder for now
                name: name.to_string(),
                size: egui::Vec2::new(64.0, 64.0),
                path: format!("builtin:{}", name.to_lowercase().replace(' ', "_")),
            });
        }
        
        textures
    }

    /// Transition to playing state
    fn start_play(&mut self) {
        if self.play_state == PlayState::Editing {
            self.play_state = PlayState::Playing;
            self.game_start_time = Some(std::time::Instant::now());
            self.last_frame_time = std::time::Instant::now();
            self.delta_time = 0.0;
            self.console_messages.push(ConsoleMessage::info("▶️ Play mode started"));
        }
    }
    
    /// Pause the game (only from playing state)
    fn pause_play(&mut self) {
        if self.play_state == PlayState::Playing {
            self.play_state = PlayState::Paused;
            self.console_messages.push(ConsoleMessage::info("⏸️ Play mode paused"));
        }
    }
    
    /// Resume from paused state
    fn resume_play(&mut self) {
        if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
            self.last_frame_time = std::time::Instant::now(); // Reset delta time
            self.console_messages.push(ConsoleMessage::info("▶️ Play mode resumed"));
        }
    }
    
    /// Stop play mode and return to editing
    fn stop_play(&mut self) {
        if self.play_state != PlayState::Editing {
            self.play_state = PlayState::Editing;
            self.game_start_time = None;
            self.delta_time = 0.0;
            self.console_messages.push(ConsoleMessage::info("⏹️ Play mode stopped - returned to editing"));
        }
    }
    
    /// Update delta time for game loop
    fn update_delta_time(&mut self) {
        let now = std::time::Instant::now();
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        // Scene navigation needs delta time even in editing mode
        // Only pause delta time calculation during actual pause state
        if self.play_state == PlayState::Paused {
            self.delta_time = 0.0;
        }
    }
    
    /// Get current play time in seconds
    fn get_play_time(&self) -> f32 {
        if let Some(start_time) = self.game_start_time {
            start_time.elapsed().as_secs_f32()
        } else {
            0.0
        }
    }

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
        
        // Initialize ECS v2 World with some test entities
        let mut world = World::new();
        
        // Create camera entity
        let camera_entity = world.create_entity();
        world.add_component(camera_entity, Transform {
            position: [0.0, 0.0, 5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        world.add_component(camera_entity, Name::new("Main Camera")).unwrap();
        world.add_component(camera_entity, Camera::default()).unwrap();
        
        // Create cube entity with mesh and material
        let cube_entity = world.create_entity();
        world.add_component(cube_entity, Transform {
            position: [1.0, 0.0, 0.0],
            rotation: [0.0, 45.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        world.add_component(cube_entity, Name::new("Cube")).unwrap();
        world.add_component(cube_entity, Mesh {
            mesh_type: MeshType::Cube,
        }).unwrap();
        world.add_component(cube_entity, Material {
            color: [0.8, 0.2, 0.2, 1.0], // Red cube
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(cube_entity, Visibility::default()).unwrap();
        
        // Create sphere entity with mesh and material
        let sphere_entity = world.create_entity();
        world.add_component(sphere_entity, Transform {
            position: [-1.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.5, 1.5, 1.5],
        }).unwrap();
        world.add_component(sphere_entity, Name::new("Sphere")).unwrap();
        world.add_component(sphere_entity, Mesh {
            mesh_type: MeshType::Sphere,
        }).unwrap();
        world.add_component(sphere_entity, Material {
            color: [0.2, 0.8, 0.2, 1.0], // Green sphere
            metallic: 0.1,
            roughness: 0.3,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(sphere_entity, Visibility::default()).unwrap();
        
        // Create plane entity (ground)
        let plane_entity = world.create_entity();
        world.add_component(plane_entity, Transform {
            position: [0.0, -1.5, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [5.0, 1.0, 5.0],
        }).unwrap();
        world.add_component(plane_entity, Name::new("Ground Plane")).unwrap();
        world.add_component(plane_entity, Mesh {
            mesh_type: MeshType::Plane,
        }).unwrap();
        world.add_component(plane_entity, Material {
            color: [0.6, 0.6, 0.6, 1.0], // Gray ground
            metallic: 0.0,
            roughness: 0.8,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(plane_entity, Visibility::default()).unwrap();
        
        // Create sprite entities to test sprite rendering
        let red_sprite_entity = world.create_entity();
        world.add_component(red_sprite_entity, Transform {
            position: [-2.0, 0.5, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.5, 1.5, 1.0],
        }).unwrap();
        world.add_component(red_sprite_entity, Name::new("Red Sprite")).unwrap();
        world.add_component(red_sprite_entity, SpriteRenderer {
            sprite: Sprite::new().with_texture(1001).with_color(1.0, 0.8, 0.8, 1.0), // Red texture with light tint
            layer: 0,
            material_override: None,
            enabled: true,
        }).unwrap();
        world.add_component(red_sprite_entity, Visibility::default()).unwrap();
        
        let blue_sprite_entity = world.create_entity();
        world.add_component(blue_sprite_entity, Transform {
            position: [2.0, 0.5, 0.0],
            rotation: [0.0, 0.0, 15.0], // Slightly rotated
            scale: [1.0, 2.0, 1.0], // Tall sprite
        }).unwrap();
        world.add_component(blue_sprite_entity, Name::new("Blue Sprite")).unwrap();
        world.add_component(blue_sprite_entity, SpriteRenderer {
            sprite: Sprite::new().with_texture(1003), // Blue texture
            layer: 1,
            material_override: None,
            enabled: true,
        }).unwrap();
        world.add_component(blue_sprite_entity, Visibility::default()).unwrap();
        
        let yellow_sprite_entity = world.create_entity();
        world.add_component(yellow_sprite_entity, Transform {
            position: [0.0, 2.0, -1.0], // Higher up and further back
            rotation: [0.0, 0.0, 0.0],
            scale: [0.8, 0.8, 1.0], // Smaller sprite
        }).unwrap();
        world.add_component(yellow_sprite_entity, Name::new("Yellow Sprite")).unwrap();
        world.add_component(yellow_sprite_entity, SpriteRenderer {
            sprite: Sprite::new().with_texture(1004).with_color(1.0, 1.0, 0.5, 0.9), // Yellow with slight transparency
            layer: 2,
            material_override: None,
            enabled: true,
        }).unwrap();
        world.add_component(yellow_sprite_entity, Visibility::default()).unwrap();
        
        Self {
            dock_state,
            world,
            selected_entity: Some(camera_entity),
            selected_object: None,
            console_messages: vec![
                ConsoleMessage::info("🎮 Unity Editor initialized with dockable panels"),
                ConsoleMessage::info("✅ EGUI docking system active"),
                ConsoleMessage::info("🚀 ECS v2 World created with 3 entities!"),
                ConsoleMessage::info("📝 Debug logs are being written to debug_console.log"),
                ConsoleMessage::info("💡 Use 📋 Copy All or 💾 Export buttons to get logs"),
                ConsoleMessage::info("🔧 Select an entity and use the move tool to test gizmos"),
            ],
            hierarchy_objects: vec![
                HierarchyObject::new("📱 Main Camera", ObjectType::Camera),
                HierarchyObject::new("☀️ Directional Light", ObjectType::Light),
                HierarchyObject::parent("📦 Game Objects", vec![
                    HierarchyObject::new("🧊 Cube", ObjectType::GameObject),
                    HierarchyObject::new("⚽ Sphere", ObjectType::GameObject),
                    HierarchyObject::new("🔺 Plane", ObjectType::GameObject),
                ]),
            ],
            project_assets: vec![
                ProjectAsset::folder("📁 Scripts", vec![
                    ProjectAsset::file("📄 PlayerController.cs"),
                    ProjectAsset::file("📄 GameManager.cs"),
                    ProjectAsset::file("📄 UIController.cs"),
                ]),
                ProjectAsset::folder("📁 Materials", vec![
                    ProjectAsset::file("🎨 DefaultMaterial.mat"),
                    ProjectAsset::file("🎨 WoodTexture.mat"),
                    ProjectAsset::file("🎨 MetalSurface.mat"),
                ]),
                ProjectAsset::folder("📁 Textures", vec![
                    ProjectAsset::file("🖼️ grass.png"),
                    ProjectAsset::file("🖼️ brick_wall.jpg"),
                    ProjectAsset::file("🖼️ sky_gradient.png"),
                ]),
            ],
            texture_assets: Self::create_default_textures(),
            next_texture_handle: 1000, // Start texture handles at 1000
            play_state: PlayState::default(),
            game_start_time: None,
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.0,
            scene_view_active: true,
            show_add_component_dialog: false,
            inspector_panel: panels::inspector::InspectorPanel::new(),
            hierarchy_panel: panels::hierarchy::HierarchyPanel::new(),
            console_panel: panels::console::ConsolePanel::new(),
            project_panel: panels::project::ProjectPanel::new(),
            toolbar: ui::toolbar::Toolbar::new(),
            menu_bar: ui::menu_bar::MenuBar::new(),
            game_view_panel: panels::game_view::GameViewPanel::new(),
            gizmo_system: GizmoSystem::new(),
            scene_navigation: SceneNavigation::default(),
            scene_renderer: SceneRenderer::new(),
            last_rendered_entity_count: 0,
        }
    }
}

impl eframe::App for UnityEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update play state timing
        self.update_delta_time();
        
        // Apply custom styling based on play state
        if self.play_state != PlayState::Editing {
            // Apply Unity-style play mode tint (subtle blue)
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(45, 45, 55, 240);
            ctx.set_style(style);
        } else {
            apply_unity_style(ctx);
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
                .show_inside(ui, &mut EditorTabViewer { editor: self });
            
            // Put dock_state back
            self.dock_state = dock_state;
        });
    }
}

/// Wrapper to avoid borrowing conflicts
struct EditorTabViewer<'a> {
    editor: &'a mut UnityEditor,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = PanelType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelType::Hierarchy => "🏗️ Hierarchy".into(),
            PanelType::Inspector => "🔍 Inspector".into(),
            PanelType::SceneView => "🎨 Scene".into(),
            PanelType::GameView => "🎮 Game".into(),
            PanelType::Console => "🖥️ Console".into(),
            PanelType::Project => "📁 Project".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelType::Hierarchy => self.editor.show_hierarchy_panel(ui),
            PanelType::Inspector => self.editor.show_inspector_panel(ui),
            PanelType::SceneView => self.editor.show_scene_view(ui),
            PanelType::GameView => self.editor.show_game_view(ui),
            PanelType::Console => self.editor.show_console_panel(ui),
            PanelType::Project => self.editor.show_project_panel(ui),
        }
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, _surface: SurfaceIndex, _node: NodeIndex) {
        if ui.button("Close Tab").clicked() {
            self.editor.console_messages.push(ConsoleMessage::info(&format!("🗑️ Closed {:?} panel", tab)));
            ui.close_menu();
        }
        if ui.button("Duplicate Tab").clicked() {
            // Note: We can't modify dock_state here since it's already borrowed
            self.editor.console_messages.push(ConsoleMessage::info(&format!("📋 Duplicated {:?} panel", tab)));
            ui.close_menu();
        }
    }
}

impl UnityEditor {
    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the menu bar module
        let messages = self.menu_bar.show(ui, &mut self.dock_state);
        self.console_messages.extend(messages);
    }
    
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        // Delegate to the toolbar module
        let actions = self.toolbar.show(
            ui,
            &mut self.play_state,
            &mut self.gizmo_system,
            &self.world,
            self.selected_entity,
            &self.selected_object,
        );
        
        // Handle toolbar actions
        if actions.start_play {
            self.start_play();
        }
        if actions.pause_play {
            self.pause_play();
        }
        if actions.resume_play {
            self.resume_play();
        }
        if actions.stop_play {
            self.stop_play();
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
    
    fn show_hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        // Delegate to the hierarchy panel module
        let messages = self.hierarchy_panel.show(ui, &mut self.world, &mut self.selected_entity, &mut self.gizmo_system);
        self.console_messages.extend(messages);
    }
    
    fn show_hierarchy_object(&mut self, ui: &mut egui::Ui, object: &HierarchyObject) {
        match &object.children {
            Some(children) => {
                // Parent object with children
                ui.collapsing(&object.name, |ui| {
                    for child in children {
                        self.show_hierarchy_object(ui, child);
                    }
                });
            }
            None => {
                // Leaf object
                let selected = self.selected_object.as_ref() == Some(&object.name);
                if ui.selectable_label(selected, &object.name).clicked() {
                    self.selected_object = Some(object.name.clone());
                    self.console_messages.push(ConsoleMessage::info(&format!("🎯 Selected: {}", object.name)));
                }
            }
        }
    }
    
    fn show_inspector_panel(&mut self, ui: &mut egui::Ui) {
        // Delegate to the inspector panel module
        let messages = self.inspector_panel.show(ui, &mut self.world, self.selected_entity);
        self.console_messages.extend(messages);
        return;
    }
    
    fn show_scene_view(&mut self, ui: &mut egui::Ui) {
        // TODO: This entire massive function (1700+ lines!) needs to be moved to panels/scene_view/
        // For now, keeping it here to avoid breaking everything at once
        // Scene view toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.scene_view_active, true, "Scene");
            ui.selectable_value(&mut self.scene_view_active, false, "Game");
            
            ui.separator();
            
            if ui.button("🔍").on_hover_text("Focus on selected (F)").clicked() {
                self.focus_on_selected_object();
            }
        });
        
        ui.separator();
        
        // Main view area
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());
        
        // Draw background
        ui.painter().rect_filled(
            response.rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(35, 35, 35)
        );
        
        // Scene content
        ui.allocate_ui_at_rect(response.rect, |ui| {
            if self.scene_view_active {
                // Draw a simple 3D scene visualization
                self.draw_simple_scene_view(ui, response.rect);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Runtime game preview");
                        ui.small("Press Play to see game running");
                    });
                });
            }
        });
        
        // Handle keyboard shortcuts for scene view
        ui.input(|i| {
            // F key to focus on selected object
            if i.key_pressed(egui::Key::F) && self.selected_entity.is_some() {
                self.focus_on_selected_object();
            }
        });
        
        // Handle gizmo and scene interactions
        self.handle_scene_input(ui, &response, response.rect);
    }
    
    /// Get the color for a sprite based on its texture handle and tint
    fn get_sprite_color(&self, sprite: &Sprite) -> egui::Color32 {
        // Start with sprite tint color
        let mut final_color = sprite.color;
        
        // If sprite has a texture, blend with texture color
        if let Some(texture_handle) = sprite.texture_handle {
            if let Some(_texture) = self.texture_assets.get(&texture_handle) {
                // In a real implementation, we would sample the texture here
                // For now, we'll use the sprite's tint color directly
                // TODO: Implement proper texture sampling
            }
        }
        
        egui::Color32::from_rgba_unmultiplied(
            (final_color[0] * 255.0) as u8,
            (final_color[1] * 255.0) as u8,
            (final_color[2] * 255.0) as u8,
            (final_color[3] * 255.0) as u8,
        )
    }

    fn draw_simple_scene_view(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        
        // Get camera position and rotation for view transformation
        let camera_pos = self.scene_navigation.scene_camera_transform.position;
        let camera_rot = self.scene_navigation.scene_camera_transform.rotation;
        
        // Draw grid background
        let _grid_size = 50.0;
        let view_center = rect.center();
        
        // Apply camera offset to grid rendering
        let camera_offset_x = -camera_pos[0] * 50.0; // 50 pixels per world unit
        let camera_offset_y = camera_pos[2] * 50.0;  // Z becomes Y in screen space
        
        // Draw grid lines with camera offset
        painter.line_segment(
            [egui::pos2(rect.left(), view_center.y + camera_offset_y), 
             egui::pos2(rect.right(), view_center.y + camera_offset_y)],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
        painter.line_segment(
            [egui::pos2(view_center.x + camera_offset_x, rect.top()), 
             egui::pos2(view_center.x + camera_offset_x, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
        
        // Draw scene objects (simplified 2D representation with camera transform)
        // Phase 10.2: Query entities with both Transform AND Mesh components
        // This is the ECS-to-renderer bridge implementation
        let entities_with_transforms: Vec<_> = self.world.entities_with_component::<Transform>();
        
        // Track entity changes for Phase 10.2
        let current_entity_count = entities_with_transforms.len();
        if current_entity_count != self.last_rendered_entity_count {
            self.console_messages.push(ConsoleMessage::info(&format!(
                "🎮 Phase 10.2: Total entities with Transform: {}",
                current_entity_count
            )));
            self.last_rendered_entity_count = current_entity_count;
        }
        
        for entity in entities_with_transforms {
            let Some(transform) = self.world.get_component::<Transform>(entity) else {
                continue;
            };
            
            // Get object's rotation
            let obj_rot_x = transform.rotation[0].to_radians();
            let obj_rot_y = transform.rotation[1].to_radians();
            let obj_rot_z = transform.rotation[2].to_radians();
            
            // Apply camera transformation to object positions
            let relative_pos = [
                transform.position[0] - camera_pos[0],
                transform.position[1] - camera_pos[1], 
                transform.position[2] - camera_pos[2]
            ];
            
            // Apply camera rotation to the relative position
            let yaw = camera_rot[1];
            let pitch = camera_rot[0];
            
            // Rotate around Y-axis (yaw) first - only affects X and Z
            let cos_yaw = yaw.cos();
            let sin_yaw = yaw.sin();
            let rotated_x = relative_pos[0] * cos_yaw + relative_pos[2] * sin_yaw;
            let rotated_z = -relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
            
            // Apply pitch rotation (rotate around X-axis) - only affects Y and Z
            let cos_pitch = pitch.cos();
            let sin_pitch = pitch.sin();
            let final_y = relative_pos[1] * cos_pitch + rotated_z * sin_pitch;
            let final_z = -relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
            
            // Simple perspective projection
            // Use the final rotated Z for depth (after both rotations)
            let depth = final_z; // Distance from camera along view direction
            
            // Skip objects behind camera
            if depth <= 0.1 {
                continue;
            }
            
            // Perspective projection with field of view
            let fov_scale = 100.0; // Base scale for FOV
            let perspective_scale = fov_scale / depth;
            
            // Project 3D position to 2D screen coordinates with perspective
            let screen_x = view_center.x + (rotated_x * perspective_scale);
            let screen_y = view_center.y - (final_y * perspective_scale); // Y remains Y in screen space after rotation
            let screen_pos = egui::pos2(screen_x, screen_y);
            
            // Get entity info
            let name = self.world.get_component::<Name>(entity)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
            // Determine object type and rendering style
            let is_selected = self.selected_entity == Some(entity);
            
            if self.world.get_component::<Camera>(entity).is_some() {
                // Draw camera
                let size = 12.0;
                let color = if is_selected { egui::Color32::YELLOW } else { egui::Color32::BLUE };
                painter.circle_filled(screen_pos, size, color);
                painter.text(
                    screen_pos + egui::vec2(size + 5.0, -size),
                    egui::Align2::LEFT_CENTER,
                    format!("📷 {}", name),
                    egui::FontId::proportional(12.0),
                    color
                );
            } else if let Some(mesh) = self.world.get_component::<Mesh>(entity) {
                // Phase 10.2: Extract component data and convert to renderer-compatible format
                // Get material color if available
                let base_color = if let Some(material) = self.world.get_component::<Material>(entity) {
                    egui::Color32::from_rgba_unmultiplied(
                        (material.color[0] * 255.0) as u8,
                        (material.color[1] * 255.0) as u8,
                        (material.color[2] * 255.0) as u8,
                        (material.color[3] * 255.0) as u8,
                    )
                } else {
                    egui::Color32::from_rgb(180, 180, 180)
                };
                
                let color = if is_selected { egui::Color32::YELLOW } else { base_color };
                
                // Draw pseudo-3D representations based on mesh type
                match mesh.mesh_type {
                    MeshType::Cube => {
                        // Draw cube with pseudo-3D effect
                        let base_size = 20.0;
                        let size = base_size * transform.scale[0] * (perspective_scale / 2.0); // Scale with perspective
                        
                        // Define cube vertices in local space (before rotation)
                        let half_size = 0.5;
                        let vertices = [
                            // Front face vertices
                            [-half_size, -half_size,  half_size],
                            [ half_size, -half_size,  half_size],
                            [ half_size,  half_size,  half_size],
                            [-half_size,  half_size,  half_size],
                            // Back face vertices
                            [-half_size, -half_size, -half_size],
                            [ half_size, -half_size, -half_size],
                            [ half_size,  half_size, -half_size],
                            [-half_size,  half_size, -half_size],
                        ];
                        
                        // Apply object rotation to vertices
                        let mut rotated_vertices = [[0.0; 3]; 8];
                        for (i, vertex) in vertices.iter().enumerate() {
                            // Apply Y rotation (yaw)
                            let cos_y = obj_rot_y.cos();
                            let sin_y = obj_rot_y.sin();
                            let x1 = vertex[0] * cos_y - vertex[2] * sin_y;
                            let z1 = vertex[0] * sin_y + vertex[2] * cos_y;
                            
                            // Apply X rotation (pitch)
                            let cos_x = obj_rot_x.cos();
                            let sin_x = obj_rot_x.sin();
                            let y2 = vertex[1] * cos_x - z1 * sin_x;
                            let z2 = vertex[1] * sin_x + z1 * cos_x;
                            
                            // Apply Z rotation (roll)
                            let cos_z = obj_rot_z.cos();
                            let sin_z = obj_rot_z.sin();
                            let x3 = x1 * cos_z - y2 * sin_z;
                            let y3 = x1 * sin_z + y2 * cos_z;
                            
                            rotated_vertices[i] = [x3, y3, z2];
                        }
                        
                        // Transform vertices to camera view space for proper depth sorting and culling
                        let mut view_vertices = [[0.0; 3]; 8];
                        for (i, vertex) in rotated_vertices.iter().enumerate() {
                            // Apply camera transformations
                            // Camera yaw (rotate around Y axis)
                            let cam_cos_yaw = (-yaw).cos(); // Negative because camera rotation is inverse
                            let cam_sin_yaw = (-yaw).sin();
                            let view_x = vertex[0] * cam_cos_yaw - vertex[2] * cam_sin_yaw;
                            let view_z = vertex[0] * cam_sin_yaw + vertex[2] * cam_cos_yaw;
                            
                            // Camera pitch (rotate around X axis)
                            let cam_cos_pitch = (-pitch).cos(); // Negative because camera rotation is inverse
                            let cam_sin_pitch = (-pitch).sin();
                            let view_y = vertex[1] * cam_cos_pitch - view_z * cam_sin_pitch;
                            let final_view_z = vertex[1] * cam_sin_pitch + view_z * cam_cos_pitch;
                            
                            view_vertices[i] = [view_x, view_y, final_view_z];
                        }
                        
                        // Project vertices to screen and find visible faces
                        let mut screen_vertices = [egui::Pos2::ZERO; 8];
                        for (i, vertex) in rotated_vertices.iter().enumerate() {
                            // Apply perspective to rotated vertices
                            let proj_x = vertex[0] * size;
                            let proj_y = -vertex[1] * size; // Flip Y for screen coordinates
                            screen_vertices[i] = screen_pos + egui::vec2(proj_x, proj_y);
                        }
                        
                        // Define faces with correct winding order (counter-clockwise when viewed from outside)
                        let faces = [
                            // Front face (+Z) - vertices 0,1,2,3
                            ([0, 1, 2, 3], color),
                            // Back face (-Z) - vertices 4,7,6,5
                            ([4, 7, 6, 5], color.gamma_multiply(0.6)),
                            // Top face (+Y) - vertices 3,2,6,7
                            ([3, 2, 6, 7], egui::Color32::from_rgba_unmultiplied(
                                ((color.r() as f32 * 1.2).min(255.0)) as u8,
                                ((color.g() as f32 * 1.2).min(255.0)) as u8,
                                ((color.b() as f32 * 1.2).min(255.0)) as u8,
                                color.a(),
                            )),
                            // Bottom face (-Y) - vertices 4,5,1,0
                            ([4, 5, 1, 0], color.gamma_multiply(0.5)),
                            // Right face (+X) - vertices 1,5,6,2
                            ([1, 5, 6, 2], color.gamma_multiply(0.8)),
                            // Left face (-X) - vertices 0,3,7,4
                            ([0, 3, 7, 4], color.gamma_multiply(0.7)),
                        ];
                        
                        // Draw faces sorted by average Z depth (painter's algorithm)
                        let mut face_depths: Vec<(usize, f32)> = faces.iter().enumerate().map(|(i, (indices, _))| {
                            // Use view-space Z for depth sorting
                            let avg_z = indices.iter()
                                .map(|&idx| view_vertices[idx][2])
                                .sum::<f32>() / 4.0;
                            (i, avg_z)
                        }).collect();
                        
                        // Sort faces by depth (back to front)
                        face_depths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                        
                        // Draw faces in order
                        for (face_idx, _) in face_depths {
                            let (indices, face_color) = &faces[face_idx];
                            
                            // Simple backface culling based on winding order in screen space
                            // Calculate screen space area to determine if face is front or back facing
                            let v0 = screen_vertices[indices[0]];
                            let v1 = screen_vertices[indices[1]];
                            let v2 = screen_vertices[indices[2]];
                            
                            // 2D cross product for winding order
                            let area = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);
                            
                            // Counter-clockwise winding = positive area = front facing
                            if area > 0.0 {
                                let face_points: Vec<egui::Pos2> = indices.iter()
                                    .map(|&idx| screen_vertices[idx])
                                    .collect();
                                
                                painter.add(egui::Shape::convex_polygon(
                                    face_points,
                                    *face_color,
                                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60))
                                ));
                            }
                        }
                        
                        painter.text(
                            screen_pos + egui::vec2(size * 0.7 + 10.0, -size * 0.7),
                            egui::Align2::LEFT_CENTER,
                            format!("⬜ {}", name),
                            egui::FontId::proportional(12.0),
                            color
                        );
                    }
                    MeshType::Sphere => {
                        // Draw sphere with shading
                        let base_radius = 15.0;
                        let radius = base_radius * transform.scale[0] * (perspective_scale / 2.0); // Scale with perspective
                        
                        // Main sphere
                        painter.circle_filled(screen_pos, radius, color);
                        
                        // Draw rotation indicator (band around sphere)
                        if obj_rot_x.abs() > 0.1 || obj_rot_y.abs() > 0.1 || obj_rot_z.abs() > 0.1 {
                            // Calculate band position based on rotation
                            let band_y_offset = (obj_rot_x.sin() * radius * 0.3).abs();
                            let band_width = radius * 2.0 * obj_rot_y.cos().abs().max(0.2);
                            let band_height = radius * 0.2;
                            
                            let band_rect = egui::Rect::from_center_size(
                                screen_pos + egui::vec2(0.0, band_y_offset),
                                egui::vec2(band_width, band_height)
                            );
                            painter.rect_filled(
                                band_rect,
                                egui::Rounding::same(band_height / 2.0),
                                color.gamma_multiply(0.7)
                            );
                        }
                        
                        // Highlight for 3D effect
                        let highlight_angle = obj_rot_y - std::f32::consts::PI / 4.0;
                        let highlight_pos = screen_pos + egui::vec2(
                            highlight_angle.cos() * radius * 0.3,
                            -radius * 0.3
                        );
                        painter.circle_filled(
                            highlight_pos,
                            radius * 0.3,
                            egui::Color32::from_rgba_unmultiplied(
                                ((base_color.r() as f32 * 1.5).min(255.0)) as u8,
                                ((base_color.g() as f32 * 1.5).min(255.0)) as u8,
                                ((base_color.b() as f32 * 1.5).min(255.0)) as u8,
                                base_color.a(),
                            )
                        );
                        
                        // Shadow
                        painter.circle_stroke(
                            screen_pos,
                            radius,
                            egui::Stroke::new(2.0, color.gamma_multiply(0.6))
                        );
                        
                        painter.text(
                            screen_pos + egui::vec2(radius + 5.0, -radius),
                            egui::Align2::LEFT_CENTER,
                            format!("⚫ {}", name),
                            egui::FontId::proportional(12.0),
                            color
                        );
                    }
                    MeshType::Plane => {
                        // Draw plane with rotation
                        let base_width = 30.0;
                        let base_height = 30.0; // Make it square for better rotation visibility
                        let size = base_width * transform.scale[0] * (perspective_scale / 2.0);
                        
                        // Define plane corners in local space
                        let half_size = 0.5;
                        let corners = [
                            [-half_size, 0.0, -half_size],
                            [ half_size, 0.0, -half_size],
                            [ half_size, 0.0,  half_size],
                            [-half_size, 0.0,  half_size],
                        ];
                        
                        // Apply object rotation to corners
                        let mut rotated_corners = [[0.0; 3]; 4];
                        for (i, corner) in corners.iter().enumerate() {
                            // Apply Y rotation (yaw)
                            let cos_y = obj_rot_y.cos();
                            let sin_y = obj_rot_y.sin();
                            let x1 = corner[0] * cos_y - corner[2] * sin_y;
                            let z1 = corner[0] * sin_y + corner[2] * cos_y;
                            
                            // Apply X rotation (pitch)
                            let cos_x = obj_rot_x.cos();
                            let sin_x = obj_rot_x.sin();
                            let y2 = corner[1] * cos_x - z1 * sin_x;
                            let z2 = corner[1] * sin_x + z1 * cos_x;
                            
                            // Apply Z rotation (roll)
                            let cos_z = obj_rot_z.cos();
                            let sin_z = obj_rot_z.sin();
                            let x3 = x1 * cos_z - y2 * sin_z;
                            let y3 = x1 * sin_z + y2 * cos_z;
                            
                            rotated_corners[i] = [x3, y3, z2];
                        }
                        
                        // Project corners to screen
                        let screen_corners: Vec<egui::Pos2> = rotated_corners.iter()
                            .map(|corner| {
                                let proj_x = corner[0] * size;
                                let proj_y = -corner[1] * size; // Flip Y for screen
                                screen_pos + egui::vec2(proj_x, proj_y)
                            })
                            .collect();
                        
                        // Draw the plane as a polygon
                        painter.add(egui::Shape::convex_polygon(
                            screen_corners.clone(),
                            color,
                            egui::Stroke::new(1.0, color.gamma_multiply(0.6))
                        ));
                        
                        // Draw grid lines on the plane for better rotation visibility
                        if rotated_corners[0][1].abs() < 0.9 { // Only if not too edge-on
                            // Draw cross lines
                            painter.line_segment(
                                [(screen_corners[0] + screen_corners[2].to_vec2()) / 2.0,
                                 (screen_corners[1] + screen_corners[3].to_vec2()) / 2.0],
                                egui::Stroke::new(1.0, color.gamma_multiply(0.4))
                            );
                            painter.line_segment(
                                [(screen_corners[0] + screen_corners[1].to_vec2()) / 2.0,
                                 (screen_corners[2] + screen_corners[3].to_vec2()) / 2.0],
                                egui::Stroke::new(1.0, color.gamma_multiply(0.4))
                            );
                        }
                        
                        painter.text(
                            screen_pos + egui::vec2(size * 0.7 + 5.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            format!("▭ {}", name),
                            egui::FontId::proportional(12.0),
                            color
                        );
                    }
                    _ => {
                        // Default mesh representation
                        painter.circle_filled(screen_pos, 10.0, color);
                        painter.text(
                            screen_pos + egui::vec2(15.0, -10.0),
                            egui::Align2::LEFT_CENTER,
                            format!("📦 {}", name),
                            egui::FontId::proportional(12.0),
                            color
                        );
                    }
                }
            } else {
                // Default object
                let color = if is_selected { egui::Color32::YELLOW } else { egui::Color32::from_rgb(150, 150, 150) };
                painter.circle_filled(screen_pos, 8.0, color);
                painter.text(
                    screen_pos + egui::vec2(10.0, -8.0),
                    egui::Align2::LEFT_CENTER,
                    format!("📍 {}", name),
                    egui::FontId::proportional(12.0),
                    color
                );
            }
        }
        
        // Draw sprite objects - using safe iteration approach
        for (entity, _transform) in self.world.query::<Transform>() {
            // Check if this entity has a SpriteRenderer component
            if let Some(sprite_renderer) = self.world.get_component::<SpriteRenderer>(entity) {
                let transform = _transform; // We already have transform from the query
                
                // Apply camera transformation to sprite positions
                let relative_pos = [
                    transform.position[0] - camera_pos[0],
                    transform.position[1] - camera_pos[1], 
                    transform.position[2] - camera_pos[2]
                ];
                
                // Apply camera rotation to the relative position (same as regular objects)
                let yaw = camera_rot[1];
                let pitch = camera_rot[0];
                
                // Rotate around Y-axis (yaw) - only affects X and Z
                let cos_yaw = yaw.cos();
                let sin_yaw = yaw.sin();
                let rotated_x = relative_pos[0] * cos_yaw + relative_pos[2] * sin_yaw;
                let rotated_z = -relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
                
                // Apply pitch rotation (rotate around X-axis) - only affects Y and Z
                let cos_pitch = pitch.cos();
                let sin_pitch = pitch.sin();
                let final_y = relative_pos[1] * cos_pitch + rotated_z * sin_pitch;
                let final_z = -relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
                
                // Simple perspective projection for sprites
                let depth = final_z;
                
                // Skip sprites behind camera
                if depth <= 0.1 {
                    continue;
                }
                
                // Perspective projection with field of view
                let fov_scale = 100.0; // Base scale for FOV
                let perspective_scale = fov_scale / depth;
                
                // Project sprite position to 2D screen coordinates with perspective
                let screen_x = view_center.x + (rotated_x * perspective_scale);
                let screen_y = view_center.y - (final_y * perspective_scale); // Y remains Y in screen space after rotation
                let screen_pos = egui::pos2(screen_x, screen_y);
                
                // Calculate sprite size in screen space with perspective
                let world_scale = (transform.scale[0] + transform.scale[1]) * 0.5; // Average X and Y scale
                let base_size = 32.0;
                let sprite_size = egui::vec2(
                    base_size * world_scale * (perspective_scale / 2.0), 
                    base_size * world_scale * (perspective_scale / 2.0)
                );
                
                // Get sprite color - simplified for stability
                let sprite_color = egui::Color32::from_rgba_unmultiplied(
                    (sprite_renderer.sprite.color[0] * 255.0) as u8,
                    (sprite_renderer.sprite.color[1] * 255.0) as u8,
                    (sprite_renderer.sprite.color[2] * 255.0) as u8,
                    (sprite_renderer.sprite.color[3] * 255.0) as u8,
                );
                
                // Highlight if selected
                let is_selected = self.selected_entity == Some(entity);
                let final_color = if is_selected {
                    egui::Color32::YELLOW
                } else {
                    sprite_color
                };
                
                // Draw sprite as a rectangle
                let sprite_rect = egui::Rect::from_center_size(screen_pos, sprite_size);
                painter.rect_filled(sprite_rect, egui::Rounding::same(2.0), final_color);
                
                // Draw sprite border
                painter.rect_stroke(
                    sprite_rect,
                    egui::Rounding::same(2.0),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100))
                );
                
                // Draw selection outline
                if is_selected {
                    painter.rect_stroke(
                        sprite_rect,
                        egui::Rounding::same(2.0),
                        egui::Stroke::new(3.0, egui::Color32::WHITE)
                    );
                }
                
                // Draw sprite icon and name
                let name = self.world.get_component::<Name>(entity)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("Sprite {}", entity.id()));
                
                painter.text(
                    screen_pos + egui::vec2(sprite_size.x * 0.5 + 5.0, -sprite_size.y * 0.5),
                    egui::Align2::LEFT_CENTER,
                    format!("🖼️ {}", name),
                    egui::FontId::proportional(12.0),
                    final_color
                );
            }
        }
        
        // Draw scene camera position indicator (showing where the navigation camera is)
        painter.circle_filled(
            view_center, 
            8.0, 
            egui::Color32::from_rgba_unmultiplied(255, 255, 0, 200)
        );
        painter.text(
            view_center + egui::vec2(12.0, -8.0),
            egui::Align2::LEFT_CENTER,
            format!("📷 Scene Camera [{:.1}, {:.1}, {:.1}]", camera_pos[0], camera_pos[1], camera_pos[2]),
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE
        );
        
        // Draw camera view indicator for selected entities
        if let Some(camera_entity) = self.selected_entity {
            if self.world.get_component::<Camera>(camera_entity).is_some() {
                if let Some(camera_transform) = self.world.get_component::<Transform>(camera_entity) {
                    // Apply camera transformation to entity camera positions too
                    let relative_cam_pos = [
                        camera_transform.position[0] - camera_pos[0],
                        camera_transform.position[1] - camera_pos[1], 
                        camera_transform.position[2] - camera_pos[2]
                    ];
                    
                    let scale = 50.0;
                    let camera_screen_x = view_center.x + relative_cam_pos[0] * scale;
                    let camera_screen_y = view_center.y - relative_cam_pos[2] * scale;
                    let entity_camera_pos = egui::pos2(camera_screen_x, camera_screen_y);
                    
                    // Draw camera view frustum (simplified)
                    let frustum_width = 80.0;
                    let frustum_height = 60.0;
                    let frustum_rect = egui::Rect::from_center_size(
                        entity_camera_pos + egui::vec2(40.0, 0.0),
                        egui::vec2(frustum_width, frustum_height)
                    );
                    
                    painter.rect_stroke(
                        frustum_rect,
                        egui::Rounding::ZERO,
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(0, 255, 255, 150))
                    );
                    
                    // Draw view direction line
                    painter.line_segment(
                        [entity_camera_pos, frustum_rect.left_center()],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255))
                    );
                }
            }
        }
        
        // Draw gizmos for selected entity
        if self.gizmo_system.get_active_tool() == SceneTool::Move {
            if let Some(selected_entity) = self.selected_entity {
                if let Some(transform) = self.world.get_component::<Transform>(selected_entity) {
                    self.render_move_gizmo(ui, rect, view_center, transform.position);
                }
            }
        }
        
        // Draw info overlay with play state information
        ui.allocate_ui_at_rect(egui::Rect::from_min_size(rect.min, egui::vec2(350.0, 120.0)), |ui| {
            ui.vertical(|ui| {
                // Play state indicator
                match self.play_state {
                    PlayState::Editing => {
                        ui.label("🎨 Scene View (Editor Mode)");
                    }
                    PlayState::Playing => {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), 
                            format!("▶️ Scene View (Playing - {:.1}s)", self.get_play_time()));
                        ui.small(format!("Delta: {:.3}ms", self.delta_time * 1000.0));
                    }
                    PlayState::Paused => {
                        ui.colored_label(egui::Color32::from_rgb(200, 200, 100), 
                            format!("⏸️ Scene View (Paused - {:.1}s)", self.get_play_time()));
                    }
                };
                
                ui.label(format!("📦 {} objects", self.world.entity_count()));
                if let Some(entity) = self.selected_entity {
                    if let Some(transform) = self.world.get_component::<Transform>(entity) {
                        ui.label(format!("📍 Selected: {:.1}, {:.1}, {:.1}", 
                            transform.position[0], transform.position[1], transform.position[2]));
                    }
                }
                
                match self.play_state {
                    PlayState::Editing => ui.small("Click objects to select • Drag to orbit camera"),
                    PlayState::Playing => ui.small("Game running • Properties locked in Inspector"),
                    PlayState::Paused => ui.small("Game paused • Limited editing available"),
                };
            });
        });
    }
    
    /// Render the move gizmo for the selected entity
    fn render_move_gizmo(&mut self, ui: &mut egui::Ui, rect: egui::Rect, scene_center: egui::Pos2, world_position: [f32; 3]) {
        let painter = ui.painter();
        
        // Apply camera transformation to gizmo position
        let camera_pos = self.scene_navigation.scene_camera_transform.position;
        let relative_pos = [
            world_position[0] - camera_pos[0],
            world_position[1] - camera_pos[1], 
            world_position[2] - camera_pos[2]
        ];
        
        // Convert world position to screen coordinates (simple orthographic projection)
        let scale = 50.0; // Same scale as scene objects
        let gizmo_screen_x = scene_center.x + relative_pos[0] * scale;
        let gizmo_screen_y = scene_center.y - relative_pos[2] * scale; // Z becomes Y in screen space
        let gizmo_center = egui::pos2(gizmo_screen_x, gizmo_screen_y);
        
        // Check if gizmo is within screen bounds
        if !rect.contains(gizmo_center) {
            return;
        }
        
        // Gizmo size - scale based on distance for consistent screen size
        let base_size = 40.0;
        let arrow_length = base_size;
        let arrow_width = 4.0;
        let arrow_head_size = 8.0;
        
        // Get current interaction state
        let hovered_component = self.gizmo_system.get_move_gizmo()
            .and_then(|gizmo| gizmo.get_hovered_component());
        
        // Define Unity standard colors
        let x_color = egui::Color32::from_rgb(255, 0, 0);   // Red for X-axis
        let y_color = egui::Color32::from_rgb(0, 255, 0);   // Green for Y-axis  
        let z_color = egui::Color32::from_rgb(0, 0, 255);   // Blue for Z-axis
        let highlight_color = egui::Color32::from_rgb(255, 255, 0); // Yellow for selection
        
        // X-Axis Arrow (Red - Right direction)
        let x_axis_color = if matches!(hovered_component, Some(GizmoComponent::Axis(GizmoAxis::X))) {
            highlight_color
        } else {
            x_color
        };
        
        let x_start = gizmo_center;
        let x_end = egui::pos2(gizmo_center.x + arrow_length, gizmo_center.y);
        
        // Draw X-axis arrow shaft
        painter.line_segment(
            [x_start, x_end],
            egui::Stroke::new(arrow_width, x_axis_color)
        );
        
        // Draw X-axis arrow head
        let x_head_points = [
            egui::pos2(x_end.x, x_end.y),
            egui::pos2(x_end.x - arrow_head_size, x_end.y - arrow_head_size/2.0),
            egui::pos2(x_end.x - arrow_head_size, x_end.y + arrow_head_size/2.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            x_head_points.to_vec(),
            x_axis_color,
            egui::Stroke::NONE
        ));
        
        // Y-Axis Arrow (Green - Up direction)
        let y_axis_color = if matches!(hovered_component, Some(GizmoComponent::Axis(GizmoAxis::Y))) {
            highlight_color
        } else {
            y_color
        };
        
        let y_start = gizmo_center;
        let y_end = egui::pos2(gizmo_center.x, gizmo_center.y - arrow_length);
        
        // Draw Y-axis arrow shaft
        painter.line_segment(
            [y_start, y_end],
            egui::Stroke::new(arrow_width, y_axis_color)
        );
        
        // Draw Y-axis arrow head
        let y_head_points = [
            egui::pos2(y_end.x, y_end.y),
            egui::pos2(y_end.x - arrow_head_size/2.0, y_end.y + arrow_head_size),
            egui::pos2(y_end.x + arrow_head_size/2.0, y_end.y + arrow_head_size),
        ];
        painter.add(egui::Shape::convex_polygon(
            y_head_points.to_vec(),
            y_axis_color,
            egui::Stroke::NONE
        ));
        
        // Z-Axis Arrow (Blue - Forward direction, diagonal up-right in 2D view)
        let z_axis_color = if matches!(hovered_component, Some(GizmoComponent::Axis(GizmoAxis::Z))) {
            highlight_color
        } else {
            z_color
        };
        
        let z_offset_x = arrow_length * 0.7; // Diagonal direction for Z
        let z_offset_y = -arrow_length * 0.7;
        let z_start = gizmo_center;
        let z_end = egui::pos2(gizmo_center.x + z_offset_x, gizmo_center.y + z_offset_y);
        
        // Draw Z-axis arrow shaft
        painter.line_segment(
            [z_start, z_end],
            egui::Stroke::new(arrow_width, z_axis_color)
        );
        
        // Draw Z-axis arrow head
        let z_head_angle = std::f32::consts::PI / 4.0; // 45 degrees
        let z_head_points = [
            egui::pos2(z_end.x, z_end.y),
            egui::pos2(z_end.x - arrow_head_size * z_head_angle.cos(), z_end.y - arrow_head_size * z_head_angle.sin()),
            egui::pos2(z_end.x - arrow_head_size * (-z_head_angle).cos(), z_end.y - arrow_head_size * (-z_head_angle).sin()),
        ];
        painter.add(egui::Shape::convex_polygon(
            z_head_points.to_vec(),
            z_axis_color,
            egui::Stroke::NONE
        ));
        
        // Draw planar movement squares
        let square_size = 12.0;
        let square_offset = arrow_length * 0.3;
        
        // XY Plane (Blue square - Z locked)
        let xy_plane_color = if matches!(hovered_component, Some(GizmoComponent::Plane(GizmoPlane::XY))) {
            highlight_color
        } else {
            z_color.gamma_multiply(0.6) // Dimmed blue for plane
        };
        
        let xy_square_center = egui::pos2(gizmo_center.x + square_offset, gizmo_center.y - square_offset);
        let xy_square = egui::Rect::from_center_size(xy_square_center, egui::vec2(square_size, square_size));
        painter.rect_filled(xy_square, egui::Rounding::same(2.0), xy_plane_color);
        
        // XZ Plane (Green square - Y locked)
        let xz_plane_color = if matches!(hovered_component, Some(GizmoComponent::Plane(GizmoPlane::XZ))) {
            highlight_color
        } else {
            y_color.gamma_multiply(0.6) // Dimmed green for plane
        };
        
        let xz_square_center = egui::pos2(gizmo_center.x + square_offset, gizmo_center.y + square_offset * 0.5);
        let xz_square = egui::Rect::from_center_size(xz_square_center, egui::vec2(square_size, square_size));
        painter.rect_filled(xz_square, egui::Rounding::same(2.0), xz_plane_color);
        
        // YZ Plane (Red square - X locked)  
        let yz_plane_color = if matches!(hovered_component, Some(GizmoComponent::Plane(GizmoPlane::YZ))) {
            highlight_color
        } else {
            x_color.gamma_multiply(0.6) // Dimmed red for plane
        };
        
        let yz_square_center = egui::pos2(gizmo_center.x - square_offset * 0.5, gizmo_center.y - square_offset);
        let yz_square = egui::Rect::from_center_size(yz_square_center, egui::vec2(square_size, square_size));
        painter.rect_filled(yz_square, egui::Rounding::same(2.0), yz_plane_color);
        
        // Center handle for screen-space movement
        let center_color = if matches!(hovered_component, Some(GizmoComponent::Center)) {
            highlight_color
        } else {
            egui::Color32::WHITE
        };
        
        painter.circle_filled(gizmo_center, 6.0, center_color);
        painter.circle_stroke(gizmo_center, 6.0, egui::Stroke::new(2.0, egui::Color32::BLACK));
        
        // Draw axis labels
        painter.text(
            egui::pos2(x_end.x + 10.0, x_end.y),
            egui::Align2::LEFT_CENTER,
            "X",
            egui::FontId::proportional(14.0),
            x_axis_color
        );
        
        painter.text(
            egui::pos2(y_end.x, y_end.y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            "Y",
            egui::FontId::proportional(14.0),
            y_axis_color
        );
        
        painter.text(
            egui::pos2(z_end.x + 5.0, z_end.y - 5.0),
            egui::Align2::LEFT_BOTTOM,
            "Z",
            egui::FontId::proportional(14.0),
            z_axis_color
        );
    }
    
    /// Handle scene navigation input (right mouse + WASD for Unity/Unreal style navigation)
    fn handle_scene_navigation(&mut self, ui: &egui::Ui, response: &egui::Response, rect: egui::Rect) {
        // Check for right mouse button to start navigation
        let (rmb_down, pointer_pos, pointer_delta) = ui.input(|i| {
            (i.pointer.secondary_down(), 
             i.pointer.hover_pos(),
             i.pointer.delta())
        });
        
        // Only handle navigation if mouse is within the scene view
        let mouse_in_rect = pointer_pos.map_or(false, |pos| rect.contains(pos));
        
        // Start navigation if RMB is pressed (not just down) and mouse is in rect
        if ui.input(|i| i.pointer.secondary_pressed()) && mouse_in_rect && !self.scene_navigation.is_navigating {
            if let Some(mouse_pos) = pointer_pos {
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 Starting navigation at ({:.1}, {:.1})", mouse_pos.x, mouse_pos.y
                )));
                self.start_scene_navigation(mouse_pos);
                
                // Capture mouse to prevent interference from other UI elements
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            }
        }
        
        // Check for right mouse button release
        if !rmb_down && self.scene_navigation.is_navigating {
            self.end_scene_navigation();
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }
        
        // Handle navigation input during active navigation
        if self.scene_navigation.is_navigating && rmb_down {
            // Use pointer delta for more accurate mouse movement tracking
            if pointer_delta != egui::Vec2::ZERO {
                self.apply_mouse_look(pointer_delta);
            }
            
            // Handle WASD movement
            self.handle_wasd_movement(ui);
        }
        
        // Handle scroll wheel for speed adjustment (even when not actively navigating)
        self.handle_navigation_speed_control(ui);
    }
    
    /// Start scene navigation mode
    fn start_scene_navigation(&mut self, mouse_pos: egui::Pos2) {
        if !self.scene_navigation.enabled {
            return;
        }
        
        self.scene_navigation.is_navigating = true;
        self.scene_navigation.last_mouse_pos = Some(mouse_pos);
        
        self.console_messages.push(ConsoleMessage::info(&format!(
            "🎮 Started scene navigation at ({:.1}, {:.1})", 
            mouse_pos.x, mouse_pos.y
        )));
    }
    
    /// End scene navigation mode
    fn end_scene_navigation(&mut self) {
        self.scene_navigation.is_navigating = false;
        self.scene_navigation.last_mouse_pos = None;
        
        self.console_messages.push(ConsoleMessage::info("🎮 Ended scene navigation"));
    }
    
    
    /// Apply mouse movement to camera rotation
    fn apply_mouse_look(&mut self, mouse_delta: egui::Vec2) {
        let sensitivity = self.scene_navigation.rotation_sensitivity;
        
        // Horizontal rotation (Y-axis - yaw)
        let yaw_delta = -mouse_delta.x * sensitivity;
        
        // Vertical rotation (X-axis - pitch)  
        let pitch_delta = -mouse_delta.y * sensitivity;
        
        // Apply rotations to scene camera - no clamping for scene editing
        self.scene_navigation.scene_camera_transform.rotation[1] += yaw_delta;
        self.scene_navigation.scene_camera_transform.rotation[0] += pitch_delta;
    }
    
    /// Handle WASD movement input during navigation mode
    fn handle_wasd_movement(&mut self, ui: &egui::Ui) {
        let movement_speed = self.scene_navigation.movement_speed;
        let delta_time = self.delta_time;
        let fast_multiplier = if ui.input(|i| i.modifiers.shift) {
            self.scene_navigation.fast_movement_multiplier
        } else {
            1.0
        };
        
        let actual_speed = movement_speed * delta_time * fast_multiplier;
        let mut movement = [0.0, 0.0, 0.0];
        let mut any_movement = false;
        
        
        
        // Check WASD keys
        ui.input(|i| {
            if i.key_down(egui::Key::W) {
                movement[2] -= actual_speed; // Forward (negative Z)
                any_movement = true;
            }
            if i.key_down(egui::Key::S) {
                movement[2] += actual_speed; // Backward (positive Z)
                any_movement = true;
            }
            if i.key_down(egui::Key::A) {
                movement[0] -= actual_speed; // Left (negative X)
                any_movement = true;
            }
            if i.key_down(egui::Key::D) {
                movement[0] += actual_speed; // Right (positive X)
                any_movement = true;
            }
            if i.key_down(egui::Key::Q) {
                movement[1] -= actual_speed; // Down (negative Y)
                any_movement = true;
            }
            if i.key_down(egui::Key::E) {
                movement[1] += actual_speed; // Up (positive Y)
                any_movement = true;
            }
        });
        
        if any_movement {
            // Transform movement relative to camera orientation
            let transformed_movement = self.transform_movement_by_camera(movement);
            
            // Apply movement to camera position
            self.scene_navigation.scene_camera_transform.position[0] += transformed_movement[0];
            self.scene_navigation.scene_camera_transform.position[1] += transformed_movement[1];
            self.scene_navigation.scene_camera_transform.position[2] += transformed_movement[2];
            
            // Log camera position occasionally (every 10th frame or significant change)
            static mut MOVEMENT_COUNTER: u32 = 0;
            unsafe {
                MOVEMENT_COUNTER += 1;
                if MOVEMENT_COUNTER % 10 == 0 {
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🎮 Camera moved to [{:.1}, {:.1}, {:.1}] delta:{:.3}{}",
                        self.scene_navigation.scene_camera_transform.position[0],
                        self.scene_navigation.scene_camera_transform.position[1],
                        self.scene_navigation.scene_camera_transform.position[2],
                        delta_time,
                        if fast_multiplier > 1.0 { " (FAST)" } else { "" }
                    )));
                }
            }
        }
    }
    
    /// Transform movement vector by camera orientation for relative movement
    fn transform_movement_by_camera(&self, movement: [f32; 3]) -> [f32; 3] {
        let rotation = &self.scene_navigation.scene_camera_transform.rotation;
        let pitch = rotation[0]; // X-axis rotation (up/down)
        let yaw = rotation[1]; // Y-axis rotation (left/right)
        
        // Calculate forward and right vectors based on camera rotation
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        
        // Forward vector (camera's -Z direction in world space, affected by pitch)
        let forward = [
            -sin_yaw * cos_pitch,
            sin_pitch,
            -cos_yaw * cos_pitch,
        ];
        
        // Right vector (camera's +X direction in world space, not affected by pitch)
        let right = [cos_yaw, 0.0, -sin_yaw];
        
        // Up vector (always world up for now)
        let up = [0.0, 1.0, 0.0];
        
        // Transform movement vector
        [
            movement[0] * right[0] + movement[1] * up[0] + movement[2] * forward[0],
            movement[0] * right[1] + movement[1] * up[1] + movement[2] * forward[1],
            movement[0] * right[2] + movement[1] * up[2] + movement[2] * forward[2],
        ]
    }
    
    /// Focus the scene camera on the selected object
    fn focus_on_selected_object(&mut self) {
        if let Some(selected_entity) = self.selected_entity {
            if let Some(transform) = self.world.get_component::<Transform>(selected_entity) {
                // Get object position
                let object_pos = transform.position;
                
                // Calculate a good viewing distance based on object scale
                let avg_scale = (transform.scale[0] + transform.scale[1] + transform.scale[2]) / 3.0;
                let view_distance = avg_scale * 5.0 + 3.0; // Base distance of 3 units plus scale factor
                
                // Get current camera rotation to maintain viewing angle
                let camera_rot = self.scene_navigation.scene_camera_transform.rotation;
                let pitch = camera_rot[0];
                let yaw = camera_rot[1];
                
                // Calculate camera position offset from object
                let cos_yaw = yaw.cos();
                let sin_yaw = yaw.sin();
                let cos_pitch = pitch.cos();
                let sin_pitch = pitch.sin();
                
                // Camera looks along -Z, so we position it behind the object along its forward vector
                let camera_offset = [
                    sin_yaw * cos_pitch * view_distance,
                    -sin_pitch * view_distance,
                    cos_yaw * cos_pitch * view_distance,
                ];
                
                // Set new camera position
                self.scene_navigation.scene_camera_transform.position = [
                    object_pos[0] + camera_offset[0],
                    object_pos[1] + camera_offset[1],
                    object_pos[2] + camera_offset[2],
                ];
                
                // Get object name for logging
                let name = self.world.get_component::<Name>(selected_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| format!("Entity {}", selected_entity.id()));
                
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🔍 Focused on {} at [{:.1}, {:.1}, {:.1}] (distance: {:.1})",
                    name, object_pos[0], object_pos[1], object_pos[2], view_distance
                )));
            } else {
                self.console_messages.push(ConsoleMessage::info("⚠️ Selected entity has no transform"));
            }
        } else {
            self.console_messages.push(ConsoleMessage::info("⚠️ No object selected to focus on"));
        }
    }
    
    /// Handle mouse wheel for navigation speed control
    fn handle_navigation_speed_control(&mut self, ui: &egui::Ui) {
        ui.input(|i| {
            let scroll_delta = i.raw_scroll_delta.y;
            if scroll_delta != 0.0 && self.scene_navigation.enabled {
                let speed_adjustment = scroll_delta * 0.1; // Sensitivity adjustment
                let old_speed = self.scene_navigation.movement_speed;
                
                // Adjust speed with limits
                self.scene_navigation.movement_speed = 
                    (self.scene_navigation.movement_speed + speed_adjustment).clamp(0.5, 50.0);
                
                if old_speed != self.scene_navigation.movement_speed {
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🎮 Navigation speed: {:.1} units/sec {}",
                        self.scene_navigation.movement_speed,
                        if scroll_delta > 0.0 { "📈" } else { "📉" }
                    )));
                }
            }
        });
    }
    
    /// Handle mouse input for scene view including navigation and gizmo interactions
    fn handle_scene_input(&mut self, ui: &egui::Ui, response: &egui::Response, rect: egui::Rect) {
        // DEBUG: Log basic input state
        if let Some(mouse_pos) = response.hover_pos() {
            // Only log occasionally to avoid spam
            if response.clicked() || response.dragged() || response.drag_stopped() || response.secondary_clicked() {
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🖱️ Mouse input: pos=({:.1}, {:.1}), clicked={}, dragged={}, drag_stopped={}, secondary_clicked={}",
                    mouse_pos.x, mouse_pos.y, response.clicked(), response.dragged(), response.drag_stopped(), response.secondary_clicked()
                )));
            }
        }
        
        // Handle scene navigation (right mouse button + WASD)
        self.handle_scene_navigation(ui, response, rect);
        
        // Skip gizmo interaction if we're in navigation mode
        if self.scene_navigation.is_navigating {
            return;
        }
        
        // Only handle input if we have a selected entity and move tool is active
        if self.gizmo_system.get_active_tool() != SceneTool::Move {
            // Only log if there's actual input
            if response.clicked() || response.dragged() {
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🔧 Not handling input: tool={:?}", self.gizmo_system.get_active_tool()
                )));
            }
            return;
        }
        
        let Some(selected_entity) = self.selected_entity else {
            if response.clicked() || response.dragged() {
                self.console_messages.push(ConsoleMessage::info("🔧 No selected entity"));
            }
            return;
        };
        
        let Some(transform) = self.world.get_component::<Transform>(selected_entity).cloned() else {
            if response.clicked() || response.dragged() {
                self.console_messages.push(ConsoleMessage::info("🔧 No transform component on selected entity"));
            }
            return;
        };
        
        // Only log when we're actually processing input
        if response.clicked() || response.dragged() || response.drag_stopped() {
            self.console_messages.push(ConsoleMessage::info(&format!(
                "🔧 Processing input for entity {} at position [{:.2}, {:.2}, {:.2}]",
                selected_entity.id(), transform.position[0], transform.position[1], transform.position[2]
            )));
        }
        
        // Calculate scene center and gizmo position
        let scene_center = rect.center();
        let scale = 50.0;
        let gizmo_screen_x = scene_center.x + transform.position[0] * scale;
        let gizmo_screen_y = scene_center.y - transform.position[2] * scale;
        let gizmo_center = egui::pos2(gizmo_screen_x, gizmo_screen_y);
        
        // Handle mouse interaction
        if let Some(mouse_pos) = response.hover_pos() {
            // Test for gizmo component hits
            let hit_component = self.test_gizmo_hit(mouse_pos, gizmo_center);
            
            // DEBUG: Log gizmo hit testing (only when relevant)
            if response.clicked() || hit_component.is_some() {
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🎯 Gizmo hit test: mouse=({:.1}, {:.1}), gizmo_center=({:.1}, {:.1}), hit={:?}",
                    mouse_pos.x, mouse_pos.y, gizmo_center.x, gizmo_center.y, hit_component
                )));
            }
            
            // Check for mouse press (start of drag) or click
            if response.clicked() || (response.drag_started() && hit_component.is_some()) {
                if response.clicked() {
                    self.console_messages.push(ConsoleMessage::info("🖱️ Mouse CLICKED"));
                } else {
                    self.console_messages.push(ConsoleMessage::info("🖱️ Mouse DRAG_STARTED"));
                }
                
                if let Some(component) = hit_component {
                    // Start dragging
                    if let Some(gizmo) = self.gizmo_system.get_move_gizmo_mut() {
                        gizmo.set_interaction_state(GizmoInteractionState::Dragging {
                            component,
                            start_mouse_pos: mouse_pos,
                            start_object_pos: transform.position,
                        });
                        self.console_messages.push(ConsoleMessage::info(&format!(
                            "🔗 Started dragging {:?} at mouse=({:.1}, {:.1}), object_pos=[{:.2}, {:.2}, {:.2}]", 
                            component, mouse_pos.x, mouse_pos.y,
                            transform.position[0], transform.position[1], transform.position[2]
                        )));
                    } else {
                        self.console_messages.push(ConsoleMessage::info("❌ Failed to get mutable gizmo"));
                    }
                } else {
                    self.console_messages.push(ConsoleMessage::info("🔗 Input detected but no gizmo component hit"));
                }
            } else if response.drag_stopped() {
                // Stop dragging (fixed deprecated method)
                if let Some(gizmo) = self.gizmo_system.get_move_gizmo_mut() {
                    if gizmo.is_interacting() {
                        gizmo.set_interaction_state(GizmoInteractionState::Idle);
                        self.console_messages.push(ConsoleMessage::info("🔗 Finished dragging"));
                    }
                }
            } else if response.dragged() {
                self.console_messages.push(ConsoleMessage::info("🖱️ Mouse DRAGGED"));
                
                // Handle dragging - extract values to avoid borrowing conflicts
                let mut new_position = transform.position;
                let mut should_update = false;
                
                if let Some(gizmo) = self.gizmo_system.get_move_gizmo() {
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🔧 Gizmo state: {:?}", gizmo.get_interaction_state()
                    )));
                    
                    if let GizmoInteractionState::Dragging { component, start_mouse_pos, start_object_pos } = gizmo.get_interaction_state() {
                        let mouse_delta = mouse_pos - *start_mouse_pos;
                        let delta_vec2 = egui::Vec2::new(mouse_delta.x, mouse_delta.y);
                        new_position = self.calculate_new_position(*start_object_pos, delta_vec2, *component, scale);
                        should_update = true;
                        
                        // Debug output
                        self.console_messages.push(ConsoleMessage::info(&format!(
                            "🖱️ Dragging {:?}: delta=({:.1}, {:.1}), start=[{:.2}, {:.2}, {:.2}], new=[{:.2}, {:.2}, {:.2}]",
                            component, mouse_delta.x, mouse_delta.y,
                            start_object_pos[0], start_object_pos[1], start_object_pos[2],
                            new_position[0], new_position[1], new_position[2]
                        )));
                    } else {
                        self.console_messages.push(ConsoleMessage::info("🔧 Not in dragging state"));
                    }
                } else {
                    self.console_messages.push(ConsoleMessage::info("❌ No gizmo available"));
                }
                
                if should_update {
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🔄 About to update transform from [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
                        transform.position[0], transform.position[1], transform.position[2],
                        new_position[0], new_position[1], new_position[2]
                    )));
                    
                    // Apply snapping if enabled
                    let snap_enabled = self.gizmo_system.is_snap_enabled();
                    let snap_increment = self.gizmo_system.get_snap_increment();
                    
                    if snap_enabled {
                        let old_new_pos = new_position;
                        new_position[0] = (new_position[0] / snap_increment).round() * snap_increment;
                        new_position[1] = (new_position[1] / snap_increment).round() * snap_increment;
                        new_position[2] = (new_position[2] / snap_increment).round() * snap_increment;
                        self.console_messages.push(ConsoleMessage::info(&format!(
                            "🔄 Snapping applied: [{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]",
                            old_new_pos[0], old_new_pos[1], old_new_pos[2],
                            new_position[0], new_position[1], new_position[2]
                        )));
                    }
                    
                    // Update transform in ECS
                    self.console_messages.push(ConsoleMessage::info(&format!(
                        "🔄 Attempting ECS mutation for entity {}", selected_entity.id()
                    )));
                    
                    match self.world.get_component_mut::<Transform>(selected_entity) {
                        Some(transform_mut) => {
                            let old_position = transform_mut.position;
                            transform_mut.position = new_position;
                            self.console_messages.push(ConsoleMessage::info(&format!(
                                "✅ Transform updated: [{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]", 
                                old_position[0], old_position[1], old_position[2],
                                transform_mut.position[0], transform_mut.position[1], transform_mut.position[2]
                            )));
                        }
                        None => {
                            self.console_messages.push(ConsoleMessage::info("❌ Failed to get mutable transform - entity doesn't exist or no transform component"));
                        }
                    }
                    
                    // Update gizmo position
                    if let Some(gizmo) = self.gizmo_system.get_move_gizmo_mut() {
                        gizmo.set_position(new_position);
                    }
                }
            } else {
                // Handle hovering
                if let Some(gizmo) = self.gizmo_system.get_move_gizmo_mut() {
                    if !gizmo.is_interacting() {
                        gizmo.set_interaction_state(if let Some(component) = hit_component {
                            GizmoInteractionState::Hovering(component)
                        } else {
                            GizmoInteractionState::Idle
                        });
                    }
                }
            }
        }
    }
    
    /// Test if mouse position hits any gizmo component
    fn test_gizmo_hit(&self, mouse_pos: egui::Pos2, gizmo_center: egui::Pos2) -> Option<GizmoComponent> {
        let arrow_length = 40.0;
        let arrow_width = 8.0; // Slightly wider for easier clicking
        let square_size = 16.0; // Slightly larger for easier clicking
        let square_offset = arrow_length * 0.3;
        let center_radius = 10.0;
        
        // Test center handle first (highest priority)
        if (mouse_pos - gizmo_center).length() <= center_radius {
            return Some(GizmoComponent::Center);
        }
        
        // Test planar movement squares
        let xy_square_center = egui::pos2(gizmo_center.x + square_offset, gizmo_center.y - square_offset);
        let xy_square = egui::Rect::from_center_size(xy_square_center, egui::vec2(square_size, square_size));
        if xy_square.contains(mouse_pos) {
            return Some(GizmoComponent::Plane(GizmoPlane::XY));
        }
        
        let xz_square_center = egui::pos2(gizmo_center.x + square_offset, gizmo_center.y + square_offset * 0.5);
        let xz_square = egui::Rect::from_center_size(xz_square_center, egui::vec2(square_size, square_size));
        if xz_square.contains(mouse_pos) {
            return Some(GizmoComponent::Plane(GizmoPlane::XZ));
        }
        
        let yz_square_center = egui::pos2(gizmo_center.x - square_offset * 0.5, gizmo_center.y - square_offset);
        let yz_square = egui::Rect::from_center_size(yz_square_center, egui::vec2(square_size, square_size));
        if yz_square.contains(mouse_pos) {
            return Some(GizmoComponent::Plane(GizmoPlane::YZ));
        }
        
        // Test axis arrows
        // X-Axis (horizontal right)
        let x_end = egui::pos2(gizmo_center.x + arrow_length, gizmo_center.y);
        if self.point_to_line_distance(mouse_pos, gizmo_center, x_end) <= arrow_width {
            return Some(GizmoComponent::Axis(GizmoAxis::X));
        }
        
        // Y-Axis (vertical up)
        let y_end = egui::pos2(gizmo_center.x, gizmo_center.y - arrow_length);
        if self.point_to_line_distance(mouse_pos, gizmo_center, y_end) <= arrow_width {
            return Some(GizmoComponent::Axis(GizmoAxis::Y));
        }
        
        // Z-Axis (diagonal up-right)
        let z_offset_x = arrow_length * 0.7;
        let z_offset_y = -arrow_length * 0.7;
        let z_end = egui::pos2(gizmo_center.x + z_offset_x, gizmo_center.y + z_offset_y);
        if self.point_to_line_distance(mouse_pos, gizmo_center, z_end) <= arrow_width {
            return Some(GizmoComponent::Axis(GizmoAxis::Z));
        }
        
        None
    }
    
    /// Calculate distance from point to line segment
    fn point_to_line_distance(&self, point: egui::Pos2, line_start: egui::Pos2, line_end: egui::Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        
        let line_len_sq = line_vec.length_sq();
        if line_len_sq < 1e-6 {
            return (point - line_start).length();
        }
        
        let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
        let projection = line_start + t * line_vec;
        (point - projection).length()
    }
    
    /// Calculate new position based on mouse delta and movement constraint
    fn calculate_new_position(&self, start_pos: [f32; 3], mouse_delta: egui::Vec2, component: GizmoComponent, scale: f32) -> [f32; 3] {
        let mut new_pos = start_pos;
        
        match component {
            GizmoComponent::Axis(GizmoAxis::X) => {
                // X-axis movement (horizontal)
                new_pos[0] = start_pos[0] + mouse_delta.x / scale;
            }
            GizmoComponent::Axis(GizmoAxis::Y) => {
                // Y-axis movement (vertical, inverted because screen Y is flipped)
                new_pos[1] = start_pos[1] - mouse_delta.y / scale;
            }
            GizmoComponent::Axis(GizmoAxis::Z) => {
                // Z-axis movement (diagonal, using both X and Y mouse movement)
                let z_movement = (mouse_delta.x - mouse_delta.y) / scale * 0.7; // Scaled for diagonal
                new_pos[2] = start_pos[2] + z_movement;
            }
            GizmoComponent::Plane(GizmoPlane::XY) => {
                // XY plane movement (Z locked)
                new_pos[0] = start_pos[0] + mouse_delta.x / scale;
                new_pos[1] = start_pos[1] - mouse_delta.y / scale;
            }
            GizmoComponent::Plane(GizmoPlane::XZ) => {
                // XZ plane movement (Y locked) 
                new_pos[0] = start_pos[0] + mouse_delta.x / scale;
                new_pos[2] = start_pos[2] - mouse_delta.y / scale;
            }
            GizmoComponent::Plane(GizmoPlane::YZ) => {
                // YZ plane movement (X locked)
                new_pos[1] = start_pos[1] - mouse_delta.y / scale * 0.5; // Reduced sensitivity
                new_pos[2] = start_pos[2] + mouse_delta.x / scale * 0.5;
            }
            GizmoComponent::Center => {
                // Screen-space movement (maintain world depth)
                new_pos[0] = start_pos[0] + mouse_delta.x / scale;
                new_pos[1] = start_pos[1] - mouse_delta.y / scale;
            }
        }
        
        new_pos
    }

    /// Render the scene from the main camera's perspective
    fn render_camera_perspective(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        
        // Draw background
        painter.rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgb(25, 25, 35) // Darker background for game view
        );
        
        // Find the main camera
        let main_camera_entity = self.find_main_camera_entity();
        
        if let Some(camera_entity) = main_camera_entity {
            if let Some(camera_transform) = self.world.get_component::<Transform>(camera_entity).cloned() {
                if let Some(camera) = self.world.get_component::<Camera>(camera_entity).cloned() {
                    // Calculate camera projection parameters
                    let aspect_ratio = rect.width() / rect.height();
                    let view_center = rect.center();
                    
                    // Render objects from camera perspective
                    self.render_scene_from_camera(ui, rect, &camera_transform, &camera, view_center);
                    
                    // Draw camera info overlay
                    ui.allocate_ui_at_rect(egui::Rect::from_min_size(rect.min, egui::vec2(250.0, 80.0)), |ui| {
                        ui.vertical(|ui| {
                            ui.colored_label(egui::Color32::WHITE, 
                                format!("📷 Camera View (FOV: {:.0}°)", camera.fov));
                            ui.small(format!("Position: [{:.1}, {:.1}, {:.1}]", 
                                camera_transform.position[0], 
                                camera_transform.position[1], 
                                camera_transform.position[2]));
                            ui.small(format!("Aspect: {:.2} | Near: {:.1} | Far: {:.0}", 
                                aspect_ratio, camera.near, camera.far));
                        });
                    });
                } else {
                    self.show_no_camera_message(ui, rect, "Camera component missing");
                }
            } else {
                self.show_no_camera_message(ui, rect, "Camera transform missing");
            }
        } else {
            self.show_no_camera_message(ui, rect, "No main camera found");
        }
    }
    
    /// Find the main camera entity in the scene
    fn find_main_camera_entity(&self) -> Option<Entity> {
        // Look for entity with Camera component that has is_main = true
        for (entity, _transform) in self.world.query::<Transform>() {
            if let Some(camera) = self.world.get_component::<Camera>(entity) {
                if camera.is_main {
                    return Some(entity);
                }
            }
        }
        
        // If no main camera found, return the first camera entity
        for (entity, _transform) in self.world.query::<Transform>() {
            if self.world.get_component::<Camera>(entity).is_some() {
                return Some(entity);
            }
        }
        
        None
    }
    
    /// Render the scene from the camera's perspective using perspective projection
    fn render_scene_from_camera(&mut self, ui: &mut egui::Ui, rect: egui::Rect, 
                                camera_transform: &Transform, camera: &Camera, view_center: egui::Pos2) {
        let painter = ui.painter();
        
        // Camera position and view parameters
        let camera_pos = camera_transform.position;
        let fov_rad = camera.fov.to_radians();
        let aspect_ratio = rect.width() / rect.height();
        
        // Calculate view frustum dimensions at different depths
        let render_scale = 100.0; // Scale factor for rendering
        
        // Render all entities with transforms
        for (entity, transform) in self.world.query::<Transform>() {
            // Skip the camera itself
            if let Some(camera_check) = self.world.get_component::<Camera>(entity) {
                if camera_check.is_main {
                    continue;
                }
            }
            
            // Calculate relative position from camera
            let relative_pos = [
                transform.position[0] - camera_pos[0],
                transform.position[1] - camera_pos[1], 
                transform.position[2] - camera_pos[2]
            ];
            
            // Simple perspective projection (assuming camera looks down -Z axis)
            let depth = -relative_pos[2]; // Distance from camera
            
            // Skip objects behind camera or too far away
            if depth <= camera.near || depth > camera.far {
                continue;
            }
            
            // Perspective projection to screen space
            let proj_x = relative_pos[0] / depth;
            let proj_y = relative_pos[1] / depth;
            
            // Convert to screen coordinates
            let screen_x = view_center.x + proj_x * render_scale;
            let screen_y = view_center.y - proj_y * render_scale; // Flip Y for screen space
            
            // Check if object is within screen bounds (frustum culling)
            let screen_pos = egui::pos2(screen_x, screen_y);
            if !rect.contains(screen_pos) {
                continue;
            }
            
            // Calculate object size based on depth (perspective scaling)
            let base_size = 20.0;
            let size_scale = camera.near / depth; // Objects get smaller with distance
            let object_size = base_size * size_scale * transform.scale[0];
            
            // Get entity info for rendering
            let name = self.world.get_component::<Name>(entity)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
            // Determine object color and shape based on components
            let (color, shape) = if let Some(sprite_renderer) = self.world.get_component::<SpriteRenderer>(entity) {
                // Render sprite
                let sprite_color = egui::Color32::from_rgba_unmultiplied(
                    (sprite_renderer.sprite.color[0] * 255.0) as u8,
                    (sprite_renderer.sprite.color[1] * 255.0) as u8,
                    (sprite_renderer.sprite.color[2] * 255.0) as u8,
                    (sprite_renderer.sprite.color[3] * 255.0) as u8,
                );
                (sprite_color, "square") // Sprites as squares
            } else if let Some(_mesh) = self.world.get_component::<Mesh>(entity) {
                // Render mesh object
                (egui::Color32::from_rgb(180, 180, 180), "circle") // Meshes as circles
            } else {
                // Default object
                (egui::Color32::YELLOW, "circle")
            };
            
            // Draw object
            if shape == "square" {
                let size_vec = egui::vec2(object_size, object_size);
                let obj_rect = egui::Rect::from_center_size(screen_pos, size_vec);
                painter.rect_filled(obj_rect, egui::Rounding::same(2.0), color);
            } else {
                painter.circle_filled(screen_pos, object_size * 0.5, color);
            }
            
            // Draw object label (smaller for distant objects)
            let label_size = (12.0 * size_scale).max(8.0);
            painter.text(
                screen_pos + egui::vec2(object_size * 0.5 + 2.0, -object_size * 0.5),
                egui::Align2::LEFT_CENTER,
                &name,
                egui::FontId::proportional(label_size),
                color
            );
        }
        
        // Draw depth indication lines for reference
        self.draw_depth_reference_lines(ui, rect, view_center, render_scale);
    }
    
    /// Draw reference lines to show depth in the camera view
    fn draw_depth_reference_lines(&self, ui: &mut egui::Ui, rect: egui::Rect, center: egui::Pos2, scale: f32) {
        let painter = ui.painter();
        
        // Draw horizon line
        painter.line_segment(
            [egui::pos2(rect.left(), center.y), egui::pos2(rect.right(), center.y)],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30))
        );
        
        // Draw vertical center line
        painter.line_segment(
            [egui::pos2(center.x, rect.top()), egui::pos2(center.x, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30))
        );
        
        // Draw perspective grid lines
        let grid_spacing = scale * 0.3;
        for i in 1..5 {
            let offset = grid_spacing * i as f32;
            
            // Horizontal grid lines
            if center.y + offset < rect.bottom() {
                painter.line_segment(
                    [egui::pos2(rect.left(), center.y + offset), egui::pos2(rect.right(), center.y + offset)],
                    egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15))
                );
            }
            if center.y - offset > rect.top() {
                painter.line_segment(
                    [egui::pos2(rect.left(), center.y - offset), egui::pos2(rect.right(), center.y - offset)],
                    egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15))
                );
            }
        }
    }
    
    /// Show message when no camera is available
    fn show_no_camera_message(&self, ui: &mut egui::Ui, rect: egui::Rect, message: &str) {
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Camera Issue");
                    ui.label(message);
                    ui.small("Add a Camera component to an entity");
                    ui.small("Set 'is_main' to true for main camera");
                });
            });
        });
    }

    fn show_game_view(&mut self, ui: &mut egui::Ui) {
        // Delegate to the game view panel
        let (messages, render_rect) = self.game_view_panel.show(ui, self.play_state);
        self.console_messages.extend(messages);
        
        // If we got a rect back, render the camera perspective
        if let Some(rect) = render_rect {
            self.render_camera_perspective(ui, rect);
        }
    }
    
    fn show_console_panel(&mut self, ui: &mut egui::Ui) {
        // Delegate to the console panel module
        self.console_panel.show(ui, &mut self.console_messages);
    }
    
    fn show_project_panel(&mut self, ui: &mut egui::Ui) {
        // Delegate to the project panel module
        self.project_panel.show(ui, &self.project_assets, &mut self.console_messages);
    }
}


/// Hierarchy object representation
#[derive(Clone)]
struct HierarchyObject {
    name: String,
    object_type: ObjectType,
    children: Option<Vec<HierarchyObject>>,
}

impl HierarchyObject {
    fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            name: name.to_string(),
            object_type,
            children: None,
        }
    }
    
    fn parent(name: &str, children: Vec<HierarchyObject>) -> Self {
        Self {
            name: name.to_string(),
            object_type: ObjectType::GameObject,
            children: Some(children),
        }
    }
}

#[derive(Clone)]
enum ObjectType {
    GameObject,
    Camera,
    Light,
}


/// Console message types

/// Setup custom fonts for Unity-like appearance
fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    
    // For now, use default fonts with adjusted sizes
    // TODO: Add custom Unity-like fonts later
    
    ctx.set_fonts(fonts);
}

/// Setup Unity-like visual style
fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Unity dark theme colors
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
    style.visuals.panel_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.window_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(26, 26, 26);
    style.visuals.button_frame = true;
    
    // Compact spacing like Unity
    style.spacing.item_spacing = egui::vec2(4.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.menu_margin = egui::Margin::same(4.0);
    
    ctx.set_style(style);
}

/// Apply Unity-style colors during runtime
fn apply_unity_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Update colors for Unity-like appearance
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 122, 255);
    
    // Selection colors
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 122, 255);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 122, 255));
    
    ctx.set_style(style);
}