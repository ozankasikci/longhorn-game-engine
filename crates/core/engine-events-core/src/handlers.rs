//! Event handler utilities and common implementations

use crate::{Event, EventHandlerTrait, EventTypeId, HandlerPriority};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Multi-threaded event handler wrapper
pub struct ThreadSafeHandler<F> {
    handler: Arc<Mutex<F>>,
    priority: HandlerPriority,
    event_type: EventTypeId,
}

impl<F> ThreadSafeHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync + 'static,
{
    /// Create a new thread-safe handler
    pub fn new(handler: F, event_type: EventTypeId) -> Self {
        Self {
            handler: Arc::new(Mutex::new(handler)),
            priority: HandlerPriority::Normal,
            event_type,
        }
    }

    /// Create a new thread-safe handler with priority
    pub fn with_priority(handler: F, event_type: EventTypeId, priority: HandlerPriority) -> Self {
        Self {
            handler: Arc::new(Mutex::new(handler)),
            priority,
            event_type,
        }
    }
}

impl<F> EventHandlerTrait for ThreadSafeHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync + 'static,
{
    fn handle(&self, event: &dyn Event) -> bool {
        if let Ok(handler) = self.handler.lock() {
            handler(event)
        } else {
            false
        }
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

/// Conditional event handler that only processes events matching a condition
pub struct ConditionalHandler<F, C> {
    handler: F,
    condition: C,
    priority: HandlerPriority,
    event_type: EventTypeId,
}

impl<F, C> ConditionalHandler<F, C>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
    C: Fn(&dyn Event) -> bool + Send + Sync,
{
    /// Create a new conditional handler
    pub fn new(handler: F, condition: C, event_type: EventTypeId) -> Self {
        Self {
            handler,
            condition,
            priority: HandlerPriority::Normal,
            event_type,
        }
    }

    /// Create a new conditional handler with priority
    pub fn with_priority(
        handler: F,
        condition: C,
        event_type: EventTypeId,
        priority: HandlerPriority,
    ) -> Self {
        Self {
            handler,
            condition,
            priority,
            event_type,
        }
    }
}

impl<F, C> EventHandlerTrait for ConditionalHandler<F, C>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
    C: Fn(&dyn Event) -> bool + Send + Sync,
{
    fn handle(&self, event: &dyn Event) -> bool {
        if (self.condition)(event) {
            (self.handler)(event)
        } else {
            false
        }
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

/// Event handler that forwards events to multiple sub-handlers
pub struct CompositeHandler {
    handlers: Vec<Box<dyn EventHandlerTrait>>,
    event_type: EventTypeId,
    priority: HandlerPriority,
    /// How to combine results from sub-handlers
    combination_mode: CombinationMode,
}

/// How to combine results from multiple handlers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombinationMode {
    /// Return true if any handler returns true
    Any,
    /// Return true only if all handlers return true
    All,
    /// Return true if the first handler returns true (short-circuit)
    First,
}

impl CompositeHandler {
    /// Create a new composite handler
    pub fn new(event_type: EventTypeId) -> Self {
        Self {
            handlers: Vec::new(),
            event_type,
            priority: HandlerPriority::Normal,
            combination_mode: CombinationMode::Any,
        }
    }

    /// Add a sub-handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandlerTrait>) {
        self.handlers.push(handler);
    }

    /// Set combination mode
    pub fn with_combination_mode(mut self, mode: CombinationMode) -> Self {
        self.combination_mode = mode;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: HandlerPriority) -> Self {
        self.priority = priority;
        self
    }
}

impl EventHandlerTrait for CompositeHandler {
    fn handle(&self, event: &dyn Event) -> bool {
        match self.combination_mode {
            CombinationMode::Any => self.handlers.iter().any(|h| h.handle(event)),
            CombinationMode::All => self.handlers.iter().all(|h| h.handle(event)),
            CombinationMode::First => self.handlers.first().is_some_and(|h| h.handle(event)),
        }
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

/// Event handler that counts how many times it's been called
pub struct CountingHandler<F> {
    handler: F,
    count: Arc<Mutex<usize>>,
    priority: HandlerPriority,
    event_type: EventTypeId,
}

impl<F> CountingHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    /// Create a new counting handler
    pub fn new(handler: F, event_type: EventTypeId) -> Self {
        Self {
            handler,
            count: Arc::new(Mutex::new(0)),
            priority: HandlerPriority::Normal,
            event_type,
        }
    }

    /// Get the current count
    pub fn get_count(&self) -> usize {
        *self
            .count
            .lock()
            .unwrap_or_else(|_| panic!("Failed to lock count"))
    }

    /// Reset the count
    pub fn reset_count(&self) {
        if let Ok(mut count) = self.count.lock() {
            *count = 0;
        }
    }
}

impl<F> EventHandlerTrait for CountingHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    fn handle(&self, event: &dyn Event) -> bool {
        if let Ok(mut count) = self.count.lock() {
            *count += 1;
        }
        (self.handler)(event)
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

/// Event handler that logs events for debugging
pub struct LoggingHandler {
    event_type: EventTypeId,
    priority: HandlerPriority,
    log_level: LogLevel,
}

/// Log levels for the logging handler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LoggingHandler {
    /// Create a new logging handler
    pub fn new(event_type: EventTypeId) -> Self {
        Self {
            event_type,
            priority: HandlerPriority::Low,
            log_level: LogLevel::Debug,
        }
    }

    /// Set log level
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }
}

impl EventHandlerTrait for LoggingHandler {
    fn handle(&self, event: &dyn Event) -> bool {
        let message = format!("Event handled: {:?}", event);

        match self.log_level {
            LogLevel::Debug => log::debug!("{}", message),
            LogLevel::Info => log::info!("{}", message),
            LogLevel::Warn => log::warn!("{}", message),
            LogLevel::Error => log::error!("{}", message),
        }

        false // Logging handlers don't consume events
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}

/// Event handler that only processes events a limited number of times
pub struct LimitedHandler<F> {
    handler: F,
    max_count: usize,
    current_count: Arc<Mutex<usize>>,
    priority: HandlerPriority,
    event_type: EventTypeId,
}

impl<F> LimitedHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    /// Create a new limited handler
    pub fn new(handler: F, event_type: EventTypeId, max_count: usize) -> Self {
        Self {
            handler,
            max_count,
            current_count: Arc::new(Mutex::new(0)),
            priority: HandlerPriority::Normal,
            event_type,
        }
    }

    /// Check if the handler can still process events
    pub fn can_handle(&self) -> bool {
        *self
            .current_count
            .lock()
            .unwrap_or_else(|_| panic!("Failed to lock count"))
            < self.max_count
    }

    /// Reset the count
    pub fn reset(&self) {
        if let Ok(mut count) = self.current_count.lock() {
            *count = 0;
        }
    }
}

impl<F> EventHandlerTrait for LimitedHandler<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    fn handle(&self, event: &dyn Event) -> bool {
        if let Ok(mut count) = self.current_count.lock() {
            if *count >= self.max_count {
                return false;
            }
            *count += 1;
            (self.handler)(event)
        } else {
            false
        }
    }

    fn event_type(&self) -> EventTypeId {
        self.event_type
    }

    fn priority(&self) -> HandlerPriority {
        self.priority
    }
}
