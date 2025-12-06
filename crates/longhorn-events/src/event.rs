use serde::{Deserialize, Serialize};

/// Target for an event - who should receive it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTarget {
    /// Broadcast to all listeners.
    Global,
    /// Target a specific entity by ID.
    Entity(u64),
}

/// Categories of built-in events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Input events
    TouchStarted,
    TouchMoved,
    TouchEnded,
    KeyPressed,
    KeyReleased,

    // Lifecycle events
    GameStarting,
    GameStopping,
    FrameBegin,
    FrameEnd,

    // World events
    EntitySpawned,
    EntityDespawned,
    ComponentAdded,
    ComponentChanged,

    // Custom script event (name stored in event data)
    Custom(String),
}

/// An event with its payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub target: EventTarget,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event {
            event_type: EventType::TouchStarted,
            target: EventTarget::Global,
            data: serde_json::json!({"x": 100.0, "y": 200.0}),
            timestamp: 12345,
        };

        assert_eq!(event.event_type, EventType::TouchStarted);
        assert_eq!(event.target, EventTarget::Global);
    }

    #[test]
    fn test_custom_event_type() {
        let event_type = EventType::Custom("playerDied".to_string());
        assert_eq!(event_type, EventType::Custom("playerDied".to_string()));
    }

    #[test]
    fn test_entity_target() {
        let target = EventTarget::Entity(42);
        assert_eq!(target, EventTarget::Entity(42));
    }
}
