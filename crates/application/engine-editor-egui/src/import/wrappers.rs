use async_trait::async_trait;
use engine_asset_import::{AssetImporter, ImportContext, ImportError, ImportResult};
use engine_audio_import::{AudioData, AudioImporter};
use engine_mesh_import::{MeshData, ObjImporter};
use engine_texture_import::{TextureData, TextureImporter};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Wrapper for mesh importers that converts MeshData to Vec<u8>
#[allow(dead_code)]
pub struct MeshImporterWrapper<I: AssetImporter<Asset = MeshData>> {
    inner: I,
}

#[allow(dead_code)]
impl<I: AssetImporter<Asset = MeshData>> MeshImporterWrapper<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

#[allow(dead_code)]
#[async_trait]
impl<I: AssetImporter<Asset = MeshData> + Send + Sync> AssetImporter for MeshImporterWrapper<I> {
    type Asset = Vec<u8>;

    fn supported_extensions(&self) -> &[&str] {
        self.inner.supported_extensions()
    }

    fn can_import(&self, path: &Path) -> bool {
        self.inner.can_import(path)
    }

    async fn import(&self, path: &Path, context: &ImportContext) -> ImportResult<Self::Asset> {
        // Import the mesh data
        let mesh_data = self.inner.import(path, context).await?;

        // Serialize to bytes using bincode
        bincode::serialize(&mesh_data).map_err(|e| {
            ImportError::ProcessingError(format!("Failed to serialize mesh data: {}", e))
        })
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Wrapper for texture importers that converts TextureData to Vec<u8>
#[allow(dead_code)]
pub struct TextureImporterWrapper<I: AssetImporter<Asset = TextureData>> {
    inner: I,
}

#[allow(dead_code)]
impl<I: AssetImporter<Asset = TextureData>> TextureImporterWrapper<I> {
    #[allow(dead_code)]
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

#[allow(dead_code)]
#[async_trait]
impl<I: AssetImporter<Asset = TextureData> + Send + Sync> AssetImporter
    for TextureImporterWrapper<I>
{
    type Asset = Vec<u8>;

    fn supported_extensions(&self) -> &[&str] {
        self.inner.supported_extensions()
    }

    fn can_import(&self, path: &Path) -> bool {
        self.inner.can_import(path)
    }

    async fn import(&self, path: &Path, context: &ImportContext) -> ImportResult<Self::Asset> {
        // Import the texture data
        let texture_data = self.inner.import(path, context).await?;

        // Serialize to bytes using bincode
        bincode::serialize(&texture_data).map_err(|e| {
            ImportError::ProcessingError(format!("Failed to serialize texture data: {}", e))
        })
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Wrapper for audio importers that converts AudioData to Vec<u8>
#[allow(dead_code)]
pub struct AudioImporterWrapper<I: AssetImporter<Asset = AudioData>> {
    inner: I,
}

#[allow(dead_code)]
impl<I: AssetImporter<Asset = AudioData>> AudioImporterWrapper<I> {
    #[allow(dead_code)]
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

#[allow(dead_code)]
#[async_trait]
impl<I: AssetImporter<Asset = AudioData> + Send + Sync> AssetImporter for AudioImporterWrapper<I> {
    type Asset = Vec<u8>;

    fn supported_extensions(&self) -> &[&str] {
        self.inner.supported_extensions()
    }

    fn can_import(&self, path: &Path) -> bool {
        self.inner.can_import(path)
    }

    async fn import(&self, path: &Path, context: &ImportContext) -> ImportResult<Self::Asset> {
        // Import the audio data
        let audio_data = self.inner.import(path, context).await?;

        // Serialize to bytes using bincode
        bincode::serialize(&audio_data).map_err(|e| {
            ImportError::ProcessingError(format!("Failed to serialize audio data: {}", e))
        })
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

// Convenience type aliases for commonly used wrappers
pub type ObjImporterWrapper = MeshImporterWrapper<ObjImporter>;
#[allow(dead_code)]
pub type StandardTextureImporterWrapper = TextureImporterWrapper<TextureImporter>;
#[allow(dead_code)]
pub type StandardAudioImporterWrapper = AudioImporterWrapper<AudioImporter>;

// Factory functions for creating wrapped importers
#[allow(dead_code)]
impl ObjImporterWrapper {
    pub fn create() -> Self {
        Self::new(ObjImporter::new())
    }
}

#[allow(dead_code)]
impl StandardTextureImporterWrapper {
    #[allow(dead_code)]
    pub fn create() -> Self {
        Self::new(TextureImporter::new())
    }
}

#[allow(dead_code)]
impl StandardAudioImporterWrapper {
    #[allow(dead_code)]
    pub fn create() -> Self {
        Self::new(AudioImporter::new())
    }
}

/// Enum to represent different asset data types in their serialized form
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedAssetData {
    Mesh(Vec<u8>),
    Texture(Vec<u8>),
    Audio(Vec<u8>),
}

#[allow(dead_code)]
impl SerializedAssetData {
    /// Deserialize mesh data from bytes
    #[allow(dead_code)]
    pub fn deserialize_mesh(&self) -> Result<MeshData, bincode::Error> {
        match self {
            SerializedAssetData::Mesh(data) => bincode::deserialize(data),
            _ => Err(Box::new(bincode::ErrorKind::Custom(
                "Not mesh data".to_string(),
            ))),
        }
    }

    /// Deserialize texture data from bytes
    #[allow(dead_code)]
    pub fn deserialize_texture(&self) -> Result<TextureData, bincode::Error> {
        match self {
            SerializedAssetData::Texture(data) => bincode::deserialize(data),
            _ => Err(Box::new(bincode::ErrorKind::Custom(
                "Not texture data".to_string(),
            ))),
        }
    }

    /// Deserialize audio data from bytes
    #[allow(dead_code)]
    pub fn deserialize_audio(&self) -> Result<AudioData, bincode::Error> {
        match self {
            SerializedAssetData::Audio(data) => bincode::deserialize(data),
            _ => Err(Box::new(bincode::ErrorKind::Custom(
                "Not audio data".to_string(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_obj_importer_wrapper() {
        let wrapper = ObjImporterWrapper::create();

        // Test supported extensions
        assert_eq!(wrapper.supported_extensions(), &["obj"]);

        // Test can_import
        assert!(wrapper.can_import(Path::new("test.obj")));
        assert!(!wrapper.can_import(Path::new("test.png")));
    }

    #[tokio::test]
    async fn test_texture_importer_wrapper() {
        let wrapper = StandardTextureImporterWrapper::create();

        // Test supported extensions
        let extensions = wrapper.supported_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));

        // Test can_import
        assert!(wrapper.can_import(Path::new("test.png")));
        assert!(wrapper.can_import(Path::new("test.jpg")));
        assert!(!wrapper.can_import(Path::new("test.obj")));
    }

    #[tokio::test]
    async fn test_audio_importer_wrapper() {
        let wrapper = StandardAudioImporterWrapper::create();

        // Test supported extensions
        let extensions = wrapper.supported_extensions();
        assert!(extensions.contains(&"wav"));
        assert!(extensions.contains(&"mp3"));

        // Test can_import
        assert!(wrapper.can_import(Path::new("test.wav")));
        assert!(wrapper.can_import(Path::new("test.mp3")));
        assert!(!wrapper.can_import(Path::new("test.png")));
    }

    #[test]
    fn test_serialized_asset_data() {
        use engine_mesh_import::types::{Material, Vertex};

        // Create test mesh data
        let mesh_data = MeshData {
            name: "Test Mesh".to_string(),
            vertices: vec![Vertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            }],
            indices: vec![0],
            material: Some(Material::default()),
        };

        // Serialize and create SerializedAssetData
        let serialized = bincode::serialize(&mesh_data).unwrap();
        let asset_data = SerializedAssetData::Mesh(serialized);

        // Test deserialization
        let deserialized = asset_data.deserialize_mesh().unwrap();
        assert_eq!(deserialized.name, "Test Mesh");
        assert_eq!(deserialized.vertices.len(), 1);

        // Test wrong type deserialization
        assert!(asset_data.deserialize_texture().is_err());
        assert!(asset_data.deserialize_audio().is_err());
    }
}
