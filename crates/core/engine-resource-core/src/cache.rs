//! Resource caching abstractions with eviction policies

use crate::manager::CachePolicy;
use crate::{ResourceHandle, ResourceId, WeakResourceHandle};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Trait for resource cache implementations
pub trait ResourceCache: Send + Sync {
    /// Insert a resource into the cache
    fn insert(&mut self, id: ResourceId, resource: Arc<dyn Any + Send + Sync>) -> CacheResult<()>;

    /// Get a resource from the cache
    fn get(&mut self, id: ResourceId) -> Option<Arc<dyn Any + Send + Sync>>;

    /// Remove a specific resource from the cache
    fn remove(&mut self, id: ResourceId) -> CacheResult<bool>;

    /// Clear all resources from the cache
    fn clear(&mut self);

    /// Get the current cache size in bytes
    fn size_bytes(&self) -> usize;

    /// Get the number of resources in the cache
    fn len(&self) -> usize;

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Set the cache policy
    fn set_policy(&mut self, policy: CachePolicy);

    /// Get the current cache policy
    fn policy(&self) -> CachePolicy;

    /// Force eviction of resources based on policy
    fn evict(&mut self, target_size: Option<usize>) -> CacheResult<Vec<ResourceId>>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;

    /// Check if cache can accommodate a resource of given size
    fn can_fit(&self, size_bytes: usize) -> bool;

    /// Get all resource IDs currently in cache
    fn resource_ids(&self) -> Vec<ResourceId>;

    /// Update access time for a resource (for LRU policies)
    fn touch(&mut self, id: ResourceId);

    /// Get memory pressure level (0.0 = no pressure, 1.0 = maximum pressure)
    fn memory_pressure(&self) -> f32;
}

/// Cache operation errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache is full and cannot accommodate resource")]
    CacheFull,

    #[error("Resource not found in cache: {id}")]
    NotFound { id: ResourceId },

    #[error("Cache size limit exceeded")]
    SizeLimitExceeded,

    #[error("Invalid cache policy configuration")]
    InvalidPolicy,

    #[error("Memory allocation failed")]
    AllocationFailed,

    #[error("Cache corruption detected")]
    CorruptionDetected,

    #[error("Eviction failed: {reason}")]
    EvictionFailed { reason: String },

    #[error("Cache is locked for maintenance")]
    CacheLocked,
}

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache statistics and metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,

    /// Total number of cache misses
    pub misses: u64,

    /// Number of resources evicted
    pub evictions: u64,

    /// Total bytes of resources evicted
    pub evicted_bytes: u64,

    /// Current cache size in bytes
    pub current_size_bytes: usize,

    /// Maximum cache size in bytes (if limited)
    pub max_size_bytes: Option<usize>,

    /// Number of resources currently cached
    pub resource_count: usize,

    /// Average resource size in bytes
    pub average_resource_size: f64,

    /// Cache hit rate as percentage
    pub hit_rate: f64,

    /// Time spent on cache operations (microseconds)
    pub total_operation_time_us: u64,

    /// Number of cache operations performed
    pub operation_count: u64,

    /// Memory fragmentation level (0.0 to 1.0)
    pub fragmentation: f32,
}

impl CacheStats {
    /// Calculate hit rate percentage
    pub fn calculate_hit_rate(&mut self) {
        let total_requests = self.hits + self.misses;
        self.hit_rate = if total_requests > 0 {
            (self.hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Calculate average resource size
    pub fn calculate_average_size(&mut self) {
        self.average_resource_size = if self.resource_count > 0 {
            self.current_size_bytes as f64 / self.resource_count as f64
        } else {
            0.0
        };
    }

    /// Get average operation time in microseconds
    pub fn average_operation_time_us(&self) -> f64 {
        if self.operation_count > 0 {
            self.total_operation_time_us as f64 / self.operation_count as f64
        } else {
            0.0
        }
    }

    /// Check if cache is under memory pressure
    pub fn is_under_pressure(&self) -> bool {
        if let Some(max_size) = self.max_size_bytes {
            let usage_ratio = self.current_size_bytes as f64 / max_size as f64;
            usage_ratio > 0.8 // Consider 80% usage as pressure
        } else {
            false
        }
    }
}

/// Cache entry metadata for eviction policies
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Resource ID
    pub id: ResourceId,

    /// The cached resource data
    pub resource: Arc<dyn Any + Send + Sync>,

    /// Size of the resource in bytes
    pub size_bytes: usize,

    /// When the resource was first cached
    pub created_at: SystemTime,

    /// When the resource was last accessed
    pub last_accessed: SystemTime,

    /// Number of times this resource has been accessed
    pub access_count: u64,

    /// Priority level for eviction (higher = keep longer)
    pub priority: i32,

    /// Whether this resource should never be evicted
    pub pinned: bool,

    /// Custom metadata for eviction decisions
    pub metadata: HashMap<String, String>,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(id: ResourceId, resource: Arc<dyn Any + Send + Sync>, size_bytes: usize) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            resource,
            size_bytes,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            priority: 0,
            pinned: false,
            metadata: HashMap::new(),
        }
    }

    /// Update access information
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }

    /// Get age of this entry
    pub fn age(&self) -> Option<Duration> {
        SystemTime::now().duration_since(self.created_at).ok()
    }

    /// Get time since last access
    pub fn time_since_access(&self) -> Option<Duration> {
        SystemTime::now().duration_since(self.last_accessed).ok()
    }

    /// Calculate eviction score (higher = more likely to be evicted)
    pub fn eviction_score(&self, policy: CachePolicy) -> f64 {
        if self.pinned {
            return 0.0; // Never evict pinned resources
        }

        match policy {
            CachePolicy::LeastRecentlyUsed => self
                .time_since_access()
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            CachePolicy::LeastFrequentlyUsed => {
                // Invert access count so lower access = higher score
                1.0 / (self.access_count as f64 + 1.0)
            }
            CachePolicy::FirstInFirstOut => self.age().map(|d| d.as_secs_f64()).unwrap_or(0.0),
            CachePolicy::Custom { .. } => {
                // Weighted combination of factors
                let age_factor = self.age().map(|d| d.as_secs_f64()).unwrap_or(0.0) * 0.3;
                let access_factor = (1.0 / (self.access_count as f64 + 1.0)) * 0.4;
                let size_factor = (self.size_bytes as f64 / (1024.0 * 1024.0)) * 0.2; // MB
                let priority_factor = (-self.priority as f64) * 0.1;

                age_factor + access_factor + size_factor + priority_factor
            }
            CachePolicy::NoEviction => 0.0, // Never evict
        }
    }
}

/// Eviction strategy implementation
pub trait EvictionStrategy: Send + Sync {
    /// Select resources to evict based on the strategy
    fn select_for_eviction(&self, entries: &[CacheEntry], target_bytes: usize) -> Vec<ResourceId>;

    /// Get the name of this eviction strategy
    fn name(&self) -> &'static str;

    /// Check if this strategy supports the given cache policy
    fn supports_policy(&self, policy: CachePolicy) -> bool;
}

/// LRU (Least Recently Used) eviction strategy
pub struct LruEvictionStrategy;

impl EvictionStrategy for LruEvictionStrategy {
    fn select_for_eviction(&self, entries: &[CacheEntry], target_bytes: usize) -> Vec<ResourceId> {
        let mut sortable_entries: Vec<_> = entries.iter().collect();

        // Sort by last accessed time (oldest first)
        sortable_entries.sort_by(|a, b| a.last_accessed.cmp(&b.last_accessed));

        let mut selected = Vec::new();
        let mut freed_bytes = 0;

        for entry in sortable_entries {
            if !entry.pinned && freed_bytes < target_bytes {
                selected.push(entry.id);
                freed_bytes += entry.size_bytes;
            }
        }

        selected
    }

    fn name(&self) -> &'static str {
        "LRU"
    }

    fn supports_policy(&self, policy: CachePolicy) -> bool {
        matches!(policy, CachePolicy::LeastRecentlyUsed)
    }
}

/// LFU (Least Frequently Used) eviction strategy
pub struct LfuEvictionStrategy;

impl EvictionStrategy for LfuEvictionStrategy {
    fn select_for_eviction(&self, entries: &[CacheEntry], target_bytes: usize) -> Vec<ResourceId> {
        let mut sortable_entries: Vec<_> = entries.iter().collect();

        // Sort by access count (least accessed first)
        sortable_entries.sort_by(|a, b| a.access_count.cmp(&b.access_count));

        let mut selected = Vec::new();
        let mut freed_bytes = 0;

        for entry in sortable_entries {
            if !entry.pinned && freed_bytes < target_bytes {
                selected.push(entry.id);
                freed_bytes += entry.size_bytes;
            }
        }

        selected
    }

    fn name(&self) -> &'static str {
        "LFU"
    }

    fn supports_policy(&self, policy: CachePolicy) -> bool {
        matches!(policy, CachePolicy::LeastFrequentlyUsed)
    }
}

/// FIFO (First In, First Out) eviction strategy
pub struct FifoEvictionStrategy;

impl EvictionStrategy for FifoEvictionStrategy {
    fn select_for_eviction(&self, entries: &[CacheEntry], target_bytes: usize) -> Vec<ResourceId> {
        let mut sortable_entries: Vec<_> = entries.iter().collect();

        // Sort by creation time (oldest first)
        sortable_entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let mut selected = Vec::new();
        let mut freed_bytes = 0;

        for entry in sortable_entries {
            if !entry.pinned && freed_bytes < target_bytes {
                selected.push(entry.id);
                freed_bytes += entry.size_bytes;
            }
        }

        selected
    }

    fn name(&self) -> &'static str {
        "FIFO"
    }

    fn supports_policy(&self, policy: CachePolicy) -> bool {
        matches!(policy, CachePolicy::FirstInFirstOut)
    }
}

/// Cache configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes
    pub max_size_bytes: Option<usize>,

    /// Maximum number of resources to cache
    pub max_resource_count: Option<usize>,

    /// Cache policy to use
    pub policy: CachePolicy,

    /// Enable cache statistics collection
    pub enable_stats: bool,

    /// Automatic cleanup interval in seconds
    pub cleanup_interval_seconds: f32,

    /// Memory pressure threshold for triggering eviction
    pub pressure_threshold: f32,

    /// Pre-allocate cache storage
    pub preallocate: bool,

    /// Enable cache validation checks
    pub enable_validation: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: Some(512 * 1024 * 1024), // 512MB default
            max_resource_count: Some(10000),
            policy: CachePolicy::LeastRecentlyUsed,
            enable_stats: true,
            cleanup_interval_seconds: 30.0,
            pressure_threshold: 0.8,
            preallocate: false,
            enable_validation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_entry() {
        let id = ResourceId::new(123);
        let resource: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test"));
        let mut entry = CacheEntry::new(id, resource, 100);

        assert_eq!(entry.access_count, 1);

        thread::sleep(Duration::from_millis(10));
        entry.touch();

        assert_eq!(entry.access_count, 2);
        assert!(entry.time_since_access().unwrap() < Duration::from_millis(5));
    }

    #[test]
    fn test_eviction_score() {
        let id = ResourceId::new(456);
        let resource: Arc<dyn Any + Send + Sync> = Arc::new(Vec::<u8>::new());
        let mut entry = CacheEntry::new(id, resource, 1000);

        // Test LRU scoring
        let lru_score = entry.eviction_score(CachePolicy::LeastRecentlyUsed);
        assert!(lru_score >= 0.0);

        // Touch the entry and check score changes
        thread::sleep(Duration::from_millis(10));
        entry.touch();
        let new_lru_score = entry.eviction_score(CachePolicy::LeastRecentlyUsed);
        // Since we just touched it, the time since access should be smaller (score should be smaller)
        assert!(new_lru_score <= lru_score);

        // Test pinned entry
        entry.pinned = true;
        assert_eq!(entry.eviction_score(CachePolicy::LeastRecentlyUsed), 0.0);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        stats.hits = 80;
        stats.misses = 20;
        stats.resource_count = 100;
        stats.current_size_bytes = 1024 * 1024; // 1MB

        stats.calculate_hit_rate();
        stats.calculate_average_size();

        assert_eq!(stats.hit_rate, 80.0);
        assert_eq!(stats.average_resource_size, 10485.76); // 1MB / 100 resources
    }

    #[test]
    fn test_lru_eviction_strategy() {
        let strategy = LruEvictionStrategy;
        assert_eq!(strategy.name(), "LRU");
        assert!(strategy.supports_policy(CachePolicy::LeastRecentlyUsed));
        assert!(!strategy.supports_policy(CachePolicy::LeastFrequentlyUsed));
    }
}
