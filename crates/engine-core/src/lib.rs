// Engine Core - Core data structures and systems for the mobile game engine

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod ecs;
pub mod ecs_v2; // New data-oriented ECS
pub mod math;
pub mod time;
pub mod memory;
pub mod components;

// Re-export ECS types (legacy)
pub use ecs::{Entity, Component, World};

// Re-export new ECS types
pub use ecs_v2::{Entity as EntityV2, Component as ComponentV2, World as WorldV2, ArchetypeId};

// Re-export common components
pub use components::{Mesh, MeshType, Material, Name, Visibility, Camera, Light, LightType};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3], 
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

// Make Transform a component for both ECS systems
impl Component for Transform {}
impl ComponentV2 for Transform {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    pub transform: Transform,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
    pub active: bool,
}

impl GameObject {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            transform: Transform::default(),
            children: Vec::new(),
            parent: None,
            active: true,
        }
    }
}

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

pub struct EditorState {
    pub scene_objects: HashMap<u32, GameObject>,
    pub selected_object: Option<u32>,
    pub next_object_id: u32,
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
}

impl Default for EditorState {
    fn default() -> Self {
        let mut state = Self {
            scene_objects: HashMap::new(),
            selected_object: None,
            next_object_id: 1,
            console_messages: Vec::new(),
            scene_name: "Untitled Scene".to_string(),
            hierarchy_open: true,
            inspector_open: true,
            project_open: true,
            console_open: true,
            scene_pan: [0.0, 0.0],
            scene_zoom: 1.0,
        };
        
        // Create default scene objects
        state.add_default_objects();
        state
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_default_objects(&mut self) {
        // Main Camera
        let camera = GameObject::new(self.next_object_id, "Main Camera".to_string());
        self.scene_objects.insert(self.next_object_id, camera);
        self.next_object_id += 1;
        
        // Directional Light
        let mut light = GameObject::new(self.next_object_id, "Directional Light".to_string());
        light.transform.rotation = [50.0, -30.0, 0.0];
        self.scene_objects.insert(self.next_object_id, light);
        self.next_object_id += 1;
        
        self.log_info("Scene initialized with default objects");
    }
    
    pub fn create_object(&mut self, name: String) -> u32 {
        let obj = GameObject::new(self.next_object_id, name.clone());
        let id = self.next_object_id;
        self.scene_objects.insert(id, obj);
        self.next_object_id += 1;
        self.log_info(&format!("Created object: {}", name));
        id
    }
    
    pub fn delete_object(&mut self, id: u32) -> bool {
        if let Some(obj) = self.scene_objects.remove(&id) {
            self.log_info(&format!("Deleted object: {}", obj.name));
            if self.selected_object == Some(id) {
                self.selected_object = None;
            }
            true
        } else {
            false
        }
    }
    
    pub fn select_object(&mut self, id: u32) -> bool {
        if self.scene_objects.contains_key(&id) {
            self.selected_object = Some(id);
            if let Some(obj) = self.scene_objects.get(&id) {
                self.log_info(&format!("Selected: {}", obj.name));
            }
            true
        } else {
            false
        }
    }
    
    pub fn get_object(&self, id: u32) -> Option<&GameObject> {
        self.scene_objects.get(&id)
    }
    
    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut GameObject> {
        self.scene_objects.get_mut(&id)
    }
    
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
    
    pub fn object_count(&self) -> usize {
        self.scene_objects.len()
    }
    
    pub fn message_count(&self) -> usize {
        self.console_messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_gameobject_creation() {
        let obj = GameObject::new(1, "Test Object".to_string());
        assert_eq!(obj.id, 1);
        assert_eq!(obj.name, "Test Object");
        assert_eq!(obj.transform, Transform::default());
        assert!(obj.children.is_empty());
        assert_eq!(obj.parent, None);
        assert!(obj.active);
    }

    #[test]
    fn test_editor_state_default() {
        let state = EditorState::default();
        assert_eq!(state.selected_object, None);
        assert_eq!(state.next_object_id, 3); // Should be 3 after creating camera and light
        assert_eq!(state.scene_name, "Untitled Scene");
        assert!(state.hierarchy_open);
        assert!(state.inspector_open);
        assert!(state.project_open);
        assert!(state.console_open);
        assert_eq!(state.scene_pan, [0.0, 0.0]);
        assert_eq!(state.scene_zoom, 1.0);
        
        // Should have default objects
        assert_eq!(state.object_count(), 2);
        assert!(state.message_count() > 0); // Should have initialization message
    }

    #[test]
    fn test_object_creation() {
        let mut state = EditorState::new();
        let initial_count = state.object_count();
        
        let cube_id = state.create_object("Cube".to_string());
        assert_eq!(state.object_count(), initial_count + 1);
        
        let cube = state.get_object(cube_id).unwrap();
        assert_eq!(cube.name, "Cube");
        assert_eq!(cube.id, cube_id);
        assert!(cube.active);
    }

    #[test]
    fn test_object_selection() {
        let mut state = EditorState::new();
        let cube_id = state.create_object("Cube".to_string());
        
        // Test successful selection
        assert!(state.select_object(cube_id));
        assert_eq!(state.selected_object, Some(cube_id));
        
        // Test selection of non-existent object
        assert!(!state.select_object(999));
        assert_eq!(state.selected_object, Some(cube_id)); // Should remain unchanged
    }

    #[test]
    fn test_object_deletion() {
        let mut state = EditorState::new();
        let cube_id = state.create_object("Cube".to_string());
        let initial_count = state.object_count();
        
        // Select the object first
        state.select_object(cube_id);
        assert_eq!(state.selected_object, Some(cube_id));
        
        // Delete the object
        assert!(state.delete_object(cube_id));
        assert_eq!(state.object_count(), initial_count - 1);
        assert_eq!(state.selected_object, None); // Should be deselected
        
        // Try to delete non-existent object
        assert!(!state.delete_object(999));
    }

    #[test]
    fn test_console_logging() {
        let mut state = EditorState::new();
        let initial_count = state.message_count();
        
        state.log_info("Test info message");
        state.log_warning("Test warning message");
        state.log_error("Test error message");
        
        assert_eq!(state.message_count(), initial_count + 3);
        
        // Check last three messages
        let messages = &state.console_messages;
        let len = messages.len();
        assert_eq!(messages[len-3].message, "Test info message");
        assert_eq!(messages[len-3].message_type, ConsoleMessageType::Info);
        assert_eq!(messages[len-2].message, "Test warning message");
        assert_eq!(messages[len-2].message_type, ConsoleMessageType::Warning);
        assert_eq!(messages[len-1].message, "Test error message");
        assert_eq!(messages[len-1].message_type, ConsoleMessageType::Error);
    }
}