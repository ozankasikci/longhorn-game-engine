// Hierarchy panel - shows entity tree

use eframe::egui;
use engine_ecs_core::{World, Entity};
use engine_components_3d::{Transform, Material, Light, Visibility, MeshFilter, MeshRenderer, Mesh, MeshType, LightType};
use engine_components_2d::{SpriteRenderer};
use engine_components_ui::{Canvas, Name};
use engine_renderer_3d::Camera;
use crate::types::{SceneTool, HierarchyObject};
use crate::editor_state::ConsoleMessage;

pub struct HierarchyPanel {
    selected_object: Option<String>,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
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
                ui.menu_button("+", |ui| {
                    ui.set_min_width(150.0);
                    ui.label("3D Objects");
                    ui.separator();
                    
                    if ui.button("Cube").clicked() {
                        create_cube_entity(world);
                        ui.close_menu();
                    }
                    
                    if ui.button("Sphere").clicked() {
                        create_sphere_entity(world);
                        ui.close_menu();
                    }
                    
                    if ui.button("Plane").clicked() {
                        create_plane_entity(world);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    if ui.button("Empty GameObject").clicked() {
                        let entity = world.spawn();
                        world.add_component(entity, Transform::default()).unwrap();
                        world.add_component(entity, Name::new("GameObject")).unwrap();
                        ui.close_menu();
                    }
                });
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
                if world.get_component::<MeshFilter>(entity).is_some() { components.push("M"); }
                if world.get_component::<Material>(entity).is_some() { components.push("Mat"); }
                
                let component_str = if components.is_empty() { "-".to_string() } else { components.join("") };
                
                // Get entity name if available
                let entity_name = if let Some(name) = world.get_component::<Name>(entity) {
                    name.name.clone()
                } else {
                    format!("Entity {}", entity.id())
                };
                
                let label = format!("{} [{}]", entity_name, component_str);
                
                if ui.selectable_label(selected, &label).clicked() {
                    *selected_entity = Some(entity);
                    // Entity selected
                    
                    // Update gizmo position if move tool is active
                    if gizmo_system.get_active_tool() == SceneTool::Move {
                        if let Some(transform) = world.get_component::<Transform>(entity) {
                            gizmo_system.enable_move_gizmo(transform.position);
                        }
                    }
                }
            }
        });
        
        // Return empty messages
        Vec::new()
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
                    // Object selected
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
                    // Settings clicked
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

// Helper functions for creating entities

use engine_resource_core::{ResourceId, ResourceHandle};
use engine_geometry_core::{MeshData, Vertex};
use glam::{Vec3, Vec2};

fn create_cube_entity(world: &mut World) -> Entity {
    
    let entity = world.spawn();
    
    // Add transform at a random position to avoid overlapping
    let offset_x = (world.entity_count() as f32 - 2.0) * 2.0; // Spread cubes horizontally
    world.add_component(entity, Transform {
        position: [offset_x, 0.5, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    }).unwrap();
    
    // Generate cube mesh data
    let mesh_data = create_cube_mesh_data(1.0);
    
    // Create mesh handle (in a real system, this would be managed by a resource manager)
    let mesh_id = 1000 + world.entity_count() as u64; // Unique ID for each cube
    let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(mesh_id));
    
    // Add Mesh component for rendering
    world.add_component(entity, Mesh {
        mesh_type: MeshType::Cube,
    }).unwrap();
    
    // Add MeshFilter component
    world.add_component(entity, MeshFilter::new(mesh_handle)).unwrap();
    
    // Add MeshRenderer component with default material
    world.add_component(entity, MeshRenderer::default()).unwrap();
    
    // Add material component
    world.add_component(entity, Material {
        color: [0.8, 0.2, 0.2, 1.0], // Red cube for new ones
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    
    world.add_component(entity, Visibility::default()).unwrap();
    world.add_component(entity, Name::new(format!("Cube {}", world.entity_count()))).unwrap();
    
    entity
}

fn create_sphere_entity(world: &mut World) -> Entity {
    let entity = world.spawn();
    
    let offset_x = (world.entity_count() as f32 - 2.0) * 2.0;
    world.add_component(entity, Transform {
        position: [offset_x, 0.5, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    }).unwrap();
    
    // Generate sphere mesh data
    let mesh_data = create_sphere_mesh_data(1.0, 16, 32);
    
    // Create mesh handle
    let mesh_id = 2000 + world.entity_count() as u64;
    let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(mesh_id));
    
    // Add Mesh component for rendering
    world.add_component(entity, Mesh {
        mesh_type: MeshType::Sphere,
    }).unwrap();
    
    // Add MeshFilter and MeshRenderer
    world.add_component(entity, MeshFilter::new(mesh_handle)).unwrap();
    world.add_component(entity, MeshRenderer::default()).unwrap();
    
    world.add_component(entity, Material {
        color: [0.2, 0.8, 0.2, 1.0], // Green sphere
        metallic: 0.0,
        roughness: 0.5,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    
    world.add_component(entity, Visibility::default()).unwrap();
    world.add_component(entity, Name::new(format!("Sphere {}", world.entity_count()))).unwrap();
    
    entity
}

fn create_plane_entity(world: &mut World) -> Entity {
    let entity = world.spawn();
    
    world.add_component(entity, Transform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [10.0, 1.0, 10.0], // Large ground plane
    }).unwrap();
    
    // Generate plane mesh data
    let mesh_data = create_plane_mesh_data(10.0, 10.0, 1, 1);
    
    // Create mesh handle
    let mesh_id = 3000 + world.entity_count() as u64;
    let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(mesh_id));
    
    // Add Mesh component for rendering
    world.add_component(entity, Mesh {
        mesh_type: MeshType::Plane,
    }).unwrap();
    
    // Add MeshFilter and MeshRenderer
    world.add_component(entity, MeshFilter::new(mesh_handle)).unwrap();
    world.add_component(entity, MeshRenderer::default()).unwrap();
    
    world.add_component(entity, Material {
        color: [0.5, 0.5, 0.5, 1.0], // Gray plane
        metallic: 0.0,
        roughness: 0.8,
        emissive: [0.0, 0.0, 0.0],
    }).unwrap();
    
    world.add_component(entity, Visibility::default()).unwrap();
    world.add_component(entity, Name::new("Ground Plane")).unwrap();
    
    entity
}

/// Create cube mesh data
fn create_cube_mesh_data(size: f32) -> MeshData {
    let half_size = size * 0.5;
    let vertices = vec![
        // Front face
        Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::Z).with_uv(Vec2::new(0.0, 1.0)),
        
        // Back face
        Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::NEG_Z).with_uv(Vec2::new(0.0, 1.0)),
        
        // Left face
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::NEG_X).with_uv(Vec2::new(0.0, 1.0)),
        
        // Right face
        Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::X).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::X).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::X).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::X).with_uv(Vec2::new(0.0, 1.0)),
        
        // Top face
        Vertex::new(Vec3::new(-half_size, half_size, half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, half_size, -half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, half_size, -half_size)).with_normal(Vec3::Y).with_uv(Vec2::new(0.0, 1.0)),
        
        // Bottom face
        Vertex::new(Vec3::new(-half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, -half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(half_size, -half_size, half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-half_size, -half_size, half_size)).with_normal(Vec3::NEG_Y).with_uv(Vec2::new(0.0, 1.0)),
    ];
    
    let indices = vec![
        // Front face
        0, 1, 2, 0, 2, 3,
        // Back face
        4, 5, 6, 4, 6, 7,
        // Left face
        8, 9, 10, 8, 10, 11,
        // Right face
        12, 13, 14, 12, 14, 15,
        // Top face
        16, 17, 18, 16, 18, 19,
        // Bottom face
        20, 21, 22, 20, 22, 23,
    ];
    
    MeshData::new("Cube".to_string(), vertices, indices)
}

/// Create sphere mesh data
fn create_sphere_mesh_data(radius: f32, rings: u32, sectors: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let ring_step = std::f32::consts::PI / rings as f32;
    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    
    // Generate vertices
    for i in 0..=rings {
        let ring_angle = i as f32 * ring_step;
        let y = radius * ring_angle.cos();
        let ring_radius = radius * ring_angle.sin();
        
        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = ring_radius * sector_angle.cos();
            let z = ring_radius * sector_angle.sin();
            
            let position = Vec3::new(x, y, z);
            let normal = position.normalize();
            let uv = Vec2::new(j as f32 / sectors as f32, i as f32 / rings as f32);
            
            vertices.push(Vertex::new(position).with_normal(normal).with_uv(uv));
        }
    }
    
    // Generate indices
    for i in 0..rings {
        for j in 0..sectors {
            let current = i * (sectors + 1) + j;
            let next = current + sectors + 1;
            
            // Two triangles per quad
            indices.extend([current, next, current + 1]);
            indices.extend([current + 1, next, next + 1]);
        }
    }
    
    MeshData::new("Sphere".to_string(), vertices, indices)
}

/// Create plane mesh data
fn create_plane_mesh_data(width: f32, height: f32, subdivisions_x: u32, subdivisions_y: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Generate vertices
    for y in 0..=subdivisions_y {
        for x in 0..=subdivisions_x {
            let u = x as f32 / subdivisions_x as f32;
            let v = y as f32 / subdivisions_y as f32;
            
            let position = Vec3::new(
                (u - 0.5) * width,
                0.0,
                (v - 0.5) * height,
            );
            
            let normal = Vec3::Y;
            let uv = Vec2::new(u, v);
            
            vertices.push(Vertex::new(position).with_normal(normal).with_uv(uv));
        }
    }
    
    // Generate indices
    for y in 0..subdivisions_y {
        for x in 0..subdivisions_x {
            let i = y * (subdivisions_x + 1) + x;
            
            // Two triangles per quad
            indices.extend([i, i + 1, i + subdivisions_x + 1]);
            indices.extend([i + 1, i + subdivisions_x + 2, i + subdivisions_x + 1]);
        }
    }
    
    MeshData::new("Plane".to_string(), vertices, indices)
}