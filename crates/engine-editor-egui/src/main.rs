// Unity-style game editor built with EGUI and dockable panels
// Provides a modern, responsive Unity-like interface with drag-and-drop docking

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};
use engine_core::{Transform, WorldV2, EntityV2, Read, Write, Name, Visibility, Camera, Light};

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
    world: WorldV2,
    selected_entity: Option<EntityV2>,
    
    // Editor state
    selected_object: Option<String>,
    console_messages: Vec<ConsoleMessage>,
    
    // Panel data
    hierarchy_objects: Vec<HierarchyObject>,
    project_assets: Vec<ProjectAsset>,
    
    // UI state
    scene_view_active: bool,
    show_add_component_dialog: bool,
}

impl UnityEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create Unity-style dock layout
        let mut dock_state = DockState::new(vec![PanelType::SceneView]);
        
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
        
        // Create some entities with Transform components
        let camera_entity = world.spawn();
        world.add_component(camera_entity, Transform {
            position: [0.0, 0.0, 5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        
        let cube_entity = world.spawn();
        world.add_component(cube_entity, Transform {
            position: [1.0, 0.0, 0.0],
            rotation: [0.0, 45.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        
        let sphere_entity = world.spawn();
        world.add_component(sphere_entity, Transform {
            position: [-1.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.5, 1.5, 1.5],
        }).unwrap();
        
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
            scene_view_active: true,
            show_add_component_dialog: false,
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
                    // Reset to Unity-style layout
                    let mut dock_state = DockState::new(vec![PanelType::SceneView]);
                    
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
                
                let component_str = if components.is_empty() { "-".to_string() } else { components.join("") };
                let label = format!("🎲 Entity {} [{}]", entity.id(), component_str);
                
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