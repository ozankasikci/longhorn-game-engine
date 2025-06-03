// Hierarchy panel - shows entity tree

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Mesh, Light, Visibility};
use engine_components_2d::{SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_camera::{Camera, Camera2D, CameraComponent};
use crate::types::{SceneTool, HierarchyObject};
use crate::editor_state::ConsoleMessage;

pub struct HierarchyPanel {
    console_messages: Vec<ConsoleMessage>,
    selected_object: Option<String>,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            console_messages: Vec::new(),
            selected_object: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        gizmo_system: &mut crate::types::GizmoSystem,
    ) -> Vec<ConsoleMessage> {
        ui.horizontal(|ui| {
            ui.label("ECS Entities");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕").on_hover_text("Create new entity").clicked() {
                    // Create new entity with ECS v2
                    let entity = world.spawn_with(Transform::default());
                    self.console_messages.push(ConsoleMessage::info(&format!("➕ Created Entity {:?}", entity)));
                }
            });
        });
        ui.separator();
        
        ui.label(format!("🎯 Entity Count: {}", world.entity_count()));
        ui.label(format!("📦 Entities: {}", world.entity_count()));
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Show all entities with Transform components using ECS v2 query
            for (entity, _transform) in world.query_legacy::<Transform>() {
                let selected = *selected_entity == Some(entity);
                
                // Build component indicator string
                let mut components = Vec::new();
                if world.get_component::<Transform>(entity).is_some() { components.push("T"); }
                if world.get_component::<Name>(entity).is_some() { components.push("N"); }
                if world.get_component::<Visibility>(entity).is_some() { components.push("V"); }
                if world.get_component::<Camera>(entity).is_some() { components.push("C"); }
                if world.get_component::<Light>(entity).is_some() { components.push("L"); }
                if world.get_component::<SpriteRenderer>(entity).is_some() { components.push("Spr"); }
                if world.get_component::<Canvas>(entity).is_some() { components.push("Canvas"); }
                if world.get_component::<Camera2D>(entity).is_some() { components.push("C2D"); }
                if world.get_component::<CameraComponent>(entity).is_some() { components.push("Cam"); }
                if world.get_component::<Mesh>(entity).is_some() { components.push("M"); }
                if world.get_component::<Material>(entity).is_some() { components.push("Mat"); }
                
                let component_str = if components.is_empty() { "-".to_string() } else { components.join("") };
                
                // Get entity name if available
                let entity_name = if let Some(name) = world.get_component::<Name>(entity) {
                    name.name.clone()
                } else {
                    format!("Entity {}", entity.id())
                };
                
                let label = format!("📦 {} [{}]", entity_name, component_str);
                
                if ui.selectable_label(selected, &label).clicked() {
                    *selected_entity = Some(entity);
                    self.console_messages.push(ConsoleMessage::info(&format!("🎯 Selected Entity {:?}", entity)));
                    
                    // Update gizmo position if move tool is active
                    if gizmo_system.get_active_tool() == SceneTool::Move {
                        if let Some(transform) = world.get_component::<Transform>(entity) {
                            gizmo_system.enable_move_gizmo(transform.position);
                        }
                    }
                }
            }
        });
        
        // Return any console messages
        let mut messages = Vec::new();
        messages.append(&mut self.console_messages);
        messages
    }
    
    /// Display a hierarchy object tree recursively
    pub fn show_hierarchy_object(&mut self, ui: &mut egui::Ui, object: &HierarchyObject) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        match &object.children {
            Some(children) => {
                // Parent object with children
                ui.collapsing(&object.name, |ui| {
                    for child in children {
                        let child_messages = self.show_hierarchy_object(ui, child);
                        messages.extend(child_messages);
                    }
                });
            }
            None => {
                // Leaf object
                let selected = self.selected_object.as_ref() == Some(&object.name);
                if ui.selectable_label(selected, &object.name).clicked() {
                    self.selected_object = Some(object.name.clone());
                    messages.push(ConsoleMessage::info(&format!("🎯 Selected: {}", object.name)));
                }
            }
        }
        
        messages
    }
    
    /// Show hierarchy objects in the panel (legacy view mode)
    pub fn show_hierarchy_objects(
        &mut self,
        ui: &mut egui::Ui,
        hierarchy_objects: &[HierarchyObject],
    ) -> Vec<ConsoleMessage> {
        let mut messages = Vec::new();
        
        ui.horizontal(|ui| {
            ui.label("Scene Hierarchy");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙️").on_hover_text("Hierarchy settings").clicked() {
                    messages.push(ConsoleMessage::info("🔧 Hierarchy settings not implemented"));
                }
            });
        });
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for object in hierarchy_objects {
                let obj_messages = self.show_hierarchy_object(ui, object);
                messages.extend(obj_messages);
            }
        });
        
        messages
    }
}