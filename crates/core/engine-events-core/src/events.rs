//! Core event abstractions

use serde::{Serialize, Deserialize};
use glam::{Vec2, Vec3, Quat};

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
    fn is_consumable(&self) -> bool { true }
    
    /// Get event priority (higher = more priority)
    fn priority(&self) -> EventPriority { EventPriority::Normal }
    
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
    SceneLoaded { scene_id: String },
    SceneUnloaded { scene_id: String },
    SceneChanged { from: String, to: String },
    
    /// Entity lifecycle events
    EntityCreated { entity: u32 },
    EntityDestroyed { entity: u32 },
    EntityEnabled { entity: u32 },
    EntityDisabled { entity: u32 },
    
    /// Component events
    ComponentAdded { entity: u32, component_type: String },
    ComponentRemoved { entity: u32, component_type: String },
    ComponentChanged { entity: u32, component_type: String },
    
    /// Transform events
    TransformChanged { entity: u32, position: Vec3, rotation: Quat, scale: Vec3 },
    
    /// Collision events
    CollisionEnter { entity1: u32, entity2: u32, point: Vec3, normal: Vec3 },
    CollisionExit { entity1: u32, entity2: u32 },
    CollisionStay { entity1: u32, entity2: u32, point: Vec3, normal: Vec3 },
    
    /// Trigger events
    TriggerEnter { trigger: u32, other: u32 },
    TriggerExit { trigger: u32, other: u32 },
    TriggerStay { trigger: u32, other: u32 },
}

/// Input events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard events
    KeyPressed { key: String, modifiers: InputModifiers },
    KeyReleased { key: String, modifiers: InputModifiers },
    
    /// Mouse events
    MouseButtonPressed { button: MouseButton, position: Vec2 },
    MouseButtonReleased { button: MouseButton, position: Vec2 },
    MouseMoved { position: Vec2, delta: Vec2 },
    MouseScrolled { delta: Vec2 },
    
    /// Touch events
    TouchStarted { id: u32, position: Vec2 },
    TouchMoved { id: u32, position: Vec2, delta: Vec2 },
    TouchEnded { id: u32, position: Vec2 },
    TouchCancelled { id: u32, position: Vec2 },
    
    /// Gamepad events
    GamepadConnected { id: u32 },
    GamepadDisconnected { id: u32 },
    GamepadButtonPressed { id: u32, button: String },
    GamepadButtonReleased { id: u32, button: String },
    GamepadAxisChanged { id: u32, axis: String, value: f32 },
}

/// Audio events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Audio playback events
    AudioStarted { source: u32 },
    AudioStopped { source: u32 },
    AudioPaused { source: u32 },
    AudioResumed { source: u32 },
    AudioFinished { source: u32 },
    
    /// Audio system events
    AudioDeviceConnected { device_id: String },
    AudioDeviceDisconnected { device_id: String },
    AudioDeviceChanged { device_id: String },
}

/// Rendering events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderEvent {
    /// Render pipeline events
    FrameStarted,
    FrameEnded,
    RenderPassStarted { pass_name: String },
    RenderPassEnded { pass_name: String },
    
    /// Resource events
    TextureLoaded { texture_id: String },
    TextureUnloaded { texture_id: String },
    ShaderCompiled { shader_id: String },
    ShaderError { shader_id: String, error: String },
    
    /// Camera events
    CameraChanged { entity: u32 },
    ViewportResized { width: u32, height: u32 },
}

/// UI events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UIEvent {
    /// Widget events
    ButtonClicked { widget_id: String },
    TextChanged { widget_id: String, text: String },
    SliderChanged { widget_id: String, value: f32 },
    
    /// Window events
    WindowResized { width: u32, height: u32 },
    WindowMoved { x: i32, y: i32 },
    WindowFocused,
    WindowUnfocused,
    WindowClosed,
}

/// Network events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Connection events
    Connected { peer_id: String },
    Disconnected { peer_id: String },
    ConnectionFailed { peer_id: String, error: String },
    
    /// Data events
    DataReceived { peer_id: String, data: Vec<u8> },
    DataSent { peer_id: String, bytes_sent: usize },
}

/// Input modifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for InputModifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }
}