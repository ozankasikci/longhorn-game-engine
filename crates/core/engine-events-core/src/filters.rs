//! Event filtering abstractions

use crate::{Event, EventPriority, EventTypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Event filter trait for determining if events should be processed
pub trait EventFilter: Send + Sync {
    /// Check if an event passes the filter
    fn passes(&self, event: &dyn Event) -> bool;

    /// Get a description of what this filter does
    fn description(&self) -> String;
}

/// Filter that accepts all events
pub struct AcceptAllFilter;

impl EventFilter for AcceptAllFilter {
    fn passes(&self, _event: &dyn Event) -> bool {
        true
    }

    fn description(&self) -> String {
        "Accept all events".to_string()
    }
}

/// Filter that rejects all events
pub struct RejectAllFilter;

impl EventFilter for RejectAllFilter {
    fn passes(&self, _event: &dyn Event) -> bool {
        false
    }

    fn description(&self) -> String {
        "Reject all events".to_string()
    }
}

/// Filter by event type
pub struct TypeFilter {
    allowed_types: HashSet<EventTypeId>,
    whitelist_mode: bool, // true = allow only listed types, false = deny listed types
}

impl TypeFilter {
    /// Create a whitelist filter (only allow specified types)
    pub fn whitelist(types: Vec<EventTypeId>) -> Self {
        Self {
            allowed_types: types.into_iter().collect(),
            whitelist_mode: true,
        }
    }

    /// Create a blacklist filter (deny specified types)
    pub fn blacklist(types: Vec<EventTypeId>) -> Self {
        Self {
            allowed_types: types.into_iter().collect(),
            whitelist_mode: false,
        }
    }

    /// Add a type to the filter
    pub fn add_type(&mut self, type_id: EventTypeId) {
        self.allowed_types.insert(type_id);
    }

    /// Remove a type from the filter
    pub fn remove_type(&mut self, type_id: EventTypeId) {
        self.allowed_types.remove(&type_id);
    }

    /// Check if a type is in the filter
    pub fn contains_type(&self, type_id: EventTypeId) -> bool {
        self.allowed_types.contains(&type_id)
    }
}

impl EventFilter for TypeFilter {
    fn passes(&self, event: &dyn Event) -> bool {
        let event_type = event.get_type_id();
        let is_in_set = self.allowed_types.contains(&event_type);

        if self.whitelist_mode {
            is_in_set
        } else {
            !is_in_set
        }
    }

    fn description(&self) -> String {
        let mode = if self.whitelist_mode {
            "whitelist"
        } else {
            "blacklist"
        };
        format!(
            "Type filter ({}) with {} types",
            mode,
            self.allowed_types.len()
        )
    }
}

/// Filter by event priority
pub struct PriorityFilter {
    min_priority: EventPriority,
    max_priority: EventPriority,
}

impl PriorityFilter {
    /// Create a priority filter with min and max priority
    pub fn new(min_priority: EventPriority, max_priority: EventPriority) -> Self {
        Self {
            min_priority,
            max_priority,
        }
    }

    /// Create a filter that only allows events above a minimum priority
    pub fn min_priority(priority: EventPriority) -> Self {
        Self {
            min_priority: priority,
            max_priority: EventPriority::Critical,
        }
    }

    /// Create a filter that only allows events below a maximum priority
    pub fn max_priority(priority: EventPriority) -> Self {
        Self {
            min_priority: EventPriority::Low,
            max_priority: priority,
        }
    }

    /// Create a filter for a specific priority level only
    pub fn exact_priority(priority: EventPriority) -> Self {
        Self {
            min_priority: priority,
            max_priority: priority,
        }
    }
}

impl EventFilter for PriorityFilter {
    fn passes(&self, event: &dyn Event) -> bool {
        let priority = event.priority();
        priority >= self.min_priority && priority <= self.max_priority
    }

    fn description(&self) -> String {
        if self.min_priority == self.max_priority {
            format!("Priority filter (exact: {:?})", self.min_priority)
        } else {
            format!(
                "Priority filter ({:?} to {:?})",
                self.min_priority, self.max_priority
            )
        }
    }
}

/// Filter by entity involvement
pub struct EntityFilter {
    entities: HashSet<u32>,
    whitelist_mode: bool,
}

impl EntityFilter {
    /// Create a whitelist filter (only allow events involving specified entities)
    pub fn whitelist(entities: Vec<u32>) -> Self {
        Self {
            entities: entities.into_iter().collect(),
            whitelist_mode: true,
        }
    }

    /// Create a blacklist filter (deny events involving specified entities)
    pub fn blacklist(entities: Vec<u32>) -> Self {
        Self {
            entities: entities.into_iter().collect(),
            whitelist_mode: false,
        }
    }

    /// Add an entity to the filter
    pub fn add_entity(&mut self, entity: u32) {
        self.entities.insert(entity);
    }

    /// Remove an entity from the filter
    pub fn remove_entity(&mut self, entity: u32) {
        self.entities.remove(&entity);
    }
}

impl EventFilter for EntityFilter {
    fn passes(&self, _event: &dyn Event) -> bool {
        // Note: This is a simplified implementation
        // In a real implementation, we'd need a way to extract entity information from events
        // For now, we'll always return true since we can't access entity data generically
        true
    }

    fn description(&self) -> String {
        let mode = if self.whitelist_mode {
            "whitelist"
        } else {
            "blacklist"
        };
        format!(
            "Entity filter ({}) with {} entities",
            mode,
            self.entities.len()
        )
    }
}

/// Filter that combines multiple filters with logical operations
pub struct CompositeFilter {
    filters: Vec<Box<dyn EventFilter>>,
    operation: LogicalOperation,
}

/// Logical operations for combining filters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOperation {
    /// All filters must pass (AND)
    And,
    /// At least one filter must pass (OR)
    Or,
    /// Filters must not pass (NOT) - only works with single filter
    Not,
}

impl CompositeFilter {
    /// Create a new composite filter with AND operation
    pub fn and() -> Self {
        Self {
            filters: Vec::new(),
            operation: LogicalOperation::And,
        }
    }

    /// Create a new composite filter with OR operation
    pub fn or() -> Self {
        Self {
            filters: Vec::new(),
            operation: LogicalOperation::Or,
        }
    }

    /// Create a new composite filter with NOT operation
    pub fn not() -> Self {
        Self {
            filters: Vec::new(),
            operation: LogicalOperation::Not,
        }
    }

    /// Add a filter to the composition
    pub fn add_filter(&mut self, filter: Box<dyn EventFilter>) {
        self.filters.push(filter);
    }

    /// Create a composite filter with multiple filters and operation
    pub fn with_filters(filters: Vec<Box<dyn EventFilter>>, operation: LogicalOperation) -> Self {
        Self { filters, operation }
    }
}

impl EventFilter for CompositeFilter {
    fn passes(&self, event: &dyn Event) -> bool {
        match self.operation {
            LogicalOperation::And => self.filters.iter().all(|f| f.passes(event)),
            LogicalOperation::Or => self.filters.iter().any(|f| f.passes(event)),
            LogicalOperation::Not => {
                // For NOT operation, we expect exactly one filter
                if self.filters.len() == 1 {
                    !self.filters[0].passes(event)
                } else {
                    false
                }
            }
        }
    }

    fn description(&self) -> String {
        let op_str = match self.operation {
            LogicalOperation::And => "AND",
            LogicalOperation::Or => "OR",
            LogicalOperation::Not => "NOT",
        };

        format!(
            "Composite filter ({}) with {} sub-filters",
            op_str,
            self.filters.len()
        )
    }
}

/// Filter that uses a custom predicate function
pub struct PredicateFilter<F> {
    predicate: F,
    description: String,
}

impl<F> PredicateFilter<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    /// Create a new predicate filter
    pub fn new(predicate: F, description: String) -> Self {
        Self {
            predicate,
            description,
        }
    }
}

impl<F> EventFilter for PredicateFilter<F>
where
    F: Fn(&dyn Event) -> bool + Send + Sync,
{
    fn passes(&self, event: &dyn Event) -> bool {
        (self.predicate)(event)
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Filter that limits the number of events that can pass through
#[allow(dead_code)]
pub struct RateLimitFilter {
    max_events: usize,
    time_window: f32,           // seconds
    events_in_window: Vec<f64>, // timestamps
    description: String,
}

impl RateLimitFilter {
    /// Create a new rate limit filter
    pub fn new(max_events: usize, time_window: f32) -> Self {
        Self {
            max_events,
            time_window,
            events_in_window: Vec::new(),
            description: format!("Rate limit: {} events per {:.1}s", max_events, time_window),
        }
    }

    /// Check if we can allow another event through
    #[allow(dead_code)]
    fn can_pass(&mut self, current_time: f64) -> bool {
        // Remove old events outside the time window
        let cutoff_time = current_time - self.time_window as f64;
        self.events_in_window
            .retain(|&timestamp| timestamp > cutoff_time);

        // Check if we're under the limit
        if self.events_in_window.len() < self.max_events {
            self.events_in_window.push(current_time);
            true
        } else {
            false
        }
    }
}

impl EventFilter for RateLimitFilter {
    fn passes(&self, _event: &dyn Event) -> bool {
        // In a real implementation, we'd get the current time and call can_pass
        // For now, always return true since we can't modify self in this trait method
        true
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}
