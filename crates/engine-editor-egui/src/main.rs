// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};

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
    
    // Editor state
    selected_object: Option<String>,
    console_messages: Vec<ConsoleMessage>,
    
    // Panel data
    hierarchy_objects: Vec<HierarchyObject>,
    project_assets: Vec<ProjectAsset>,
    
    // UI state
    scene_view_active: bool,
}

impl UnityEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create initial dock layout similar to Unity
        let dock_state = DockState::new(vec![
            PanelType::SceneView,
            PanelType::Hierarchy, 
            PanelType::Inspector,
            PanelType::Console,
            PanelType::Project,
        ]);
        
        Self {
            dock_state,
            selected_object: None,
            console_messages: vec![
                ConsoleMessage::info("🎮 Unity Editor initialized with dockable panels"),
                ConsoleMessage::info("✅ EGUI docking system active"),
                ConsoleMessage::info("🚀 Drag panel tabs to rearrange layout"),
                ConsoleMessage::info("💡 Try dragging tabs to different areas to dock them"),
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
            scene_view_active: true,
        }
    }
}

impl eframe::App for UnityEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply custom styling
        apply_unity_style(ctx);
        
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
                ui.separator();
                if ui.button("Reset Layout").clicked() {
                    self.dock_state = DockState::new(vec![
                        PanelType::SceneView,
                        PanelType::Hierarchy, 
                        PanelType::Inspector,
                        PanelType::Console,
                        PanelType::Project,
                    ]);
                    self.console_messages.push(ConsoleMessage::info("🔄 Layout reset to default"));
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
            
            // Play controls
            if ui.button("▶️").on_hover_text("Play").clicked() {
                self.console_messages.push(ConsoleMessage::info("▶️ Game started"));
            }
            if ui.button("⏸️").on_hover_text("Pause").clicked() {
                self.console_messages.push(ConsoleMessage::info("⏸️ Game paused"));
            }
            if ui.button("⏹️").on_hover_text("Stop").clicked() {
                self.console_messages.push(ConsoleMessage::info("⏹️ Game stopped"));
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
            ui.label("Scene Objects");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕").on_hover_text("Create new object").clicked() {
                    self.console_messages.push(ConsoleMessage::info("➕ Create object menu"));
                }
            });
        });
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let objects = self.hierarchy_objects.clone();
            for object in &objects {
                self.show_hierarchy_object(ui, object);
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
        ui.label("Object Inspector");
        ui.separator();
        
        if let Some(ref selected) = self.selected_object {
            ui.label(format!("Selected: {}", selected));
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Transform component (always present)
                ui.collapsing("📐 Transform", |ui| {
                    egui::Grid::new("transform_grid").show(ui, |ui| {
                        ui.label("Position:");
                        ui.end_row();
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut 0.0f32).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut 0.0f32).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut 0.0f32).speed(0.1));
                        ui.end_row();
                    });
                });
                
                // Type-specific components
                if selected.contains("Camera") {
                    ui.collapsing("📷 Camera", |ui| {
                        ui.label("Field of View:");
                        ui.add(egui::Slider::new(&mut 60.0f32, 1.0..=179.0).suffix("°"));
                    });
                } else if selected.contains("Light") {
                    ui.collapsing("💡 Light", |ui| {
                        ui.label("Color:");
                        let mut color = [1.0, 1.0, 1.0];
                        ui.color_edit_button_rgb(&mut color);
                        ui.label("Intensity:");
                        ui.add(egui::Slider::new(&mut 1.0f32, 0.0..=8.0));
                    });
                } else {
                    ui.collapsing("🎨 Mesh Renderer", |ui| {
                        ui.label("Material:");
                        egui::ComboBox::from_label("")
                            .selected_text("Default")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "", "Default", "Default");
                                ui.selectable_value(&mut "", "Wood", "Wood");
                                ui.selectable_value(&mut "", "Metal", "Metal");
                            });
                    });
                }
                
                ui.separator();
                if ui.button("➕ Add Component").clicked() {
                    self.console_messages.push(ConsoleMessage::info("➕ Add Component dialog"));
                }
            });
        } else {
            ui.label("No object selected");
            ui.label("Select an object in the Hierarchy to view its properties.");
        }
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
            ui.centered_and_justified(|ui| {
                if self.scene_view_active {
                    ui.vertical_centered(|ui| {
                        ui.label("🎨 Scene View");
                        ui.label("Your 3D scene editor");
                        ui.small("Drag objects from hierarchy • Use transform tools");
                        
                        if let Some(ref selected) = self.selected_object {
                            ui.separator();
                            ui.label(format!("Selected: {}", selected));
                        }
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label("🎮 Game View");
                        ui.label("Runtime preview");
                        ui.small("Press Play to see your game");
                    });
                }
            });
        });
        
        // Handle scene interactions
        if response.dragged() {
            self.console_messages.push(ConsoleMessage::info("🖱️ Scene view interaction"));
        }
    }
    
    fn show_game_view(&mut self, ui: &mut egui::Ui) {
        self.scene_view_active = false;
        self.show_scene_view(ui);
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