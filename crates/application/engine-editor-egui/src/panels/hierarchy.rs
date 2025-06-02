// Hierarchy panel - shows entity tree

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Mesh, Light, Visibility};
use engine_components_2d::{SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_camera::{Camera, Camera2D, CameraComponent};
use crate::types::SceneTool;
use crate::editor_state::ConsoleMessage;

pub struct HierarchyPanel {
    console_messages: Vec<ConsoleMessage>,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            console_messages: Vec::new(),
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
                    let entity = world.create_entity();
                    world.add_component(entity, Transform::default()).unwrap();
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
            for (entity, _transform) in world.query::<Transform>() {
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
}

// Legacy hierarchy object structure (not currently used but kept for future reference)
#[derive(Debug, Clone)]
pub struct HierarchyObject {
    pub name: String,
    pub object_type: ObjectType,
    pub children: Option<Vec<HierarchyObject>>,
}

#[derive(Debug, Clone)]
pub enum ObjectType {
    Empty,
    Cube,
    Sphere,
    Light,
    Camera,
}

impl HierarchyObject {
    pub fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            name: name.to_string(),
            object_type,
            children: None,
        }
    }
    
    pub fn with_children(mut self, children: Vec<HierarchyObject>) -> Self {
        self.children = Some(children);
        self
    }
}