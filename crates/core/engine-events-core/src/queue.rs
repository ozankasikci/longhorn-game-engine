//! Event queue abstractions for deferred event processing

use crate::{Event, EventError, EventId, EventPriority, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

/// Queued event wrapper
#[derive(Debug)]
pub struct QueuedEvent {
    /// The actual event
    pub event: Box<dyn Event>,
    /// Event priority for ordering
    pub priority: EventPriority,
    /// Event ID for tracking
    pub id: EventId,
    /// Timestamp when event was queued
    pub queued_at: f64,
}

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for QueuedEvent {}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap behavior (higher priority first)
        self.priority.cmp(&other.priority).reverse().then_with(|| {
            self.queued_at
                .partial_cmp(&other.queued_at)
                .unwrap_or(Ordering::Equal)
        })
    }
}

/// Event queue configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventQueueConfig {
    /// Maximum number of events in queue
    pub max_size: usize,
    /// Whether to use priority ordering
    pub priority_ordering: bool,
    /// Whether to drop old events when queue is full
    pub drop_old_on_full: bool,
    /// Maximum age of events before they're automatically dropped (seconds)
    pub max_event_age: Option<f32>,
}

impl Default for EventQueueConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            priority_ordering: true,
            drop_old_on_full: true,
            max_event_age: Some(10.0),
        }
    }
}

/// Event queue statistics
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventQueueStats {
    /// Current number of events in queue
    pub current_size: usize,
    /// Total events ever queued
    pub total_queued: u64,
    /// Total events processed
    pub total_processed: u64,
    /// Events dropped due to queue overflow
    pub events_dropped: u64,
    /// Events dropped due to age
    pub events_expired: u64,
    /// Average time events spend in queue (ms)
    pub avg_queue_time: f32,
}

/// Priority-based event queue
pub struct EventQueue {
    /// Events stored in priority order
    events: BinaryHeap<QueuedEvent>,
    /// Configuration
    config: EventQueueConfig,
    /// Statistics
    stats: EventQueueStats,
    /// Next event ID
    next_id: EventId,
}

impl EventQueue {
    /// Create a new event queue
    pub fn new() -> Self {
        Self::with_config(EventQueueConfig::default())
    }

    /// Create a new event queue with configuration
    pub fn with_config(config: EventQueueConfig) -> Self {
        Self {
            events: if config.priority_ordering {
                BinaryHeap::with_capacity(config.max_size.min(1000))
            } else {
                BinaryHeap::new()
            },
            config,
            stats: EventQueueStats::default(),
            next_id: 1,
        }
    }

    /// Add an event to the queue
    pub fn enqueue(&mut self, event: Box<dyn Event>) -> Result<EventId> {
        // Check if queue is full
        if self.events.len() >= self.config.max_size {
            if self.config.drop_old_on_full {
                // Remove oldest event
                self.drop_oldest_event();
            } else {
                self.stats.events_dropped += 1;
                return Err(EventError::QueueFull);
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let queued_event = QueuedEvent {
            priority: event.priority(),
            id,
            queued_at: get_current_time(),
            event,
        };

        self.events.push(queued_event);
        self.stats.total_queued += 1;
        self.stats.current_size = self.events.len();

        Ok(id)
    }

    /// Remove and return the highest priority event
    pub fn dequeue(&mut self) -> Option<Box<dyn Event>> {
        // Clean up expired events first
        self.cleanup_expired_events();

        if let Some(queued_event) = self.events.pop() {
            let queue_time = (get_current_time() - queued_event.queued_at) * 1000.0;

            // Update statistics
            self.stats.total_processed += 1;
            self.stats.current_size = self.events.len();
            self.update_avg_queue_time(queue_time as f32);

            Some(queued_event.event)
        } else {
            None
        }
    }

    /// Peek at the highest priority event without removing it
    pub fn peek(&self) -> Option<&dyn Event> {
        self.events.peek().map(|qe| qe.event.as_ref())
    }

    /// Get the number of events in the queue
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.events.len() >= self.config.max_size
    }

    /// Clear all events from the queue
    pub fn clear(&mut self) {
        self.events.clear();
        self.stats.current_size = 0;
    }

    /// Drain all events from the queue
    pub fn drain(&mut self) -> Vec<Box<dyn Event>> {
        let mut events = Vec::with_capacity(self.events.len());

        while let Some(event) = self.dequeue() {
            events.push(event);
        }

        events
    }

    /// Process up to `max_events` from the queue with a handler
    pub fn process_events<F>(&mut self, max_events: usize, mut handler: F) -> usize
    where
        F: FnMut(&dyn Event) -> bool,
    {
        let mut processed = 0;

        while processed < max_events {
            if let Some(event) = self.dequeue() {
                handler(event.as_ref());
                processed += 1;
            } else {
                break;
            }
        }

        processed
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> &EventQueueStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = EventQueueStats {
            current_size: self.stats.current_size,
            ..Default::default()
        };
    }

    /// Get queue configuration
    pub fn get_config(&self) -> &EventQueueConfig {
        &self.config
    }

    /// Update queue configuration
    pub fn set_config(&mut self, config: EventQueueConfig) {
        self.config = config;

        // If max size decreased, drop events if needed
        while self.events.len() > self.config.max_size {
            self.drop_oldest_event();
        }
    }

    /// Remove events matching a predicate
    pub fn remove_events<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&dyn Event) -> bool,
    {
        let old_len = self.events.len();

        // Convert to vec, filter, then rebuild heap
        let mut events: Vec<_> = self.events.drain().collect();
        events.retain(|qe| !predicate(qe.event.as_ref()));

        self.events = events.into_iter().collect();
        self.stats.current_size = self.events.len();

        old_len - self.events.len()
    }

    /// Get events by priority level
    pub fn count_by_priority(&self, priority: EventPriority) -> usize {
        self.events
            .iter()
            .filter(|qe| qe.priority == priority)
            .count()
    }

    fn drop_oldest_event(&mut self) {
        // Find and remove the oldest event (highest queued_at time)
        if let Some(oldest_idx) = self
            .events
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.queued_at
                    .partial_cmp(&b.1.queued_at)
                    .unwrap_or(Ordering::Equal)
            })
            .map(|(idx, _)| idx)
        {
            // Convert to vec to remove by index
            let mut events: Vec<_> = self.events.drain().collect();
            events.remove(oldest_idx);
            self.events = events.into_iter().collect();

            self.stats.events_dropped += 1;
            self.stats.current_size = self.events.len();
        }
    }

    fn cleanup_expired_events(&mut self) {
        if let Some(max_age) = self.config.max_event_age {
            let current_time = get_current_time();
            // Convert to vec, filter expired events, then rebuild heap
            let mut events: Vec<_> = self.events.drain().collect();
            let old_len = events.len();

            events.retain(|qe| {
                let age = current_time - qe.queued_at;
                age <= max_age as f64
            });

            let expired_count = old_len - events.len();
            self.events = events.into_iter().collect();

            self.stats.events_expired += expired_count as u64;
            self.stats.current_size = self.events.len();
        }
    }

    fn update_avg_queue_time(&mut self, queue_time: f32) {
        let weight = 0.1;
        self.stats.avg_queue_time =
            self.stats.avg_queue_time * (1.0 - weight) + queue_time * weight;
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple FIFO event queue for when priority ordering is not needed
pub struct FifoEventQueue {
    events: VecDeque<Box<dyn Event>>,
    config: EventQueueConfig,
    stats: EventQueueStats,
}

impl FifoEventQueue {
    /// Create a new FIFO event queue
    pub fn new() -> Self {
        Self::with_config(EventQueueConfig {
            priority_ordering: false,
            ..Default::default()
        })
    }

    /// Create a new FIFO event queue with configuration
    pub fn with_config(config: EventQueueConfig) -> Self {
        Self {
            events: VecDeque::with_capacity(config.max_size.min(1000)),
            config,
            stats: EventQueueStats::default(),
        }
    }

    /// Add an event to the back of the queue
    pub fn enqueue(&mut self, event: Box<dyn Event>) -> Result<()> {
        if self.events.len() >= self.config.max_size {
            if self.config.drop_old_on_full {
                self.events.pop_front();
                self.stats.events_dropped += 1;
            } else {
                self.stats.events_dropped += 1;
                return Err(EventError::QueueFull);
            }
        }

        self.events.push_back(event);
        self.stats.total_queued += 1;
        self.stats.current_size = self.events.len();

        Ok(())
    }

    /// Remove and return the front event
    pub fn dequeue(&mut self) -> Option<Box<dyn Event>> {
        if let Some(event) = self.events.pop_front() {
            self.stats.total_processed += 1;
            self.stats.current_size = self.events.len();
            Some(event)
        } else {
            None
        }
    }

    /// Get the number of events in the queue
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.stats.current_size = 0;
    }
}

impl Default for FifoEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current time in seconds (implementation would use actual time source)
fn get_current_time() -> f64 {
    // In a real implementation, this would use std::time::SystemTime or similar
    // For now, return a placeholder
    0.0
}
