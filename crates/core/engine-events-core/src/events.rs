//! Core event abstractions

use glam::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Unique identifier for event types
pub type EventTypeId = u32;

/// Unique identifier for event instances
pub type EventId = u64;

/// Base event trait that all events must implement
pub trait Event: Send + Sync + std::fmt::Debug + std::any::Any {
    /// Get the type ID for this event
    fn get_type_id(&self) -> EventTypeId;

    /// Get a unique identifier for this event instance
    fn event_id(&self) -> EventId;

    /// Get the timestamp when this event was created
    fn timestamp(&self) -> f64;

    /// Check if this event should be consumed after handling
    fn is_consumable(&self) -> bool {
        true
    }

    /// Get event priority (higher = more priority)
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }

    /// Get the event as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Base event structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseEvent {
    /// Unique event ID
    pub id: EventId,
    /// Event timestamp
    pub timestamp: f64,
    /// Event priority
    pub priority: EventPriority,
    /// Whether event should be consumed after handling
    pub consumable: bool,
    /// Source entity (if applicable)
    pub source_entity: Option<u32>,
    /// Target entity (if applicable)
    pub target_entity: Option<u32>,
}

/// Core engine events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineEvent {
    /// Application lifecycle events
    AppStarted,
    AppShutdown,
    AppPaused,
    AppResumed,

    /// Scene events
    SceneLoaded {
        scene_id: String,
    },
    SceneUnloaded {
        scene_id: String,
    },
    SceneChanged {
        from: String,
        to: String,
    },

    /// Entity lifecycle events
    EntityCreated {
        entity: u32,
    },
    EntityDestroyed {
        entity: u32,
    },
    EntityEnabled {
        entity: u32,
    },
    EntityDisabled {
        entity: u32,
    },

    /// Component events
    ComponentAdded {
        entity: u32,
        component_type: String,
    },
    ComponentRemoved {
        entity: u32,
        component_type: String,
    },
    ComponentChanged {
        entity: u32,
        component_type: String,
    },

    /// Transform events
    TransformChanged {
        entity: u32,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
    },

    /// Collision events
    CollisionEnter {
        entity1: u32,
        entity2: u32,
        point: Vec3,
        normal: Vec3,
    },
    CollisionExit {
        entity1: u32,
        entity2: u32,
    },
    CollisionStay {
        entity1: u32,
        entity2: u32,
        point: Vec3,
        normal: Vec3,
    },

    /// Trigger events
    TriggerEnter {
        trigger: u32,
        other: u32,
    },
    TriggerExit {
        trigger: u32,
        other: u32,
    },
    TriggerStay {
        trigger: u32,
        other: u32,
    },
}

/// Input events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard events
    KeyPressed {
        key: String,
        modifiers: InputModifiers,
    },
    KeyReleased {
        key: String,
        modifiers: InputModifiers,
    },

    /// Mouse events
    MouseButtonPressed {
        button: MouseButton,
        position: Vec2,
    },
    MouseButtonReleased {
        button: MouseButton,
        position: Vec2,
    },
    MouseMoved {
        position: Vec2,
        delta: Vec2,
    },
    MouseScrolled {
        delta: Vec2,
    },

    /// Touch events
    TouchStarted {
        id: u32,
        position: Vec2,
    },
    TouchMoved {
        id: u32,
        position: Vec2,
        delta: Vec2,
    },
    TouchEnded {
        id: u32,
        position: Vec2,
    },
    TouchCancelled {
        id: u32,
        position: Vec2,
    },

    /// Gamepad events
    GamepadConnected {
        id: u32,
    },
    GamepadDisconnected {
        id: u32,
    },
    GamepadButtonPressed {
        id: u32,
        button: String,
    },
    GamepadButtonReleased {
        id: u32,
        button: String,
    },
    GamepadAxisChanged {
        id: u32,
        axis: String,
        value: f32,
    },
}

/// Audio events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Audio playback events
    AudioStarted {
        source: u32,
    },
    AudioStopped {
        source: u32,
    },
    AudioPaused {
        source: u32,
    },
    AudioResumed {
        source: u32,
    },
    AudioFinished {
        source: u32,
    },

    /// Audio system events
    AudioDeviceConnected {
        device_id: String,
    },
    AudioDeviceDisconnected {
        device_id: String,
    },
    AudioDeviceChanged {
        device_id: String,
    },
}

/// Rendering events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderEvent {
    /// Render pipeline events
    FrameStarted,
    FrameEnded,
    RenderPassStarted {
        pass_name: String,
    },
    RenderPassEnded {
        pass_name: String,
    },

    /// Resource events
    TextureLoaded {
        texture_id: String,
    },
    TextureUnloaded {
        texture_id: String,
    },
    ShaderCompiled {
        shader_id: String,
    },
    ShaderError {
        shader_id: String,
        error: String,
    },

    /// Camera events
    CameraChanged {
        entity: u32,
    },
    ViewportResized {
        width: u32,
        height: u32,
    },
}

/// UI events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UIEvent {
    /// Widget events
    ButtonClicked {
        widget_id: String,
    },
    TextChanged {
        widget_id: String,
        text: String,
    },
    SliderChanged {
        widget_id: String,
        value: f32,
    },

    /// Window events
    WindowResized {
        width: u32,
        height: u32,
    },
    WindowMoved {
        x: i32,
        y: i32,
    },
    WindowFocused,
    WindowUnfocused,
    WindowClosed,
}

/// Network events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Connection events
    Connected {
        peer_id: String,
    },
    Disconnected {
        peer_id: String,
    },
    ConnectionFailed {
        peer_id: String,
        error: String,
    },

    /// Data events
    DataReceived {
        peer_id: String,
        data: Vec<u8>,
    },
    DataSent {
        peer_id: String,
        bytes_sent: usize,
    },
}

/// Input modifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InputModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Custom user events wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEvent {
    /// Event type name
    pub event_type: String,
    /// Event data as JSON
    pub data: serde_json::Value,
    /// Base event information
    pub base: BaseEvent,
}

impl BaseEvent {
    /// Create a new base event
    pub fn new(id: EventId, timestamp: f64) -> Self {
        Self {
            id,
            timestamp,
            priority: EventPriority::Normal,
            consumable: true,
            source_entity: None,
            target_entity: None,
        }
    }

    /// Set event priority
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set source entity
    pub fn with_source(mut self, entity: u32) -> Self {
        self.source_entity = Some(entity);
        self
    }

    /// Set target entity
    pub fn with_target(mut self, entity: u32) -> Self {
        self.target_entity = Some(entity);
        self
    }

    /// Make event non-consumable
    pub fn persistent(mut self) -> Self {
        self.consumable = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Low < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Critical);
    }

    #[test]
    fn test_base_event_creation() {
        let event = BaseEvent::new(123, 456.789);
        assert_eq!(event.id, 123);
        assert_eq!(event.timestamp, 456.789);
        assert_eq!(event.priority, EventPriority::Normal);
        assert!(event.consumable);
        assert!(event.source_entity.is_none());
        assert!(event.target_entity.is_none());
    }

    #[test]
    fn test_base_event_builder() {
        let event = BaseEvent::new(1, 100.0)
            .with_priority(EventPriority::High)
            .with_source(10)
            .with_target(20)
            .persistent();

        assert_eq!(event.priority, EventPriority::High);
        assert_eq!(event.source_entity, Some(10));
        assert_eq!(event.target_entity, Some(20));
        assert!(!event.consumable);
    }

    #[test]
    fn test_input_modifiers() {
        let modifiers = InputModifiers {
            ctrl: true,
            alt: false,
            shift: true,
            meta: false,
        };
        assert!(modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(modifiers.shift);
        assert!(!modifiers.meta);

        let default_modifiers = InputModifiers::default();
        assert!(!default_modifiers.ctrl);
        assert!(!default_modifiers.alt);
        assert!(!default_modifiers.shift);
        assert!(!default_modifiers.meta);
    }

    #[test]
    fn test_mouse_button() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_ne!(MouseButton::Left, MouseButton::Right);
        assert_ne!(MouseButton::Middle, MouseButton::Other(4));
        assert_eq!(MouseButton::Other(4), MouseButton::Other(4));
    }

    #[test]
    fn test_engine_events() {
        let event = EngineEvent::SceneLoaded {
            scene_id: "main_menu".to_string(),
        };
        match event {
            EngineEvent::SceneLoaded { scene_id } => {
                assert_eq!(scene_id, "main_menu");
            }
            _ => panic!("Wrong event type"),
        }

        let entity_event = EngineEvent::EntityCreated { entity: 42 };
        match entity_event {
            EngineEvent::EntityCreated { entity } => {
                assert_eq!(entity, 42);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_collision_event() {
        let collision = EngineEvent::CollisionEnter {
            entity1: 10,
            entity2: 20,
            point: Vec3::new(1.0, 2.0, 3.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
        };

        match collision {
            EngineEvent::CollisionEnter {
                entity1,
                entity2,
                point,
                normal,
            } => {
                assert_eq!(entity1, 10);
                assert_eq!(entity2, 20);
                assert_eq!(point, Vec3::new(1.0, 2.0, 3.0));
                assert_eq!(normal, Vec3::new(0.0, 1.0, 0.0));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_input_events() {
        let key_event = InputEvent::KeyPressed {
            key: "Space".to_string(),
            modifiers: InputModifiers {
                ctrl: true,
                ..Default::default()
            },
        };

        match key_event {
            InputEvent::KeyPressed { key, modifiers } => {
                assert_eq!(key, "Space");
                assert!(modifiers.ctrl);
                assert!(!modifiers.alt);
            }
            _ => panic!("Wrong event type"),
        }

        let mouse_event = InputEvent::MouseMoved {
            position: Vec2::new(100.0, 200.0),
            delta: Vec2::new(10.0, -5.0),
        };

        match mouse_event {
            InputEvent::MouseMoved { position, delta } => {
                assert_eq!(position, Vec2::new(100.0, 200.0));
                assert_eq!(delta, Vec2::new(10.0, -5.0));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_audio_events() {
        let audio_event = AudioEvent::AudioStarted { source: 123 };
        match audio_event {
            AudioEvent::AudioStarted { source } => {
                assert_eq!(source, 123);
            }
            _ => panic!("Wrong event type"),
        }

        let device_event = AudioEvent::AudioDeviceConnected {
            device_id: "headphones".to_string(),
        };
        match device_event {
            AudioEvent::AudioDeviceConnected { device_id } => {
                assert_eq!(device_id, "headphones");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_render_events() {
        let viewport_event = RenderEvent::ViewportResized {
            width: 1920,
            height: 1080,
        };
        match viewport_event {
            RenderEvent::ViewportResized { width, height } => {
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
            }
            _ => panic!("Wrong event type"),
        }

        let shader_event = RenderEvent::ShaderError {
            shader_id: "main_shader".to_string(),
            error: "Compilation failed".to_string(),
        };
        match shader_event {
            RenderEvent::ShaderError { shader_id, error } => {
                assert_eq!(shader_id, "main_shader");
                assert_eq!(error, "Compilation failed");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_ui_events() {
        let button_event = UIEvent::ButtonClicked {
            widget_id: "submit_button".to_string(),
        };
        match button_event {
            UIEvent::ButtonClicked { widget_id } => {
                assert_eq!(widget_id, "submit_button");
            }
            _ => panic!("Wrong event type"),
        }

        let slider_event = UIEvent::SliderChanged {
            widget_id: "volume_slider".to_string(),
            value: 0.75,
        };
        match slider_event {
            UIEvent::SliderChanged { widget_id, value } => {
                assert_eq!(widget_id, "volume_slider");
                assert_eq!(value, 0.75);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_network_events() {
        let connect_event = NetworkEvent::Connected {
            peer_id: "player_123".to_string(),
        };
        match connect_event {
            NetworkEvent::Connected { peer_id } => {
                assert_eq!(peer_id, "player_123");
            }
            _ => panic!("Wrong event type"),
        }

        let data_event = NetworkEvent::DataReceived {
            peer_id: "server".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };
        match data_event {
            NetworkEvent::DataReceived { peer_id, data } => {
                assert_eq!(peer_id, "server");
                assert_eq!(data, vec![1, 2, 3, 4, 5]);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_custom_event() {
        let custom = CustomEvent {
            event_type: "player_level_up".to_string(),
            data: serde_json::json!({
                "player_id": 123,
                "new_level": 10,
                "experience": 5000
            }),
            base: BaseEvent::new(999, 1000.0),
        };

        assert_eq!(custom.event_type, "player_level_up");
        assert_eq!(custom.data["player_id"], 123);
        assert_eq!(custom.data["new_level"], 10);
        assert_eq!(custom.data["experience"], 5000);
        assert_eq!(custom.base.id, 999);
    }
}
