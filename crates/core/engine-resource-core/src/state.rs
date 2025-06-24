//! Resource loading states and lifecycle management

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Current state of a resource in the loading pipeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    /// Resource is not loaded and not being loaded
    Unloaded,

    /// Resource loading has been requested but not yet started
    Queued,

    /// Resource is currently being loaded
    Loading,

    /// Resource has been successfully loaded and is ready for use
    Loaded,

    /// Resource loading failed with an error message
    Failed(String),
}

impl ResourceState {
    /// Check if the resource is available for use
    pub fn is_loaded(&self) -> bool {
        matches!(self, ResourceState::Loaded)
    }

    /// Check if the resource is currently being loaded
    pub fn is_loading(&self) -> bool {
        matches!(self, ResourceState::Loading | ResourceState::Queued)
    }

    /// Check if the resource loading failed
    pub fn is_failed(&self) -> bool {
        matches!(self, ResourceState::Failed(_))
    }

    /// Check if the resource is not loaded
    pub fn is_unloaded(&self) -> bool {
        matches!(self, ResourceState::Unloaded)
    }

    /// Get the error message if the resource failed to load
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ResourceState::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Detailed loading state with timing and progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingState {
    /// Current state of the resource
    pub state: ResourceState,

    /// When the loading was requested
    pub requested_at: Option<SystemTime>,

    /// When loading actually started
    pub started_at: Option<SystemTime>,

    /// When loading completed (successfully or with failure)
    pub completed_at: Option<SystemTime>,

    /// Progress percentage (0.0 to 1.0) for loading operations that support it
    pub progress: f32,

    /// Number of retry attempts if loading failed
    pub retry_count: u32,

    /// Priority level for loading (higher values = higher priority)
    pub priority: LoadingPriority,
}

/// Priority levels for resource loading
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum LoadingPriority {
    /// Background loading, no rush
    Low = 0,

    /// Normal priority
    #[default]
    Normal = 1,

    /// High priority, load as soon as possible
    High = 2,

    /// Critical priority, block other loading if necessary
    Critical = 3,
}

impl LoadingState {
    /// Create a new loading state in the unloaded state
    pub fn new() -> Self {
        Self {
            state: ResourceState::Unloaded,
            requested_at: None,
            started_at: None,
            completed_at: None,
            progress: 0.0,
            retry_count: 0,
            priority: LoadingPriority::Normal,
        }
    }

    /// Create a new loading state with specified priority
    pub fn with_priority(priority: LoadingPriority) -> Self {
        Self {
            priority,
            ..Self::new()
        }
    }

    /// Mark the resource as requested for loading
    pub fn mark_requested(&mut self) {
        self.state = ResourceState::Queued;
        self.requested_at = Some(SystemTime::now());
    }

    /// Mark the resource as starting to load
    pub fn mark_loading(&mut self) {
        self.state = ResourceState::Loading;
        self.started_at = Some(SystemTime::now());
        self.progress = 0.0;
    }

    /// Update the loading progress
    pub fn update_progress(&mut self, progress: f32) {
        if matches!(self.state, ResourceState::Loading) {
            self.progress = progress.clamp(0.0, 1.0);
        }
    }

    /// Mark the resource as successfully loaded
    pub fn mark_loaded(&mut self) {
        self.state = ResourceState::Loaded;
        self.completed_at = Some(SystemTime::now());
        self.progress = 1.0;
    }

    /// Mark the resource as failed to load
    pub fn mark_failed(&mut self, error: String) {
        self.state = ResourceState::Failed(error);
        self.completed_at = Some(SystemTime::now());
        self.retry_count += 1;
    }

    /// Mark the resource as unloaded
    pub fn mark_unloaded(&mut self) {
        self.state = ResourceState::Unloaded;
        // Keep timing information for debugging
    }

    /// Get the total loading time if completed
    pub fn loading_duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        }
    }

    /// Get the time since loading was requested
    pub fn time_since_requested(&self) -> Option<Duration> {
        self.requested_at
            .and_then(|start| SystemTime::now().duration_since(start).ok())
    }

    /// Check if this loading state should be retried
    pub fn should_retry(&self, max_retries: u32) -> bool {
        self.state.is_failed() && self.retry_count < max_retries
    }

    /// Reset for retry attempt
    pub fn reset_for_retry(&mut self) {
        self.state = ResourceState::Queued;
        self.started_at = None;
        self.completed_at = None;
        self.progress = 0.0;
        // Keep retry_count and requested_at for tracking
    }
}

impl Default for LoadingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about resource loading performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadingStats {
    /// Total number of resources loaded
    pub total_loaded: u64,

    /// Total number of failed loads
    pub total_failed: u64,

    /// Total number of resources currently loading
    pub currently_loading: u32,

    /// Average loading time in milliseconds
    pub average_loading_time_ms: f64,

    /// Peak memory usage in bytes
    pub peak_memory_usage: usize,

    /// Current memory usage in bytes
    pub current_memory_usage: usize,
}

impl LoadingStats {
    /// Update stats with a completed loading operation
    pub fn record_completed_load(&mut self, duration: Duration, success: bool) {
        if success {
            self.total_loaded += 1;
        } else {
            self.total_failed += 1;
        }

        // Update average loading time
        let total_operations = self.total_loaded + self.total_failed;
        if total_operations > 0 {
            let new_time_ms = duration.as_millis() as f64;
            self.average_loading_time_ms =
                (self.average_loading_time_ms * (total_operations - 1) as f64 + new_time_ms)
                    / total_operations as f64;
        }
    }

    /// Update memory usage statistics
    pub fn update_memory_usage(&mut self, current: usize) {
        self.current_memory_usage = current;
        self.peak_memory_usage = self.peak_memory_usage.max(current);
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_loaded + self.total_failed;
        if total > 0 {
            (self.total_loaded as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_resource_state() {
        assert!(ResourceState::Loaded.is_loaded());
        assert!(!ResourceState::Loading.is_loaded());

        assert!(ResourceState::Loading.is_loading());
        assert!(ResourceState::Queued.is_loading());
        assert!(!ResourceState::Loaded.is_loading());

        let failed_state = ResourceState::Failed("test error".to_string());
        assert!(failed_state.is_failed());
        assert_eq!(failed_state.error_message(), Some("test error"));
    }

    #[test]
    fn test_loading_state() {
        let mut state = LoadingState::new();

        assert!(state.state.is_unloaded());

        state.mark_requested();
        assert_eq!(state.state, ResourceState::Queued);
        assert!(state.requested_at.is_some());

        state.mark_loading();
        assert_eq!(state.state, ResourceState::Loading);
        assert!(state.started_at.is_some());

        state.update_progress(0.5);
        assert_eq!(state.progress, 0.5);

        state.mark_loaded();
        assert!(state.state.is_loaded());
        assert!(state.completed_at.is_some());
        assert_eq!(state.progress, 1.0);
    }

    #[test]
    fn test_loading_priority() {
        assert!(LoadingPriority::Critical > LoadingPriority::High);
        assert!(LoadingPriority::High > LoadingPriority::Normal);
        assert!(LoadingPriority::Normal > LoadingPriority::Low);
    }

    #[test]
    fn test_retry_logic() {
        let mut state = LoadingState::new();

        state.mark_failed("test error".to_string());
        assert!(state.should_retry(3));
        assert_eq!(state.retry_count, 1);

        state.reset_for_retry();
        assert_eq!(state.state, ResourceState::Queued);
        assert_eq!(state.retry_count, 1); // Should keep retry count
    }

    #[test]
    fn test_loading_stats() {
        let mut stats = LoadingStats::default();

        stats.record_completed_load(Duration::from_millis(100), true);
        stats.record_completed_load(Duration::from_millis(200), false);

        assert_eq!(stats.total_loaded, 1);
        assert_eq!(stats.total_failed, 1);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.average_loading_time_ms, 150.0);
    }
}
