// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

mod editor_state;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};
use engine_ecs_core::{Transform, WorldV2, EntityV2, Read, Write, Name, Visibility, Light, Sprite, SpriteRenderer, Canvas, Material};
use engine_camera::{CameraComponent, CameraType, Viewport, Camera, Camera2D};
use editor_state::{EditorState, GameObject, ConsoleMessage, ConsoleMessageType};
use std::io::Write as IoWrite;

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

/// Scene navigation state for Unity/Unreal style camera controls
#[derive(Debug, Clone)]
pub struct SceneNavigation {
    pub enabled: bool,
    pub is_navigating: bool,
    pub movement_speed: f32,
    pub rotation_sensitivity: f32,
    pub fast_movement_multiplier: f32,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub scene_camera_transform: Transform,
}

impl Default for SceneNavigation {
    fn default() -> Self {
        Self {
            enabled: true,
            is_navigating: false,
            movement_speed: 5.0,                    // Units per second
            rotation_sensitivity: 0.005,            // Radians per pixel - increased for better responsiveness
            fast_movement_multiplier: 3.0,          // Shift speed boost
            last_mouse_pos: None,
            scene_camera_transform: Transform {
                position: [0.0, 2.0, 5.0],          // Default camera position
                rotation: [0.0, 0.0, 0.0],          // Looking forward
                scale: [1.0, 1.0, 1.0],
            },
        }
    }
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Editing
    }
}

/// Scene manipulation tool types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneTool {
    Select,   // Q - Selection tool (default)
    Move,     // W - Move tool with XYZ gizmo
    Rotate,   // E - Rotation tool (future)
    Scale,    // R - Scale tool (future)
}

impl Default for SceneTool {
    fn default() -> Self {
        Self::Select
    }
}

/// Gizmo axis selection for movement constraints
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoAxis {
    X,   // Red axis - Left/Right
    Y,   // Green axis - Up/Down  
    Z,   // Blue axis - Forward/Backward
}

/// Gizmo plane selection for planar movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoPlane {
    XY,  // Blue square - Z locked
    XZ,  // Green square - Y locked
    YZ,  // Red square - X locked
}

/// Gizmo component that can be interacted with
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoComponent {
    Axis(GizmoAxis),
    Plane(GizmoPlane),
    Center,  // Screen-space movement
}

/// Current gizmo interaction state
#[derive(Debug, Clone)]
pub enum GizmoInteractionState {
    Idle,                                    // No interaction
    Hovering(GizmoComponent),               // Mouse over component
    Dragging {
        component: GizmoComponent,
        start_mouse_pos: egui::Pos2,
        start_object_pos: [f32; 3],
    },
}

impl Default for GizmoInteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Move gizmo for 3D object manipulation
#[derive(Debug, Clone)]
pub struct MoveGizmo {
    /// World position of the gizmo (object center)
    position: [f32; 3],
    /// Scale factor based on camera distance for consistent screen size
    scale: f32,
    /// Current interaction state
    interaction_state: GizmoInteractionState,
    /// Whether gizmo is visible and active
    enabled: bool,
}

impl MoveGizmo {
    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            scale: 1.0,
            interaction_state: GizmoInteractionState::default(),
            enabled: true,
        }
    }
    
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }
    
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
    
    pub fn is_interacting(&self) -> bool {
        matches!(self.interaction_state, GizmoInteractionState::Dragging { .. })
    }
    
    pub fn get_hovered_component(&self) -> Option<GizmoComponent> {
        match &self.interaction_state {
            GizmoInteractionState::Hovering(component) => Some(*component),
            GizmoInteractionState::Dragging { component, .. } => Some(*component),
            _ => None,
        }
    }
}

/// Gizmo system for managing scene manipulation tools
#[derive(Debug, Clone)]
pub struct GizmoSystem {
    /// Currently active scene tool
    active_tool: SceneTool,
    /// Move gizmo instance
    move_gizmo: Option<MoveGizmo>,
    /// Whether gizmos should be rendered
    enabled: bool,
    /// Grid snapping settings
    snap_enabled: bool,
    snap_increment: f32,
}

impl Default for GizmoSystem {
    fn default() -> Self {
        Self {
            active_tool: SceneTool::default(),
            move_gizmo: None,
            enabled: true,
            snap_enabled: false,
            snap_increment: 1.0,
        }
    }
}

impl GizmoSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_active_tool(&mut self, tool: SceneTool) {
        self.active_tool = tool;
    }
    
    pub fn get_active_tool(&self) -> SceneTool {
        self.active_tool
    }
    
    pub fn enable_move_gizmo(&mut self, position: [f32; 3]) {
        if self.active_tool == SceneTool::Move {
            self.move_gizmo = Some(MoveGizmo::new(position));
        }
    }
    
    pub fn disable_move_gizmo(&mut self) {
        self.move_gizmo = None;
    }
    
    pub fn get_move_gizmo_mut(&mut self) -> Option<&mut MoveGizmo> {
        self.move_gizmo.as_mut()
    }
    
    pub fn get_move_gizmo(&self) -> Option<&MoveGizmo> {
        self.move_gizmo.as_ref()
    }
    
    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
    }
    
    pub fn set_snap_increment(&mut self, increment: f32) {
        self.snap_increment = increment;
    }
    
    pub fn apply_snap(&self, value: f32) -> f32 {
        if self.snap_enabled {
            (value / self.snap_increment).round() * self.snap_increment
        } else {
            value
        }
    }
}

/// Ray for 3D intersection testing
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self { origin, direction }
    }
    
    /// Get point along ray at distance t
    pub fn at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
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
    
    // Gizmo system
    gizmo_system: GizmoSystem,
    
    // Scene navigation system
    scene_navigation: SceneNavigation,
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
            gizmo_system: GizmoSystem::new(),
            scene_navigation: SceneNavigation::default(),
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
            
            // Scene manipulation tools
            let current_tool = self.gizmo_system.get_active_tool();
            
            // Selection tool (Q)
            let select_pressed = ui.add(
                egui::Button::new("🎯")
                    .fill(if current_tool == SceneTool::Select { 
                        egui::Color32::from_rgb(100, 150, 255) 
                    } else { 
                        egui::Color32::TRANSPARENT 
                    })
            ).on_hover_text("Select Tool (Q)").clicked();
            
            if select_pressed {
                self.gizmo_system.set_active_tool(SceneTool::Select);
                self.gizmo_system.disable_move_gizmo();
                self.console_messages.push(ConsoleMessage::info("🎯 Selection tool activated"));
            }
            
            // Move tool (W)
            let move_pressed = ui.add(
                egui::Button::new("🔗")
                    .fill(if current_tool == SceneTool::Move { 
                        egui::Color32::from_rgb(100, 150, 255) 
                    } else { 
                        egui::Color32::TRANSPARENT 
                    })
            ).on_hover_text("Move Tool (W)").clicked();
            
            if move_pressed {
                self.gizmo_system.set_active_tool(SceneTool::Move);
                // Enable move gizmo if an entity is selected
                if let Some(entity) = self.selected_entity {
                    if let Some(transform) = self.world.get_component::<Transform>(entity) {
                        self.gizmo_system.enable_move_gizmo(transform.position);
                    }
                }
                self.console_messages.push(ConsoleMessage::info("🔗 Move tool activated"));
            }
            
            // Rotate tool (E) - Future implementation
            let rotate_pressed = ui.add(
                egui::Button::new("🔄")
                    .fill(if current_tool == SceneTool::Rotate { 
                        egui::Color32::from_rgb(100, 150, 255) 
                    } else { 
                        egui::Color32::TRANSPARENT 
                    })
            ).on_hover_text("Rotate Tool (E) - Coming Soon").clicked();
            
            if rotate_pressed {
                self.gizmo_system.set_active_tool(SceneTool::Rotate);
                self.console_messages.push(ConsoleMessage::info("🔄 Rotate tool - coming soon!"));
            }
            
            // Scale tool (R) - Future implementation
            let scale_pressed = ui.add(
                egui::Button::new("📐")
                    .fill(if current_tool == SceneTool::Scale { 
                        egui::Color32::from_rgb(100, 150, 255) 
                    } else { 
                        egui::Color32::TRANSPARENT 
                    })
            ).on_hover_text("Scale Tool (R) - Coming Soon").clicked();
            
            if scale_pressed {
                self.gizmo_system.set_active_tool(SceneTool::Scale);
                self.console_messages.push(ConsoleMessage::info("📐 Scale tool - coming soon!"));
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
            
            // DEBUG: Test transform mutation
            if ui.button("🔧 Test Move").on_hover_text("Debug: Move selected object 1 unit in X").clicked() {
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
                    
                    // Update gizmo position if move tool is active
                    if self.gizmo_system.get_active_tool() == SceneTool::Move {
                        if let Some(transform) = self.world.get_component::<Transform>(entity) {
                            self.gizmo_system.enable_move_gizmo(transform.position);
                        }
                    }
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
                
                // Basic 3D Camera Component
                if ui.button("📷 3D Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if self.world.get_component::<Camera>(entity).is_some() {
                        self.console_messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let camera = Camera::new().with_fov(60.0);
                        
                        match self.world.add_component(entity, camera) {
                            Ok(_) => {
                                self.console_messages.push(ConsoleMessage::info("✅ Added 3D Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add 3D Camera: {}", e)));
                            }
                        }
                    }
                }
                
                // Main Camera shortcut
                if ui.button("🎥 Main Camera Component").clicked() {
                    // Check if entity already has a camera component
                    if self.world.get_component::<Camera>(entity).is_some() {
                        self.console_messages.push(ConsoleMessage::info("⚠️ Entity already has a Camera component"));
                    } else {
                        let camera = Camera::main_camera();
                        
                        match self.world.add_component(entity, camera) {
                            Ok(_) => {
                                self.console_messages.push(ConsoleMessage::info("✅ Added Main Camera component"));
                                self.show_add_component_dialog = false;
                            }
                            Err(e) => {
                                self.console_messages.push(ConsoleMessage::info(&format!("❌ Failed to add Main Camera: {}", e)));
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
        
        // Handle gizmo and scene interactions
        self.handle_scene_input(ui, &response, response.rect);
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
        
        // Get camera position for view transformation
        let camera_pos = self.scene_navigation.scene_camera_transform.position;
        let camera_rot = self.scene_navigation.scene_camera_transform.rotation;
        
        // Draw grid background
        let grid_size = 50.0;
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
        // Use direct component access to ensure we see the latest transform values
        let entities_with_transforms: Vec<_> = self.world.query::<Read<Transform>>().iter().map(|(e, _)| e).collect();
        for entity in entities_with_transforms {
            let Some(transform) = self.world.get_component::<Transform>(entity) else {
                continue;
            };
            
            // Apply camera transformation to object positions
            let relative_pos = [
                transform.position[0] - camera_pos[0],
                transform.position[1] - camera_pos[1], 
                transform.position[2] - camera_pos[2]
            ];
            
            // Apply camera rotation to the relative position
            let yaw = camera_rot[1];
            let pitch = camera_rot[0];
            
            // Rotate around Y-axis (yaw)
            let cos_yaw = yaw.cos();
            let sin_yaw = yaw.sin();
            let rotated_x = relative_pos[0] * cos_yaw - relative_pos[2] * sin_yaw;
            let rotated_z = relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
            
            // Apply pitch rotation (simplified for 2D view)
            let cos_pitch = pitch.cos();
            let sin_pitch = pitch.sin();
            let final_y = relative_pos[1] * cos_pitch - rotated_z * sin_pitch;
            let final_z = relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
            
            // Project 3D position to 2D screen coordinates
            let scale = 50.0; // Pixels per world unit
            let screen_x = view_center.x + rotated_x * scale;
            let screen_y = view_center.y - final_z * scale; // Z becomes Y in screen space
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
                
                // Apply camera transformation to sprite positions
                let relative_pos = [
                    transform.position[0] - camera_pos[0],
                    transform.position[1] - camera_pos[1], 
                    transform.position[2] - camera_pos[2]
                ];
                
                // Apply camera rotation to the relative position (same as regular objects)
                let yaw = camera_rot[1];
                let pitch = camera_rot[0];
                
                // Rotate around Y-axis (yaw)
                let cos_yaw = yaw.cos();
                let sin_yaw = yaw.sin();
                let rotated_x = relative_pos[0] * cos_yaw - relative_pos[2] * sin_yaw;
                let rotated_z = relative_pos[0] * sin_yaw + relative_pos[2] * cos_yaw;
                
                // Apply pitch rotation (simplified for 2D view)
                let cos_pitch = pitch.cos();
                let sin_pitch = pitch.sin();
                let final_y = relative_pos[1] * cos_pitch - rotated_z * sin_pitch;
                let final_z = relative_pos[1] * sin_pitch + rotated_z * cos_pitch;
                
                // Project sprite position to 2D screen coordinates
                let scale = 50.0; // Pixels per world unit
                let screen_x = view_center.x + rotated_x * scale;
                let screen_y = view_center.y - final_z * scale; // Z becomes Y in screen space
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
    fn handle_scene_navigation(&mut self, ui: &egui::Ui, response: &egui::Response, _rect: egui::Rect) {
        // Check for right mouse button to start navigation
        let rmb_down = ui.input(|i| i.pointer.secondary_down());
        let current_mouse_pos = response.hover_pos();
        
        // DEBUG: Log RMB state occasionally
        static mut RMB_COUNTER: u32 = 0;
        unsafe {
            RMB_COUNTER += 1;
            if RMB_COUNTER % 60 == 0 && rmb_down {  // Log every 60 frames when RMB is down
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 DEBUG: RMB down, Nav: {}, Pos: {:?}", 
                    self.scene_navigation.is_navigating, current_mouse_pos
                )));
            }
        }
        
        // Start navigation if RMB is down and we're not already navigating
        if rmb_down && !self.scene_navigation.is_navigating {
            if let Some(mouse_pos) = current_mouse_pos {
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 Starting navigation at ({:.1}, {:.1})", mouse_pos.x, mouse_pos.y
                )));
                self.start_scene_navigation(mouse_pos);
            }
        }
        
        // Check for right mouse button release
        if !rmb_down && self.scene_navigation.is_navigating {
            self.end_scene_navigation();
        }
        
        // Handle navigation input during active navigation
        if self.scene_navigation.is_navigating {
            // DEBUG: Log that we're processing navigation
            static mut NAV_DEBUG_COUNTER: u32 = 0;
            unsafe {
                NAV_DEBUG_COUNTER += 1;
                if NAV_DEBUG_COUNTER % 30 == 0 {  // Log every 30 frames
                    self.console_messages.push(ConsoleMessage::info("🎮 Processing navigation input..."));
                }
            }
            self.process_navigation_input(ui, response);
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
    
    /// Process navigation input during active navigation
    fn process_navigation_input(&mut self, ui: &egui::Ui, response: &egui::Response) {
        // Handle mouse look (rotation)
        if let Some(current_mouse_pos) = response.hover_pos() {
            if let Some(last_mouse_pos) = self.scene_navigation.last_mouse_pos {
                let mouse_delta = current_mouse_pos - last_mouse_pos;
                self.apply_mouse_look(mouse_delta);
            }
            self.scene_navigation.last_mouse_pos = Some(current_mouse_pos);
        }
        
        // Handle WASD movement (will implement in next phase)
        self.handle_wasd_movement(ui);
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
        
        // No rotation limits for scene navigation - complete freedom
        
        // Debug log rotation changes
        if mouse_delta.length() > 0.5 {
            self.console_messages.push(ConsoleMessage::info(&format!(
                "🎥 Mouse delta: ({:.1}, {:.1}) → yaw={:.1}°, pitch={:.1}°",
                mouse_delta.x, mouse_delta.y,
                self.scene_navigation.scene_camera_transform.rotation[1].to_degrees(),
                self.scene_navigation.scene_camera_transform.rotation[0].to_degrees()
            )));
        }
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
        
        // DEBUG: Always log WASD state during navigation
        let (w, a, s, d, q, e) = ui.input(|i| {
            (i.key_down(egui::Key::W), i.key_down(egui::Key::A), i.key_down(egui::Key::S), 
             i.key_down(egui::Key::D), i.key_down(egui::Key::Q), i.key_down(egui::Key::E))
        });
        
        static mut DEBUG_COUNTER: u32 = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            if DEBUG_COUNTER % 15 == 0 {  // Log every 15 frames
                self.console_messages.push(ConsoleMessage::info(&format!(
                    "🎮 WASD DEBUG: W:{} A:{} S:{} D:{} Q:{} E:{} | speed:{:.1} delta:{:.4} actual:{:.4}",
                    w, a, s, d, q, e, movement_speed, delta_time, actual_speed
                )));
            }
        }
        
        
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
        } else {
            // DEBUG: Log when keys are pressed but no movement happens
            let any_key_pressed = ui.input(|i| {
                i.key_down(egui::Key::W) || i.key_down(egui::Key::A) || 
                i.key_down(egui::Key::S) || i.key_down(egui::Key::D) ||
                i.key_down(egui::Key::Q) || i.key_down(egui::Key::E)
            });
            
            if any_key_pressed {
                static mut KEY_DEBUG_COUNTER: u32 = 0;
                unsafe {
                    KEY_DEBUG_COUNTER += 1;
                    if KEY_DEBUG_COUNTER % 30 == 0 {  // Log every 30 frames
                        self.console_messages.push(ConsoleMessage::info(&format!(
                            "🎮 Keys pressed but no movement - Delta:{:.3} Speed:{:.1}",
                            delta_time, movement_speed
                        )));
                    }
                }
            }
        }
    }
    
    /// Transform movement vector by camera orientation for relative movement
    fn transform_movement_by_camera(&self, movement: [f32; 3]) -> [f32; 3] {
        let rotation = &self.scene_navigation.scene_camera_transform.rotation;
        let yaw = rotation[1]; // Y-axis rotation
        
        // Calculate forward and right vectors based on camera yaw
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        
        // Forward vector (camera's -Z direction in world space)
        let forward = [-sin_yaw, 0.0, -cos_yaw];
        // Right vector (camera's +X direction in world space)
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
                        gizmo.interaction_state = GizmoInteractionState::Dragging {
                            component,
                            start_mouse_pos: mouse_pos,
                            start_object_pos: transform.position,
                        };
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
                        gizmo.interaction_state = GizmoInteractionState::Idle;
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
                        "🔧 Gizmo state: {:?}", gizmo.interaction_state
                    )));
                    
                    if let GizmoInteractionState::Dragging { component, start_mouse_pos, start_object_pos } = &gizmo.interaction_state {
                        let mouse_delta = mouse_pos - *start_mouse_pos;
                        new_position = self.calculate_new_position(*start_object_pos, mouse_delta, *component, scale);
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
                    let snap_enabled = self.gizmo_system.snap_enabled;
                    let snap_increment = self.gizmo_system.snap_increment;
                    
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
                        gizmo.interaction_state = if let Some(component) = hit_component {
                            GizmoInteractionState::Hovering(component)
                        } else {
                            GizmoInteractionState::Idle
                        };
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
                
                if ui.button("📋 Copy All").on_hover_text("Copy all logs to clipboard").clicked() {
                    let all_logs = ConsoleMessage::get_all_logs_as_string(&self.console_messages);
                    ui.output_mut(|o| o.copied_text = all_logs);
                    self.console_messages.push(ConsoleMessage::info("📋 Logs copied to clipboard"));
                }
                
                if ui.button("💾 Export").on_hover_text("Export logs to file").clicked() {
                    let all_logs = ConsoleMessage::get_all_logs_as_string(&self.console_messages);
                    match std::fs::write("console_export.log", all_logs) {
                        Ok(_) => self.console_messages.push(ConsoleMessage::info("💾 Logs exported to console_export.log")),
                        Err(e) => self.console_messages.push(ConsoleMessage::info(&format!("❌ Export failed: {}", e))),
                    }
                }
            });
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.console_messages {
                    let color = match message.message_type {
                        ConsoleMessageType::Info => egui::Color32::WHITE,
                        ConsoleMessageType::Warning => egui::Color32::YELLOW,
                        ConsoleMessageType::Error => egui::Color32::RED,
                    };
                    
                    ui.colored_label(color, &message.message);
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