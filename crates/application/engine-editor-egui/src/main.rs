// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};
use engine_ecs_core::{Transform, WorldV2, EntityV2, Read, Write, Name, Visibility, Camera, Light, Sprite, SpriteRenderer, Canvas, Camera2D, Material};
use engine_camera::{CameraComponent, CameraType, Viewport};

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

/// Play state for editor mode management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayState {
    Editing,   // Normal editor mode - full editing capabilities
    Playing,   // Game running - properties locked, runtime active  
    Paused,    // Game paused - can inspect state, limited editing
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Editing
    }
}

/// Basic texture asset for sprites
#[derive(Debug, Clone)]
struct TextureAsset {
    handle: u64,
    name: String,
    color: [f32; 4], // RGBA color for simple colored textures
    size: [u32; 2],  // Width, Height
}

/// Unity-style editor application with dockable panels
struct UnityEditor {
    // Docking system
    dock_state: DockState<PanelType>,
    
    // ECS v2 Integration
    world: WorldV2,
    selected_entity: Option<EntityV2>,
    
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
        
        for (handle, name, color) in textures_data {
            textures.insert(handle, TextureAsset {
                handle,
                name: name.to_string(),
                color,
                size: [64, 64], // 64x64 pixel textures
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
        if self.play_state == PlayState::Playing {
            let now = std::time::Instant::now();
            self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;
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
        let mut world = WorldV2::new();
        
        // Create camera entity
        let camera_entity = world.spawn();
        world.add_component(camera_entity, Transform {
            position: [0.0, 0.0, 5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        world.add_component(camera_entity, Name::new("Main Camera")).unwrap();
        world.add_component(camera_entity, Camera::default()).unwrap();
        
        // Create cube entity with mesh and material
        let cube_entity = world.spawn();
        world.add_component(cube_entity, Transform {
            position: [1.0, 0.0, 0.0],
            rotation: [0.0, 45.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        world.add_component(cube_entity, Name::new("Cube")).unwrap();
        world.add_component(cube_entity, engine_ecs_core::Mesh {
            mesh_type: engine_ecs_core::MeshType::Cube,
        }).unwrap();
        world.add_component(cube_entity, Material {
            color: [0.8, 0.2, 0.2, 1.0], // Red cube
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(cube_entity, Visibility::default()).unwrap();
        
        // Create sphere entity with mesh and material
        let sphere_entity = world.spawn();
        world.add_component(sphere_entity, Transform {
            position: [-1.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.5, 1.5, 1.5],
        }).unwrap();
        world.add_component(sphere_entity, Name::new("Sphere")).unwrap();
        world.add_component(sphere_entity, engine_ecs_core::Mesh {
            mesh_type: engine_ecs_core::MeshType::Sphere,
        }).unwrap();
        world.add_component(sphere_entity, Material {
            color: [0.2, 0.8, 0.2, 1.0], // Green sphere
            metallic: 0.1,
            roughness: 0.3,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(sphere_entity, Visibility::default()).unwrap();
        
        // Create plane entity (ground)
        let plane_entity = world.spawn();
        world.add_component(plane_entity, Transform {
            position: [0.0, -1.5, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [5.0, 1.0, 5.0],
        }).unwrap();
        world.add_component(plane_entity, Name::new("Ground Plane")).unwrap();
        world.add_component(plane_entity, engine_ecs_core::Mesh {
            mesh_type: engine_ecs_core::MeshType::Plane,
        }).unwrap();
        world.add_component(plane_entity, Material {
            color: [0.6, 0.6, 0.6, 1.0], // Gray ground
            metallic: 0.0,
            roughness: 0.8,
            emissive: [0.0, 0.0, 0.0],
        }).unwrap();
        world.add_component(plane_entity, Visibility::default()).unwrap();
        
        // Create sprite entities to test sprite rendering
        let red_sprite_entity = world.spawn();
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
        
        let blue_sprite_entity = world.spawn();
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
        
        let yellow_sprite_entity = world.spawn();
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
                ConsoleMessage::info("💡 Try selecting entities in the hierarchy"),
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
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Scene").clicked() {
                    self.console_messages.push(ConsoleMessage::info("📄 Created new scene"));
                    ui.close_menu();
                }
                if ui.button("Open Scene").clicked() {
                    self.console_messages.push(ConsoleMessage::info("📂 Opening scene..."));
                    ui.close_menu();
                }
                if ui.button("Save Scene").clicked() {
                    self.console_messages.push(ConsoleMessage::info("💾 Scene saved"));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
            
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    self.console_messages.push(ConsoleMessage::info("↶ Undo"));
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    self.console_messages.push(ConsoleMessage::info("↷ Redo"));
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Window", |ui| {
                ui.label("Dockable Panels:");
                ui.separator();
                if ui.button("Add Hierarchy Panel").clicked() {
                    self.dock_state.add_window(vec![PanelType::Hierarchy]);
                    self.console_messages.push(ConsoleMessage::info("➕ Added Hierarchy panel"));
                    ui.close_menu();
                }
                if ui.button("Add Inspector Panel").clicked() {
                    self.dock_state.add_window(vec![PanelType::Inspector]);
                    self.console_messages.push(ConsoleMessage::info("➕ Added Inspector panel"));
                    ui.close_menu();
                }
                if ui.button("Add Console Panel").clicked() {
                    self.dock_state.add_window(vec![PanelType::Console]);
                    self.console_messages.push(ConsoleMessage::info("➕ Added Console panel"));
                    ui.close_menu();
                }
                if ui.button("Add Project Panel").clicked() {
                    self.dock_state.add_window(vec![PanelType::Project]);
                    self.console_messages.push(ConsoleMessage::info("➕ Added Project panel"));
                    ui.close_menu();
                }
                if ui.button("Add Game View Panel").clicked() {
                    self.dock_state.add_window(vec![PanelType::GameView]);
                    self.console_messages.push(ConsoleMessage::info("➕ Added Game View panel"));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Reset Layout").clicked() {
                    // Reset to Unity-style layout with Scene and Game views
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
                    
                    self.dock_state = dock_state;
                    self.console_messages.push(ConsoleMessage::info("🔄 Layout reset to Unity default"));
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Help", |ui| {
                ui.label("💡 Drag panel tabs to rearrange");
                ui.label("🔄 Drop tabs on different areas to dock");
                ui.label("➕ Use Window menu to add panels");
                ui.label("🖱️ Right-click tabs for options");
            });
        });
    }
    
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            
            // Transform tools
            if ui.button("🔗").on_hover_text("Move Tool").clicked() {
                self.console_messages.push(ConsoleMessage::info("🔗 Move tool selected"));
            }
            if ui.button("🔄").on_hover_text("Rotate Tool").clicked() {
                self.console_messages.push(ConsoleMessage::info("🔄 Rotate tool selected"));
            }
            if ui.button("📐").on_hover_text("Scale Tool").clicked() {
                self.console_messages.push(ConsoleMessage::info("📐 Scale tool selected"));
            }
            
            ui.separator();
            
            // Play controls - state-aware buttons
            match self.play_state {
                PlayState::Editing => {
                    if ui.button("▶️").on_hover_text("Play").clicked() {
                        self.start_play();
                    }
                    // Show disabled pause/stop buttons
                    ui.add_enabled(false, egui::Button::new("⏸️"));
                    ui.add_enabled(false, egui::Button::new("⏹️"));
                }
                PlayState::Playing => {
                    // Show highlighted play button (active state)
                    ui.add_enabled(false, egui::Button::new("▶️").fill(egui::Color32::from_rgb(100, 200, 100)));
                    if ui.button("⏸️").on_hover_text("Pause").clicked() {
                        self.pause_play();
                    }
                    if ui.button("⏹️").on_hover_text("Stop").clicked() {
                        self.stop_play();
                    }
                }
                PlayState::Paused => {
                    if ui.button("▶️").on_hover_text("Resume").clicked() {
                        self.resume_play();
                    }
                    // Show highlighted pause button (active state)
                    ui.add_enabled(false, egui::Button::new("⏸️").fill(egui::Color32::from_rgb(200, 200, 100)));
                    if ui.button("⏹️").on_hover_text("Stop").clicked() {
                        self.stop_play();
                    }
                }
            }
            
            ui.separator();
            
            // View options
            ui.label("Layers:");
            egui::ComboBox::from_id_source("layers")
                .selected_text("Default")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut "", "Default", "Default");
                    ui.selectable_value(&mut "", "UI", "UI");
                    ui.selectable_value(&mut "", "Background", "Background");
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🎯 Focus Selected").on_hover_text("Focus camera on selected object").clicked() {
                    if let Some(ref obj) = self.selected_object {
                        self.console_messages.push(ConsoleMessage::info(&format!("🎯 Focused on {}", obj)));
                    }
                }
                
                ui.separator();
                
                ui.label("Layout:");
                egui::ComboBox::from_id_source("layout")
                    .selected_text("Default")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut "", "Default", "Default");
                        ui.selectable_value(&mut "", "2 by 3", "2 by 3");
                        ui.selectable_value(&mut "", "4 Split", "4 Split");
                    });
            });
        });
    }
    
    fn show_hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("ECS Entities");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕").on_hover_text("Create new entity").clicked() {
                    // Create new entity with ECS v2
                    let entity = self.world.spawn();
                    self.world.add_component(entity, Transform::default()).unwrap();
                    self.console_messages.push(ConsoleMessage::info(&format!("➕ Created Entity {:?}", entity)));
                }
            });
        });
        ui.separator();
        
        ui.label(format!("🎯 Entity Count: {}", self.world.entity_count()));
        ui.label(format!("📦 Archetypes: {}", self.world.archetype_count()));
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Show all entities with Transform components using ECS v2 query
            for (entity, _transform) in self.world.query::<Read<Transform>>().iter() {
                let selected = self.selected_entity == Some(entity);
                
                // Build component indicator string
                let mut components = Vec::new();
                if self.world.get_component::<Transform>(entity).is_some() { components.push("T"); }
                if self.world.get_component::<Name>(entity).is_some() { components.push("N"); }
                if self.world.get_component::<Visibility>(entity).is_some() { components.push("V"); }
                if self.world.get_component::<Camera>(entity).is_some() { components.push("C"); }
                if self.world.get_component::<Light>(entity).is_some() { components.push("L"); }
                if self.world.get_component::<SpriteRenderer>(entity).is_some() { components.push("Spr"); }
                if self.world.get_component::<Canvas>(entity).is_some() { components.push("Canvas"); }
                if self.world.get_component::<Camera2D>(entity).is_some() { components.push("C2D"); }
                if self.world.get_component::<CameraComponent>(entity).is_some() { components.push("Cam"); }
                if self.world.get_component::<engine_ecs_core::Mesh>(entity).is_some() { components.push("M"); }
                if self.world.get_component::<Material>(entity).is_some() { components.push("Mat"); }
                
                let component_str = if components.is_empty() { "-".to_string() } else { components.join("") };
                
                // Get entity name if available
                let entity_name = if let Some(name) = self.world.get_component::<Name>(entity) {
                    name.name.clone()
                } else {
                    format!("Entity {}", entity.id())
                };
                
                let label = format!("📦 {} [{}]", entity_name, component_str);
                
                if ui.selectable_label(selected, &label).clicked() {
                    self.selected_entity = Some(entity);
                    self.console_messages.push(ConsoleMessage::info(&format!("🎯 Selected Entity {:?}", entity)));
                }
            }
        });
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
        ui.label("Entity Inspector");
        ui.separator();
        
        if let Some(selected_entity) = self.selected_entity {
            ui.label(format!("Entity ID: {}", selected_entity.id()));
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Get Transform component from ECS v2 (clone it to avoid borrowing issues)
                if let Some(transform) = self.world.get_component::<Transform>(selected_entity).cloned() {
                    ui.collapsing("📐 Transform", |ui| {
                        // Clone the transform to make it mutable for editing
                        let mut pos = transform.position;
                        let mut rot = transform.rotation;
                        let mut scale = transform.scale;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("transform_grid").show(ui, |ui| {
                            // Position
                            ui.label("Position:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[0]).speed(0.1)).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[1]).speed(0.1)).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut pos[2]).speed(0.1)).changed();
                            ui.end_row();
                            
                            // Rotation
                            ui.label("Rotation:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[0]).speed(1.0).suffix("°")).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[1]).speed(1.0).suffix("°")).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut rot[2]).speed(1.0).suffix("°")).changed();
                            ui.end_row();
                            
                            // Scale
                            ui.label("Scale:");
                            ui.end_row();
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[0]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.label("Y:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[1]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut scale[2]).speed(0.01).range(0.01..=10.0)).changed();
                            ui.end_row();
                        });
                        
                        // Update the ECS component if values changed
                        if changed {
                            if let Some(transform_mut) = self.world.get_component_mut::<Transform>(selected_entity) {
                                transform_mut.position = pos;
                                transform_mut.rotation = rot;
                                transform_mut.scale = scale;
                            }
                        }
                    });
                } else {
                    ui.label("❌ No Transform component");
                }
                
                // Name Component
                if let Some(name) = self.world.get_component::<Name>(selected_entity) {
                    ui.collapsing("📝 Name", |ui| {
                        ui.label(format!("Name: {}", name.name));
                    });
                }
                
                // Visibility Component
                if let Some(visibility) = self.world.get_component::<Visibility>(selected_entity) {
                    ui.collapsing("👁️ Visibility", |ui| {
                        ui.label(format!("Visible: {}", visibility.visible));
                    });
                }
                
                // Camera Component
                if let Some(camera) = self.world.get_component::<Camera>(selected_entity) {
                    ui.collapsing("📷 Camera", |ui| {
                        ui.label(format!("FOV: {:.1}°", camera.fov));
                        ui.label(format!("Near: {:.2}", camera.near));
                        ui.label(format!("Far: {:.0}", camera.far));
                        ui.label(format!("Main Camera: {}", camera.is_main));
                    });
                }
                
                // Light Component
                if let Some(light) = self.world.get_component::<Light>(selected_entity) {
                    ui.collapsing("💡 Light", |ui| {
                        ui.label(format!("Type: {:?}", light.light_type));
                        ui.label(format!("Color: [{:.2}, {:.2}, {:.2}]", 
                                 light.color[0], light.color[1], light.color[2]));
                        ui.label(format!("Intensity: {:.2}", light.intensity));
                    });
                }
                
                // Sprite Renderer Component
                if let Some(sprite_renderer) = self.world.get_component::<SpriteRenderer>(selected_entity).cloned() {
                    ui.collapsing("🖼️ Sprite Renderer", |ui| {
                        let mut enabled = sprite_renderer.enabled;
                        let mut layer = sprite_renderer.layer;
                        let mut color = sprite_renderer.sprite.color;
                        let mut flip_x = sprite_renderer.sprite.flip_x;
                        let mut flip_y = sprite_renderer.sprite.flip_y;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("sprite_renderer_grid").show(ui, |ui| {
                            // Enabled checkbox
                            ui.label("Enabled:");
                            changed |= ui.checkbox(&mut enabled, "").changed();
                            ui.end_row();
                            
                            // Layer
                            ui.label("Layer:");
                            changed |= ui.add(egui::DragValue::new(&mut layer).range(-32768..=32767)).changed();
                            ui.end_row();
                            
                            // Color tint
                            ui.label("Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                            
                            // Flip options
                            ui.label("Flip X:");
                            changed |= ui.checkbox(&mut flip_x, "").changed();
                            ui.end_row();
                            ui.label("Flip Y:");
                            changed |= ui.checkbox(&mut flip_y, "").changed();
                            ui.end_row();
                        });
                        
                        // Show texture handle if present
                        if let Some(handle) = sprite_renderer.sprite.texture_handle {
                            ui.label(format!("Texture Handle: {}", handle));
                        } else {
                            ui.label("No texture assigned");
                        }
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(sprite_mut) = self.world.get_component_mut::<SpriteRenderer>(selected_entity) {
                                sprite_mut.enabled = enabled;
                                sprite_mut.layer = layer;
                                sprite_mut.sprite.color = color;
                                sprite_mut.sprite.flip_x = flip_x;
                                sprite_mut.sprite.flip_y = flip_y;
                            }
                        }
                    });
                }
                
                // Canvas Component
                if let Some(canvas) = self.world.get_component::<Canvas>(selected_entity) {
                    ui.collapsing("🎨 Canvas", |ui| {
                        ui.label(format!("Render Mode: {:?}", canvas.render_mode));
                        ui.label(format!("Sorting Layer: {}", canvas.sorting_layer));
                        ui.label(format!("Order in Layer: {}", canvas.order_in_layer));
                        ui.label(format!("Pixel Perfect: {}", canvas.pixel_perfect));
                    });
                }
                
                // Camera2D Component
                if let Some(camera_2d) = self.world.get_component::<Camera2D>(selected_entity).cloned() {
                    ui.collapsing("📷 Camera 2D", |ui| {
                        let mut size = camera_2d.size;
                        let mut aspect_ratio = camera_2d.aspect_ratio;
                        let mut near = camera_2d.near;
                        let mut far = camera_2d.far;
                        let mut is_main = camera_2d.is_main;
                        let mut bg_color = camera_2d.background_color;
                        
                        let mut changed = false;
                        
                        egui::Grid::new("camera_2d_grid").show(ui, |ui| {
                            // Orthographic size
                            ui.label("Size:");
                            changed |= ui.add(egui::DragValue::new(&mut size).speed(0.1).range(0.1..=100.0)).changed();
                            ui.end_row();
                            
                            // Aspect ratio
                            ui.label("Aspect Ratio:");
                            changed |= ui.add(egui::DragValue::new(&mut aspect_ratio).speed(0.01).range(0.0..=10.0)).changed();
                            ui.end_row();
                            
                            // Near/Far clipping
                            ui.label("Near:");
                            changed |= ui.add(egui::DragValue::new(&mut near).speed(0.1).range(-100.0..=100.0)).changed();
                            ui.end_row();
                            ui.label("Far:");
                            changed |= ui.add(egui::DragValue::new(&mut far).speed(0.1).range(-100.0..=100.0)).changed();
                            ui.end_row();
                            
                            // Main camera
                            ui.label("Main Camera:");
                            changed |= ui.checkbox(&mut is_main, "").changed();
                            ui.end_row();
                            
                            // Background color
                            ui.label("Background Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut bg_color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                        });
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(camera_mut) = self.world.get_component_mut::<Camera2D>(selected_entity) {
                                camera_mut.size = size;
                                camera_mut.aspect_ratio = aspect_ratio;
                                camera_mut.near = near;
                                camera_mut.far = far;
                                camera_mut.is_main = is_main;
                                camera_mut.background_color = bg_color;
                            }
                        }
                    });
                }
                
                // Camera Component (Advanced)
                if let Some(camera_comp) = self.world.get_component::<CameraComponent>(selected_entity).cloned() {
                    ui.collapsing("📷 Camera (Advanced)", |ui| {
                        let mut is_main = camera_comp.is_main;
                        let mut camera_type = camera_comp.camera.camera_type().clone();
                        let mut clear_color = camera_comp.camera.clear_color();
                        let mut render_order = camera_comp.camera.render_order();
                        let mut enabled = camera_comp.camera.enabled();
                        
                        let mut changed = false;
                        
                        egui::Grid::new("camera_comp_grid").show(ui, |ui| {
                            // Main camera checkbox
                            ui.label("Main Camera:");
                            changed |= ui.checkbox(&mut is_main, "").changed();
                            ui.end_row();
                            
                            // Enabled checkbox
                            ui.label("Enabled:");
                            changed |= ui.checkbox(&mut enabled, "").changed();
                            ui.end_row();
                            
                            // Render order
                            ui.label("Render Order:");
                            changed |= ui.add(egui::DragValue::new(&mut render_order).range(-100..=100)).changed();
                            ui.end_row();
                            
                            // Camera type
                            ui.label("Camera Type:");
                            ui.end_row();
                            
                            // Camera type specific settings
                            match &mut camera_type {
                                CameraType::Orthographic2D { size, near, far } => {
                                    ui.label("Type: Orthographic 2D");
                                    ui.end_row();
                                    ui.label("Size:");
                                    changed |= ui.add(egui::DragValue::new(size).speed(0.1).range(0.1..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Near:");
                                    changed |= ui.add(egui::DragValue::new(near).speed(0.1).range(-100.0..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Far:");
                                    changed |= ui.add(egui::DragValue::new(far).speed(0.1).range(-100.0..=100.0)).changed();
                                    ui.end_row();
                                }
                                CameraType::Perspective3D { fov_degrees, near, far } => {
                                    ui.label("Type: Perspective 3D");
                                    ui.end_row();
                                    ui.label("FOV (degrees):");
                                    changed |= ui.add(egui::DragValue::new(fov_degrees).speed(1.0).range(1.0..=179.0)).changed();
                                    ui.end_row();
                                    ui.label("Near:");
                                    changed |= ui.add(egui::DragValue::new(near).speed(0.01).range(0.01..=100.0)).changed();
                                    ui.end_row();
                                    ui.label("Far:");
                                    changed |= ui.add(egui::DragValue::new(far).speed(1.0).range(1.0..=10000.0)).changed();
                                    ui.end_row();
                                }
                                CameraType::Custom { .. } => {
                                    ui.label("Type: Custom Matrix");
                                    ui.end_row();
                                    ui.label("(Custom matrices not editable)");
                                    ui.end_row();
                                }
                            }
                            
                            // Clear color
                            ui.label("Clear Color:");
                            ui.end_row();
                            ui.label("R:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[0]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("G:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[1]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("B:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[2]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.label("A:");
                            changed |= ui.add(egui::DragValue::new(&mut clear_color[3]).range(0.0..=1.0).speed(0.01)).changed();
                            ui.end_row();
                        });
                        
                        // Show viewport info (read-only)
                        ui.separator();
                        ui.label("Viewport Information:");
                        let viewport = camera_comp.camera.viewport();
                        ui.label(format!("Size: {}x{}", viewport.width, viewport.height));
                        ui.label(format!("Aspect Ratio: {:.2}", viewport.aspect_ratio()));
                        
                        // Update the component if values changed
                        if changed {
                            if let Some(camera_mut) = self.world.get_component_mut::<CameraComponent>(selected_entity) {
                                camera_mut.is_main = is_main;
                                camera_mut.camera.set_camera_type(camera_type);
                                camera_mut.camera.set_clear_color(clear_color);
                                camera_mut.camera.set_render_order(render_order);
                                camera_mut.camera.set_enabled(enabled);
                                
                                // Update projection matrix if camera type changed
                                if let Err(e) = camera_mut.camera.update_projection_matrix() {
                                    self.console_messages.push(ConsoleMessage::info(&format!("⚠️ Camera update error: {}", e)));
                                }
                            }
                        }
                    });
                }
                
                // ECS v2 Entity Info
                ui.separator();
                ui.collapsing("🔧 Entity Debug", |ui| {
                    ui.label(format!("Entity ID: {}", selected_entity.id()));
                    ui.label(format!("Generation: {}", selected_entity.generation()));
                    
                    // Count components
                    let mut component_count = 0;
                    let mut component_list = Vec::new();
                    
                    if self.world.get_component::<Transform>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Transform");
                    }
                    if self.world.get_component::<Name>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Name");
                    }
                    if self.world.get_component::<Visibility>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Visibility");
                    }
                    if self.world.get_component::<Camera>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Camera");
                    }
                    if self.world.get_component::<Light>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Light");
                    }
                    if self.world.get_component::<SpriteRenderer>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("SpriteRenderer");
                    }
                    if self.world.get_component::<Canvas>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Canvas");
                    }
                    if self.world.get_component::<Camera2D>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("Camera2D");
                    }
                    if self.world.get_component::<CameraComponent>(selected_entity).is_some() {
                        component_count += 1;
                        component_list.push("CameraComponent");
                    }
                    
                    ui.label(format!("Component Count: {}", component_count));
                    ui.label(format!("Components: {}", component_list.join(", ")));
                });
                
                ui.separator();
                if ui.button("➕ Add Component").clicked() {
                    self.show_add_component_dialog = true;
                }
                
                // Add Component Dialog
                if self.show_add_component_dialog {
                    self.show_add_component_dialog(ui, selected_entity);
                }
            });
        } else {
            ui.label("No entity selected");
            ui.label("Select an entity in the Hierarchy to view its components.");
        }
    }
    
    fn show_add_component_dialog(&mut self, ui: &mut egui::Ui, entity: EntityV2) {
        let mut dialog_open = self.show_add_component_dialog;
        egui::Window::new("Add Component")
            .open(&mut dialog_open)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Choose a component to add:");
                ui.separator();
                
                // Name Component
                if ui.button("📝 Name Component").clicked() {
                    match self.world.add_component(entity, Name::new("New Object")) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Name component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Name: {}", e)));
                        }
                    }
                }
                
                // Visibility Component
                if ui.button("👁️ Visibility Component").clicked() {
                    match self.world.add_component(entity, Visibility::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Visibility component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Visibility: {}", e)));
                        }
                    }
                }
                
                // Camera Component
                if ui.button("📷 Camera Component").clicked() {
                    match self.world.add_component(entity, Camera::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Camera component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Camera: {}", e)));
                        }
                    }
                }
                
                // Light Component
                if ui.button("💡 Light Component").clicked() {
                    match self.world.add_component(entity, Light::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Light component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Light: {}", e)));
                        }
                    }
                }
                
                ui.separator();
                ui.label("Camera Components:");
                
                // Advanced Camera Component
                if ui.button("📷 Advanced Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if self.world.get_component::<CameraComponent>(entity).is_some() {
                        self.console_messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let viewport = Viewport::new(800, 600);
                        let camera = engine_camera::Camera::orthographic_2d(5.0, viewport);
                        let camera_comp = CameraComponent::new(camera);
                        
                        match self.world.add_component(entity, camera_comp) {
                            Ok(_) => {
                                self.console_messages.push(ConsoleMessage::info("✅ Added Advanced Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Advanced Camera: {}", e)));
                            }
                        }
                    }
                }
                
                // Perspective Camera shortcut
                if ui.button("🎥 Perspective Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if self.world.get_component::<CameraComponent>(entity).is_some() {
                        self.console_messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let viewport = Viewport::new(800, 600);
                        let camera = engine_camera::Camera::perspective_3d(60.0, viewport);
                        let camera_comp = CameraComponent::new(camera);
                        
                        match self.world.add_component(entity, camera_comp) {
                            Ok(_) => {
                                self.console_messages.push(ConsoleMessage::info("✅ Added Perspective Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Perspective Camera: {}", e)));
                            }
                        }
                    }
                }
                
                ui.separator();
                ui.label("2D Components:");
                
                // Sprite Renderer Component
                if ui.button("🖼️ Sprite Renderer Component").clicked() {
                    match self.world.add_component(entity, SpriteRenderer::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Sprite Renderer component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Sprite Renderer: {}", e)));
                        }
                    }
                }
                
                // Canvas Component
                if ui.button("🎨 Canvas Component").clicked() {
                    match self.world.add_component(entity, Canvas::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Canvas component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Canvas: {}", e)));
                        }
                    }
                }
                
                // Camera2D Component
                if ui.button("📷 Camera 2D Component").clicked() {
                    match self.world.add_component(entity, Camera2D::default()) {
                        Ok(_) => {
                            self.console_messages.push(ConsoleMessage::info("✅ Added Camera 2D component"));
                            self.show_add_component_dialog = false;
                        }
                        Err(e) => {
                            self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Camera 2D: {}", e)));
                        }
                    }
                }
                
                ui.separator();
                if ui.button("Cancel").clicked() {
                    self.show_add_component_dialog = false;
                }
            });
        self.show_add_component_dialog = dialog_open;
    }
    
    fn show_scene_view(&mut self, ui: &mut egui::Ui) {
        // Scene view toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.scene_view_active, true, "Scene");
            ui.selectable_value(&mut self.scene_view_active, false, "Game");
            
            ui.separator();
            
            if ui.button("🔍").on_hover_text("Focus on selected").clicked() {
                if let Some(ref obj) = self.selected_object {
                    self.console_messages.push(ConsoleMessage::info(&format!("🔍 Focused on {}", obj)));
                }
            }
        });
        
        ui.separator();
        
        // Main view area
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::drag());
        
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
        
        // Handle scene interactions
        if response.dragged() {
            self.console_messages.push(ConsoleMessage::info("🖱️ Scene view interaction"));
        }
    }
    
    /// Get the color for a sprite based on its texture handle and tint
    fn get_sprite_color(&self, sprite: &Sprite) -> egui::Color32 {
        // Start with sprite tint color
        let mut final_color = sprite.color;
        
        // If sprite has a texture, blend with texture color
        if let Some(texture_handle) = sprite.texture_handle {
            if let Some(texture) = self.texture_assets.get(&texture_handle) {
                // Multiply sprite tint with texture color
                final_color[0] *= texture.color[0];
                final_color[1] *= texture.color[1];
                final_color[2] *= texture.color[2];
                final_color[3] *= texture.color[3];
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
        
        // Draw grid background
        let grid_size = 50.0;
        let center = rect.center();
        
        // Draw grid lines
        painter.line_segment(
            [egui::pos2(rect.left(), center.y), egui::pos2(rect.right(), center.y)],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
        painter.line_segment(
            [egui::pos2(center.x, rect.top()), egui::pos2(center.x, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        );
        
        // Draw scene objects (simplified 2D representation)
        for (entity, transform) in self.world.query::<Read<Transform>>().iter() {
            // Project 3D position to 2D screen coordinates (simple orthographic)
            let scale = 50.0; // Pixels per world unit
            let screen_x = center.x + transform.position[0] * scale;
            let screen_y = center.y - transform.position[2] * scale; // Z becomes Y in screen space
            let screen_pos = egui::pos2(screen_x, screen_y);
            
            // Get entity info
            let name = self.world.get_component::<Name>(entity)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
            // Determine object type and color
            let (color, icon, size) = if self.world.get_component::<Camera>(entity).is_some() {
                (egui::Color32::BLUE, "📷", 12.0)
            } else if self.world.get_component::<engine_ecs_core::Mesh>(entity).is_some() {
                let mesh = self.world.get_component::<engine_ecs_core::Mesh>(entity).unwrap();
                match mesh.mesh_type {
                    engine_ecs_core::MeshType::Cube => (egui::Color32::RED, "⬜", 15.0),
                    engine_ecs_core::MeshType::Sphere => (egui::Color32::GREEN, "⚫", 15.0),
                    engine_ecs_core::MeshType::Plane => (egui::Color32::GRAY, "▭", 20.0),
                    _ => (egui::Color32::WHITE, "📦", 10.0),
                }
            } else {
                (egui::Color32::YELLOW, "📍", 8.0)
            };
            
            // Highlight if selected
            let is_selected = self.selected_entity == Some(entity);
            let final_color = if is_selected {
                egui::Color32::YELLOW
            } else {
                color
            };
            
            // Draw object
            painter.circle_filled(screen_pos, size, final_color);
            
            // Draw selection outline
            if is_selected {
                painter.circle_stroke(
                    screen_pos,
                    size + 3.0,
                    egui::Stroke::new(2.0, egui::Color32::WHITE)
                );
            }
            
            // Draw label
            painter.text(
                screen_pos + egui::vec2(size + 5.0, -size),
                egui::Align2::LEFT_CENTER,
                format!("{} {}", icon, name),
                egui::FontId::proportional(12.0),
                final_color
            );
        }
        
        // Draw sprite objects - using safe iteration approach
        for (entity, _transform) in self.world.query::<Read<Transform>>().iter() {
            // Check if this entity has a SpriteRenderer component
            if let Some(sprite_renderer) = self.world.get_component::<SpriteRenderer>(entity) {
                let transform = _transform; // We already have transform from the query
                
                // Project sprite position to 2D screen coordinates
                let scale = 50.0; // Pixels per world unit
                let screen_x = center.x + transform.position[0] * scale;
                let screen_y = center.y - transform.position[2] * scale; // Z becomes Y in screen space
                let screen_pos = egui::pos2(screen_x, screen_y);
                
                // Calculate sprite size in screen space
                let world_scale = (transform.scale[0] + transform.scale[1]) * 0.5; // Average X and Y scale
                let sprite_size = egui::vec2(32.0 * world_scale, 32.0 * world_scale); // Base size 32x32 pixels
                
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
        
        // Draw camera view indicator
        if let Some(camera_entity) = self.selected_entity {
            if self.world.get_component::<Camera>(camera_entity).is_some() {
                if let Some(camera_transform) = self.world.get_component::<Transform>(camera_entity) {
                    let scale = 50.0;
                    let camera_screen_x = center.x + camera_transform.position[0] * scale;
                    let camera_screen_y = center.y - camera_transform.position[2] * scale;
                    let camera_pos = egui::pos2(camera_screen_x, camera_screen_y);
                    
                    // Draw camera view frustum (simplified)
                    let frustum_width = 80.0;
                    let frustum_height = 60.0;
                    let frustum_rect = egui::Rect::from_center_size(
                        camera_pos + egui::vec2(40.0, 0.0),
                        egui::vec2(frustum_width, frustum_height)
                    );
                    
                    painter.rect_stroke(
                        frustum_rect,
                        egui::Rounding::ZERO,
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(0, 255, 255, 150))
                    );
                    
                    // Draw view direction line
                    painter.line_segment(
                        [camera_pos, frustum_rect.left_center()],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255))
                    );
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
    fn find_main_camera_entity(&self) -> Option<EntityV2> {
        // Look for entity with Camera component that has is_main = true
        for (entity, _transform) in self.world.query::<Read<Transform>>().iter() {
            if let Some(camera) = self.world.get_component::<Camera>(entity) {
                if camera.is_main {
                    return Some(entity);
                }
            }
        }
        
        // If no main camera found, return the first camera entity
        for (entity, _transform) in self.world.query::<Read<Transform>>().iter() {
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
        for (entity, transform) in self.world.query::<Read<Transform>>().iter() {
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
            } else if let Some(_mesh) = self.world.get_component::<engine_ecs_core::Mesh>(entity) {
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
        // Game View header
        ui.horizontal(|ui| {
            ui.label("🎮 Game View");
            
            ui.separator();
            
            // Aspect ratio selector  
            ui.label("Aspect:");
            egui::ComboBox::from_id_source("game_view_aspect")
                .selected_text("16:9")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut "", "16:9", "16:9");
                    ui.selectable_value(&mut "", "4:3", "4:3");
                    ui.selectable_value(&mut "", "Free", "Free Aspect");
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔊").on_hover_text("Audio toggle").clicked() {
                    self.console_messages.push(ConsoleMessage::info("🔊 Game audio toggled"));
                }
                if ui.button("📊").on_hover_text("Stats").clicked() {
                    self.console_messages.push(ConsoleMessage::info("📊 Game view stats"));
                }
            });
        });
        
        ui.separator();
        
        // Main game view area
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::hover());
        
        if self.play_state == PlayState::Editing {
            // Show "Press Play" message when not in play mode
            ui.allocate_ui_at_rect(response.rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Press Play button to see game from camera");
                        ui.small("This view shows what the player will see");
                    });
                });
            });
        } else {
            // Render from main camera perspective when playing
            self.render_camera_perspective(ui, response.rect);
        }
    }
    
    fn show_console_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Output Log");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🧹 Clear").clicked() {
                    self.console_messages.clear();
                    self.console_messages.push(ConsoleMessage::info("🧹 Console cleared"));
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.console_messages {
                    let color = match message.level {
                        LogLevel::Info => egui::Color32::WHITE,
                        LogLevel::Warning => egui::Color32::YELLOW,
                        LogLevel::Error => egui::Color32::RED,
                    };
                    
                    ui.colored_label(color, &message.text);
                }
            });
    }
    
    fn show_project_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Asset Browser");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").on_hover_text("Refresh assets").clicked() {
                    self.console_messages.push(ConsoleMessage::info("🔄 Refreshing project assets"));
                }
                if ui.button("➕").on_hover_text("Create new asset").clicked() {
                    self.console_messages.push(ConsoleMessage::info("➕ Create asset menu"));
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let assets = self.project_assets.clone();
            for asset in &assets {
                self.show_project_asset(ui, asset);
            }
        });
    }
    
    fn show_project_asset(&mut self, ui: &mut egui::Ui, asset: &ProjectAsset) {
        match &asset.children {
            Some(children) => {
                // Folder with children
                ui.collapsing(&asset.name, |ui| {
                    for child in children {
                        self.show_project_asset(ui, child);
                    }
                });
            }
            None => {
                // File asset
                if ui.selectable_label(false, &asset.name).clicked() {
                    self.console_messages.push(ConsoleMessage::info(&format!("📄 Selected asset: {}", asset.name)));
                }
            }
        }
    }
}

/// Different types of dockable panels
#[derive(Debug, Clone, Copy, PartialEq)]
enum PanelType {
    Hierarchy,
    Inspector,
    SceneView,
    GameView,
    Console,
    Project,
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

/// Project asset representation
#[derive(Clone)]
struct ProjectAsset {
    name: String,
    children: Option<Vec<ProjectAsset>>,
}

impl ProjectAsset {
    fn file(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: None,
        }
    }
    
    fn folder(name: &str, children: Vec<ProjectAsset>) -> Self {
        Self {
            name: name.to_string(),
            children: Some(children),
        }
    }
}

/// Console message types
#[derive(Clone)]
struct ConsoleMessage {
    text: String,
    level: LogLevel,
}

#[derive(Clone)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

impl ConsoleMessage {
    fn info(text: &str) -> Self {
        Self {
            text: text.to_string(),
            level: LogLevel::Info,
        }
    }
    
    fn _warning(text: &str) -> Self {
        Self {
            text: text.to_string(),
            level: LogLevel::Warning,
        }
    }
    
    fn _error(text: &str) -> Self {
        Self {
            text: text.to_string(),
            level: LogLevel::Error,
        }
    }
}

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