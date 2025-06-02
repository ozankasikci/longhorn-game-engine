// Editor State - Application-specific state management for the Unity-style editor

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use engine_ecs_core::Transform;

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

impl ConsoleMessage {
    pub fn info(message: &str) -> Self {
        Self {
            message: message.to_string(),
            message_type: ConsoleMessageType::Info,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn warning(message: &str) -> Self {
        Self {
            message: message.to_string(),
            message_type: ConsoleMessageType::Warning,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            message: message.to_string(),
            message_type: ConsoleMessageType::Error,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn get_all_logs_as_string(messages: &[ConsoleMessage]) -> String {
        messages.iter()
            .map(|msg| {
                let timestamp = msg.timestamp.elapsed().as_secs();
                format!("[{}s] {:?}: {}", timestamp, msg.message_type, msg.message)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
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
        self.console_messages.push(ConsoleMessage::info(message));
    }
    
    pub fn log_warning(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage::warning(message));
    }
    
    pub fn log_error(&mut self, message: &str) {
        self.console_messages.push(ConsoleMessage::error(message));
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