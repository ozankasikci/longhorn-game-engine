// ECS-based editor state that bridges the World with the GTK interface

use engine_core::{World, Entity, Transform, Mesh, MeshType, Camera, Name, Light, LightType};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleMessageType {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ConsoleMessage {
    pub message: String,
    pub message_type: ConsoleMessageType,
    pub timestamp: std::time::Instant,
}

pub struct EcsEditorState {
    pub world: World,
    pub selected_entity: Option<Entity>,
    pub console_messages: Vec<ConsoleMessage>,
    pub scene_name: String,
    
    // Panel visibility
    pub hierarchy_open: bool,
    pub inspector_open: bool,
    pub project_open: bool,
    pub console_open: bool,
    
    // Scene view state
    pub scene_pan: [f32; 2],
    pub scene_zoom: f32,
    
    // Runtime state
    pub is_playing: bool,
}

impl Default for EcsEditorState {
    fn default() -> Self {
        let mut state = Self {
            world: World::new(),
            selected_entity: None,
            console_messages: Vec::new(),
            scene_name: "Untitled Scene".to_string(),
            hierarchy_open: true,
            inspector_open: true,
            project_open: true,
            console_open: true,
            scene_pan: [0.0, 0.0],
            scene_zoom: 1.0,
            is_playing: false,
        };
        
        // Create default scene objects
        state.add_default_objects();
        state
    }
}

impl EcsEditorState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_default_objects(&mut self) {
        // Main Camera
        let camera_entity = self.world.spawn();
        self.world.add_component(camera_entity, Camera { 
            fov: 60.0, 
            near: 0.1, 
            far: 1000.0, 
            is_main: true 
        }).unwrap();
        self.world.add_component(camera_entity, Transform {
            position: [0.0, 0.0, 5.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        self.world.add_component(camera_entity, Name::new("Main Camera")).unwrap();
        
        // Directional Light
        let light_entity = self.world.spawn();
        self.world.add_component(light_entity, Light {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        }).unwrap();
        self.world.add_component(light_entity, Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [50.0, -30.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }).unwrap();
        self.world.add_component(light_entity, Name::new("Directional Light")).unwrap();
        
        self.log_info("Scene initialized with default objects");
    }
    
    pub fn create_empty_object(&mut self, name: String) -> Entity {
        let entity = self.world.spawn();
        self.world.add_component(entity, Name::new(name.clone())).unwrap();
        self.world.add_component(entity, Transform::default()).unwrap();
        self.log_info(&format!("Created empty object: {}", name));
        entity
    }
    
    pub fn create_cube(&mut self, name: String) -> Entity {
        let entity = self.world.spawn();
        self.world.add_component(entity, Name::new(name.clone())).unwrap();
        self.world.add_component(entity, Transform::default()).unwrap();
        self.world.add_component(entity, Mesh { mesh_type: MeshType::Cube }).unwrap();
        self.log_info(&format!("Created cube: {}", name));
        entity
    }
    
    pub fn create_sphere(&mut self, name: String) -> Entity {
        let entity = self.world.spawn();
        self.world.add_component(entity, Name::new(name.clone())).unwrap();
        self.world.add_component(entity, Transform::default()).unwrap();
        self.world.add_component(entity, Mesh { mesh_type: MeshType::Sphere }).unwrap();
        self.log_info(&format!("Created sphere: {}", name));
        entity
    }
    
    pub fn delete_entity(&mut self, entity: Entity) -> bool {
        let name = self.world.get_component::<Name>(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));
            
        if self.world.remove_entity(entity) {
            self.log_info(&format!("Deleted entity: {}", name));
            if self.selected_entity == Some(entity) {
                self.selected_entity = None;
            }
            true
        } else {
            false
        }
    }
    
    pub fn select_entity(&mut self, entity: Entity) -> bool {
        if self.world.get_component::<Name>(entity).is_some() {
            self.selected_entity = Some(entity);
            if let Some(name) = self.world.get_component::<Name>(entity) {
                self.log_info(&format!("Selected: {}", name.name));
            }
            true
        } else {
            false
        }
    }
    
    // Get all entities with names for hierarchy display
    pub fn get_named_entities(&self) -> Vec<(Entity, String)> {
        self.world.query::<Name>()
            .map(|(entity, name)| (entity, name.name.clone()))
            .collect()
    }
    
    // Get the selected entity's name
    pub fn get_selected_name(&self) -> Option<String> {
        self.selected_entity
            .and_then(|entity| self.world.get_component::<Name>(entity))
            .map(|name| name.name.clone())
    }
    
    // Get the selected entity's transform for inspector
    pub fn get_selected_transform(&self) -> Option<Transform> {
        self.selected_entity
            .and_then(|entity| self.world.get_component::<Transform>(entity))
            .cloned()
    }
    
    // Update the selected entity's transform from inspector
    pub fn update_selected_transform(&mut self, transform: Transform) -> bool {
        if let Some(entity) = self.selected_entity {
            if let Some(current_transform) = self.world.get_component_mut::<Transform>(entity) {
                *current_transform = transform;
                self.log_info("Transform updated");
                return true;
            }
        }
        false
    }
    
    // Play/stop controls
    pub fn play(&mut self) {
        self.is_playing = true;
        self.log_info("Game started");
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.log_info("Game stopped");
    }
    
    pub fn pause(&mut self) {
        self.is_playing = false;
        self.log_info("Game paused");
    }
    
    // Console logging
    pub fn log_info(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Info,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn log_warning(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Warning,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn log_error(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage {
            message: message.to_string(),
            message_type: ConsoleMessageType::Error,
            timestamp: std::time::Instant::now(),
        });
    }
    
    pub fn clear_console(&mut self) {
        self.console_messages.clear();
        self.log_info("Console cleared");
    }
    
    pub fn entity_count(&self) -> usize {
        self.world.entity_count()
    }
    
    pub fn message_count(&self) -> usize {
        self.console_messages.len()
    }
}