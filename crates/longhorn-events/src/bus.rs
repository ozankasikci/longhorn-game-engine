use std::collections::HashMap;
use crate::{Event, EventType, EventTarget, RingBuffer};

/// Subscription ID for unsubscribing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

/// The central event bus for the engine.
pub struct EventBus {
    /// Events queued for processing this frame.
    pending: Vec<Event>,
    /// Event history for debugging (last 256 events).
    history: RingBuffer<Event, 256>,
    /// Frame counter for timestamps.
    frame: u64,
    /// Next subscription ID.
    next_sub_id: u64,
    /// Rust callback subscriptions.
    subscriptions: HashMap<EventType, Vec<(SubscriptionId, Box<dyn Fn(&Event) + Send + Sync>)>>,
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        Self {
            pending: Vec::with_capacity(64),
            history: RingBuffer::new(),
            frame: 0,
            next_sub_id: 1,
            subscriptions: HashMap::new(),
        }
    }

    /// Emit an event to be processed later.
    pub fn emit(&mut self, event_type: EventType, data: serde_json::Value) {
        self.emit_targeted(event_type, EventTarget::Global, data);
    }

    /// Emit an event to a specific target.
    pub fn emit_targeted(&mut self, event_type: EventType, target: EventTarget, data: serde_json::Value) {
        let event = Event {
            event_type,
            target,
            data,
            timestamp: self.frame,
        };
        self.pending.push(event);
    }

    /// Process all pending events, calling handlers and moving to history.
    /// Returns the processed events.
    pub fn process(&mut self) -> Vec<Event> {
        let events = std::mem::take(&mut self.pending);

        // Call Rust handlers
        for event in &events {
            if let Some(handlers) = self.subscriptions.get(&event.event_type) {
                for (_, handler) in handlers {
                    handler(event);
                }
            }
            // Also check for Custom event handlers by exact match
            if let EventType::Custom(_) = &event.event_type {
                if let Some(handlers) = self.subscriptions.get(&event.event_type) {
                    for (_, handler) in handlers {
                        handler(event);
                    }
                }
            }
        }

        // Move to history
        for event in &events {
            self.history.push(event.clone());
        }

        self.frame += 1;
        events
    }

    /// Subscribe to an event type with a Rust callback.
    pub fn subscribe<F>(&mut self, event_type: EventType, handler: F) -> SubscriptionId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let id = SubscriptionId(self.next_sub_id);
        self.next_sub_id += 1;

        self.subscriptions
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push((id, Box::new(handler)));

        id
    }

    /// Unsubscribe from events.
    pub fn unsubscribe(&mut self, id: SubscriptionId) {
        for handlers in self.subscriptions.values_mut() {
            handlers.retain(|(sub_id, _)| *sub_id != id);
        }
    }

    /// Number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Number of events in history.
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Iterate over event history (oldest to newest).
    pub fn history_iter(&self) -> impl Iterator<Item = &Event> {
        self.history.iter()
    }

    /// Clear all pending events without processing.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Current frame number.
    pub fn frame(&self) -> u64 {
        self.frame
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_emit_and_process() {
        let mut bus = EventBus::new();

        bus.emit(EventType::TouchStarted, serde_json::json!({"x": 10.0}));

        assert_eq!(bus.pending_count(), 1);

        let processed = bus.process();

        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].event_type, EventType::TouchStarted);
        assert_eq!(bus.pending_count(), 0);
    }

    #[test]
    fn test_history_preserved() {
        let mut bus = EventBus::new();

        bus.emit(EventType::TouchStarted, serde_json::json!({}));
        bus.process();

        assert_eq!(bus.history_count(), 1);
    }

    #[test]
    fn test_emit_order_preserved() {
        let mut bus = EventBus::new();

        bus.emit(EventType::TouchStarted, serde_json::json!({"order": 1}));
        bus.emit(EventType::TouchMoved, serde_json::json!({"order": 2}));
        bus.emit(EventType::TouchEnded, serde_json::json!({"order": 3}));

        let processed = bus.process();

        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].data["order"], 1);
        assert_eq!(processed[1].data["order"], 2);
        assert_eq!(processed[2].data["order"], 3);
    }

    #[test]
    fn test_rust_subscription() {
        let mut bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventType::TouchStarted, move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit(EventType::TouchStarted, serde_json::json!({}));
        bus.emit(EventType::TouchStarted, serde_json::json!({}));
        bus.process();

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_unsubscribe() {
        let mut bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let sub_id = bus.subscribe(EventType::TouchStarted, move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit(EventType::TouchStarted, serde_json::json!({}));
        bus.process();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        bus.unsubscribe(sub_id);

        bus.emit(EventType::TouchStarted, serde_json::json!({}));
        bus.process();
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Still 1, not called again
    }

    #[test]
    fn test_targeted_events() {
        let mut bus = EventBus::new();

        bus.emit_targeted(
            EventType::Custom("hit".to_string()),
            EventTarget::Entity(42),
            serde_json::json!({"damage": 10}),
        );

        let processed = bus.process();

        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].target, EventTarget::Entity(42));
    }
}
