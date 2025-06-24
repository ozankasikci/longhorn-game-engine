//! Event trait implementations for concrete event types

use crate::{
    AudioEvent, CustomEvent, EngineEvent, Event, EventId, EventPriority, EventTypeId, InputEvent,
    NetworkEvent, RenderEvent, UIEvent,
};

// Type IDs for different event categories
pub const ENGINE_EVENT_TYPE_ID: EventTypeId = 1;
pub const INPUT_EVENT_TYPE_ID: EventTypeId = 2;
pub const AUDIO_EVENT_TYPE_ID: EventTypeId = 3;
pub const RENDER_EVENT_TYPE_ID: EventTypeId = 4;
pub const UI_EVENT_TYPE_ID: EventTypeId = 5;
pub const NETWORK_EVENT_TYPE_ID: EventTypeId = 6;
pub const CUSTOM_EVENT_TYPE_ID: EventTypeId = 7;

impl Event for EngineEvent {
    fn get_type_id(&self) -> EventTypeId {
        ENGINE_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        // In a real implementation, this would be generated uniquely
        0
    }

    fn timestamp(&self) -> f64 {
        // In a real implementation, this would use system time
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for InputEvent {
    fn get_type_id(&self) -> EventTypeId {
        INPUT_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        0
    }

    fn timestamp(&self) -> f64 {
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for AudioEvent {
    fn get_type_id(&self) -> EventTypeId {
        AUDIO_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        0
    }

    fn timestamp(&self) -> f64 {
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for RenderEvent {
    fn get_type_id(&self) -> EventTypeId {
        RENDER_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        0
    }

    fn timestamp(&self) -> f64 {
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for UIEvent {
    fn get_type_id(&self) -> EventTypeId {
        UI_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        0
    }

    fn timestamp(&self) -> f64 {
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for NetworkEvent {
    fn get_type_id(&self) -> EventTypeId {
        NETWORK_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        0
    }

    fn timestamp(&self) -> f64 {
        0.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Event for CustomEvent {
    fn get_type_id(&self) -> EventTypeId {
        CUSTOM_EVENT_TYPE_ID
    }

    fn event_id(&self) -> EventId {
        self.base.id
    }

    fn timestamp(&self) -> f64 {
        self.base.timestamp
    }

    fn priority(&self) -> EventPriority {
        self.base.priority
    }

    fn is_consumable(&self) -> bool {
        self.base.consumable
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
