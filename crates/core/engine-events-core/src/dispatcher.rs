//! Event dispatcher abstractions

use crate::{Event, EventTypeId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event handler function signature
pub type EventHandler<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

/// Generic event handler trait
pub trait EventHandlerTrait: Send + Sync {
    /// Handle an event and return whether it was consumed
    fn handle(&self, event: &dyn Event) -> bool;

    /// Get the event type this handler can process
    fn event_type(&self) -> EventTypeId;

    /// Get handler priority
    fn priority(&self) -> HandlerPriority {
        HandlerPriority::Normal
    }
}

/// Handler priority levels (higher = processed first)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HandlerPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event dispatcher configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatcherConfig {
    /// Maximum number of events to process per frame
    pub max_events_per_frame: usize,
    /// Whether to sort events by priority
    pub sort_by_priority: bool,
    /// Whether to sort handlers by priority
    pub sort_handlers_by_priority: bool,
    /// Maximum event queue size
    pub max_queue_size: usize,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            max_events_per_frame: 1000,
            sort_by_priority: true,
            sort_handlers_by_priority: true,
            max_queue_size: 10000,
        }
    }
}

/// Event dispatcher for managing and routing events
pub struct EventDispatcher {
    /// Event handlers by type
    handlers: HashMap<EventTypeId, Vec<Box<dyn EventHandlerTrait>>>,
    /// Global handlers that receive all events
    global_handlers: Vec<Box<dyn EventHandlerTrait>>,
    /// Dispatcher configuration
    config: DispatcherConfig,
    /// Event statistics
    stats: DispatcherStats,
}

/// Event dispatcher statistics
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DispatcherStats {
    /// Total events processed
    pub events_processed: u64,
    /// Events processed this frame
    pub events_this_frame: usize,
    /// Total handlers registered
    pub handlers_registered: usize,
    /// Average processing time per event (ms)
    pub avg_processing_time: f32,
    /// Events dropped due to queue overflow
    pub events_dropped: u64,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self::with_config(DispatcherConfig::default())
    }

    /// Create a new event dispatcher with config
    pub fn with_config(config: DispatcherConfig) -> Self {
        Self {
            handlers: HashMap::new(),
            global_handlers: Vec::new(),
            config,
            stats: DispatcherStats::default(),
        }
    }

    /// Register an event handler for a specific event type
    pub fn register_handler(
        &mut self,
        type_id: EventTypeId,
        handler: Box<dyn EventHandlerTrait>,
    ) -> Result<()> {
        let handlers = self.handlers.entry(type_id).or_default();
        handlers.push(handler);

        // Sort by priority if enabled
        if self.config.sort_handlers_by_priority {
            handlers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        }

        self.stats.handlers_registered = self.count_handlers();
        Ok(())
    }

    /// Register a global handler that receives all events
    pub fn register_global_handler(&mut self, handler: Box<dyn EventHandlerTrait>) {
        self.global_handlers.push(handler);

        // Sort by priority if enabled
        if self.config.sort_handlers_by_priority {
            self.global_handlers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        }

        self.stats.handlers_registered = self.count_handlers();
    }

    /// Unregister all handlers for a specific event type
    pub fn unregister_handlers(&mut self, type_id: EventTypeId) {
        self.handlers.remove(&type_id);
        self.stats.handlers_registered = self.count_handlers();
    }

    /// Clear all handlers
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
        self.global_handlers.clear();
        self.stats.handlers_registered = 0;
    }

    /// Dispatch an event to all registered handlers
    pub fn dispatch(&mut self, event: &dyn Event) -> bool {
        let start_time = std::time::Instant::now();
        let mut consumed = false;

        // Get event type
        let type_id = event.get_type_id();

        // Process specific handlers first
        if let Some(handlers) = self.handlers.get(&type_id) {
            for handler in handlers {
                if handler.handle(event) {
                    consumed = true;
                    if event.is_consumable() {
                        break;
                    }
                }
            }
        }

        // Process global handlers if event wasn't consumed
        if !consumed || !event.is_consumable() {
            for handler in &self.global_handlers {
                if handler.handle(event) {
                    consumed = true;
                    if event.is_consumable() {
                        break;
                    }
                }
            }
        }

        // Update statistics
        let processing_time = start_time.elapsed().as_secs_f32() * 1000.0;
        self.update_stats(processing_time);

        consumed
    }

    /// Get dispatcher statistics
    pub fn get_stats(&self) -> &DispatcherStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DispatcherStats {
            handlers_registered: self.stats.handlers_registered,
            ..Default::default()
        };
    }

    /// Get dispatcher configuration
    pub fn get_config(&self) -> &DispatcherConfig {
        &self.config
    }

    /// Update dispatcher configuration
    pub fn set_config(&mut self, config: DispatcherConfig) {
        self.config = config;

        // Re-sort handlers if priority sorting was enabled
        if self.config.sort_handlers_by_priority {
            for handlers in self.handlers.values_mut() {
                handlers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
            }
            self.global_handlers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        }
    }

    /// Check if any handlers are registered for an event type
    pub fn has_handlers(&self, type_id: EventTypeId) -> bool {
        self.handlers.contains_key(&type_id) || !self.global_handlers.is_empty()
    }

    /// Get number of handlers for a specific event type
    pub fn handler_count(&self, type_id: EventTypeId) -> usize {
        self.handlers.get(&type_id).map_or(0, |h| h.len()) + self.global_handlers.len()
    }

    /// Get total number of registered handlers
    pub fn total_handler_count(&self) -> usize {
        self.count_handlers()
    }

    fn count_handlers(&self) -> usize {
        self.handlers.values().map(|h| h.len()).sum::<usize>() + self.global_handlers.len()
    }

    fn update_stats(&mut self, processing_time: f32) {
        self.stats.events_processed += 1;
        self.stats.events_this_frame += 1;

        // Update rolling average processing time
        let weight = 0.1;
        self.stats.avg_processing_time =
            self.stats.avg_processing_time * (1.0 - weight) + processing_time * weight;
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple event handler implementation
pub struct SimpleEventHandler<T: Event> {
    handler: EventHandler<T>,
    priority: HandlerPriority,
    type_id: EventTypeId,
}

impl<T: Event + 'static> SimpleEventHandler<T> {
    /// Create a new simple event handler
    pub fn new(handler: EventHandler<T>, type_id: EventTypeId) -> Self {
        Self {
            handler,
            priority: HandlerPriority::Normal,
            type_id,
        }
    }

    /// Create a new simple event handler with priority
    pub fn with_priority(
        handler: EventHandler<T>,
        type_id: EventTypeId,
        priority: HandlerPriority,
    ) -> Self {
        Self {
            handler,
            priority,
            type_id,
        }
    }
}

impl<T: Event + 'static> EventHandlerTrait for SimpleEventHandler<T> {
    fn handle(&self, event: &dyn Event) -> bool {
        // Attempt to downcast to our specific event type
        if let Some(typed_event) = event.as_any().downcast_ref::<T>() {
            (self.handler)(typed_event)
        } else {
            false
        }
    }

    fn event_type(&self) -> EventTypeId {
        self.type_id
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}
