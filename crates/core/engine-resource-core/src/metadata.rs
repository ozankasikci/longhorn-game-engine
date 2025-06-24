//! Resource metadata and type information

use crate::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Metadata associated with a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// Unique identifier for this resource
    pub id: ResourceId,

    /// Original file path
    pub path: PathBuf,

    /// Resource type information
    pub resource_type: ResourceType,

    /// File size in bytes
    pub file_size: usize,

    /// Estimated memory size when loaded
    pub memory_size: Option<usize>,

    /// File modification time
    pub modified_time: Option<SystemTime>,

    /// Time when metadata was created
    pub created_time: SystemTime,

    /// Resource version (for cache invalidation)
    pub version: String,

    /// Checksum/hash of the resource data
    pub checksum: Option<String>,

    /// MIME type if known
    pub mime_type: Option<String>,

    /// Dependencies on other resources
    pub dependencies: Vec<ResourceDependency>,

    /// Custom metadata fields
    pub custom_fields: HashMap<String, MetadataValue>,

    /// Loading hints for optimization
    pub loading_hints: LoadingHints,

    /// Tags for categorization and searching
    pub tags: Vec<String>,

    /// Whether this resource can be hot-reloaded
    pub supports_hot_reload: bool,

    /// Compression information if applicable
    pub compression: Option<CompressionInfo>,

    /// Platform-specific information
    pub platform_info: PlatformInfo,
}

impl ResourceMetadata {
    /// Create new metadata for a resource
    pub fn new(id: ResourceId, path: PathBuf, resource_type: ResourceType) -> Self {
        Self {
            id,
            path,
            resource_type,
            file_size: 0,
            memory_size: None,
            modified_time: None,
            created_time: SystemTime::now(),
            version: "1.0".to_string(),
            checksum: None,
            mime_type: None,
            dependencies: Vec::new(),
            custom_fields: HashMap::new(),
            loading_hints: LoadingHints::default(),
            tags: Vec::new(),
            supports_hot_reload: true,
            compression: None,
            platform_info: PlatformInfo::default(),
        }
    }

    /// Add a custom metadata field
    pub fn add_custom_field(&mut self, key: String, value: MetadataValue) {
        self.custom_fields.insert(key, value);
    }

    /// Get a custom metadata field
    pub fn get_custom_field(&self, key: &str) -> Option<&MetadataValue> {
        self.custom_fields.get(key)
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dependency: ResourceDependency) {
        self.dependencies.push(dependency);
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Check if metadata has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Calculate memory efficiency (memory_size / file_size ratio)
    pub fn memory_efficiency(&self) -> Option<f32> {
        if let Some(memory_size) = self.memory_size {
            if self.file_size > 0 {
                Some(memory_size as f32 / self.file_size as f32)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if the resource needs to be reloaded based on file modification time
    pub fn needs_reload(&self, file_modified_time: SystemTime) -> bool {
        if !self.supports_hot_reload {
            return false;
        }

        match self.modified_time {
            Some(cached_time) => file_modified_time > cached_time,
            None => true, // No cached time, assume needs reload
        }
    }

    /// Update modification time
    pub fn update_modified_time(&mut self, time: SystemTime) {
        self.modified_time = Some(time);
    }

    /// Get the file extension from path
    pub fn file_extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Get estimated loading time based on file size and type
    pub fn estimated_load_time(&self) -> Duration {
        let base_time = match self.resource_type {
            ResourceType::Texture => Duration::from_millis(10),
            ResourceType::Audio => Duration::from_millis(50),
            ResourceType::Model => Duration::from_millis(100),
            ResourceType::Script => Duration::from_millis(5),
            ResourceType::Shader => Duration::from_millis(20),
            ResourceType::Font => Duration::from_millis(30),
            ResourceType::Video => Duration::from_millis(200),
            ResourceType::Animation => Duration::from_millis(80),
            ResourceType::Scene => Duration::from_millis(150),
            ResourceType::Material => Duration::from_millis(10),
            ResourceType::Custom(_) => Duration::from_millis(50),
        };

        // Scale by file size (rough estimate: 1MB per 100ms base)
        let size_factor = (self.file_size as f64 / (1024.0 * 1024.0)).max(1.0);
        Duration::from_millis((base_time.as_millis() as f64 * size_factor) as u64)
    }
}

/// Types of resources that can be loaded
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Image/texture resources (PNG, JPG, etc.)
    Texture,

    /// Audio resources (WAV, MP3, OGG, etc.)
    Audio,

    /// 3D model resources (OBJ, FBX, GLTF, etc.)
    Model,

    /// Script resources (Lua, JavaScript, etc.)
    Script,

    /// Shader resources (GLSL, HLSL, WGSL, etc.)
    Shader,

    /// Font resources (TTF, OTF, etc.)
    Font,

    /// Video resources (MP4, AVI, etc.)
    Video,

    /// Animation resources
    Animation,

    /// Scene/level files
    Scene,

    /// Material definitions
    Material,

    /// Custom resource type
    Custom(String),
}

impl ResourceType {
    /// Get the canonical name of this resource type
    pub fn name(&self) -> &str {
        match self {
            ResourceType::Texture => "Texture",
            ResourceType::Audio => "Audio",
            ResourceType::Model => "Model",
            ResourceType::Script => "Script",
            ResourceType::Shader => "Shader",
            ResourceType::Font => "Font",
            ResourceType::Video => "Video",
            ResourceType::Animation => "Animation",
            ResourceType::Scene => "Scene",
            ResourceType::Material => "Material",
            ResourceType::Custom(name) => name,
        }
    }

    /// Detect resource type from file extension
    pub fn from_extension(extension: &str) -> Option<ResourceType> {
        match extension.to_lowercase().as_str() {
            // Texture formats
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "dds" | "ktx" | "astc" => {
                Some(ResourceType::Texture)
            }

            // Audio formats
            "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => Some(ResourceType::Audio),

            // Model formats
            "obj" | "fbx" | "dae" | "gltf" | "glb" | "blend" | "3ds" | "ply" => {
                Some(ResourceType::Model)
            }

            // Script formats
            "lua" | "js" | "ts" | "py" | "cs" | "cpp" | "c" | "h" => Some(ResourceType::Script),

            // Shader formats
            "glsl" | "hlsl" | "wgsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => {
                Some(ResourceType::Shader)
            }

            // Font formats
            "ttf" | "otf" | "woff" | "woff2" | "eot" => Some(ResourceType::Font),

            // Video formats
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => Some(ResourceType::Video),

            // Animation formats
            "anim" | "bvh" | "x3d" => Some(ResourceType::Animation),

            // Scene formats
            "scene" | "level" | "map" | "world" => Some(ResourceType::Scene),

            // Material formats
            "mat" | "material" | "mtl" => Some(ResourceType::Material),

            _ => None,
        }
    }

    /// Get common file extensions for this resource type
    pub fn common_extensions(&self) -> Vec<&'static str> {
        match self {
            ResourceType::Texture => vec!["png", "jpg", "jpeg", "bmp", "tga"],
            ResourceType::Audio => vec!["wav", "mp3", "ogg", "flac"],
            ResourceType::Model => vec!["obj", "fbx", "gltf", "glb"],
            ResourceType::Script => vec!["lua", "js", "py"],
            ResourceType::Shader => vec!["glsl", "vert", "frag", "wgsl"],
            ResourceType::Font => vec!["ttf", "otf"],
            ResourceType::Video => vec!["mp4", "avi", "mov"],
            ResourceType::Animation => vec!["anim", "fbx"],
            ResourceType::Scene => vec!["scene", "level"],
            ResourceType::Material => vec!["mat", "mtl"],
            ResourceType::Custom(_) => vec![],
        }
    }

    /// Check if this resource type typically uses a lot of memory
    pub fn is_memory_intensive(&self) -> bool {
        matches!(
            self,
            ResourceType::Texture | ResourceType::Model | ResourceType::Video | ResourceType::Audio
        )
    }

    /// Check if this resource type supports streaming
    pub fn supports_streaming(&self) -> bool {
        matches!(
            self,
            ResourceType::Audio | ResourceType::Video | ResourceType::Model
        )
    }
}

/// Custom metadata value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<MetadataValue>),
    Object(HashMap<String, MetadataValue>),
}

impl MetadataValue {
    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            MetadataValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            MetadataValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            MetadataValue::Float(f) => Some(*f),
            MetadataValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Convert to boolean if possible
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            MetadataValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Resource dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    /// Path to the dependent resource
    pub path: String,

    /// Type of dependency
    pub dependency_type: DependencyType,

    /// Whether this dependency is required for loading
    pub required: bool,

    /// Version requirement if applicable
    pub version_requirement: Option<String>,
}

/// Types of resource dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Direct reference (e.g., texture used by material)
    Reference,

    /// Include/import (e.g., script includes another script)
    Include,

    /// Template/parent (e.g., prefab instance)
    Template,

    /// Plugin/extension
    Plugin,

    /// Custom dependency type
    Custom(String),
}

/// Hints for optimizing resource loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingHints {
    /// Preferred loading priority
    pub priority: crate::state::LoadingPriority,

    /// Whether to preload this resource
    pub preload: bool,

    /// Whether to keep in memory permanently
    pub persistent: bool,

    /// Whether to compress in memory
    pub compress_in_memory: bool,

    /// Preferred quality level (0.0 to 1.0)
    pub quality_level: Option<f32>,

    /// Target platform optimizations
    pub platform_optimizations: Vec<String>,

    /// Memory pool hint
    pub memory_pool: Option<String>,

    /// Whether to generate mipmaps (for textures)
    pub generate_mipmaps: bool,
}

impl Default for LoadingHints {
    fn default() -> Self {
        Self {
            priority: crate::state::LoadingPriority::Normal,
            preload: false,
            persistent: false,
            compress_in_memory: false,
            quality_level: None,
            platform_optimizations: Vec::new(),
            memory_pool: None,
            generate_mipmaps: true,
        }
    }
}

/// Compression information for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Compression algorithm used
    pub algorithm: String,

    /// Compression ratio achieved
    pub ratio: f32,

    /// Original uncompressed size
    pub original_size: usize,

    /// Compressed size
    pub compressed_size: usize,

    /// Whether the resource is stored compressed
    pub stored_compressed: bool,
}

/// Platform-specific resource information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Supported platforms
    pub supported_platforms: Vec<String>,

    /// Platform-specific variants
    pub variants: HashMap<String, PathBuf>,

    /// Quality levels per platform
    pub quality_levels: HashMap<String, f32>,

    /// Platform-specific loading hints
    pub platform_hints: HashMap<String, HashMap<String, String>>,
}

/// Resource metadata database for efficient queries
pub trait MetadataDatabase: Send + Sync {
    /// Store metadata for a resource
    fn store(&mut self, metadata: ResourceMetadata) -> Result<(), MetadataError>;

    /// Retrieve metadata by resource ID
    fn get(&self, id: ResourceId) -> Option<ResourceMetadata>;

    /// Retrieve metadata by file path
    fn get_by_path(&self, path: &str) -> Option<ResourceMetadata>;

    /// Find resources by type
    fn find_by_type(&self, resource_type: ResourceType) -> Vec<ResourceMetadata>;

    /// Find resources by tag
    fn find_by_tag(&self, tag: &str) -> Vec<ResourceMetadata>;

    /// Find resources with custom field
    fn find_by_custom_field(&self, key: &str, value: &MetadataValue) -> Vec<ResourceMetadata>;

    /// Get all tracked resources
    fn all_resources(&self) -> Vec<ResourceMetadata>;

    /// Remove metadata for a resource
    fn remove(&mut self, id: ResourceId) -> Result<bool, MetadataError>;

    /// Clear all metadata
    fn clear(&mut self) -> Result<(), MetadataError>;

    /// Update metadata for an existing resource
    fn update(&mut self, metadata: ResourceMetadata) -> Result<(), MetadataError>;

    /// Get database statistics
    fn stats(&self) -> DatabaseStats;
}

/// Metadata database errors
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("Resource not found: {id}")]
    NotFound { id: ResourceId },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Invalid metadata format")]
    InvalidFormat,

    #[error("Database is read-only")]
    ReadOnly,

    #[error("Concurrent access conflict")]
    ConcurrencyError,
}

/// Database statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total number of resources tracked
    pub total_resources: usize,

    /// Number of resources by type
    pub by_type: HashMap<String, usize>,

    /// Total metadata size in bytes
    pub metadata_size_bytes: usize,

    /// Number of dependencies tracked
    pub total_dependencies: usize,

    /// Number of custom fields
    pub total_custom_fields: usize,

    /// Database version
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_detection() {
        assert_eq!(
            ResourceType::from_extension("png"),
            Some(ResourceType::Texture)
        );
        assert_eq!(
            ResourceType::from_extension("wav"),
            Some(ResourceType::Audio)
        );
        assert_eq!(
            ResourceType::from_extension("obj"),
            Some(ResourceType::Model)
        );
        assert_eq!(ResourceType::from_extension("unknown"), None);
    }

    #[test]
    fn test_resource_metadata() {
        let id = ResourceId::new(123);
        let path = PathBuf::from("test.png");
        let mut metadata = ResourceMetadata::new(id, path, ResourceType::Texture);

        metadata.add_tag("ui".to_string());
        metadata.add_tag("button".to_string());

        assert!(metadata.has_tag("ui"));
        assert!(metadata.has_tag("button"));
        assert!(!metadata.has_tag("nonexistent"));

        metadata.add_custom_field("width".to_string(), MetadataValue::Integer(512));
        metadata.add_custom_field("height".to_string(), MetadataValue::Integer(256));

        assert_eq!(
            metadata.get_custom_field("width").unwrap().as_integer(),
            Some(512)
        );
        assert_eq!(
            metadata.get_custom_field("height").unwrap().as_integer(),
            Some(256)
        );
    }

    #[test]
    fn test_metadata_value() {
        let string_val = MetadataValue::String("test".to_string());
        let int_val = MetadataValue::Integer(42);
        let float_val = MetadataValue::Float(3.14);
        let bool_val = MetadataValue::Boolean(true);

        assert_eq!(string_val.as_string(), Some("test"));
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(float_val.as_float(), Some(3.14));
        assert_eq!(bool_val.as_boolean(), Some(true));

        // Test type conversion
        assert_eq!(int_val.as_float(), Some(42.0));
    }

    #[test]
    fn test_loading_hints() {
        let hints = LoadingHints::default();
        assert_eq!(hints.priority, crate::state::LoadingPriority::Normal);
        assert!(!hints.preload);
        assert!(!hints.persistent);
        assert!(hints.generate_mipmaps);
    }

    #[test]
    fn test_dependency() {
        let dependency = ResourceDependency {
            path: "material.mat".to_string(),
            dependency_type: DependencyType::Reference,
            required: true,
            version_requirement: Some(">=1.0".to_string()),
        };

        assert_eq!(dependency.dependency_type, DependencyType::Reference);
        assert!(dependency.required);
    }

    #[test]
    fn test_memory_efficiency() {
        let id = ResourceId::new(456);
        let path = PathBuf::from("test.wav");
        let mut metadata = ResourceMetadata::new(id, path, ResourceType::Audio);

        metadata.file_size = 1000;
        metadata.memory_size = Some(2000);

        assert_eq!(metadata.memory_efficiency(), Some(2.0));

        metadata.memory_size = None;
        assert_eq!(metadata.memory_efficiency(), None);
    }
}
