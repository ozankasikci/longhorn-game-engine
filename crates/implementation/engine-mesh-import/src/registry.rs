use engine_asset_import::AssetImporter;
use std::collections::HashMap;

pub struct MeshImportRegistry {
    importers: HashMap<String, ImporterType>,
}

enum ImporterType {
    Obj(crate::ObjImporter),
    Gltf(crate::GltfImporter),
    Fbx(crate::FbxImporter),
}

impl MeshImportRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            importers: HashMap::new(),
        };

        // Register default importers
        registry.importers.insert(
            "obj".to_string(),
            ImporterType::Obj(crate::ObjImporter::new()),
        );
        registry.importers.insert(
            "gltf".to_string(),
            ImporterType::Gltf(crate::GltfImporter::new()),
        );
        registry.importers.insert(
            "glb".to_string(),
            ImporterType::Gltf(crate::GltfImporter::new()),
        );
        registry.importers.insert(
            "fbx".to_string(),
            ImporterType::Fbx(crate::FbxImporter::new()),
        );

        registry
    }

    pub fn get_importer(
        &self,
        extension: &str,
    ) -> Option<&dyn AssetImporter<Asset = crate::MeshData>> {
        self.importers
            .get(&extension.to_lowercase())
            .map(|importer| match importer {
                ImporterType::Obj(imp) => imp as &dyn AssetImporter<Asset = crate::MeshData>,
                ImporterType::Gltf(_) => panic!("GLTF returns Vec<MeshData>, not MeshData"),
                ImporterType::Fbx(_) => panic!("FBX returns Vec<MeshData>, not MeshData"),
            })
    }

    pub fn supported_formats(&self) -> Vec<&str> {
        self.importers.keys().map(|s| s.as_str()).collect()
    }
}
