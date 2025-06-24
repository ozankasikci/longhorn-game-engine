// Editor State - Application-specific state management for the Longhorn-style editor

use crate::types::SceneObject;
use engine_components_3d::Transform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub enum ConsoleMessage {
    Message {
        message: String,
        message_type: ConsoleMessageType,
        timestamp: instant::Instant,
    },
    UserAction(String),
}

impl ConsoleMessage {
    pub fn info(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Info,
            timestamp: instant::Instant::now(),
        }
    }

    pub fn warning(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Warning,
            timestamp: instant::Instant::now(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Error,
            timestamp: instant::Instant::now(),
        }
    }

    pub fn get_all_logs_as_string(messages: &[ConsoleMessage]) -> String {
        messages
            .iter()
            .filter_map(|msg| match msg {
                ConsoleMessage::Message {
                    message,
                    message_type,
                    timestamp,
                } => {
                    let elapsed = timestamp.elapsed().as_secs();
                    Some(format!("[{}s] {:?}: {}", elapsed, message_type, message))
                }
                ConsoleMessage::UserAction(_) => None,
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

        // Scene initialized with default objects
    }

    pub fn create_object(&mut self, name: &str, _object: SceneObject) -> u32 {
        let obj = GameObject::new(self.next_object_id, name.to_string());
        let id = self.next_object_id;
        self.scene_objects.insert(id, obj);
        self.next_object_id += 1;
        // Object created
        id
    }

    pub fn delete_object(&mut self, id: u32) -> bool {
        if let Some(_obj) = self.scene_objects.remove(&id) {
            // Object deleted
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
            if let Some(_obj) = self.scene_objects.get(&id) {
                // Object selected
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

    pub fn log_info(&mut self, _message: &str) {
        // Logging disabled
    }

    pub fn log_warning(&mut self, _message: &str) {
        // Logging disabled
    }

    pub fn log_error(&mut self, _message: &str) {
        // Logging disabled
    }

    pub fn clear_console(&mut self) {
        self.console_messages.clear();
        // Console cleared
    }

    pub fn object_count(&self) -> usize {
        self.scene_objects.len()
    }

    pub fn message_count(&self) -> usize {
        self.console_messages.len()
    }
}
