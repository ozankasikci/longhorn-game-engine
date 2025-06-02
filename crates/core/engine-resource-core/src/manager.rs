//! Resource manager trait definitions and core abstractions

use crate::{ResourceHandle, WeakResourceHandle, ResourceId, ResourceState, LoadingState};
use crate::cache::ResourceCache;
use crate::loader::ResourceLoader;
use crate::metadata::ResourceMetadata;
use std::any::Any;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Core trait for resource managers
pub trait ResourceManager: Send + Sync {
    /// Load a resource by path, returning a handle
    fn load<T: 'static>(&mut self, path: &str) -> ResourceResult<ResourceHandle<T>>;
    
    /// Load a resource with specific priority
    fn load_with_priority<T: 'static>(
        &mut self, 
        path: &str, 
        priority: crate::state::LoadingPriority
    ) -> ResourceResult<ResourceHandle<T>>;
    
    /// Get a resource if it's already loaded
    fn get<T: 'static>(&self, handle: &ResourceHandle<T>) -> Option<Arc<T>>;
    
    /// Check if a resource is loaded
    fn is_loaded<T>(&self, handle: &ResourceHandle<T>) -> bool;
    
    /// Get the loading state of a resource
    fn get_loading_state(&self, id: ResourceId) -> Option<LoadingState>;
    
    /// Unload a specific resource
    fn unload(&mut self, id: ResourceId) -> ResourceResult<()>;
    
    /// Unload all resources of a specific type
    fn unload_type<T: 'static>(&mut self) -> ResourceResult<()>;
    
    /// Force garbage collection of unused resources
    fn collect_garbage(&mut self) -> usize;
    
    /// Get memory usage statistics
    fn memory_usage(&self) -> MemoryUsage;
    
    /// Set the cache policy for resource management
    fn set_cache_policy(&mut self, policy: CachePolicy);
    
    /// Register a custom loader for a file extension
    fn register_loader(&mut self, extension: &str, loader: Box<dyn ResourceLoader>);
    
    /// Preload resources from a manifest file
    fn preload_manifest(&mut self, manifest_path: &str) -> ResourceResult<Vec<ResourceId>>;
    
    /// Wait for all pending loads to complete
    fn wait_for_all(&self) -> ResourceResult<()>;
    
    /// Cancel loading of a specific resource
    fn cancel_loading(&mut self, id: ResourceId) -> ResourceResult<()>;
    
    /// Get all resources currently being tracked
    fn tracked_resources(&self) -> Vec<ResourceId>;
    
    /// Clear all resources and reset the manager
    fn clear(&mut self);
}

/// Resource manager errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceManagerError {
    #[error("No loader registered for extension: {extension}")]
    NoLoaderForExtension { extension: String },
    
    #[error("Resource already exists: {id}")]
    ResourceExists { id: ResourceId },
    
    #[error("Resource manager is shutting down")]
    ShuttingDown,
    
    #[error("Invalid manifest file: {path}")]
    InvalidManifest { path: String },
    
    #[error("Dependency resolution failed: {reason}")]
    DependencyResolutionFailed { reason: String },
    
    #[error("Resource type not supported: {type_name}")]
    UnsupportedType { type_name: String },
    
    #[error("Cache policy violation: {reason}")]
    CachePolicyViolation { reason: String },
    
    #[error("Resource locked by another operation")]
    ResourceLocked,
    
    #[error("Loading queue is full")]
    QueueFull,
    
    #[error("Resource manager configuration error: {reason}")]
    ConfigurationError { reason: String },
}

/// Cache policy for resource management
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Never evict resources automatically
    NoEviction,
    
    /// Evict least recently used resources when memory limit is reached
    LeastRecentlyUsed,
    
    /// Evict least frequently used resources when memory limit is reached
    LeastFrequentlyUsed,
    
    /// Evict oldest resources when memory limit is reached
    FirstInFirstOut,
    
    /// Custom eviction policy with configurable parameters
    Custom {
        max_memory_mb: u32,
        max_resource_count: u32,
        eviction_threshold: f32, // 0.0 to 1.0
    },
}

impl Default for CachePolicy {
    fn default() -> Self {
        CachePolicy::LeastRecentlyUsed
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Total memory used by all resources in bytes
    pub total_bytes: usize,
    
    /// Number of resources currently loaded
    pub resource_count: u32,
    
    /// Memory used by different resource types
    pub by_type: std::collections::HashMap<String, usize>,
    
    /// Peak memory usage since manager creation
    pub peak_bytes: usize,
    
    /// Memory limit in bytes (if set)
    pub limit_bytes: Option<usize>,
    
    /// Number of resources waiting to be loaded
    pub queued_count: u32,
    
    /// Number of resources currently being loaded
    pub loading_count: u32,
    
    /// Number of failed loading attempts
    pub failed_count: u32,
    
    /// Average loading time in milliseconds
    pub average_load_time_ms: f64,
}

impl MemoryUsage {
    /// Calculate memory usage percentage if limit is set
    pub fn usage_percentage(&self) -> Option<f32> {
        self.limit_bytes.map(|limit| {
            if limit > 0 {
                (self.total_bytes as f32 / limit as f32) * 100.0
            } else {
                0.0
            }
        })
    }
    
    /// Check if memory usage is above threshold
    pub fn is_above_threshold(&self, threshold: f32) -> bool {
        self.usage_percentage()
            .map(|usage| usage > threshold)
            .unwrap_or(false)
    }
    
    /// Get the largest resource type by memory usage
    pub fn largest_type(&self) -> Option<(&String, &usize)> {
        self.by_type.iter().max_by_key(|(_, &size)| size)
    }
}

/// Resource loading progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingProgress {
    /// Resources that have completed loading
    pub completed: u32,
    
    /// Resources currently being loaded
    pub in_progress: u32,
    
    /// Resources queued for loading
    pub queued: u32,
    
    /// Resources that failed to load
    pub failed: u32,
    
    /// Overall progress percentage (0.0 to 1.0)
    pub progress: f32,
    
    /// Estimated time remaining in seconds
    pub estimated_time_remaining: Option<f32>,
}

impl LoadingProgress {
    /// Calculate total number of resources
    pub fn total(&self) -> u32 {
        self.completed + self.in_progress + self.queued + self.failed
    }
    
    /// Check if all loading operations are complete
    pub fn is_complete(&self) -> bool {
        self.in_progress == 0 && self.queued == 0
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        let finished = self.completed + self.failed;
        if finished > 0 {
            self.completed as f32 / finished as f32
        } else {
            0.0
        }
    }
}

/// Configuration for resource manager behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagerConfig {
    /// Maximum memory limit in bytes
    pub memory_limit: Option<usize>,
    
    /// Maximum number of concurrent loading operations
    pub max_concurrent_loads: u32,
    
    /// Default cache policy
    pub cache_policy: CachePolicy,
    
    /// Number of retry attempts for failed loads
    pub max_retries: u32,
    
    /// Timeout for individual resource loads in seconds
    pub load_timeout_seconds: f32,
    
    /// Enable/disable automatic garbage collection
    pub auto_gc: bool,
    
    /// Garbage collection frequency in seconds
    pub gc_interval_seconds: f32,
    
    /// Threshold for triggering garbage collection (memory usage percentage)
    pub gc_threshold: f32,
    
    /// Enable detailed loading statistics
    pub enable_statistics: bool,
    
    /// Buffer size for async loading operations
    pub async_buffer_size: usize,
}

impl Default for ResourceManagerConfig {
    fn default() -> Self {
        Self {
            memory_limit: None,
            max_concurrent_loads: 8,
            cache_policy: CachePolicy::default(),
            max_retries: 3,
            load_timeout_seconds: 30.0,
            auto_gc: true,
            gc_interval_seconds: 10.0,
            gc_threshold: 80.0,
            enable_statistics: true,
            async_buffer_size: 1024 * 1024, // 1MB
        }
    }
}

/// Result type for resource manager operations
pub type ResourceResult<T> = Result<T, ResourceManagerError>;

/// Resource dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    /// The resource that depends on others
    pub resource_id: ResourceId,
    
    /// Resources that this resource depends on
    pub dependencies: Vec<ResourceId>,
    
    /// Whether dependencies must be loaded before this resource
    pub hard_dependencies: bool,
    
    /// Priority boost when dependencies are loaded
    pub priority_boost: i32,
}

/// Resource manifest for batch loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// Version of the manifest format
    pub version: String,
    
    /// Resources to preload
    pub resources: Vec<ManifestEntry>,
    
    /// Resource dependencies
    pub dependencies: Vec<ResourceDependency>,
    
    /// Default loading priority for unlisted resources
    pub default_priority: crate::state::LoadingPriority,
}

/// Entry in a resource manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Unique identifier for this resource
    pub id: String,
    
    /// File path to the resource
    pub path: String,
    
    /// Resource type hint
    pub resource_type: Option<String>,
    
    /// Loading priority
    pub priority: crate::state::LoadingPriority,
    
    /// Whether to keep this resource loaded permanently
    pub persistent: bool,
    
    /// Custom metadata for this resource
    pub metadata: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage() {
        let mut usage = MemoryUsage::default();
        usage.total_bytes = 1024 * 1024; // 1MB
        usage.limit_bytes = Some(10 * 1024 * 1024); // 10MB
        
        assert_eq!(usage.usage_percentage(), Some(10.0));
        assert!(!usage.is_above_threshold(50.0));
        assert!(usage.is_above_threshold(5.0));
    }
    
    #[test]
    fn test_loading_progress() {
        let progress = LoadingProgress {
            completed: 8,
            in_progress: 1,
            queued: 1,
            failed: 0,
            progress: 0.8,
            estimated_time_remaining: Some(10.0),
        };
        
        assert_eq!(progress.total(), 10);
        assert!(!progress.is_complete());
        assert_eq!(progress.success_rate(), 1.0);
    }
    
    #[test]
    fn test_cache_policy() {
        let policy = CachePolicy::Custom {
            max_memory_mb: 512,
            max_resource_count: 1000,
            eviction_threshold: 0.8,
        };
        
        assert!(matches!(policy, CachePolicy::Custom { .. }));
    }
}