// Menu bar - Longhorn-style editor menu bar

use eframe::egui;
use egui_dock::{DockState, NodeIndex};
use crate::editor_state::ConsoleMessage;
use crate::types::PanelType;

pub struct MenuBar {
    console_messages: Vec<ConsoleMessage>,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            console_messages: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, dock_state: &mut DockState<PanelType>) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Scene").clicked() {
                    messages.push(ConsoleMessage::info("📄 Created new scene"));
                    ui.close_menu();
                }
                if ui.button("Open Scene").clicked() {
                    messages.push(ConsoleMessage::info("📂 Opening scene..."));
                    ui.close_menu();
                }
                if ui.button("Save Scene").clicked() {
                    messages.push(ConsoleMessage::info("💾 Scene saved"));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
            
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    messages.push(ConsoleMessage::info("↶ Undo"));
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    messages.push(ConsoleMessage::info("↷ Redo"));
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Window", |ui| {
                ui.label("Dockable Panels:");
                ui.separator();
                if ui.button("Add Hierarchy Panel").clicked() {
                    dock_state.add_window(vec![PanelType::Hierarchy]);
                    messages.push(ConsoleMessage::info("➕ Added Hierarchy panel"));
                    ui.close_menu();
                }
                if ui.button("Add Inspector Panel").clicked() {
                    dock_state.add_window(vec![PanelType::Inspector]);
                    messages.push(ConsoleMessage::info("➕ Added Inspector panel"));
                    ui.close_menu();
                }
                if ui.button("Add Console Panel").clicked() {
                    dock_state.add_window(vec![PanelType::Console]);
                    messages.push(ConsoleMessage::info("➕ Added Console panel"));
                    ui.close_menu();
                }
                if ui.button("Add Project Panel").clicked() {
                    dock_state.add_window(vec![PanelType::Project]);
                    messages.push(ConsoleMessage::info("➕ Added Project panel"));
                    ui.close_menu();
                }
                if ui.button("Add Game View Panel").clicked() {
                    dock_state.add_window(vec![PanelType::GameView]);
                    messages.push(ConsoleMessage::info("➕ Added Game View panel"));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Reset Layout").clicked() {
                    // Reset to Longhorn-style layout with Scene and Game views
                    let mut new_dock_state = DockState::new(vec![PanelType::SceneView, PanelType::GameView]);
                    
                    // Add Hierarchy to the left
                    let [_main, _left] = new_dock_state.main_surface_mut().split_left(
                        NodeIndex::root(),
                        0.2,
                        vec![PanelType::Hierarchy]
                    );
                    
                    // Add Inspector to the right
                    let [_main, _right] = new_dock_state.main_surface_mut().split_right(
                        NodeIndex::root(),
                        0.8,
                        vec![PanelType::Inspector]
                    );
                    
                    // Add Console to the bottom
                    let [_main, _bottom] = new_dock_state.main_surface_mut().split_below(
                        NodeIndex::root(),
                        0.7,
                        vec![PanelType::Console]
                    );
                    
                    *dock_state = new_dock_state;
                    messages.push(ConsoleMessage::info("🔄 Layout reset to Longhorn default"));
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
        
        messages
    }
}