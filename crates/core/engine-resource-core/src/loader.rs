//! Resource loader trait definitions and abstractions

use crate::metadata::ResourceMetadata;
use crate::state::{LoadingPriority, LoadingState};
use crate::{ResourceHandle, ResourceId};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::io;
use std::path::Path;
use std::sync::Arc;

/// Core trait for resource loaders
pub trait ResourceLoader: Send + Sync {
    /// Load a resource from the given path
    fn load(
        &self,
        path: &Path,
        metadata: &ResourceMetadata,
    ) -> LoaderResult<Arc<dyn Any + Send + Sync>>;

    /// Load a resource asynchronously
    fn load_async(&self, path: &Path, metadata: &ResourceMetadata) -> LoaderResult<LoaderFuture>;

    /// Check if this loader can handle the given file extension
    fn can_load(&self, extension: &str) -> bool;

    /// Get the supported file extensions
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// Get the resource type this loader produces
    fn resource_type(&self) -> &'static str;

    /// Estimate the memory size of a resource before loading
    fn estimate_size(&self, path: &Path) -> LoaderResult<usize>;

    /// Validate that a file can be loaded without actually loading it
    fn validate(&self, path: &Path) -> LoaderResult<()>;

    /// Get loader-specific configuration options
    fn config(&self) -> LoaderConfig;

    /// Set loader-specific configuration
    fn set_config(&mut self, config: LoaderConfig) -> LoaderResult<()>;

    /// Get dependencies for this resource (other resources it needs)
    fn dependencies(&self, path: &Path) -> LoaderResult<Vec<String>>;

    /// Load only metadata without the full resource
    fn load_metadata(&self, path: &Path) -> LoaderResult<ResourceMetadata>;

    /// Support for streaming/progressive loading
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Start streaming load (for large resources)
    fn start_streaming(&self, path: &Path) -> LoaderResult<Box<dyn StreamingLoader>> {
        Err(LoaderError::StreamingNotSupported)
    }

    /// Cache the loader's state for faster subsequent loads
    fn cache_state(&mut self) -> LoaderResult<()> {
        Ok(())
    }

    /// Clear any cached state
    fn clear_cache(&mut self) -> LoaderResult<()> {
        Ok(())
    }
}

/// Future type for async loading operations
pub type LoaderFuture =
    Box<dyn std::future::Future<Output = LoaderResult<Arc<dyn Any + Send + Sync>>> + Send + Unpin>;

/// Trait for streaming resource loaders
pub trait StreamingLoader: Send + Sync {
    /// Get the next chunk of data
    fn next_chunk(&mut self) -> LoaderResult<Option<Vec<u8>>>;

    /// Get loading progress (0.0 to 1.0)
    fn progress(&self) -> f32;

    /// Check if loading is complete
    fn is_complete(&self) -> bool;

    /// Get the partially loaded resource
    fn partial_resource(&self) -> Option<Arc<dyn Any + Send + Sync>>;

    /// Finalize and get the complete resource
    fn finalize(self: Box<Self>) -> LoaderResult<Arc<dyn Any + Send + Sync>>;

    /// Cancel the streaming operation
    fn cancel(&mut self) -> LoaderResult<()>;

    /// Estimate total size of the resource
    fn estimated_total_size(&self) -> Option<usize>;

    /// Get current loaded size
    fn current_loaded_size(&self) -> usize;
}

/// Resource loader errors
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Invalid file format: {path}")]
    InvalidFormat { path: String },

    #[error("Unsupported file type: {extension}")]
    UnsupportedType { extension: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Dependency missing: {dependency}")]
    DependencyMissing { dependency: String },

    #[error("Loading timeout: {path}")]
    Timeout { path: String },

    #[error("Memory allocation failed")]
    OutOfMemory,

    #[error("Loader configuration error: {message}")]
    ConfigError { message: String },

    #[error("Streaming not supported by this loader")]
    StreamingNotSupported,

    #[error("Loading cancelled")]
    Cancelled,

    #[error("Corruption detected in: {path}")]
    CorruptedData { path: String },

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Custom loader error: {message}")]
    Custom { message: String },
}

/// Result type for loader operations
pub type LoaderResult<T> = Result<T, LoaderError>;

/// Configuration for resource loaders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderConfig {
    /// Maximum file size this loader will handle
    pub max_file_size: Option<usize>,

    /// Timeout for loading operations in seconds
    pub timeout_seconds: f32,

    /// Buffer size for file reading
    pub buffer_size: usize,

    /// Enable compression detection
    pub detect_compression: bool,

    /// Enable file validation
    pub validate_files: bool,

    /// Cache parsed data for faster reloading
    pub enable_caching: bool,

    /// Custom loader-specific options
    pub custom_options: std::collections::HashMap<String, String>,

    /// Thread pool size for async operations
    pub thread_pool_size: Option<usize>,

    /// Enable progress reporting
    pub enable_progress: bool,

    /// Chunk size for streaming operations
    pub streaming_chunk_size: usize,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            timeout_seconds: 30.0,
            buffer_size: 64 * 1024, // 64KB
            detect_compression: true,
            validate_files: true,
            enable_caching: true,
            custom_options: std::collections::HashMap::new(),
            thread_pool_size: None, // Use system default
            enable_progress: true,
            streaming_chunk_size: 1024 * 1024, // 1MB chunks
        }
    }
}

/// Registry for resource loaders
pub trait LoaderRegistry: Send + Sync {
    /// Register a loader for specific file extensions
    fn register(&mut self, extensions: Vec<String>, loader: Box<dyn ResourceLoader>);

    /// Get a loader for the given file extension
    fn get_loader(&self, extension: &str) -> Option<&dyn ResourceLoader>;

    /// Get a mutable loader for configuration
    fn get_loader_mut(&mut self, extension: &str) -> Option<&mut dyn ResourceLoader>;

    /// Unregister a loader for specific extensions
    fn unregister(&mut self, extensions: &[String]) -> LoaderResult<()>;

    /// Get all registered extensions
    fn registered_extensions(&self) -> Vec<String>;

    /// Check if an extension is supported
    fn supports_extension(&self, extension: &str) -> bool;

    /// Get loader information for debugging
    fn loader_info(&self) -> Vec<LoaderInfo>;

    /// Clear all registered loaders
    fn clear(&mut self);

    /// Load resource using appropriate loader
    fn load_resource(&self, path: &Path) -> LoaderResult<Arc<dyn Any + Send + Sync>>;

    /// Load resource metadata using appropriate loader
    fn load_metadata(&self, path: &Path) -> LoaderResult<ResourceMetadata>;
}

/// Information about a registered loader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderInfo {
    /// Extensions handled by this loader
    pub extensions: Vec<String>,

    /// Resource type produced
    pub resource_type: String,

    /// Whether streaming is supported
    pub supports_streaming: bool,

    /// Configuration options
    pub config: LoaderConfig,

    /// Loader-specific metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Common file format detection utilities
pub struct FormatDetector;

impl FormatDetector {
    /// Detect file format from extension
    pub fn from_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Detect file format from magic bytes
    pub fn from_magic_bytes(data: &[u8]) -> Option<String> {
        if data.len() < 4 {
            return None;
        }

        match &data[0..4] {
            [0x89, 0x50, 0x4E, 0x47] => Some("png".to_string()),
            [0xFF, 0xD8, 0xFF, _] => Some("jpg".to_string()),
            [0x47, 0x49, 0x46, 0x38] => Some("gif".to_string()),
            [0x52, 0x49, 0x46, 0x46] => Some("wav".to_string()),
            [0x4F, 0x67, 0x67, 0x53] => Some("ogg".to_string()),
            _ => None,
        }
    }

    /// Detect compression format
    pub fn detect_compression(data: &[u8]) -> Option<String> {
        if data.len() < 4 {
            return None;
        }

        match &data[0..4] {
            [0x1F, 0x8B, _, _] => Some("gzip".to_string()),
            [0x42, 0x5A, 0x68, _] => Some("bzip2".to_string()),
            [0xFD, 0x37, 0x7A, 0x58] => Some("xz".to_string()),
            [0x50, 0x4B, 0x03, 0x04] => Some("zip".to_string()),
            _ => None,
        }
    }

    /// Validate file header
    pub fn validate_header(data: &[u8], expected_format: &str) -> bool {
        match expected_format.to_lowercase().as_str() {
            "png" => data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            "jpg" | "jpeg" => data.starts_with(&[0xFF, 0xD8, 0xFF]),
            "gif" => data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a"),
            "bmp" => data.starts_with(b"BM"),
            "wav" => data.starts_with(b"RIFF") && data[8..12] == *b"WAVE",
            "ogg" => data.starts_with(b"OggS"),
            "mp3" => data.starts_with(b"ID3") || data.starts_with(&[0xFF, 0xFB]),
            _ => true, // Unknown format, assume valid
        }
    }
}

/// Loading context for passing additional information to loaders
#[derive(Debug, Clone)]
pub struct LoadingContext {
    /// Base path for relative resource references
    pub base_path: String,

    /// Loading priority
    pub priority: LoadingPriority,

    /// Custom metadata for this loading operation
    pub metadata: std::collections::HashMap<String, String>,

    /// Whether to load dependencies automatically
    pub auto_load_dependencies: bool,

    /// Maximum recursion depth for dependencies
    pub max_dependency_depth: u32,

    /// Current recursion depth
    pub current_depth: u32,

    /// Loading session ID for tracking
    pub session_id: Option<String>,
}

impl Default for LoadingContext {
    fn default() -> Self {
        Self {
            base_path: String::new(),
            priority: LoadingPriority::Normal,
            metadata: std::collections::HashMap::new(),
            auto_load_dependencies: true,
            max_dependency_depth: 10,
            current_depth: 0,
            session_id: None,
        }
    }
}

/// Progress information for loading operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingProgress {
    /// Current progress (0.0 to 1.0)
    pub progress: f32,

    /// Current operation description
    pub description: String,

    /// Bytes loaded so far
    pub bytes_loaded: usize,

    /// Total bytes to load (if known)
    pub total_bytes: Option<usize>,

    /// Estimated time remaining in seconds
    pub eta_seconds: Option<f32>,

    /// Current loading stage
    pub stage: LoadingStage,

    /// Any warnings or non-fatal errors
    pub warnings: Vec<String>,
}

/// Stages of the loading process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadingStage {
    /// Initializing the loading operation
    Initializing,

    /// Reading file data
    Reading,

    /// Parsing/decoding the data
    Parsing,

    /// Loading dependencies
    Dependencies,

    /// Post-processing
    PostProcessing,

    /// Finalizing
    Finalizing,

    /// Completed successfully
    Complete,

    /// Failed with error
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        // Test PNG magic bytes
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(
            FormatDetector::from_magic_bytes(&png_data),
            Some("png".to_string())
        );

        // Test JPEG magic bytes
        let jpg_data = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(
            FormatDetector::from_magic_bytes(&jpg_data),
            Some("jpg".to_string())
        );

        // Test validation
        assert!(FormatDetector::validate_header(&png_data, "png"));
        assert!(!FormatDetector::validate_header(&jpg_data, "png"));
    }

    #[test]
    fn test_compression_detection() {
        // Test GZIP magic bytes
        let gzip_data = [0x1F, 0x8B, 0x08, 0x00];
        assert_eq!(
            FormatDetector::detect_compression(&gzip_data),
            Some("gzip".to_string())
        );

        // Test ZIP magic bytes
        let zip_data = [0x50, 0x4B, 0x03, 0x04];
        assert_eq!(
            FormatDetector::detect_compression(&zip_data),
            Some("zip".to_string())
        );
    }

    #[test]
    fn test_loading_context() {
        let mut context = LoadingContext::default();
        assert_eq!(context.current_depth, 0);
        assert!(context.auto_load_dependencies);

        context
            .metadata
            .insert("test".to_string(), "value".to_string());
        assert_eq!(context.metadata.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_loading_progress() {
        let progress = LoadingProgress {
            progress: 0.5,
            description: "Loading texture".to_string(),
            bytes_loaded: 512,
            total_bytes: Some(1024),
            eta_seconds: Some(10.0),
            stage: LoadingStage::Reading,
            warnings: vec!["Minor format issue".to_string()],
        };

        assert_eq!(progress.progress, 0.5);
        assert_eq!(progress.stage, LoadingStage::Reading);
        assert_eq!(progress.warnings.len(), 1);
    }

    #[test]
    fn test_loader_config() {
        let config = LoaderConfig::default();
        assert_eq!(config.max_file_size, Some(100 * 1024 * 1024));
        assert_eq!(config.timeout_seconds, 30.0);
        assert!(config.validate_files);
        assert!(config.enable_caching);
    }
}
