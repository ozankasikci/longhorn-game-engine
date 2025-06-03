//! Event system integration and ECS components

use serde::{Serialize, Deserialize};
use engine_ecs_core::Component;
use crate::{EventDispatcher, EventQueue, EventFilter, EventHandlerTrait, Result, EventError};

/// Event emitter component for entities that can generate events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEmitter {
    /// Whether the emitter is enabled
    pub enabled: bool,
    /// Types of events this entity can emit
    pub event_types: Vec<String>,
    /// Maximum events per frame this entity can emit
    pub max_events_per_frame: usize,
    /// Current events emitted this frame
    pub events_this_frame: usize,
    /// Whether to auto-reset frame counter
    pub auto_reset_frame_counter: bool,
}

impl Component for EventEmitter {}

impl Default for EventEmitter {
    fn default() -> Self {
        Self {
            enabled: true,
            event_types: Vec::new(),
            max_events_per_frame: 100,
            events_this_frame: 0,
            auto_reset_frame_counter: true,
        }
    }
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create an emitter for specific event types
    pub fn for_types(event_types: Vec<String>) -> Self {
        Self {
            event_types,
            ..Self::default()
        }
    }
    
    /// Check if this emitter can emit more events this frame
    pub fn can_emit(&self) -> bool {
        self.enabled && self.events_this_frame < self.max_events_per_frame
    }
    
    /// Record that an event was emitted
    pub fn record_emission(&mut self) {
        if self.enabled {
            self.events_this_frame += 1;
        }
    }
    
    /// Reset the frame counter
    pub fn reset_frame_counter(&mut self) {
        self.events_this_frame = 0;
    }
    
    /// Check if this emitter can emit a specific event type
    pub fn can_emit_type(&self, event_type: &str) -> bool {
        self.enabled && 
        (self.event_types.is_empty() || self.event_types.contains(&event_type.to_string())) &&
        self.can_emit()
    }
}

/// Event listener component for entities that want to receive specific events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventListener {
    /// Whether the listener is enabled
    pub enabled: bool,
    /// Types of events to listen for (empty = listen to all)
    pub event_types: Vec<String>,
    /// Maximum events per frame this entity can process
    pub max_events_per_frame: usize,
    /// Current events processed this frame
    pub events_this_frame: usize,
    /// Whether to auto-reset frame counter
    pub auto_reset_frame_counter: bool,
    /// Handler function name or identifier
    pub handler_id: Option<String>,
}

impl Component for EventListener {}

impl Default for EventListener {
    fn default() -> Self {
        Self {
            enabled: true,
            event_types: Vec::new(),
            max_events_per_frame: 1000,
            events_this_frame: 0,
            auto_reset_frame_counter: true,
            handler_id: None,
        }
    }
}

impl EventListener {
    /// Create a new event listener
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a listener for specific event types
    pub fn for_types(event_types: Vec<String>) -> Self {
        Self {
            event_types,
            ..Self::default()
        }
    }
    
    /// Create a listener with a specific handler
    pub fn with_handler(handler_id: String) -> Self {
        Self {
            handler_id: Some(handler_id),
            ..Self::default()
        }
    }
    
    /// Check if this listener can process more events this frame
    pub fn can_process(&self) -> bool {
        self.enabled && self.events_this_frame < self.max_events_per_frame
    }
    
    /// Record that an event was processed
    pub fn record_processing(&mut self) {
        if self.enabled {
            self.events_this_frame += 1;
        }
    }
    
    /// Reset the frame counter
    pub fn reset_frame_counter(&mut self) {
        self.events_this_frame = 0;
    }
    
    /// Check if this listener wants to receive a specific event type
    pub fn wants_event_type(&self, event_type: &str) -> bool {
        self.enabled && 
        (self.event_types.is_empty() || self.event_types.contains(&event_type.to_string())) &&
        self.can_process()
    }
}

/// Event system configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventSystemConfig {
    /// Maximum events to process per frame
    pub max_events_per_frame: usize,
    /// Whether to use priority-based processing
    pub use_priority_processing: bool,
    /// Whether to automatically reset frame counters
    pub auto_reset_frame_counters: bool,
    /// Global event filter
    pub global_filter: Option<String>, // Serializable filter identifier
    /// Whether to enable event profiling
    pub enable_profiling: bool,
}

impl Default for EventSystemConfig {
    fn default() -> Self {
        Self {
            max_events_per_frame: 1000,
            use_priority_processing: true,
            auto_reset_frame_counters: true,
            global_filter: None,
            enable_profiling: false,
        }
    }
}

/// Main event system that coordinates dispatching and queuing
pub struct EventSystem {
    /// Event dispatcher
    dispatcher: EventDispatcher,
    /// Event queue
    queue: EventQueue,
    /// System configuration
    config: EventSystemConfig,
    /// Global event filter
    global_filter: Option<Box<dyn EventFilter>>,
    /// System statistics
    stats: EventSystemStats,
}

/// Event system statistics
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventSystemStats {
    /// Total events processed
    pub total_events_processed: u64,
    /// Events processed this frame
    pub events_this_frame: usize,
    /// Events dropped by filters
    pub events_filtered: u64,
    /// Events dropped due to queue overflow
    pub events_dropped: u64,
    /// Average processing time per frame (ms)
    pub avg_frame_time: f32,
    /// Current active handlers
    pub active_handlers: usize,
}

impl EventSystem {
    /// Create a new event system
    pub fn new() -> Self {
        Self::with_config(EventSystemConfig::default())
    }
    
    /// Create a new event system with configuration
    pub fn with_config(config: EventSystemConfig) -> Self {
        Self {
            dispatcher: EventDispatcher::new(),
            queue: EventQueue::new(),
            config,
            global_filter: None,
            stats: EventSystemStats::default(),
        }
    }
    
    /// Register an event handler
    pub fn register_handler(&mut self, type_id: crate::EventTypeId, handler: Box<dyn EventHandlerTrait>) -> Result<()> {
        self.dispatcher.register_handler(type_id, handler)?;
        self.stats.active_handlers = self.dispatcher.total_handler_count();
        Ok(())
    }
    
    /// Queue an event for later processing
    pub fn queue_event(&mut self, event: Box<dyn crate::Event>) -> Result<crate::EventId> {
        // Apply global filter if present
        if let Some(filter) = &self.global_filter {
            if !filter.passes(event.as_ref()) {
                self.stats.events_filtered += 1;
                return Err(EventError::InvalidEventData("Event filtered".to_string()));
            }
        }
        
        self.queue.enqueue(event)
    }
    
    /// Process events from the queue
    pub fn process_events(&mut self) -> usize {
        let start_time = std::time::Instant::now();
        let max_events = self.config.max_events_per_frame;
        
        let events_processed = self.queue.process_events(max_events, |event| {
            self.dispatcher.dispatch(event);
            true
        });
        
        // Update statistics
        let frame_time = start_time.elapsed().as_secs_f32() * 1000.0;
        self.update_frame_stats(events_processed, frame_time);
        
        events_processed
    }
    
    /// Immediately dispatch an event without queuing
    pub fn dispatch_immediate(&mut self, event: &dyn crate::Event) -> bool {
        // Apply global filter if present
        if let Some(filter) = &self.global_filter {
            if !filter.passes(event) {
                self.stats.events_filtered += 1;
                return false;
            }
        }
        
        self.dispatcher.dispatch(event)
    }
    
    /// Set global event filter
    pub fn set_global_filter(&mut self, filter: Box<dyn EventFilter>) {
        self.global_filter = Some(filter);
    }
    
    /// Remove global event filter
    pub fn clear_global_filter(&mut self) {
        self.global_filter = None;
    }
    
    /// Get system statistics
    pub fn get_stats(&self) -> &EventSystemStats {
        &self.stats
    }
    
    /// Reset system statistics
    pub fn reset_stats(&mut self) {
        self.stats = EventSystemStats {
            active_handlers: self.stats.active_handlers,
            ..Default::default()
        };
        self.dispatcher.reset_stats();
        self.queue.reset_stats();
    }
    
    /// Get system configuration
    pub fn get_config(&self) -> &EventSystemConfig {
        &self.config
    }
    
    /// Update system configuration
    pub fn set_config(&mut self, config: EventSystemConfig) {
        self.config = config;
    }
    
    /// Clear all events and handlers
    pub fn clear(&mut self) {
        self.queue.clear();
        self.dispatcher.clear_handlers();
        self.stats.active_handlers = 0;
    }
    
    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }
    
    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Process frame cleanup (reset counters, etc.)
    pub fn end_frame(&mut self) {
        if self.config.auto_reset_frame_counters {
            self.stats.events_this_frame = 0;
        }
    }
    
    fn update_frame_stats(&mut self, events_processed: usize, frame_time: f32) {
        self.stats.total_events_processed += events_processed as u64;
        self.stats.events_this_frame += events_processed;
        
        // Update rolling average frame time
        let weight = 0.1;
        self.stats.avg_frame_time = 
            self.stats.avg_frame_time * (1.0 - weight) + frame_time * weight;
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}